use libghostty_vt::render::Dirty;
use libghostty_vt::{RenderState, Terminal};

use crate::parser::VtParser;

pub struct TerminalState {
    parser: VtParser,
    render_state: RenderState<'static>,
}

impl TerminalState {
    pub fn new(rows: u32, cols: u32) -> Self {
        let parser =
            VtParser::new(rows as u16, cols as u16).expect("failed to create Ghostty VT terminal");
        let render_state = RenderState::new().expect("failed to create Ghostty VT render state");
        Self {
            parser,
            render_state,
        }
    }

    pub fn rows(&self) -> u32 {
        self.parser.terminal().rows().unwrap_or(24) as u32
    }

    pub fn cols(&self) -> u32 {
        self.parser.terminal().cols().unwrap_or(80) as u32
    }

    pub fn resize(&mut self, rows: u32, cols: u32) {
        let cell_width = 8;
        let cell_height = 16;
        let _ =
            self.parser
                .terminal_mut()
                .resize(cols as u16, rows as u16, cell_width, cell_height);
    }

    pub fn process_bytes(&mut self, data: &[u8]) {
        self.parser.terminal_mut().vt_write(data);
    }

    pub fn update_render_state(&mut self) -> bool {
        // SAFETY: render_state and terminal are separate fields.
        // render_state.update() only reads from terminal and writes to render_state.
        // This is equivalent to what libghostty-vt does internally.
        let render_state_ptr: *mut RenderState<'static> = &mut self.render_state;
        let terminal_ptr: *const Terminal<'static, 'static> = self.parser.terminal();
        unsafe {
            if let Ok(snapshot) = (*render_state_ptr).update(&*terminal_ptr) {
                match snapshot.dirty() {
                    Ok(Dirty::Clean) => false,
                    Ok(Dirty::Partial) | Ok(Dirty::Full) => true,
                    Err(_) => true,
                }
            } else {
                true
            }
        }
    }

    pub fn render_state(&self) -> &RenderState<'static> {
        &self.render_state
    }

    pub fn render_state_mut(&mut self) -> &mut RenderState<'static> {
        &mut self.render_state
    }

    pub fn terminal(&self) -> &Terminal<'static, 'static> {
        self.parser.terminal()
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<'static, 'static> {
        self.parser.terminal_mut()
    }

    pub fn title(&self) -> &str {
        self.parser.terminal().title().unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_state_creation() {
        let state = TerminalState::new(24, 80);
        assert_eq!(state.rows(), 24);
        assert_eq!(state.cols(), 80);
    }

    #[test]
    fn terminal_state_resize() {
        let mut state = TerminalState::new(24, 80);
        state.resize(40, 120);
        assert_eq!(state.rows(), 40);
        assert_eq!(state.cols(), 120);
    }

    #[test]
    fn terminal_state_process_bytes() {
        let mut state = TerminalState::new(24, 80);
        state.process_bytes(b"Hello, world!\r\n");
        assert!(state.update_render_state());
    }

    #[test]
    fn terminal_state_title() {
        let mut state = TerminalState::new(24, 80);
        state.process_bytes(b"\x1b]2;Hello\x1b\\");
        assert_eq!(state.title(), "Hello");
    }
}
