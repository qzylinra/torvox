#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use torvox_terminal::terminal::TerminalState;

    let Ok(mut state) = TerminalState::new(24, 80) else {
        return;
    };
    state.process_bytes(data);
});
