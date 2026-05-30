#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use torvox_terminal::terminal::TerminalState;

    let mut state = TerminalState::new(24, 80);
    state.process_bytes(data);
});
