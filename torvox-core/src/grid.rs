use alloc::collections::VecDeque;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::cell::{Cell, DirtyMask};
use crate::line::Line;

/// 终端网格的只读快照，用于渲染。
/// 由 Grid 实现，使渲染器不依赖 terminal crate。
pub trait GridSnapshot {
    fn rows(&self) -> u32;
    fn cols(&self) -> u32;
    fn get(&self, row: u32) -> Option<&Line>;
    fn cell(&self, row: u32, col: u32) -> Option<&Cell>;
    fn dirty(&self) -> &DirtyMask;
}

impl GridSnapshot for Grid {
    fn rows(&self) -> u32 {
        self.rows
    }
    fn cols(&self) -> u32 {
        self.cols
    }
    fn get(&self, row: u32) -> Option<&Line> {
        self.lines.get(row as usize)
    }
    fn cell(&self, row: u32, col: u32) -> Option<&Cell> {
        self.lines.get(row as usize).and_then(|line| line.get(col))
    }
    fn dirty(&self) -> &DirtyMask {
        &self.dirty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Grid {
    lines: Vec<Line>,
    dirty: DirtyMask,
    rows: u32,
    cols: u32,
    scrollback: VecDeque<Line>,
    max_scrollback: usize,
}

impl Grid {
    pub fn new(rows: u32, cols: u32) -> Self {
        let lines = (0..rows).map(|_| Line::new(cols)).collect();
        let mut dirty = DirtyMask::new(rows);
        dirty.mark_all(rows);
        Self {
            lines,
            dirty,
            rows,
            cols,
            scrollback: VecDeque::new(),
            max_scrollback: 50_000,
        }
    }

    pub fn with_scrollback(rows: u32, cols: u32, max_scrollback: usize) -> Self {
        let lines = (0..rows).map(|_| Line::new(cols)).collect();
        let mut dirty = DirtyMask::new(rows);
        dirty.mark_all(rows);
        Self {
            lines,
            dirty,
            rows,
            cols,
            scrollback: VecDeque::new(),
            max_scrollback,
        }
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn get(&self, row: u32) -> Option<&Line> {
        self.lines.get(row as usize)
    }

    /// 返回 `row` 的单元格切片（零分配、零间接寻址）。
    /// 比 `get(row).map(|l| l.cells())` 更快，适用于渲染热路径。
    pub fn row_cells(&self, row: u32) -> Option<&[Cell]> {
        self.lines.get(row as usize).map(|l| l.cells())
    }

    pub fn get_mut(&mut self, row: u32) -> Option<&mut Line> {
        let line = self.lines.get_mut(row as usize)?;
        self.dirty.mark(row);
        Some(line)
    }

    pub fn dirty(&self) -> &DirtyMask {
        &self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty.clear();
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty.mark_all(self.rows);
    }

    pub fn resize(&mut self, new_rows: u32, new_cols: u32) {
        self.lines.resize(new_rows as usize, Line::new(new_cols));
        for line in &mut self.lines {
            line.resize(new_cols);
        }
        self.dirty.resize(new_rows);
        self.dirty.mark_all(new_rows);
        self.rows = new_rows;
        self.cols = new_cols;
    }

    pub fn cell_mut(&mut self, row: u32, col: u32) -> Option<&mut crate::cell::Cell> {
        let result = self
            .lines
            .get_mut(row as usize)
            .and_then(|line| line.get_mut(col));
        if result.is_some() {
            self.dirty.mark(row);
        }
        result
    }

    pub fn cell(&self, row: u32, col: u32) -> Option<&crate::cell::Cell> {
        self.lines.get(row as usize).and_then(|line| line.get(col))
    }

    pub fn mark_row_dirty(&mut self, row: u32) {
        self.dirty.mark(row);
    }

    pub fn mark_rows_dirty(&mut self, start: u32, end: u32) {
        for row in start..end {
            self.dirty.mark(row);
        }
    }

    pub fn scroll_up(&mut self, top: u32, bottom: u32, cols: u32) {
        if top >= bottom || bottom > self.rows {
            return;
        }
        let region_size = bottom - top;
        if region_size <= 1 {
            return;
        }
        let t = top as usize;
        let b = bottom as usize;
        if top == 0 {
            let removed = self.lines.remove(t);
            self.scrollback.push_back(removed);
            while self.scrollback.len() > self.max_scrollback {
                self.scrollback.pop_front();
            }
            self.lines.insert(b - 1, Line::new(cols));
        } else {
            self.lines[t..b].rotate_left(1);
            *self
                .lines
                .get_mut(b - 1)
                .expect("grid invariant: b-1 < lines.len() after rotate") = Line::new(cols);
        }
        for row in top..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn scroll_down(&mut self, top: u32, bottom: u32, cols: u32) {
        if top >= bottom || bottom > self.rows {
            return;
        }
        let t = top as usize;
        let b = bottom as usize;
        self.lines[t..b].rotate_right(1);
        *self
            .lines
            .get_mut(t)
            .expect("grid invariant: t < lines.len() after rotate") = Line::new(cols);
        for row in top..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn insert_lines(&mut self, at: u32, count: u32, bottom: u32, cols: u32) {
        if at >= bottom || count == 0 {
            return;
        }
        let actual = count.min(bottom - at);
        let a = at as usize;
        let b = bottom as usize;
        // 左旋：在 `at` 处插入空行，现有行向下推移
        self.lines[a..b].rotate_right(actual as usize);
        // 用空行填充新插入的行（现在在切片开头）
        for i in a..a + actual as usize {
            *self
                .lines
                .get_mut(i)
                .expect("grid invariant: i < lines.len() after rotate_right") = Line::new(cols);
        }
        for row in at..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn delete_lines(&mut self, at: u32, count: u32, bottom: u32, cols: u32) {
        if at >= bottom || count == 0 {
            return;
        }
        let actual = count.min(bottom - at);
        let a = at as usize;
        let b = bottom as usize;
        // 左旋：删除 `at` 处的行，从下方拉取行向上
        self.lines[a..b].rotate_left(actual as usize);
        // 用空行填充已删除的行（现在在切片末尾）
        for i in b - actual as usize..b {
            *self
                .lines
                .get_mut(i)
                .expect("grid invariant: i < lines.len() after rotate_left") = Line::new(cols);
        }
        for row in at..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn clear_cells(&mut self, row: u32, start_col: u32, end_col: u32) {
        if let Some(line) = self.lines.get_mut(row as usize) {
            for col in start_col..end_col.min(line.len()) {
                if let Some(cell) = line.get_mut(col) {
                    *cell = crate::cell::Cell::default();
                }
            }
            self.dirty.mark(row);
        }
    }

    pub fn fill_cells(&mut self, row: u32, ch: char, start_col: u32, end_col: u32) {
        if let Some(line) = self.lines.get_mut(row as usize) {
            for col in start_col..end_col.min(line.len()) {
                if let Some(cell) = line.get_mut(col) {
                    cell.char = ch;
                }
            }
            self.dirty.mark(row);
        }
    }

    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    pub fn scrollback_line(&self, index: usize) -> Option<&Line> {
        self.scrollback.get(index)
    }

    pub fn clear_scrollback(&mut self) {
        self.scrollback.clear();
    }

    pub fn max_scrollback(&self) -> usize {
        self.max_scrollback
    }

    pub fn push_scrollback(&mut self, line: Line) {
        self.scrollback.push_back(line);
        while self.scrollback.len() > self.max_scrollback {
            self.scrollback.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[test]
    fn grid_new_dimensions() {
        let g = Grid::new(24, 80);
        assert_eq!(g.rows(), 24);
        assert_eq!(g.cols(), 80);
    }

    #[test]
    fn grid_new_all_dirty() {
        let g = Grid::new(3, 5);
        for row in 0..3 {
            assert!(g.dirty().is_dirty(row));
        }
    }

    #[test]
    fn grid_mark_clean() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_get_valid_row() {
        let g = Grid::new(24, 80);
        assert!(g.get(0).is_some());
        assert!(g.get(23).is_some());
    }

    #[test]
    fn grid_get_out_of_bounds() {
        let g = Grid::new(24, 80);
        assert!(g.get(24).is_none());
    }

    #[test]
    fn grid_get_mut_marks_dirty() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        let _ = g.get_mut(1);
        assert!(g.dirty().is_dirty(1));
        assert!(!g.dirty().is_dirty(0));
    }

    #[test]
    fn grid_resize() {
        let mut g = Grid::new(24, 80);
        g.resize(30, 100);
        assert_eq!(g.rows(), 30);
        assert_eq!(g.cols(), 100);
    }

    #[test]
    fn grid_resize_marks_all_dirty() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        g.resize(5, 10);
        assert!(g.dirty().any_dirty());
    }

    #[test]
    fn grid_cell_default() {
        let g = Grid::new(1, 1);
        let cell = g.get(0).unwrap().get(0).unwrap();
        assert_eq!(cell.char, ' ');
    }

    #[test]
    fn grid_set_cell() {
        let mut g = Grid::new(1, 1);
        g.get_mut(0).unwrap().get_mut(0).unwrap().char = 'X';
        assert_eq!(g.get(0).unwrap().get(0).unwrap().char, 'X');
    }

    #[test]
    fn grid_row_cells_returns_full_slice() {
        let mut g = Grid::new(3, 5);
        g.fill_cells(1, 'X', 0, 5);
        let row1 = g.row_cells(1).unwrap();
        assert_eq!(row1.len(), 5);
        assert!(row1.iter().all(|c| c.char == 'X'));
    }

    #[test]
    fn grid_row_cells_out_of_bounds() {
        let g = Grid::new(3, 5);
        assert!(g.row_cells(3).is_none());
    }

    #[test]
    fn dirty_mask_bit_ops() {
        let mut m = DirtyMask::new(24);
        assert!(!m.any_dirty());
        m.mark(0);
        m.mark(3);
        assert!(m.is_dirty(0));
        assert!(m.is_dirty(3));
        assert!(!m.is_dirty(1));
        m.clear();
        assert!(!m.any_dirty());
        m.mark_all(5);
        for i in 0..5 {
            assert!(m.is_dirty(i));
        }
    }

    #[test]
    fn grid_scrollback_on_scroll_up() {
        let mut g = Grid::new(3, 5);
        g.fill_cells(0, 'A', 0, 5);
        g.fill_cells(1, 'B', 0, 5);
        g.fill_cells(2, 'C', 0, 5);
        g.scroll_up(0, 3, 5);
        assert_eq!(g.scrollback_len(), 1);
        assert_eq!(g.scrollback_line(0).unwrap().get(0).unwrap().char, 'A');
    }

    #[test]
    fn grid_scrollback_no_save_when_not_top() {
        let mut g = Grid::new(4, 5);
        g.fill_cells(0, 'A', 0, 5);
        g.fill_cells(1, 'B', 0, 5);
        g.fill_cells(2, 'C', 0, 5);
        g.fill_cells(3, 'D', 0, 5);
        g.scroll_up(1, 3, 5);
        assert_eq!(g.scrollback_len(), 0);
    }

    #[test]
    fn grid_scrollback_max_limit() {
        let mut g = Grid::with_scrollback(2, 5, 3);
        for i in 0..5 {
            g.fill_cells(0, (b'A' + i) as char, 0, 5);
            g.scroll_up(0, 2, 5);
        }
        assert!(g.scrollback_len() <= 3);
    }

    #[test]
    fn grid_scrollback_clear() {
        let mut g = Grid::new(2, 5);
        g.fill_cells(0, 'A', 0, 5);
        g.scroll_up(0, 2, 5);
        assert_eq!(g.scrollback_len(), 1);
        g.clear_scrollback();
        assert_eq!(g.scrollback_len(), 0);
    }

    #[quickcheck]
    fn prop_grid_resize_preserves_cols(rows: u32, cols: u32, new_rows: u32, new_cols: u32) -> bool {
        let rows = rows.clamp(1, 200);
        let cols = cols.clamp(1, 200);
        let new_rows = new_rows.clamp(1, 200);
        let new_cols = new_cols.clamp(1, 200);
        let mut g = Grid::new(rows, cols);
        g.mark_clean();
        g.resize(new_rows, new_cols);
        g.rows() == new_rows && g.cols() == new_cols
    }

    #[quickcheck]
    fn prop_dirty_mask_mark_then_clear(rows: u8) -> bool {
        let rows = rows.clamp(1, 200) as u32;
        let mut m = DirtyMask::new(rows);
        m.clear();
        for r in 0..rows {
            m.mark(r);
            if !m.is_dirty(r) {
                return false;
            }
        }
        m.clear();
        !m.any_dirty()
    }

    #[quickcheck]
    fn prop_scrollback_bounded(max_lines: u8, scrolls: u8) -> bool {
        let max_lines = (max_lines.clamp(1, 100)) as u32;
        let scrolls = scrolls.clamp(0, 200);
        let mut g = Grid::with_scrollback(2, 5, max_lines as usize);
        for _ in 0..scrolls {
            g.fill_cells(0, 'A', 0, 5);
            g.scroll_up(0, 2, 5);
        }
        g.scrollback_len() <= max_lines as usize
    }

    #[test]
    fn grid_with_scrollback_dimensions() {
        let g = Grid::with_scrollback(24, 80, 1000);
        assert_eq!(g.rows(), 24);
        assert_eq!(g.cols(), 80);
        assert_eq!(g.max_scrollback(), 1000);
    }

    #[test]
    fn grid_default_max_scrollback_50k() {
        let g = Grid::new(24, 80);
        assert_eq!(g.max_scrollback(), 50_000);
    }

    #[test]
    fn grid_scroll_up_invalid_top_equals_bottom() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.scroll_up(3, 3, 5);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_scroll_up_invalid_bottom_above_rows() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.scroll_up(0, 10, 5);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_scroll_up_region_size_one() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.scroll_up(0, 1, 5);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_scroll_up_region_marks_dirty() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.scroll_up(0, 5, 5);
        assert!(g.dirty().any_dirty());
    }

    #[test]
    fn grid_scroll_up_top_zero_appends_to_scrollback() {
        let mut g = Grid::new(2, 5);
        g.fill_cells(0, 'A', 0, 5);
        g.scroll_up(0, 2, 5);
        assert_eq!(g.scrollback_len(), 1);
        let first = g.scrollback_line(0).unwrap();
        assert_eq!(first.get(0).unwrap().char, 'A');
    }

    #[test]
    fn grid_scroll_up_top_nonzero_does_not_push_scrollback() {
        let mut g = Grid::new(4, 5);
        g.scroll_up(1, 4, 5);
        assert_eq!(g.scrollback_len(), 0);
    }

    #[test]
    fn grid_scroll_down_marks_dirty() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.scroll_down(0, 5, 5);
        assert!(g.dirty().any_dirty());
    }

    #[test]
    fn grid_scroll_down_top_equals_bottom_no_op() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.scroll_down(2, 2, 5);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_insert_lines_basic() {
        let mut g = Grid::new(4, 3);
        g.fill_cells(0, 'A', 0, 3);
        g.fill_cells(1, 'B', 0, 3);
        g.fill_cells(2, 'C', 0, 3);
        g.fill_cells(3, 'D', 0, 3);
        g.insert_lines(1, 1, 4, 3);
        // 插入后：索引 1 的行为空行，B 推到 2，C 到 3，D 丢弃
        assert_eq!(g.get(1).unwrap().get(0).unwrap().char, ' ');
        assert_eq!(g.get(2).unwrap().get(0).unwrap().char, 'B');
        assert_eq!(g.get(3).unwrap().get(0).unwrap().char, 'C');
    }

    #[test]
    fn grid_insert_lines_at_equals_bottom_no_op() {
        let mut g = Grid::new(3, 3);
        g.mark_clean();
        g.insert_lines(3, 1, 3, 3);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_insert_lines_zero_count_no_op() {
        let mut g = Grid::new(3, 3);
        g.mark_clean();
        g.insert_lines(1, 0, 3, 3);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_insert_lines_count_clamped_to_region() {
        let mut g = Grid::new(4, 3);
        g.insert_lines(0, 100, 4, 3);
        // 应插入 min(100, 4) = 4 行空行，所以全部为空
        for r in 0..4 {
            assert_eq!(g.get(r).unwrap().get(0).unwrap().char, ' ');
        }
    }

    #[test]
    fn grid_delete_lines_basic() {
        let mut g = Grid::new(4, 3);
        g.fill_cells(0, 'A', 0, 3);
        g.fill_cells(1, 'B', 0, 3);
        g.fill_cells(2, 'C', 0, 3);
        g.fill_cells(3, 'D', 0, 3);
        g.delete_lines(1, 1, 4, 3);
        // 删除后：A 留在 0，C 移到 1，D 移到 2，最后一行为空
        assert_eq!(g.get(0).unwrap().get(0).unwrap().char, 'A');
        assert_eq!(g.get(1).unwrap().get(0).unwrap().char, 'C');
        assert_eq!(g.get(2).unwrap().get(0).unwrap().char, 'D');
        assert_eq!(g.get(3).unwrap().get(0).unwrap().char, ' ');
    }

    #[test]
    fn grid_delete_lines_at_equals_bottom_no_op() {
        let mut g = Grid::new(3, 3);
        g.mark_clean();
        g.delete_lines(3, 1, 3, 3);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_delete_lines_zero_count_no_op() {
        let mut g = Grid::new(3, 3);
        g.mark_clean();
        g.delete_lines(0, 0, 3, 3);
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_clear_cells_range() {
        let mut g = Grid::new(1, 10);
        g.fill_cells(0, 'X', 0, 10);
        g.clear_cells(0, 2, 5);
        assert_eq!(g.get(0).unwrap().get(0).unwrap().char, 'X');
        assert_eq!(g.get(0).unwrap().get(1).unwrap().char, 'X');
        assert_eq!(g.get(0).unwrap().get(2).unwrap().char, ' ');
        assert_eq!(g.get(0).unwrap().get(4).unwrap().char, ' ');
        assert_eq!(g.get(0).unwrap().get(5).unwrap().char, 'X');
    }

    #[test]
    fn grid_clear_cells_out_of_range_clamps() {
        let mut g = Grid::new(1, 5);
        g.fill_cells(0, 'X', 0, 5);
        g.clear_cells(0, 3, 100);
        for c in 3..5 {
            assert_eq!(g.get(0).unwrap().get(c).unwrap().char, ' ');
        }
    }

    #[test]
    fn grid_clear_cells_invalid_row_no_panic() {
        let mut g = Grid::new(3, 5);
        g.clear_cells(100, 0, 5);
        // 无 panic，无脏标记
    }

    #[test]
    fn grid_fill_cells_partial_range() {
        let mut g = Grid::new(1, 10);
        g.fill_cells(0, 'X', 3, 7);
        for c in 0..3 {
            assert_eq!(g.get(0).unwrap().get(c).unwrap().char, ' ');
        }
        for c in 3..7 {
            assert_eq!(g.get(0).unwrap().get(c).unwrap().char, 'X');
        }
        for c in 7..10 {
            assert_eq!(g.get(0).unwrap().get(c).unwrap().char, ' ');
        }
    }

    #[test]
    fn grid_cell_mut_marks_dirty() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        let cell = g.cell_mut(1, 2).unwrap();
        cell.char = 'Z';
        assert!(g.dirty().is_dirty(1));
    }

    #[test]
    fn grid_cell_mut_out_of_bounds_returns_none() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        assert!(g.cell_mut(3, 0).is_none());
        assert!(g.cell_mut(0, 5).is_none());
        assert!(!g.dirty().any_dirty());
    }

    #[test]
    fn grid_cell_read() {
        let mut g = Grid::new(2, 3);
        g.get_mut(0).unwrap().get_mut(1).unwrap().char = 'Y';
        assert_eq!(g.cell(0, 1).unwrap().char, 'Y');
        assert!(g.cell(5, 0).is_none());
    }

    #[test]
    fn grid_mark_row_dirty() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        g.mark_row_dirty(2);
        assert!(g.dirty().is_dirty(2));
        assert!(!g.dirty().is_dirty(1));
    }

    #[test]
    fn grid_mark_rows_dirty_range() {
        let mut g = Grid::new(10, 5);
        g.mark_clean();
        g.mark_rows_dirty(3, 7);
        for r in 3..7 {
            assert!(g.dirty().is_dirty(r));
        }
        assert!(!g.dirty().is_dirty(2));
        assert!(!g.dirty().is_dirty(7));
    }

    #[test]
    fn grid_mark_all_dirty() {
        let mut g = Grid::new(5, 5);
        g.mark_clean();
        g.mark_all_dirty();
        for r in 0..5 {
            assert!(g.dirty().is_dirty(r));
        }
    }

    #[test]
    fn grid_push_scrollback_within_limit() {
        let mut g = Grid::with_scrollback(2, 3, 5);
        g.push_scrollback(Line::new(3));
        g.push_scrollback(Line::new(3));
        assert_eq!(g.scrollback_len(), 2);
    }

    #[test]
    fn grid_push_scrollback_evicts_oldest() {
        let mut g = Grid::with_scrollback(2, 3, 3);
        for _ in 0..10 {
            g.push_scrollback(Line::new(3));
        }
        assert_eq!(g.scrollback_len(), 3);
    }

    #[test]
    fn grid_scrollback_line_out_of_bounds() {
        let g = Grid::new(2, 5);
        assert!(g.scrollback_line(0).is_none());
    }

    #[test]
    fn grid_scrollback_clear_via_method() {
        let mut g = Grid::new(2, 5);
        g.fill_cells(0, 'X', 0, 5);
        g.scroll_up(0, 2, 5);
        assert!(g.scrollback_len() > 0);
        g.clear_scrollback();
        assert_eq!(g.scrollback_len(), 0);
    }

    #[test]
    fn grid_scrollback_line_index_returns_correct() {
        let mut g = Grid::with_scrollback(2, 5, 10);
        for i in 0..3 {
            g.fill_cells(0, (b'A' + i) as char, 0, 5);
            g.scroll_up(0, 2, 5);
        }
        // 最旧的在索引 0
        assert_eq!(g.scrollback_line(0).unwrap().get(0).unwrap().char, 'A');
        assert_eq!(g.scrollback_line(1).unwrap().get(0).unwrap().char, 'B');
        assert_eq!(g.scrollback_line(2).unwrap().get(0).unwrap().char, 'C');
    }

    #[test]
    fn grid_resize_shrinks_preserves_existing_data() {
        let mut g = Grid::new(10, 20);
        g.fill_cells(0, 'X', 0, 20);
        g.resize(5, 10);
        assert_eq!(g.cell(0, 0).unwrap().char, 'X');
        assert_eq!(g.cell(0, 9).unwrap().char, 'X');
    }

    #[test]
    fn grid_resize_grows_preserves_existing_data() {
        let mut g = Grid::new(5, 10);
        g.fill_cells(0, 'A', 0, 10);
        g.resize(10, 20);
        assert_eq!(g.cell(0, 0).unwrap().char, 'A');
        assert_eq!(g.cell(0, 9).unwrap().char, 'A');
    }

    #[test]
    fn grid_scroll_up_preserves_top_zero_only() {
        let mut g = Grid::new(3, 3);
        g.fill_cells(0, 'A', 0, 3);
        g.fill_cells(1, 'B', 0, 3);
        g.fill_cells(2, 'C', 0, 3);
        g.scroll_up(0, 3, 3);
        // 'A' goes to scrollback, 'B' moves to 0, 'C' moves to 1, last is blank
        assert_eq!(g.cell(0, 0).unwrap().char, 'B');
        assert_eq!(g.cell(1, 0).unwrap().char, 'C');
        assert_eq!(g.cell(2, 0).unwrap().char, ' ');
        assert_eq!(g.scrollback_line(0).unwrap().get(0).unwrap().char, 'A');
    }

    #[test]
    fn grid_resize_cols_changes_line_lengths() {
        let mut g = Grid::new(2, 5);
        g.fill_cells(0, 'X', 0, 5);
        g.resize(2, 10);
        assert_eq!(g.row_cells(0).unwrap().len(), 10);
    }

    #[test]
    fn grid_zero_zero_creates_empty() {
        let g = Grid::new(0, 0);
        assert_eq!(g.rows(), 0);
        assert_eq!(g.cols(), 0);
    }
}
