---
metadata:
  complexity: 1
  frequency: common
  confidence: high
  applicable:
    async: true
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

Holding a lock across `.await` can cause deadlocks if the task is suspended and another task tries to acquire the same lock. The async runtime may also suspend the task while holding the lock, preventing other tasks from making progress.

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

- **Prototyping**: Quick experiments where you know the async operation is fast
- **Single-threaded**: When using a single-threaded runtime with no concurrent tasks

## Related

- [`own-mutex-interior`](rules/own-mutex-interior.md) - Use Mutex for interior mutability
- [`own-rwlock-readers`](rules/own-rwlock-readers.md) - Use RwLock when reads dominate writes
- [`async-clone-before-await`](rules/async-clone-before-await.md) - Clone data before await, release locks
