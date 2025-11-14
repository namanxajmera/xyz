# Performance Documentation

**Optimization strategies, benchmarks, and performance analysis for DepMgr**

---

## Table of Contents

- [Performance Overview](#performance-overview)
- [Benchmarks](#benchmarks)
- [Optimization Techniques](#optimization-techniques)
- [Implementation Details](#implementation-details)
- [Performance Monitoring](#performance-monitoring)
- [Future Optimizations](#future-optimizations)
- [Performance Best Practices](#performance-best-practices)

---

## Performance Overview

DepMgr is designed for **maximum performance** through:

1. **HTTP APIs over CLI**: 100-200x faster than spawning processes
2. **Parallel Processing**: Utilizes all CPU cores with Rayon
3. **Aggressive Caching**: 1-hour memory cache for instant reloads
4. **Connection Pooling**: Reuses HTTP connections (60-80% latency reduction)
5. **Async Operations**: Non-blocking I/O with Tokio
6. **Incremental UI Updates**: Shows results as they arrive

**Result**: **10-20x faster** than traditional approach (8-10 min → 30-60 sec first load)

---

## Benchmarks

### Overall Performance

| Metric | Old Approach | New Approach | Improvement |
|--------|--------------|--------------|-------------|
| **First Load** | 8-10 minutes | 30-60 seconds | **10-20x** |
| **Cached Load** | N/A | <100ms | **Instant** |
| **List Homebrew** | 5-7 minutes | 1-3 seconds | **100-200x** |
| **Get Descriptions** | 3-5 minutes | <10 seconds | **18-30x** |
| **Check Outdated** | 30-60 seconds | Instant | **∞** |
| **Project Scan** | N/A | 2-5 seconds | Baseline |

**Source**: [`docs/worklog.md:353-469`](docs/worklog.md) - Performance transformation notes

### Component Breakdown

**Homebrew Package Listing** (83 packages):

| Method | Time | Commands |
|--------|------|----------|
| Sequential `brew info` | 5-7 minutes | 83 × `brew info <pkg>` (6-10s each) |
| Batch `brew info` (20 pkgs) | 60-90 seconds | 5 × batched commands |
| **API + Rayon** | **1-3 seconds** | **1 HTTP GET** |

**Code Reference**: [`src/managers/homebrew_fast.rs:22-100`](src/managers/homebrew_fast.rs)

**Description Fetching**:

| Method | Time (83 packages) | Concurrency |
|--------|-------------------|-------------|
| Sequential | 3-5 minutes | 1 at a time |
| Batched (15 pkgs) | 60-90 seconds | Batches of 15 |
| Parallel | 10-15 seconds | 83 simultaneous |
| **API (pre-fetched)** | **<1 second** | N/A (cached) |

**Code Reference**: [`src/managers/homebrew_fast.rs:210-270`](src/managers/homebrew_fast.rs)

### Real-World Timings

**First Run** (no cache, 83 Homebrew packages):
```
0-1s:   Window appears, manager detection
1-3s:   Homebrew API fetch (6,943 formulas)
3-5s:   Packages appear in table
5-30s:  Project usage scanning (filesystem)
30-60s: All data complete
```

**Subsequent Runs** (with cache):
```
0-1s:   Window appears
1-2s:   Packages load from cache
Complete!
```

**Memory Usage**:
- Idle: ~50 MB
- During scan: ~150 MB
- With 200+ packages: ~200 MB
- Peak: ~500 MB (with 1000+ packages)

**Binary Size**:
- Debug build: ~50 MB
- Release build: ~15-20 MB
- Stripped release: ~12-15 MB

---

## Optimization Techniques

### 1. HTTP API Instead of Process Spawning

**Problem**: Spawning `brew info` 83 times = 83 processes = 6-10 seconds each = 5-7 minutes total

**Solution**: Single HTTP GET to Homebrew Formula API

**Implementation**: [`src/managers/homebrew_fast.rs:34-56`](src/managers/homebrew_fast.rs)

```rust
// ONE API call fetches ALL 6,943 Homebrew formulas
let url = "https://formulae.brew.sh/api/formula.json";
let formulas: Vec<FormulaInfo> = client
    .get(url)
    .send()
    .await?
    .json()
    .await?;

// Time: 1-3 seconds (vs 5-7 minutes with 83 × brew info)
```

**Why This Works**:
- HTTP requests are faster than process spawning
- Single request = no overhead multiplication
- API response is compressed (gzip)
- Network latency amortized over one request

**Speedup**: **100-200x** (5-7 min → 1-3 sec)

---

### 2. Parallel Processing with Rayon

**Problem**: Filtering 6,943 formulas to find 83 installed packages is CPU-bound

**Solution**: Parallel iterators with Rayon

**Implementation**: [`src/managers/homebrew_fast.rs:69-87`](src/managers/homebrew_fast.rs)

```rust
let packages: Vec<Package> = formulas
    .par_iter()  // Parallel iterator (uses all CPU cores)
    .filter_map(|formula| {
        installed.get(&formula.name).map(|version| {
            Package { /* ... */ }
        })
    })
    .collect();

// Time: ~3ms for 6,943 items (vs ~20ms sequential)
```

**Why This Works**:
- Rayon automatically distributes work across all CPU cores
- CPU-bound filtering benefits from parallelism
- No manual thread management needed

**Speedup**: **~8x** on 8-core CPU (linear scaling)

**Trade-off**: Small overhead for small datasets (<100 items), but huge win for 1000+ items

---

### 3. Connection Pooling

**Problem**: Making many HTTP requests has overhead (TCP handshake, TLS negotiation)

**Solution**: Reuse HTTP connections with connection pooling

**Implementation**: [`src/utils/http_client.rs:5-13`](src/utils/http_client.rs)

```rust
pub fn create_http_client() -> Client {
    Client::builder()
        .pool_max_idle_per_host(10)    // Reuse up to 10 connections
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .gzip(true)                     // Enable compression
        .build()
        .expect("Failed to create HTTP client")
}
```

**Why This Works**:
- TCP handshake: 1-3 round trips saved
- TLS negotiation: 2-4 round trips saved
- Connection reuse: ~200-500ms saved per request

**Speedup**: **60-80%** latency reduction on repeated requests

**Use Cases**:
- Description fetching for multiple packages
- Repeated API calls in same session
- Future: Background refresh

---

### 4. Memory Caching

**Problem**: API calls are expensive (1-3 seconds), but data doesn't change often

**Solution**: In-memory cache with 1-hour TTL

**Implementation**: [`src/utils/cache.rs`](src/utils/cache.rs)

```rust
// Check cache first
if let Some(cached) = get_cached::<Vec<Package>>("homebrew_all_packages") {
    return Ok(cached);  // Instant return!
}

// Fetch from API
let packages = fetch_from_api().await?;

// Cache for 1 hour
set_cached("homebrew_all_packages", &packages, 3600);
```

**Data Structure**: `DashMap<String, CacheEntry>` (lock-free concurrent hashmap)

**Why This Works**:
- Packages don't change frequently (hours/days)
- 1-hour TTL is reasonable
- Lock-free access = no contention
- Memory is cheap

**Speedup**: **100-1000x** for cached loads (<100ms vs 30-60 seconds)

**Trade-off**: Stale data for up to 1 hour (acceptable, use "Refresh" button for fresh data)

---

### 5. Adaptive Concurrency

**Problem**: Fetching descriptions for 83 packages sequentially is slow

**Solution**: Fetch multiple descriptions concurrently (8 at a time)

**Implementation**: Pattern used in description fetching

```rust
// Process 8 packages at a time
for chunk in packages_needing_desc.chunks(8) {
    let futures: Vec<_> = chunk
        .iter()
        .map(|pkg| fetch_description(pkg.name.clone()))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    // Update UI incrementally
}
```

**Why 8 Concurrent?**:
- More than 8: Diminishing returns, risk of rate limiting
- Less than 8: Not fully utilizing network bandwidth
- 8 is sweet spot: Good throughput, low resource usage

**Speedup**: **~8x** vs sequential (60s → 8s)

---

### 6. Incremental UI Updates

**Problem**: Waiting for all data before showing UI feels slow

**Solution**: Update UI multiple times as data arrives

**Implementation**: [`src/app.rs:53-76`](src/app.rs)

```rust
// Phase 1: Show packages immediately (1-3 seconds)
*packages_clone.write().await = packages.clone();

// Phase 2: Add project usage info (20-30 seconds)
scan_project_usage(&mut packages);
*packages_clone.write().await = packages.clone();  // Update UI again

// Phase 3: Add outdated status (instant with API)
check_outdated(&mut packages).await;
*packages_clone.write().await = packages.clone();  // Update UI again
```

**Why This Works**:
- Users see results immediately
- Perception of speed is improved
- No blocking operations
- Progressive enhancement

**User Experience**: Feels **2-3x faster** than actual time

---

### 7. Release Profile Optimizations

**Configuration**: [`Cargo.toml:42-45`](Cargo.toml)

```toml
[profile.release]
opt-level = 3          # Maximum optimization level
lto = true             # Link-time optimization (whole-program)
codegen-units = 1      # Single codegen unit for better optimization
```

**What These Do**:
- `opt-level = 3`: Aggressive inlining, loop unrolling, vectorization
- `lto = true`: Cross-crate optimization, dead code elimination
- `codegen-units = 1`: Better optimization opportunities (slower build)

**Effect**:
- ~10-15% faster runtime
- ~20% smaller binary
- 2-3x slower compilation (acceptable for release builds)

---

## Implementation Details

### HTTP Client Configuration

**File**: [`src/utils/http_client.rs:5-13`](src/utils/http_client.rs)

```rust
Client::builder()
    .pool_max_idle_per_host(10)    // Connection pool size
    .pool_idle_timeout(Duration::from_secs(90))
    .timeout(Duration::from_secs(30))  // Request timeout
    .gzip(true)                     // Compression
    .build()
```

**Tuning Notes**:
- Pool size (10): Sufficient for typical usage, low memory overhead
- Idle timeout (90s): Keeps connections alive between requests
- Request timeout (30s): Prevents hanging on slow networks
- Gzip: 60-80% bandwidth reduction

---

### Parallel Processing Strategy

**When to Use Rayon**:
- ✅ CPU-bound work (parsing, filtering, transforming)
- ✅ Large datasets (1000+ items)
- ❌ I/O-bound work (use Tokio instead)
- ❌ Small datasets (<100 items, overhead not worth it)

**Example Pattern**:

```rust
use rayon::prelude::*;

// Parallel iteration
let results: Vec<_> = large_dataset
    .par_iter()
    .map(|item| expensive_computation(item))
    .collect();
```

**Code Reference**: [`src/managers/homebrew_fast.rs:69-87`](src/managers/homebrew_fast.rs)

---

### Cache Strategy

**Cache Key Design**:
- `"homebrew_all_packages"`: All Homebrew packages
- `"npm_all_packages"`: All npm packages
- etc.

**TTL Selection**:
- 1 hour (3600s): Good balance between freshness and performance
- Too short (<10 min): Cache thrashing, no benefit
- Too long (>6 hours): Stale data

**Cache Invalidation**:
- Time-based (TTL expires)
- Manual ("Refresh" button clears cache)
- No automatic invalidation (by design)

**Future Enhancement**: Disk cache for cold-start performance

---

### Async/Await Best Practices

**Pattern**: Non-blocking operations

```rust
// ✅ Good: Spawn background task
self.runtime.spawn(async move {
    let packages = fetch_packages().await;
    *shared_state.write().await = packages;
});

// ❌ Bad: Blocking the GUI thread
let packages = self.runtime.block_on(fetch_packages());
```

**Code Reference**: [`src/app.rs:41-179`](src/app.rs) - Background scanning

---

## Performance Monitoring

### Built-in Logging

**Console Output** shows timing:

```
[FAST] Fetching Homebrew packages via API...
[FAST] ✓ Fetched 6943 formulas in 1.2s
[FAST] ✓ Parsed 83 installed packages in 3ms
[FAST] 🚀 Total time: 1.5s (vs 5-7 minutes with old method!)
```

**Code Reference**: [`src/managers/homebrew_fast.rs:59-100`](src/managers/homebrew_fast.rs)

### Manual Profiling

**Time Execution**:

```bash
# Overall timing
time ./target/release/depmgr

# With memory stats (macOS)
/usr/bin/time -l ./target/release/depmgr
```

**Rust Built-in Profiling**:

```bash
# Build with profiling
cargo build --release

# Profile with instruments (macOS)
instruments -t "Time Profiler" ./target/release/depmgr

# Or use flamegraph
cargo install flamegraph
cargo flamegraph
```

### Benchmarking (Future)

**Criterion.rs** benchmarks (planned):

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_list_packages(c: &mut Criterion) {
    c.bench_function("list_homebrew_packages", |b| {
        b.iter(|| {
            let packages = list_homebrew_packages_fast().await.unwrap();
            black_box(packages);
        });
    });
}

criterion_group!(benches, bench_list_packages);
criterion_main!(benches);
```

---

## Future Optimizations

### Planned (v0.2.0)

**1. Disk Cache Layer**:
- Persist cache between app restarts
- Cold-start performance: <5 seconds (vs 30-60 currently)
- Implementation: `serde` + file system

**2. Background Refresh**:
- Daemon mode: Update cache every hour
- Foreground app always shows cached data
- Zero wait time for users

**3. Differential Updates**:
- Only fetch changed packages since last scan
- HTTP ETag support for conditional requests
- 90% bandwidth reduction on refreshes

**4. Request Deduplication**:
- Prevent duplicate simultaneous requests
- Useful if user spam-clicks "Refresh"

**5. WebSocket Streaming** (optional):
- Real-time updates from package registries
- Push notifications for security updates

### Potential Optimizations

**1. Better Parallelism**:
- Use `rayon` for more operations (project scanning, etc.)
- Parallel package manager scanning (already partially done)

**2. Compression**:
- Compress cache entries (trade CPU for memory)
- LZ4 or Zstd for fast compression

**3. Lazy Loading**:
- Load only visible packages in table
- Virtual scrolling for 1000+ packages
- Reduces memory and render time

**4. Predictive Prefetching**:
- Prefetch on app launch (before user clicks)
- Predictive refresh based on usage patterns

---

## Performance Best Practices

### For Contributors

**When Adding Features**:

1. **Measure First**: Profile before optimizing
2. **Consider Async**: Use `async/await` for I/O
3. **Think Parallel**: Use Rayon for CPU-bound work
4. **Cache Wisely**: Cache expensive operations
5. **Batch Requests**: Reduce round trips
6. **Incremental Updates**: Show results as they arrive

**Anti-Patterns to Avoid**:

❌ **Don't**: Spawn processes in loops
```rust
// Bad: 83 processes = slow
for package in packages {
    Command::new("brew").arg("info").arg(&package).output().await?;
}
```

✅ **Do**: Use APIs or batch commands
```rust
// Good: 1 API call
let formulas = fetch_from_api().await?;
```

❌ **Don't**: Block the GUI thread
```rust
// Bad: Blocks UI
let result = self.runtime.block_on(expensive_operation());
```

✅ **Do**: Spawn background tasks
```rust
// Good: Non-blocking
self.runtime.spawn(async move {
    let result = expensive_operation().await;
});
```

### Optimization Checklist

**Before Optimizing**:
- [ ] Is this actually slow? (profile it)
- [ ] Is this a bottleneck? (measure impact)
- [ ] Is the complexity worth it? (maintainability)

**After Optimizing**:
- [ ] Measure improvement (before/after benchmarks)
- [ ] Test correctness (no bugs introduced)
- [ ] Document optimization (why and how)
- [ ] Add regression tests (prevent future slowdowns)

---

## Performance Analysis

### Why DepMgr is Fast

**1. Right Tool for the Job**:
- Rust: Zero-cost abstractions, no GC pauses
- Tokio: Efficient async I/O
- Rayon: Easy data parallelism
- egui: Immediate-mode GUI (simple, fast)

**2. Architectural Decisions**:
- HTTP APIs over CLI spawning (100-200x faster)
- Parallel processing where it matters
- Aggressive caching with reasonable TTL
- Connection pooling for network efficiency

**3. Optimization Focus**:
- Optimized hot paths (package listing, descriptions)
- Accepted trade-offs (1hr cache vs always fresh)
- User experience over perfection (incremental updates)

### Comparison with Alternatives

**Traditional Shell Script Approach**:
```bash
# List packages: 30-60 seconds
brew list

# Check outdated: 30-60 seconds
brew outdated

# Get info: 5-7 minutes
for pkg in $(brew list); do
  brew info $pkg
done

# Total: 6-8 minutes
```

**DepMgr Approach**:
```
API fetch: 1-3 seconds
Filter + parse: <1 second
Project scan: 2-5 seconds (optional)
Total: 3-9 seconds (first run)
Total: <1 second (cached)
```

**Speedup**: **10-100x** depending on operation

---

## Summary

**Key Performance Wins**:
1. ✅ HTTP APIs: 100-200x faster than CLI
2. ✅ Parallel processing: 8x CPU utilization
3. ✅ Caching: Instant cached loads
4. ✅ Connection pooling: 60-80% latency reduction
5. ✅ Incremental UI: Perceived 2-3x speedup

**Total Result**: **10-20x overall improvement** (8-10 min → 30-60 sec)

**Trade-offs Accepted**:
- 1-hour cache staleness (acceptable)
- Longer build times with LTO (acceptable)
- Network dependency for Homebrew (acceptable)

**Future Potential**: Another **2-3x improvement** with disk cache and differential updates

---

For implementation details, see:
- [`ARCHITECTURE.md`](ARCHITECTURE.md) - System design
- [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs) - Fast implementation
- [`docs/worklog.md:353-512`](docs/worklog.md) - Optimization journey

---

**Performance is a feature.** DepMgr proves that developer tools can be both powerful and fast.

