# Oxide: Next-Generation High-Performance Code Editor in Rust

Oxide is a high-performance, modern development environment engineered from the ground up in Rust. It delivers the ergonomic polish and rich feature set of Visual Studio Code while eliminating Electron overhead to provide:

- **Near-Instant Cold Startup**: Under 250 ms cold boot to an editable buffer.
- **Ultra-Low Memory Footprint**: Under 45 MB idle RAM (vs 400+ MB for Electron editors).
- **Zero-Lag Typing & Editing**: B-Tree Rope backed text buffer with sub-millisecond insert/delete latency.
- **Rayon Multi-Threaded Search**: Search across 50,000+ files in fractions of a second.
- **Incremental Syntax Highlighting**: Fast, multi-language tokenization with dark & light themes.
- **100% VS Code Compatible Shortcuts**: Command palette (`Ctrl+Shift+P`), multi-cursor (`Ctrl+Alt+Down`), fuzzy find (`Ctrl+P`), and settings importer.
- **Integrated PTY Terminal**: Full interactive terminal shell running directly in the editor.
- **Git Integration & Gutter Diffs**: Branch detection, status, staged files, and real-time gutter markers (`+`, `~`, `-`).

---

## Architecture Overview

Oxide is organized as a modular Rust workspace:

```
crates/
├── app/               # Application CLI, startup benchmarking & main event loop
├── ui/                # Hardware-accelerated virtualized editor rendering & docking
├── editor-core/       # Multi-cursor, selections, edit operations, folding, indent
├── text-buffer/       # Ropey-based text buffer, metrics & undo/redo transaction history
├── syntax/            # High-performance incremental syntax tokenizer & themes
├── filesystem/        # Async file I/O, line ending normalization & atomic saving
├── workspace/         # Project directory tree, open folders & documents
├── search/            # Rayon parallel project search engine (.gitignore aware)
├── git/               # Git status, zero-overhead branch detection & diff viewer
├── language-server/   # LSP client with JSON-RPC stdio protocol & diagnostics
├── debugger/          # DAP client for breakpoints, call stacks & variable inspection
├── terminal/          # Cross-platform PTY integration & virtual ANSI screen
├── extensions/        # Sandboxed plugin manifests & capability permission system
├── settings/          # Configuration, theme loader & VS Code settings importer
└── commands/          # Command palette registry & hotkey dispatcher
```

---

## Getting Started

### Prerequisites
- Rust 1.80+ (MSRV)
- Linux (X11 / Wayland), macOS, or Windows

### Building & Running

```bash
# Build optimized release binary
cargo build --release

# Run Oxide
cargo run --release

# Open a specific file or folder
cargo run --release -- /path/to/project

# Run cold startup benchmark
cargo run --release -p app -- --bench-startup
```

---

## Documentation

- [Architecture Specification](docs/ARCHITECTURE.md)
- [Performance Benchmarks](docs/PERFORMANCE.md)
- [Keyboard Shortcuts Reference](docs/KEYBINDINGS.md)

---

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
