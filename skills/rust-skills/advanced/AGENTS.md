# Rust Skills - Advanced Level

Deep optimization for library authors and expert Rust developers (24+ months experience). Focus on compiler optimization, low-level memory patterns, SIMD, and complex edge cases.

## When to Use

This advanced-level skill set is ideal when:

- Writing libraries used by thousands of developers
- Building performance-critical systems (latency-sensitive)
- Working with unsafe code and FFI
- Complex trait systems and generic bounds
- Deep optimization and profiling work
- Competitive programming or high-frequency trading

---

## Compiler Optimization (Advanced)

- [`opt-inline-always-rare`](rules/opt-inline-always-rare.md) - Use `#[inline(always)]` sparingly
- [`opt-inline-never-cold`](rules/opt-inline-never-cold.md) - Use `#[inline(never)]` for cold paths
- [`opt-lto-release`](rules/opt-lto-release.md) - Enable LTO in release builds
- [`opt-codegen-units`](rules/opt-codegen-units.md) - Use `codegen-units = 1` for max optimization
- [`opt-pgo-profile`](rules/opt-pgo-profile.md) - Use PGO for production builds
- [`opt-target-cpu`](rules/opt-target-cpu-native.md) - Set `target-cpu=native` for local builds
- [`opt-bounds-check`](rules/opt-bounds-check.md) - Use iterators to avoid bounds checks
- [`opt-simd-portable`](rules/opt-simd-portable.md) - Use portable SIMD for data-parallel operations
- [`opt-cache-friendly`](rules/opt-cache-friendly.md) - Design cache-friendly data layouts (SoA)

---

## Memory Optimization (Advanced)

- [`mem-arena-allocator`](rules/mem-arena-allocator.md) - Use arena allocators for batch allocations
- [`mem-zero-copy`](rules/mem-zero-copy.md) - Use zero-copy patterns with slices and `Bytes`
- [`mem-compact-string`](rules/mem-compact-string.md) - Use `CompactString` for small string optimization
- [`mem-smaller-integers`](rules/mem-smaller-integers.md) - Use smallest integer type that fits
- [`mem-box-large-variant`](rules/mem-box-large-variant.md) - Box large enum variants to reduce type size
- [`mem-boxed-slice`](rules/mem-boxed-slice.md) - Use `Box<[T]>` instead of `Vec<T>` when fixed
- [`mem-thinvec`](rules/mem-thinvec.md) - Use `ThinVec` for often-empty vectors
- [`mem-arrayvec`](rules/mem-arrayvec.md) - Use `ArrayVec` for bounded-size collections
- [`mem-assert-type-size`](rules/mem-assert-type-size.md) - Assert hot type sizes to prevent regressions

---

## Advanced Ownership Patterns

- [`own-arc-shared`](rules/own-arc-shared.md) - Use `Arc<T>` for thread-safe shared ownership
- [`own-rc-single-thread`](rules/own-rc-single-thread.md) - Use `Rc<T>` for single-threaded sharing
- [`own-refcell-interior`](rules/own-refcell-interior.md) - Use `RefCell<T>` for interior mutability (single-thread)
- [`own-mutex-interior`](rules/own-mutex-interior.md) - Use `Mutex<T>` for interior mutability (multi-thread)
- [`own-rwlock-readers`](rules/own-rwlock-readers.md) - Use `RwLock<T>` when reads dominate writes
- [`own-lifetime-elision`](rules/own-lifetime-elision.md) - Rely on lifetime elision when possible
- [`own-copy-small`](rules/own-copy-small.md) - Derive `Copy` for small, trivial types

---

## Advanced Error Handling

- [`err-custom-type`](rules/err-custom-type.md) - Create custom error types, not `Box<dyn Error>`
- [`err-from-impl`](rules/err-from-impl.md) - Use `#[from]` for automatic error conversion
- [`err-source-chain`](rules/err-source-chain.md) - Use `#[source]` to chain underlying errors
- [`err-lowercase-msg`](rules/err-lowercase-msg.md) - Error messages: lowercase, no trailing punctuation
- [`err-doc-errors`](rules/err-doc-errors.md) - Document errors with `# Errors` section

---

## Advanced Async Patterns

