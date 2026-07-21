//! Structured test result parsing for convergence tracking.
//!
//! Parses test output from various formats into a uniform [`TestSuiteResult`]
//! that the convergence system can use to compute pass rates, detect regressions,
//! and drive metric-driven loop termination.
//!
//! Supported formats:
//! - Playwright JSON report
//! - JUnit XML
//! - cargo test output (text)
//! - Generic exit-code + text fallback

use serde::{Deserialize, Serialize};

/// Result of parsing a test suite's output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestSuiteResult {
    /// Total number of tests that ran.
    pub total: usize,
    /// Number of tests that passed.
    pub passed: usize,
    /// Number of tests that failed.
    pub failed: usize,
    /// Number of tests that were skipped.
    pub skipped: usize,
    /// Per-test failure details (only for failed tests).
    pub failures: Vec<TestFailure>,
    /// Pass rate as a fraction (0.0–1.0). Computed from passed/total.
    pub pass_rate: f64,
    /// Duration in milliseconds, if reported.
    pub duration_ms: Option<u64>,
    /// The format that was parsed.
    pub format: TestFormat,
}

/// A single test failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    /// Test name or identifier.
    pub name: String,
    /// File where the test lives, if known.
    pub file: Option<String>,
    /// Error message or assertion failure text.
    pub error: String,
    /// Whether this test was passing in a previous iteration (regression flag).
    /// Set by the metrics tracker, not the parser.
    #[serde(default)]
    pub is_regression: bool,
}

/// Which format was detected and parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TestFormat {
    PlaywrightJson,
    JunitXml,
    CargoTest,
    #[default]
    Unknown,
}

/// Detect the format of test output and parse it.
///
/// Tries parsers in order of specificity:
/// 1. Playwright JSON (starts with `{` and contains `suites`)
/// 2. JUnit XML (starts with `<` and contains `<testsuite`)
/// 3. cargo test text (contains `test result:`)
/// 4. Fallback: exit-code based (caller provides exit code)
pub fn parse_test_output(stdout: &str, stderr: &str, exit_code: Option<i32>) -> TestSuiteResult {
    let combined = format!("{stdout}\n{stderr}");
    let trimmed = combined.trim();

    // Playwright JSON
    if trimmed.starts_with('{') && trimmed.contains("\"suites\"") {
        if let Some(result) = parse_playwright_json(trimmed) {
            return result;
        }
    }

    // JUnit XML
    if trimmed.contains("<testsuite") || trimmed.contains("<testsuites") {
        if let Some(result) = parse_junit_xml(trimmed) {
            return result;
        }
    }

    // cargo test output
    if combined.contains("test result:") {
        if let Some(result) = parse_cargo_test(&combined) {
            return result;
        }
    }

    // Fallback: exit-code based
    parse_exit_code_fallback(exit_code)
}

// ---------------------------------------------------------------------------
// Playwright JSON
// ---------------------------------------------------------------------------

fn parse_playwright_json(text: &str) -> Option<TestSuiteResult> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let stats = v.get("stats")?;

    let expected = stats.get("expected")?.as_u64()? as usize;
    let unexpected = stats.get("unexpected")?.as_u64()? as usize;
    let skipped = stats.get("skipped")?.as_u64()? as usize;
    let total = expected + unexpected + skipped;
    let duration = stats.get("duration")?.as_u64();

    let mut failures = Vec::new();
    if unexpected > 0 {
        extract_playwright_failures(&v, &mut failures);
    }

    let pass_rate = if total > 0 {
        expected as f64 / total as f64
    } else {
        1.0
    };

    Some(TestSuiteResult {
        total,
        passed: expected,
        failed: unexpected,
        skipped,
        failures,
        pass_rate,
        duration_ms: duration,
        format: TestFormat::PlaywrightJson,
    })
}

fn extract_playwright_failures(v: &serde_json::Value, failures: &mut Vec<TestFailure>) {
    if let Some(suites) = v.get("suites").and_then(|s| s.as_array()) {
        for suite in suites {
            extract_failures_from_suite(suite, failures);
        }
    }
}

