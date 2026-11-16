# Oxide Performance Benchmark Report & Reference Targets

## 1. Executive Performance Targets vs. VS Code

| Benchmark Metric | Oxide Target | VS Code (Electron Reference) | Oxide Architecture Advantage |
| :--- | :--- | :--- | :--- |
| **Cold Startup Time** | **< 250 ms** | 1,200 – 2,800 ms | Native Rust binary, no V8 engine or Chromium shell boot overhead |
| **Warm Startup Time** | **< 80 ms** | 600 – 1,100 ms | Direct kernel page cache mapping |
| **Idle Memory (Single File)** | **< 45 MB** | 350 – 600 MB | Zero Chromium processes; lightweight Ropey B-Tree allocations |
| **Idle Memory (50k Files Workspace)** | **< 90 MB** | 750 – 1,400 MB | Shallow tree indexing; lazy file descriptor loading |
| **Idle CPU Utilization** | **0.0% – 0.1%** | 1.5% – 5.0% | Event-driven render loop (sleeps when no events arrive) |
| **Typing Latency (p99)** | **< 4 ms** | 18 – 45 ms | Synchronous in-memory rope mutation; no IPC hop to webview |
| **100-Cursor Edit Latency** | **< 6 ms** | 45 – 120 ms | In-place batched memory edits with grouped transactions |
| **Regex Search (10,000 Files)** | **< 450 ms** | 2,800 – 6,500 ms | Rayon thread-pool parallelization with `.gitignore` short-circuiting |

---

## 2. Benchmark Suite Guide

Oxide provides Criterion-based microbenchmarks in `benchmarks/`:

```bash
# Run the complete microbenchmark suite
cargo bench -p benchmarks

# Run startup benchmark
cargo run -p app --release -- --bench-startup
```

### Microbenchmark Targets:
1. `rope_insert_10000_chars`: Validates Ropey chunk rebalancing under rapid typing. Target: `< 150 µs`.
2. `syntax_highlight_rust_lines`: Validates line tokenizer throughput. Target: `> 2,000,000 lines/second`.
3. `multicursor_100_edits`: Validates multi-cursor transactional batching. Target: `< 250 µs`.
