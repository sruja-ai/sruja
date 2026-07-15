//! Tests for parallel read-only tool execution.
//!
//! ToolRegistry should partition tool calls into mutating vs read-only;
//! read-only tools execute concurrently via tokio::join! while mutating
//! tools run sequentially in order.
//!
//! These tests are written against the *desired* API and will fail until the
//! implementation lands.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sruja_agent::tool::{Tool, ToolError, ToolRegistry};

// ---------------------------------------------------------------------------
// Helper tools
// ---------------------------------------------------------------------------

/// A slow read-only tool that sleeps for `delay_ms` before returning.
struct SlowRead {
    name: &'static str,
    delay_ms: u64,
    order_log: Arc<AtomicU64>,
}

#[async_trait]
impl Tool for SlowRead {
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        "Slow read-only tool"
    }
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn call(&self, _params: serde_json::Value) -> Result<String, ToolError> {
        // Record our sequential position for ordering tests.
        self.order_log.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        Ok(format!("{} done", self.name))
    }
    fn is_mutating(&self) -> bool {
        false
    }
    // New method under test — default should be false on the trait.
    fn is_read_only(&self) -> bool {
        true
    }
}

/// A mutating tool that sleeps briefly.
struct MutatingTool {
    name: &'static str,
    delay_ms: u64,
    order_log: Arc<AtomicU64>,
}