fn extract_failures_from_suite(suite: &serde_json::Value, failures: &mut Vec<TestFailure>) {
    if let Some(specs) = suite.get("specs").and_then(|s| s.as_array()) {
        for spec in specs {
            let test_name = spec
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();
            let file = suite
                .get("file")
                .and_then(|f| f.as_str())
                .map(|s| s.to_string());

            if let Some(tests) = spec.get("tests").and_then(|t| t.as_array()) {
                for test in tests {
                    let status = test.get("status").and_then(|s| s.as_str()).unwrap_or("");
                    if status == "unexpected" || status == "failed" {
                        let error = extract_test_error(test);
                        failures.push(TestFailure {
                            name: test_name.clone(),
                            file: file.clone(),
                            error,
                            is_regression: false,
                        });
                    }
                }
            }
        }
    }

    // Recurse into nested suites
    if let Some(suites) = suite.get("suites").and_then(|s| s.as_array()) {
        for nested in suites {
            extract_failures_from_suite(nested, failures);
        }
    }
}

fn extract_test_error(test: &serde_json::Value) -> String {
    if let Some(results) = test.get("results").and_then(|r| r.as_array()) {
        if let Some(last) = results.last() {
            if let Some(errors) = last.get("errors").and_then(|e| e.as_array()) {
                return errors
                    .iter()
                    .filter_map(|e| {
                        e.get("message")
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
            }
            if let Some(status) = last.get("status").and_then(|s| s.as_str()) {
                return format!("test {status}");
            }
        }
    }
    "unknown error".to_string()
}

// ---------------------------------------------------------------------------
// JUnit XML
// ---------------------------------------------------------------------------

fn parse_junit_xml(text: &str) -> Option<TestSuiteResult> {
    // Simple XML parsing without a full XML library — extract attributes from
    // <testsuite> tags. This handles the common single-suite and multi-suite cases.
    let mut total = 0usize;
    let mut failures_count = 0usize;
    let mut skipped = 0usize;
    let mut failures = Vec::new();

    // Find all <testsuite ...> opening tags to get aggregate counts
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<testsuite ") || trimmed.starts_with("<testsuite>") {
            if let Some(t) = extract_xml_attr(trimmed, "tests") {
                total += t.parse::<usize>().unwrap_or(0);
            }
            if let Some(f) = extract_xml_attr(trimmed, "failures") {
                failures_count += f.parse::<usize>().unwrap_or(0);
            }
            if let Some(s) = extract_xml_attr(trimmed, "skipped") {
                skipped += s.parse::<usize>().unwrap_or(0);
            }
        }
    }

    // Extract individual failure details from <testcase> elements
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if !t.starts_with("<testcase ") {
            continue;
        }

        let name = extract_xml_attr(t, "name").unwrap_or_default();
        let file = extract_xml_attr(t, "classname");

        // Self-closing tags can't have child elements
        if t.ends_with("/>") {
            continue;
        }

        // Look ahead for <failure> child element
        let mut is_failure = false;
        let mut error_msg = String::new();
        for j in (i + 1)..lines.len().min(i + 20) {
            let next = lines[j].trim();
            if next.starts_with("<failure") {
                is_failure = true;
                error_msg = extract_xml_attr(next, "message").unwrap_or_default();
                if error_msg.is_empty() {
                    // Collect text content until </failure>
                    let mut content = Vec::new();
                    for k in (j + 1)..lines.len().min(j + 50) {
                        let l = lines[k].trim();
                        if l.starts_with("</failure") {
                            break;
                        }
                        content.push(l.to_string());
                    }
                    error_msg = content.join("\n").trim().to_string();
                }
                break;
            }
            if next.starts_with("</testcase") || next.starts_with("<testcase ") {
                break;
            }
        }

        if is_failure {
            failures.push(TestFailure {
                name,
                file,
                error: if error_msg.is_empty() {
                    "test failed".to_string()
                } else {
                    error_msg
                },
                is_regression: false,
            });
        }
    }

    if total == 0 && failures.is_empty() {
        return None;
    }

    let passed = total.saturating_sub(failures_count).saturating_sub(skipped);
    let pass_rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        1.0
    };

    Some(TestSuiteResult {
        total,
        passed,
        failed: failures_count,
        skipped,
        failures,
        pass_rate,
        duration_ms: None,
        format: TestFormat::JunitXml,
    })
}

