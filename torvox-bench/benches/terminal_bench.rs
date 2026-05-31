use criterion::{Criterion, criterion_group, criterion_main};
use torvox_terminal::ghostty_terminal::GhosttyTerminal;

fn bench_vt_parse_plain_text(c: &mut Criterion) {
    let input = b"Hello, World! This is a terminal benchmark.\n";
    c.bench_function("vt_parse_plain_text", |b| {
        let mut terminal = GhosttyTerminal::new(24, 80, 1000).unwrap();
        b.iter(|| {
            terminal.vt_write(input);
        });
    });
}

fn bench_vt_parse_sgr_sequences(c: &mut Criterion) {
    let input = b"\x1b[1mBold\x1b[0m \x1b[31mRed\x1b[0m \x1b[1;32mGreenBold\x1b[0m\n";
    c.bench_function("vt_parse_sgr_sequences", |b| {
        let mut terminal = GhosttyTerminal::new(24, 80, 1000).unwrap();
        b.iter(|| {
            terminal.vt_write(input);
        });
    });
}

fn bench_vt_parse_cursor_movement(c: &mut Criterion) {
    let input = b"\x1b[2A\x1b[3B\x1b[4C\x1b[5D\x1b[10;20H";
    c.bench_function("vt_parse_cursor_movement", |b| {
        let mut terminal = GhosttyTerminal::new(24, 80, 1000).unwrap();
        b.iter(|| {
            terminal.vt_write(input);
        });
    });
}

fn bench_vt_parse_large_output(c: &mut Criterion) {
    let line =
        b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n";
    let mut input = Vec::with_capacity(line.len() * 1000);
    for _ in 0..1000 {
        input.extend_from_slice(line);
    }
    c.bench_function("vt_parse_large_output_1k_lines", |b| {
        let mut terminal = GhosttyTerminal::new(24, 80, 1000).unwrap();
        b.iter(|| {
            terminal.vt_write(&input);
        });
    });
}

criterion_group!(
    benches,
    bench_vt_parse_plain_text,
    bench_vt_parse_sgr_sequences,
    bench_vt_parse_cursor_movement,
    bench_vt_parse_large_output,
);
criterion_main!(benches);