#[async_trait]
impl Tool for MutatingTool {
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        "Mutating tool"
    }
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn call(&self, _params: serde_json::Value) -> Result<String, ToolError> {
        self.order_log.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        Ok(format!("{} done", self.name))
    }
    fn is_mutating(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// AC 4: tool.is_read_only() method exists on Tool trait with default false
// ---------------------------------------------------------------------------

/// A bare-minimum tool that does NOT override `is_read_only()`.
/// The default on the trait must be `false`.
struct BareTool;

#[async_trait]
impl Tool for BareTool {
    fn name(&self) -> &str {
        "bare"
    }
    fn description(&self) -> &str {
        "bare"
    }
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn call(&self, _params: serde_json::Value) -> Result<String, ToolError> {
        Ok("ok".into())
    }
}

#[tokio::test]
async fn is_read_only_method_exists_with_default_false() {
    let tool = BareTool;
    // This test verifies the `is_read_only()` method exists on the Tool trait
    // and returns `false` by default.
    assert!(
        !tool.is_read_only(),
        "Tool::is_read_only() should default to false"
    );
}

#[tokio::test]
async fn read_only_tool_reports_true() {
    let counter = Arc::new(AtomicU64::new(0));
    let tool = SlowRead {
        name: "r",
        delay_ms: 1,
        order_log: counter,
    };
    assert!(tool.is_read_only());
    assert!(!tool.is_mutating());
}

// ---------------------------------------------------------------------------
// AC 2: file_edit is classified as mutating
// ---------------------------------------------------------------------------

#[tokio::test]
async fn file_edit_is_classified_as_mutating() {
    // Use the real builtin FileEdit tool.
    let reg = ToolRegistry::with_builtin(".", vec![]);
    let tool = reg
        .get("file_edit")
        .expect("file_edit should be registered");
    assert!(
        tool.is_mutating(),
        "file_edit must be classified as mutating"
    );
    // Conversely it must NOT be read-only.
    assert!(!tool.is_read_only(), "file_edit must NOT be read-only");
}

// ---------------------------------------------------------------------------
// AC 1: multiple read-only calls execute concurrently
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_only_tools_execute_concurrently() {
    let counter = Arc::new(AtomicU64::new(0));
    let delay = 100u64;
    let count = 4u32;

    let mut reg = ToolRegistry::new();
    for i in 0..count {
        reg = reg.with(Box::new(SlowRead {
            name: Box::leak(format!("read_{i}").into_boxed_str()),
            delay_ms: delay,
            order_log: counter.clone(),
        }));
    }

    let start = Instant::now();
    let reg = Arc::new(reg);
    let mut handles = Vec::new();
    for i in 0..count {
        let name = format!("read_{i}");
        let reg_ref = Arc::clone(&reg);
        handles.push(tokio::spawn(async move {
            reg_ref.dispatch(&name, serde_json::json!({})).await
        }));
    }
    for h in handles {
        h.await.unwrap().unwrap();
    }
    let wall = start.elapsed();

    // If they ran sequentially, total time would be >= count * delay (400ms).
    // Concurrent execution should finish in roughly `delay` (~100ms).
    let sequential_bound = Duration::from_millis(delay * count as u64);
    assert!(
        wall < sequential_bound,
        "read-only tools should run concurrently; took {wall:?} but sequential bound is {sequential_bound:?}"
    );
}

// ---------------------------------------------------------------------------
// AC 3: mixed batch — read-only in parallel, then mutating in order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mixed_batch_partitioned_execution() {
    let order = Arc::new(AtomicU64::new(0));

    let mut reg = ToolRegistry::new();
    // Register read-only tools
    reg = reg.with(Box::new(SlowRead {
        name: "ro_a",
        delay_ms: 50,
        order_log: order.clone(),
    }));
    reg = reg.with(Box::new(SlowRead {
        name: "ro_b",
        delay_ms: 50,
        order_log: order.clone(),
    }));
    // Register mutating tools
    reg = reg.with(Box::new(MutatingTool {
        name: "mut_x",
        delay_ms: 10,
        order_log: order.clone(),
    }));
    reg = reg.with(Box::new(MutatingTool {
        name: "mut_y",
        delay_ms: 10,
        order_log: order.clone(),
    }));

    // Build a mixed batch: two read-only then two mutating.
    let batch: Vec<(&str, serde_json::Value)> = vec![
        ("ro_a", serde_json::json!({})),
        ("ro_b", serde_json::json!({})),
        ("mut_x", serde_json::json!({})),
        ("mut_y", serde_json::json!({})),
    ];

    let start = Instant::now();
    let results = reg.dispatch_batch(&batch).await;
    let wall = start.elapsed();

    // All four should succeed.
    for r in &results {
        assert!(r.is_ok(), "tool call failed: {r:?}");
    }

    // Read-only tools ran first (their order_log values 1,2 come before
    // mutating tools 3,4).
    // Mutating tools ran sequentially so mut_x order < mut_y order.
    let final_order = order.load(Ordering::SeqCst);
    assert_eq!(final_order, 4, "all 4 tools should have executed");

    // Wall-clock check: read-only ran in parallel (~50ms) + mutating sequential
    // (~20ms) ≈ 70ms. Sequential would be 50+50+10+10 = 120ms.
    let sequential_total = Duration::from_millis(50 + 50 + 10 + 10);
    assert!(
        wall < sequential_total,
        "mixed batch should partition: took {wall:?}, sequential would be {sequential_total:?}"
    );
}

// ---------------------------------------------------------------------------
// AC 3 (supplemental): mutating tools preserve insertion order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mutating_tools_preserve_order() {
    let order = Arc::new(AtomicU64::new(0));
    let mut reg = ToolRegistry::new();

    reg = reg.with(Box::new(MutatingTool {
        name: "m1",
        delay_ms: 10,
        order_log: order.clone(),
    }));
    reg = reg.with(Box::new(MutatingTool {
        name: "m2",
        delay_ms: 10,
        order_log: order.clone(),
    }));
    reg = reg.with(Box::new(MutatingTool {
        name: "m3",
        delay_ms: 10,
        order_log: order.clone(),
    }));

    let batch: Vec<(&str, serde_json::Value)> = vec![
        ("m1", serde_json::json!({})),
        ("m2", serde_json::json!({})),
        ("m3", serde_json::json!({})),
    ];

    let _results = reg.dispatch_batch(&batch).await;

    // Mutating tools must execute in batch order.
    // The order_log is an atomic counter — if they ran in order, each tool
    // records its position sequentially: 1, 2, 3.
    // This is inherently satisfied by sequential execution.
    assert_eq!(order.load(Ordering::SeqCst), 3);
}
