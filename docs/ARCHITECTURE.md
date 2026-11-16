# Oxide Architecture & Systems Engineering Guide

## Overview

**Oxide** is a next-generation code editor engineered from the ground up in Rust to provide a high-performance alternative to Visual Studio Code. Its primary architectural directive is sub-millisecond responsiveness, ultralow idle resource consumption, instant startup, and effortless scaling to million-line repositories.

---

## 1. Modular Subsystem Design

Oxide is decomposed into 15 focused, decoupled crates with zero circular dependencies:

```
crates/
├── text-buffer/         # B-Tree Rope text representation, UTF-8/UTF-16 metrics, transaction log
├── editor-core/         # Multi-cursor, selections, edit operations, folding, indentation
├── filesystem/          # Async file I/O, line ending normalization, atomic file writes
├── syntax/              # Fast lexing, incremental tokenization, syntax color mapping
├── commands/            # Command palette registry, keybindings parser & dispatcher
├── settings/            # Strongly-typed configuration, theme engine, VS Code importer
├── workspace/           # Project directory tree, open documents, active tabs
├── search/              # Multi-threaded Rayon search across workspace files (.gitignore aware)
├── git/                 # Git status, zero-overhead branch detection, gutter diffs
├── language-server/     # LSP client, async JSON-RPC over stdio, diagnostics store
├── debugger/            # DAP client, breakpoint management, stack frames, variables
├── terminal/            # Cross-platform PTY (`portable-pty`), virtual ANSI screen (`vte`)
├── extensions/          # Sandboxed plugin manifests, capability-based security model
├── ui/                  # Native GPU/windowing layer (`egui`/`eframe`), line virtualization
└── app/                 # CLI entrypoint, argument parsing (`clap`), application lifecycle
```

---

## 2. Text Buffer & Rope Model

The text buffer (`crates/text-buffer`) is built on **Ropey**, an immutable-snapshotting B-Tree rope data structure:
- **Time Complexity**: Insertions and deletions take $O(\log N)$ time, preventing GUI thread pauses even in multi-gigabyte files.
- **Transactional History**: Edits are stored as invertible transactions (`Transaction` containing `EditOp::Insert` and `EditOp::Delete`). This enables grouped multi-cursor undo/redo and restores cursor positions accurately.
- **Dual Indexing**: Converts between char offsets, UTF-8 byte positions, lines/columns, and UTF-16 code units (required by the Language Server Protocol).

---

## 3. UI Virtualization Pipeline

The rendering engine (`crates/ui`) implements an immediate-retained hybrid virtualization pipeline:
- **Viewport Virtualization**: Computes `first_visible_line` and `visible_line_count` directly from viewport height and scroll offset. Only visible lines are tokenized, measured, and painted.
- **Zero-Lag Input**: Keystrokes modify the in-memory Rope buffer synchronously on the main thread, while background tasks (search, git diff, LSP diagnostics) operate on immutable snapshots without holding mutex locks on the UI loop.
- **Hardware Acceleration**: Built on `eframe` / `wgpu` with native Wayland and X11 backends on Linux, Metal on macOS, and DirectX 12 / Vulkan on Windows.
