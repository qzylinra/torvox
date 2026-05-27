use torvox_core::grid::Grid;

pub struct TerminalState {
    grid: Grid,
    #[allow(dead_code)]
    scrollback: Vec<String>,
}

impl TerminalState {
    pub fn new(rows: u32, cols: u32) -> Self {
        Self {
            grid: Grid::new(rows, cols),
            scrollback: Vec::new(),
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn resize(&mut self, rows: u32, cols: u32) {
        self.grid.resize(rows, cols);
    }
}