- [`async-bounded-channel`](rules/async-bounded-channel.md) - Use bounded channels for backpressure
- [`async-mpsc-queue`](rules/async-mpsc-queue.md) - Use `mpsc` for work queues
- [`async-broadcast-pubsub`](rules/async-broadcast-pubsub.md) - Use `broadcast` for pub/sub patterns
- [`async-watch-latest`](rules/async-watch-latest.md) - Use `watch` for latest-value sharing
- [`async-oneshot-response`](rules/async-oneshot-response.md) - Use `oneshot` for request/response
- [`async-join-parallel`](rules/async-join-parallel.md) - Use `tokio::join!` for parallel operations
- [`async-try-join`](rules/async-try-join.md) - Use `tokio::try_join!` for fallible parallel ops
- [`async-select-racing`](rules/async-select-racing.md) - Use `tokio::select!` for racing/timeouts
- [`async-joinset-structured`](rules/async-joinset-structured.md) - Use `JoinSet` for dynamic task groups
- [`async-clone-before-await`](rules/async-clone-before-await.md) - Clone data before await, release locks

---

## Advanced API Design

- [`api-typestate`](rules/api-typestate.md) - Use typestate for compile-time state machines
- [`api-sealed-trait`](rules/api-sealed-trait.md) - Seal traits to prevent external implementations
- [`api-extension-trait`](rules/api-extension-trait.md) - Use extension traits to add methods to foreign types
- [`api-non-exhaustive`](rules/api-non-exhaustive.md) - Use `#[non_exhaustive]` for future-proof enums/structs
- [`api-default-impl`](rules/api-default-impl.md) - Implement `Default` for sensible defaults
- [`api-common-traits`](rules/api-common-traits.md) - Implement `Debug`, `Clone`, `PartialEq` eagerly
- [`api-must-use`](rules/api-must-use.md) - Add `#[must_use]` to `Result` returning functions
- [`api-builder-must-use`](rules/api-builder-must-use.md) - Add `#[must_use]` to builder types

---

## Advanced Type System

- [`type-phantom-marker`](rules/type-phantom-marker.md) - Use `PhantomData<T>` for type-level markers
- [`type-never-diverge`](rules/type-never-diverge.md) - Use `!` type for functions that never return
- [`type-generic-bounds`](rules/type-generic-bounds.md) - Add trait bounds only where needed
- [`type-no-stringly`](rules/type-no-stringly.md) - Avoid stringly-typed APIs, use enums/newtypes
- [`type-repr-transparent`](rules/type-repr-transparent.md) - Use `#[repr(transparent)]` for FFI newtypes

---

## Testing (Advanced)

- [`test-proptest-properties`](rules/test-proptest-properties.md) - Use `proptest` for property-based testing
- [`test-mockall-mocking`](rules/test-mockall-mocking.md) - Use `mockall` for trait mocking
- [`test-criterion-bench`](rules/test-criterion-bench.md) - Use `criterion` for benchmarking
- [`test-tokio-async`](rules/test-tokio-async.md) - Use `#[tokio::test]` for async tests

---

## Anti-Patterns (Reference)

- [`anti-unwrap-abuse`](rules/anti-unwrap-abuse.md) - Don't use `.unwrap()` in production code
- [`anti-expect-lazy`](rules/anti-expect-lazy.md) - Don't use `.expect()` for recoverable errors
- [`anti-clone-excessive`](rules/anti-clone-excessive.md) - Don't clone when borrowing works
- [`anti-lock-across-await`](rules/anti-lock-across-await.md) - Don't hold locks across `.await`
- [`anti-string-for-str`](rules/anti-string-for-str.md) - Don't accept `&String` when `&str` works
- [`anti-vec-for-slice`](rules/anti-vec-for-slice.md) - Don't accept `&Vec<T>` when `&[T]` works
- [`anti-index-over-iter`](rules/anti-index-over-iter.md) - Don't use indexing when iterators work
- [`anti-panic-expected`](rules/anti-panic-expected.md) - Don't panic on expected/recoverable errors
- [`anti-premature-optimize`](rules/anti-premature-optimize.md) - Don't optimize before profiling

---

## Performance Optimization Techniques

### 1. SIMD with Portable Intrinsics

```rust
use std::simd::{Simd, SimdFloat, num::SimdUint};

fn sum_simd(data: &[f32]) -> f32 {
    const LANES: usize = 8;

    let mut sum = Simd::splat(0.0);
    let chunks = data.chunks_exact(LANES);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let vec = Simd::from_slice(chunk);
        sum += vec;
    }

    // Horizontal sum
    let mut result = 0.0;
    for i in 0..LANES {
        result += sum.as_array()[i];
    }

    // Add remainder
    result += remainder.iter().sum();

    result
}
```

### 2. Arena Allocation Pattern

