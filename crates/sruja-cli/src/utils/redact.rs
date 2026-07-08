//! Best-effort secrets redaction for run snapshots and context events.
//!
//! Walks JSON values recursively and replaces likely secrets with `[REDACTED]`
//! before storage. Non-destructive — the original value is not modified, only the
//! serialized snapshot is cleaned.

use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Common API key prefixes (with or without quotes around value)
        Regex::new(r"(?i)(?:api[_-]?key|apikey|api_secret|secret[_-]?key|access[_-]?key|auth[_-]?token)\s*[:=]\s*[^\s]{12,}").unwrap(),
        // Bearer tokens
        Regex::new(r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),
        // JWT tokens (header.payload.signature pattern)
        Regex::new(r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+").unwrap(),
        // Private key headers (matches the begin line)
        Regex::new(r"-----BEGIN\s+(?:RSA|EC|DSA|OPENSSH|PGP)\s+PRIVATE\s+KEY-----").unwrap(),
        // DB URLs with embedded credentials
        Regex::new(r"(?:postgres|mysql|mongodb|redis)://[^:]+:[^@]+@").unwrap(),
        // Generic "token=" or "password=" with values (8+ chars)
        Regex::new(r"(?i)(?:token|password|passwd|pwd|secret|credential)\s*[:=]\s*[^\s]{8,}").unwrap(),
        // AWS keys (AKIA + 16 uppercase alphanumeric)
        Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        // GitHub tokens (ghp_, gho_, ghu_, ghs_, ghr_ prefixes)
        Regex::new(r"(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{20,}").unwrap(),
        // OpenAI / Anthropic API keys
        Regex::new(r"(?:sk-[A-Za-z0-9]{20,}|sk-ant-[A-Za-z0-9\-_]{20,})").unwrap(),
        // env vars with _KEY, _TOKEN, _SECRET, _PASSWORD suffixes
        Regex::new(r"(?i)[A-Za-z0-9_]+(?:_KEY|_TOKEN|_SECRET|_PASSWORD)\s*=\s*[^\s]{8,}").unwrap(),
        // Generic base64-encoded-looking tokens (64+ base64 chars)
        // Note: 40-char pattern was too aggressive, matching git SHAs and other non-secrets
        Regex::new(r"[A-Za-z0-9+/=]{64,}").unwrap(),
    ]
});

static REDACTED: &str = "[REDACTED]";

pub fn redact_json_value(value: &mut Value) {
    match value {
        Value::String(s) => {
            *s = redact_string(std::mem::take(s));
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                redact_json_value(v);
            }
        }
        Value::Object(map) => {
            for (_k, v) in map.iter_mut() {
                redact_json_value(v);
            }
        }
        _ => {}
    }
}

pub fn redact_string(mut s: String) -> String {
    for pattern in SECRET_PATTERNS.iter() {
        s = pattern.replace_all(&s, REDACTED).to_string();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_bearer_token() {
        let s = redact_string("Authorization: Bearer abc123def456ghi789jkl".into());
        assert!(!s.contains("abc123def456ghi789jkl"));
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_api_key() {
        let s = redact_string("export API_KEY=sk-1234567890abcdefghij".into());
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("sk-12345"));
    }

    #[test]
    fn redacts_database_url() {
        let s = redact_string("DATABASE_URL=postgres://admin:hunter2@db.internal:5432/staging".into());
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("hunter2"));
    }

    #[test]
    fn redacts_jwt() {
        let s = redact_string("token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U".into());
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_aws_key() {
        let s = redact_string("AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE".into());
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_openai_key() {
        let s = redact_string("OPENAI_API_KEY=sk-proj-1234567890abcdefghijklmnopqrstuv".into());
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_json_object_recursively() {
        let mut v: Value = serde_json::json!({
            "env": {
                "DATABASE_URL": "postgres://user:secret@localhost/db"
            },
            "headers": {
                "Authorization": "Bearer eyJhbGci.token.signature"
            },
            "safe": "keep-this"
        });
        redact_json_value(&mut v);
        let s = v.to_string();
        assert!(!s.contains("secret"));
        assert!(!s.contains("eyJhbGci"));
        assert!(s.contains("keep-this"));
    }

    #[test]
    fn non_string_values_preserved() {
        let mut v: Value = serde_json::json!({"count": 42, "flag": true, "arr": [1, 2, 3]});
        redact_json_value(&mut v);
        assert_eq!(v["count"], 42);
        assert_eq!(v["flag"], true);
    }
}
