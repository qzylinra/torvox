use criterion::{Criterion, criterion_group, criterion_main};
use torvox_core::grid::Grid;
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

fn bench_grid_sizeof(c: &mut Criterion) {
    c.bench_function("grid_sizeof_24x80_no_scrollback", |b| {
        b.iter(|| {
            let g = Grid::new(24, 80);
            std::hint::black_box(&g);
            std::mem::size_of_val(&g)
        });
    });
    c.bench_function("grid_sizeof_24x80_50k_scrollback", |b| {
        b.iter(|| {
            let g = Grid::with_scrollback(24, 80, 50_000);
            std::hint::black_box(&g);
            std::mem::size_of_val(&g)
        });
    });
}

fn bench_grid_cell_access(c: &mut Criterion) {
    let mut g = Grid::new(24, 80);
    for r in 0..24 {
        g.fill_cells(r, 'A', 0, 80);
    }
    c.bench_function("grid_row_cells_24x80", |b| {
        b.iter(|| {
            let mut count = 0;
            for r in 0..24 {
                if let Some(cells) = g.row_cells(r) {
                    count += cells.len();
                }
            }
            std::hint::black_box(count);
        });
    });
}

criterion_group!(
    benches,
    bench_vt_parse_plain_text,
    bench_vt_parse_sgr_sequences,
    bench_vt_parse_cursor_movement,
    bench_vt_parse_large_output,
    bench_grid_sizeof,
    bench_grid_cell_access,
);

criterion_main!(benches);
