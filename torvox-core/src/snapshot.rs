use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::grid::Grid;
use crate::line::Line;

#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub visible_lines: Vec<Line>,
    pub scrollback_lines: Vec<Line>,
    pub rows: u32,
    pub cols: u32,
    pub max_scrollback: usize,
}

impl SessionSnapshot {
    pub fn from_grid(grid: &Grid) -> Self {
        let mut visible_lines = Vec::with_capacity(grid.rows() as usize);
        for r in 0..grid.rows() {
            visible_lines.push(
                grid.get(r)
                    .cloned()
                    .unwrap_or_else(|| Line::new(grid.cols())),
            );
        }
        let scrollback_len = grid.scrollback_len();
        let mut scrollback_lines = Vec::with_capacity(scrollback_len);
        for i in 0..scrollback_len {
            scrollback_lines.push(
                grid.scrollback_line(i)
                    .cloned()
                    .unwrap_or_else(|| Line::new(grid.cols())),
            );
        }
        Self {
            visible_lines,
            scrollback_lines,
            rows: grid.rows(),
            cols: grid.cols(),
            max_scrollback: grid.max_scrollback(),
        }
    }

    pub fn apply_to_scrollback(&self, grid: &mut Grid, max_lines: usize) {
        let total = self.scrollback_lines.len() + self.visible_lines.len();
        let keep = total.min(max_lines);
        let mut all: Vec<Line> = Vec::with_capacity(total);
        all.extend(self.scrollback_lines.iter().cloned());
        all.extend(self.visible_lines.iter().cloned());
        let start = total.saturating_sub(keep);
        for line in all.iter().skip(start) {
            grid.push_scrollback(line.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn session_snapshot_from_grid_roundtrip() {
        let mut grid = Grid::new(24, 80);
        grid.get_mut(0).unwrap().get_mut(0).unwrap().char = 'X';
        grid.get_mut(5).unwrap().get_mut(10).unwrap().char = 'Y';
        grid.fill_cells(0, 'A', 0, 80);
        grid.scroll_up(0, 24, 80);
        grid.scroll_up(0, 24, 80);

        let snapshot = SessionSnapshot::from_grid(&grid);
        assert_eq!(snapshot.rows, 24);
        assert_eq!(snapshot.cols, 80);
        assert!(!snapshot.visible_lines.is_empty());
        assert_eq!(snapshot.scrollback_lines.len(), 2);
    }

    #[test]
    fn session_snapshot_apply_to_scrollback() {
        let mut grid = Grid::new(24, 80);
        grid.get_mut(0).unwrap().get_mut(0).unwrap().char = 'Z';
        grid.scroll_up(0, 24, 80);

        let snapshot = SessionSnapshot::from_grid(&grid);
        let mut restored = Grid::new(24, 80);
        snapshot.apply_to_scrollback(&mut restored, 1000);

        assert!(restored.scrollback_len() > 0);
        let first = restored.scrollback_line(0).unwrap().get(0).unwrap();
        assert_eq!(first.char, 'Z');
    }

    #[cfg(feature = "rkyv")]
    #[test]
    fn session_snapshot_rkyv_roundtrip() {
        use rkyv::rancor::Error;

        let mut grid = Grid::new(24, 80);
        grid.get_mut(0).unwrap().get_mut(0).unwrap().char = 'R';
        grid.scroll_up(0, 24, 80);

        let snapshot = SessionSnapshot::from_grid(&grid);
        let bytes = rkyv::to_bytes::<Error>(&snapshot).expect("rkyv serialize failed");
        let archived = unsafe {
            rkyv::access_unchecked::<<SessionSnapshot as rkyv::Archive>::Archived>(&bytes)
        };
        let restored =
            rkyv::deserialize::<SessionSnapshot, Error>(archived).expect("rkyv deserialize failed");
        assert_eq!(restored, snapshot);
    }
}
