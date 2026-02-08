---
metadata:
  complexity: 1
  frequency: common
  confidence: high
  applicable:
    is_async: true
    embedded: false
    wasm: false
  category: critical
  level: intermediate
  rust_version: "1.0+"
  alternatives:
    - Clone data before await if necessary
    - Use RwLock if you need concurrent reads
  related_rules:
    - own-mutex-interior
    - own-rwlock-readers
    - async-join-parallel
---

# No lock across await

Never hold a Mutex or RwLock across `.await` points.

## Why

Holding a lock across `.await` can cause deadlocks if task is suspended and another task tries to acquire the same lock. The async runtime may also suspend the task while holding the lock, preventing other tasks from making progress.

## Examples

### ❌ Don't

```rust
async fn process(state: &Mutex<State>) -> Result<()> {
    let mut guard = state.lock().await;

    let result = expensive_async_operation().await;

    guard.update(result);
    Ok(())
}
```

### ✅ Do

```rust
async fn process(state: &Mutex<State>) -> Result<()> {
    let result = {
        let guard = state.lock().await;
        guard.clone_data()
    };

    let processed = expensive_async_operation(result).await?;

    let mut guard = state.lock().await;
    guard.update(processed);
    Ok(())
}
```

## When to Break This Rule

Use locks across `.await` when:

- **Prototyping**: Quick experiments where you know the async operation is fast
- **Single-threaded runtime**: When using `current_thread` runtime with no concurrent tasks
- **Guaranteed fast operations**: When async operation completes in <1μs (measure with `Instant::elapsed()`)
- **Debugging**: Temporarily for troubleshooting specific async behavior
- **Mutex re-entrancy**: When you need to acquire the same mutex multiple times in same call stack

## Cost Analysis

| Scenario                     | Deadlock Risk | Performance Impact | Recommendation                   |
| ---------------------------- | ------------- | ------------------ | -------------------------------- |
| Production with concurrency  | High          | High               | Never hold across await          |
| Single-threaded prototyping  | None          | Low                | Acceptable for quick experiments |
| Microsecond-scale operations | Low           | Low                | Acceptable if verified fast      |
| Mutex re-entrancy required   | N/A           | High               | Use re-entrant mutex             |

### Performance Impact

Holding a lock across `.await`:

- **Blocks other tasks**: Runtime can't schedule other work while lock held
- **Prevents async scheduling**: Task can't yield, defeating async benefits
- **Deadlock potential**: If another task waits for same lock
- **Priority inversion**: High-priority tasks blocked by low-priority lock holder

## Real-World Examples

### Acceptable Lock Across Await

```rust
// Single-threaded prototyping
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(Mutex::new(0));

    // In current_thread, no other tasks can run
    // So lock across await is safe
    let _guard = state.lock().await;
    let result = compute_something().await;
    state.lock().await.update(result);
}
```

### Unacceptable Lock Across Await

```rust
// Production with multi-threaded runtime
async fn process_concurrently(state: Arc<Mutex<State>>) -> Vec<()> {
    let mut tasks = Vec::new();

    for i in 0..10 {
        let state = state.clone();
        tasks.push(tokio::spawn(async move {
            // ❌ Never do this: lock across await in multi-threaded runtime
            let _guard = state.lock().await;
            let result = do_work(i).await;
            process_result(result);
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}

// ✅ Correct: minimal lock duration
async fn process_correctly(state: Arc<Mutex<State>>) -> Vec<()> {
    let mut tasks = Vec::new();

    for i in 0..10 {
        let state = state.clone();
        tasks.push(tokio::spawn(async move {
            let data = {
                let guard = state.lock().await;
                guard.clone_data()
            };

            let result = do_work(i).await;
            process_result(result);
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }
}
```

### Using RwLock for Read-Dominated Workloads

```rust
// When many concurrent readers and infrequent writes
async fn read_heavy(state: Arc<RwLock<Data>>) {
    // ✅ RwLock allows concurrent reads
    let guard = state.read().await;
    // Multiple tasks can hold read guard simultaneously
    process(&*guard);
}
```

## Debugging Lock Issues

If you suspect lock contention or deadlocks:

```rust
// Enable Tokio's lock instrumentation
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

// Use tracing instrumenting
#[instrument(skip_all)]
async fn instrumented_fn(state: Arc<Mutex<Data>>) {
    let guard = state.lock().await;
    do_work(&guard);
}
```

## Related Rules

- [`own-mutex-interior`](rules/own-mutex-interior.md) - Use Mutex for interior mutability
- [`own-rwlock-readers`](rules/own-rwlock-readers.md) - Use RwLock when reads dominate writes
- [`async-join-parallel`](rules/async-join-parallel.md) - Use tokio::join! for parallel operations
- [`async-clone-before-await`](rules/async-clone-before-await.md) - Clone data before await, release locks

## References

- [Tokio Deadlocks Guide](https://tokio.rs/tokio/topics/futures/index.html#deadlock-detection)
- [Rust Async Book](https://rust-lang.github.io/async-book/01_getting_started/04_chapter_03_shared_state.html)
- [`parking_lot`](https://github.com/Amanieu/parking_lot) - Alternative with deadlock detection
