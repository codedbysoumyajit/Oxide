use criterion::{black_box, criterion_group, criterion_main, Criterion};
use editor_core::{Cursor, Editor};
use syntax::{Highlighter, Language};
use text_buffer::TextBuffer;

fn bench_rope_insert(c: &mut Criterion) {
    c.bench_function("rope_insert_10000_chars", |b| {
        b.iter(|| {
            let mut buf = TextBuffer::new();
            for i in 0..1000 {
                buf.insert_raw(i, "a");
            }
            black_box(buf.len_chars())
        });
    });
}

fn bench_syntax_highlighting(c: &mut Criterion) {
    let source_lines = vec![
        "pub fn calculate_primes(max: usize) -> Vec<usize> {",
        "    let mut primes = Vec::new();",
        "    for n in 2..=max {",
        "        if is_prime(n) { primes.push(n); }",
        "    }",
        "    primes",
        "}",
    ];

    c.bench_function("syntax_highlight_rust_lines", |b| {
        b.iter(|| {
            for line in &source_lines {
                let tokens = Highlighter::highlight_line(line, Language::Rust);
                black_box(tokens);
            }
        });
    });
}

fn bench_multicursor_typing(c: &mut Criterion) {
    let initial_content = (0..100)
        .map(|i| format!("let var_{} = 0;\n", i))
        .collect::<String>();

    c.bench_function("multicursor_100_edits", |b| {
        b.iter(|| {
            let mut ed = Editor::from_str(&initial_content, None, "rust");
            let cursors: Vec<Cursor> = (0..100).map(|i| Cursor::new(i * 15)).collect();
            ed.selections.set_cursors(cursors);
            ed.insert_text("pub ");
            black_box(ed.buffer.len_chars())
        });
    });
}

criterion_group!(benches, bench_rope_insert, bench_syntax_highlighting, bench_multicursor_typing);
criterion_main!(benches);