fn extract_xml_attr(tag: &str, attr: &str) -> Option<String> {
    // Handle attr="value" pattern
    let pattern = format!("{attr}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// ---------------------------------------------------------------------------
// cargo test output
// ---------------------------------------------------------------------------

fn parse_cargo_test(text: &str) -> Option<TestSuiteResult> {
    // Look for the summary line: "test result: ok. X passed; Y failed; Z ignored; W measured; N filtered out"
    let summary_line = text.lines().find(|l| l.contains("test result:"))?;

    let passed = extract_cargo_count(summary_line, "passed");
    let failed = extract_cargo_count(summary_line, "failed");
    let ignored = extract_cargo_count(summary_line, "ignored");
    let measured = extract_cargo_count(summary_line, "measured");
    let filtered = extract_cargo_count(summary_line, "filtered out");

    let total = passed + failed + ignored + measured;

    // Extract individual failure details
    let mut failures = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        // Match lines like: "test some::test_name ... FAILED"
        if trimmed.ends_with("FAILED") && trimmed.starts_with("test ") {
            let parts: Vec<&str> = trimmed.splitn(3, whitespace_split).collect();
            if parts.len() >= 2 {
                let name = parts[1].to_string();
                failures.push(TestFailure {
                    name,
                    file: None,
                    error: extract_cargo_failure_detail(text, parts[1]),
                    is_regression: false,
                });
            }
        }
    }

    if total == 0 && failures.is_empty() && filtered > 0 {
        // All tests filtered out — treat as no-op (pass)
        return Some(TestSuiteResult {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            failures: vec![],
            pass_rate: 1.0,
            duration_ms: None,
            format: TestFormat::CargoTest,
        });
    }

    if total == 0 && failures.is_empty() {
        return None;
    }

    let pass_rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        1.0
    };

    Some(TestSuiteResult {
        total,
        passed,
        failed,
        skipped: ignored,
        failures,
        pass_rate,
        duration_ms: None,
        format: TestFormat::CargoTest,
    })
}

fn extract_cargo_count(line: &str, label: &str) -> usize {
    // "X passed" or "Y failed" etc.
    let pattern = format!(" {label}");
    if let Some(pos) = line.find(&pattern) {
        let before = &line[..pos];
        // Walk backwards to find the number
        let num_str: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        return num_str.parse().unwrap_or(0);
    }
    0
}