```rust
struct Arena<'a> {
    memory: Vec<u8>,
    marker: std::marker::PhantomData<&'a u8>,
}

impl<'a> Arena<'a> {
    fn new(capacity: usize) -> Self {
        Arena {
            memory: Vec::with_capacity(capacity),
            marker: std::marker::PhantomData,
        }
    }

    fn alloc<T>(&mut self, value: T) -> &'a T {
        unsafe {
            let ptr = self.memory.as_ptr() as *mut T;
            std::ptr::write(ptr, value);
            &*ptr
        }
    }
}
```

### 3. Struct of Arrays (SoA) Layout

```rust
struct ParticlesSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    mass: Vec<f32>,
}

struct ParticlesAoS {
    data: Vec<[f32; 4]>, // [x, y, z, mass]
}

// SoA is better for SIMD: operate on entire arrays
fn apply_gravity_soa(particles: &mut ParticlesSoA) {
    for i in 0..particles.x.len() {
        let m = particles.mass[i];
        particles.x[i] -= particles.x[i] * m * GRAVITY;
        particles.y[i] -= particles.y[i] * m * GRAVITY;
        particles.z[i] -= particles.z[i] * m * GRAVITY;
    }
}
```

---

## Compiler Flags for Maximum Performance

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

# PGO (Profile-Guided Optimization)
[profile.pgo]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1

# Native compilation (CPU-specific optimizations)
[profile.native]
inherits = "release"
rustflags = ["-C", "target-cpu=native"]
```

---

## Zero-Copy Patterns

### 1. Borrow from Bytes

```rust
use bytes::Bytes;

fn parse_packet(data: &Bytes) -> Result<Packet> {
    // Zero-copy parsing
    let header = PacketHeader::parse(&data[0..HEADER_SIZE])?;
    let payload = &data[HEADER_SIZE..];

    Ok(Packet { header, payload })
}
```

### 2. Conditional Ownership with Cow

```rust
use std::borrow::Cow;

fn to_uppercase(input: &str) -> Cow<str> {
    if input.chars().all(|c| c.is_uppercase()) {
        Cow::Borrowed(input) // No allocation
    } else {
        Cow::Owned(input.to_uppercase()) // Allocate
    }
}
```

---

## Typestate Pattern Example

```rust
struct Connection {
    // private fields to enforce state transitions
    stream: Option<std::net::TcpStream>,
}

struct Disconnected;
struct Connected;
struct Authenticated;

struct Connection<State> {
    _state: std::marker::PhantomData<State>,
}

impl Connection<Disconnected> {
    fn connect(addr: &str) -> Result<Connection<Connected>, Error> {
        let stream = TcpStream::connect(addr)?;
        Ok(Connection {
            _state: std::marker::PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn authenticate(self, creds: &str) -> Result<Connection<Authenticated>, Error> {
        // authentication logic
        Ok(Connection {
            _state: std::marker::PhantomData,
        })
    }
}

impl Connection<Authenticated> {
    fn send(&mut self, data: &[u8]) -> Result<(), Error> {
        // send logic
        Ok(())
    }
}
```

---

## FFI and Unsafe Guidelines

- ✅ Use `unsafe` only when absolutely necessary
- ✅ Document safety invariants thoroughly
- ✅ Use `#[repr(C)]` for FFI structs
- ✅ Check pointer validity before dereferencing
- ✅ Use `PhantomData` for lifetime tracking
- ❌ Never expose `unsafe` in public API

---

## Learning Path

1. **Master compiler optimization** - Inlining, LTO, PGO, SIMD
2. **Learn zero-copy patterns** - Cow, Bytes, arena allocators
3. **Understand type system** - PhantomData, GATs, associated types
4. **Practice benchmarking** - Criterion, flamegraphs, profiling

---

## Quick Reference

| Category              | Rule Count | Focus                |
| --------------------- | ---------- | -------------------- |
| Compiler Optimization | 9          | Performance tuning   |
| Memory Optimization   | 9          | Allocation patterns  |
| Advanced Ownership    | 7          | Complex sharing      |
| Advanced Async        | 10         | Concurrency patterns |
| Advanced API Design   | 8          | Type safety          |
| Advanced Type System  | 5          | Trait bounds         |
| Advanced Testing      | 4          | Benchmarking         |
| Anti-Patterns         | 10         | What to avoid        |

---

**Total Rules:** 60+ advanced rules
**Complexity Range:** 3-5
**Focus:** Performance, low-level, optimization

---

**See Also:**

- [Beginner Rules](../beginner/AGENTS.md) - Foundation concepts
- [Intermediate Rules](../intermediate/AGENTS.md) - Daily use patterns
- [Full Reference](../AGENTS.md) - All 179 rules

---

**Version:** 1.0.0
**Last Updated:** 2025-02-08
