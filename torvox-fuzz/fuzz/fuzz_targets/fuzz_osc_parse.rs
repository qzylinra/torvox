#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use torvox_terminal::parser::VtParser;
    use torvox_terminal::terminal::TerminalState;

    let Ok(mut state) = TerminalState::new(24, 80) else {
        return;
    };
    let mut parser = VtParser::new();

    // Prepend OSC escape sequence prefix to exercise OSC parsing paths.
    // OSC = ESC ] ... BEL (0x07) or ST (ESC \)
    let mut input = Vec::with_capacity(data.len() + 4);
    input.extend_from_slice(b"\x1b]");
    input.extend_from_slice(data);
    input.push(0x07); // BEL terminates OSC

    parser.advance(&mut state, &input);
});