fn extract_cargo_failure_detail(full_output: &str, test_name: &str) -> String {
    // Look for "---- test_name stdout ----" followed by error details
    let marker = format!("---- {test_name} stdout ----");
    if let Some(pos) = full_output.find(&marker) {
        let after = &full_output[pos + marker.len()..];
        // Collect until the next "----" or "test result:" or end
        let detail: String = after
            .lines()
            .take_while(|l| !l.starts_with("----") && !l.contains("test result:"))
            .collect::<Vec<_>>()
            .join("\n");
        let trimmed = detail.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "test failed".to_string()
}

fn whitespace_split(c: char) -> bool {
    c.is_whitespace()
}

// ---------------------------------------------------------------------------
// Fallback
// ---------------------------------------------------------------------------

fn parse_exit_code_fallback(exit_code: Option<i32>) -> TestSuiteResult {
    let passed = if exit_code == Some(0) { 1 } else { 0 };
    let failed = if exit_code == Some(0) { 0 } else { 1 };
    TestSuiteResult {
        total: 1,
        passed,
        failed,
        skipped: 0,
        failures: if failed > 0 {
            vec![TestFailure {
                name: "exit_code_check".to_string(),
                file: None,
                error: format!(
                    "process exited with code {}",
                    exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into())
                ),
                is_regression: false,
            }]
        } else {
            vec![]
        },
        pass_rate: passed as f64,
        duration_ms: None,
        format: TestFormat::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Multi-step aggregation
// ---------------------------------------------------------------------------

/// Aggregate multiple [`TestSuiteResult`]s from parallel verify steps into a
/// single combined result.
pub fn aggregate_results(results: &[TestSuiteResult]) -> TestSuiteResult {
    if results.is_empty() {
        return TestSuiteResult::default();
    }
    if results.len() == 1 {
        return results[0].clone();
    }

    let mut combined = TestSuiteResult::default();
    for r in results {
        combined.total += r.total;
        combined.passed += r.passed;
        combined.failed += r.failed;
        combined.skipped += r.skipped;
        combined.failures.extend(r.failures.clone());
        if let Some(d) = r.duration_ms {
            combined.duration_ms = Some(combined.duration_ms.unwrap_or(0) + d);
        }
    }
    combined.pass_rate = if combined.total > 0 {
        combined.passed as f64 / combined.total as f64
    } else {
        1.0
    };
    // Use the most specific format detected
    combined.format = results
        .iter()
        .find(|r| r.format != TestFormat::Unknown)
        .map(|r| r.format)
        .unwrap_or(TestFormat::Unknown);

    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cargo_test_summary() {
        let output = "\
running 10 tests
test auth::test_login ... ok
test auth::test_logout ... FAILED
test db::test_connect ... ok
test db::test_query ... FAILED
test api::test_health ... ok

failures:

---- auth::test_logout stdout ----
panicked at 'assertion failed', src/auth.rs:42

---- db::test_query stdout ----
panicked at 'timeout', src/db.rs:99

failures:
    auth::test_logout
    db::test_query

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.42s
";

        let result = parse_cargo_test(output).unwrap();
        assert_eq!(result.total, 5); // 3 passed + 2 failed + 0 ignored
        assert_eq!(result.passed, 3);
        assert_eq!(result.failed, 2);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failures.len(), 2);
        assert_eq!(result.failures[0].name, "auth::test_logout");
        assert_eq!(result.failures[1].name, "db::test_query");
        assert!((result.pass_rate - 0.6).abs() < 0.01);
        assert_eq!(result.format, TestFormat::CargoTest);
    }

    #[test]
    fn parse_cargo_test_all_passing() {
        let output = "\
running 5 tests
test a ... ok
test b ... ok
test c ... ok
test d ... ok
test e ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
";

        let result = parse_cargo_test(output).unwrap();
        assert_eq!(result.total, 5);
        assert_eq!(result.passed, 5);
        assert_eq!(result.failed, 0);
        assert!((result.pass_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_cargo_test_filtered_out() {
        let output = "\
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
";

        let result = parse_cargo_test(output).unwrap();
        assert_eq!(result.total, 0);
        assert!((result.pass_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_playwright_json_report() {
        let json = r#"{
            "stats": {
                "expected": 8,
                "unexpected": 2,
                "flaky": 0,
                "skipped": 1,
                "duration": 15000
            },
            "suites": [
                {
                    "file": "tests/login.spec.ts",
                    "specs": [
                        {
                            "title": "should login successfully",
                            "tests": [{"status": "expected", "results": [{"status": "passed"}]}]
                        },
                        {
                            "title": "should handle invalid password",
                            "tests": [{"status": "unexpected", "results": [{"status": "failed", "errors": [{"message": "Expected 'error' but got 'undefined'"}]}]}]
                        }
                    ]
                }
            ]
        }"#;

        let result = parse_playwright_json(json).unwrap();
        assert_eq!(result.total, 11); // 8+2+1
        assert_eq!(result.passed, 8);
        assert_eq!(result.failed, 2);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].name, "should handle invalid password");
        assert_eq!(
            result.failures[0].error,
            "Expected 'error' but got 'undefined'"
        );
        assert_eq!(result.duration_ms, Some(15000));
        assert_eq!(result.format, TestFormat::PlaywrightJson);
    }

    #[test]
    fn parse_junit_xml_report() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="auth" tests="4" failures="1" skipped="0" time="2.5">
    <testcase name="test_login" classname="auth" time="0.5"/>
    <testcase name="test_logout" classname="auth" time="0.3">
      <failure message="Expected true but got false">
        assertion failed at auth.rs:42
      </failure>
    </testcase>
    <testcase name="test_signup" classname="auth" time="1.0"/>
    <testcase name="test_reset" classname="auth" time="0.7"/>
  </testsuite>
</testsuites>"#;

        let result = parse_junit_xml(xml).unwrap();
        assert_eq!(result.total, 4);
        assert_eq!(result.passed, 3);
        assert_eq!(result.failed, 1);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].name, "test_logout");
        assert_eq!(result.failures[0].error, "Expected true but got false");
        assert_eq!(result.format, TestFormat::JunitXml);
    }

    #[test]
    fn auto_detect_cargo_test() {
        let stdout = "running 2 tests\ntest a ... ok\ntest b ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n";
        let result = parse_test_output(stdout, "", Some(1));
        assert_eq!(result.format, TestFormat::CargoTest);
        assert_eq!(result.total, 2);
        assert_eq!(result.failed, 1);
    }

    #[test]
    fn auto_detect_playwright_json() {
        let stdout =
            r#"{"stats":{"expected":5,"unexpected":0,"skipped":0,"duration":1000},"suites":[]}"#;
        let result = parse_test_output(stdout, "", Some(0));
        assert_eq!(result.format, TestFormat::PlaywrightJson);
        assert_eq!(result.total, 5);
    }

    #[test]
    fn fallback_on_exit_code_success() {
        let result = parse_test_output("some output", "", Some(0));
        assert_eq!(result.format, TestFormat::Unknown);
        assert_eq!(result.total, 1);
        assert_eq!(result.passed, 1);
        assert!((result.pass_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fallback_on_exit_code_failure() {
        let result = parse_test_output("some output", "error", Some(1));
        assert_eq!(result.total, 1);
        assert_eq!(result.failed, 1);
        assert!((result.pass_rate).abs() < f64::EPSILON);
        assert_eq!(result.failures.len(), 1);
    }

    #[test]
    fn aggregate_multiple_results() {
        let r1 = TestSuiteResult {
            total: 10,
            passed: 8,
            failed: 2,
            skipped: 0,
            failures: vec![TestFailure {
                name: "a".into(),
                file: None,
                error: "err".into(),
                is_regression: false,
            }],
            pass_rate: 0.8,
            duration_ms: Some(1000),
            format: TestFormat::CargoTest,
        };
        let r2 = TestSuiteResult {
            total: 5,
            passed: 5,
            failed: 0,
            skipped: 0,
            failures: vec![],
            pass_rate: 1.0,
            duration_ms: Some(500),
            format: TestFormat::JunitXml,
        };

        let combined = aggregate_results(&[r1, r2]);
        assert_eq!(combined.total, 15);
        assert_eq!(combined.passed, 13);
        assert_eq!(combined.failed, 2);
        assert_eq!(combined.failures.len(), 1);
        assert!((combined.pass_rate - 13.0 / 15.0).abs() < 0.01);
        assert_eq!(combined.duration_ms, Some(1500));
    }

    #[test]
    fn extract_xml_attr_basic() {
        assert_eq!(
            extract_xml_attr(r#"<testsuite name="auth" tests="4">"#, "tests"),
            Some("4".to_string())
        );
        assert_eq!(
            extract_xml_attr(r#"<testsuite name="auth" tests="4">"#, "name"),
            Some("auth".to_string())
        );
        assert_eq!(extract_xml_attr(r#"<testsuite>"#, "tests"), None);
    }
}
