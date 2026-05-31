#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use torvox_terminal::keyboard::{InputEngine, KeyAction, KeyEvent, SpecialKey};

    let engine = InputEngine::new();

    for &byte in data {
        let action = match byte % 3 {
            0 => KeyAction::Press,
            1 => KeyAction::Repeat,
            _ => KeyAction::Release,
        };

        let key = match byte % 20 {
            0 => KeyEvent::Special(SpecialKey::Enter),
            1 => KeyEvent::Special(SpecialKey::Tab),
            2 => KeyEvent::Special(SpecialKey::Backspace),
            3 => KeyEvent::Special(SpecialKey::Escape),
            4 => KeyEvent::Special(SpecialKey::Up),
            5 => KeyEvent::Special(SpecialKey::Down),
            6 => KeyEvent::Special(SpecialKey::Left),
            7 => KeyEvent::Special(SpecialKey::Right),
            8 => KeyEvent::Special(SpecialKey::Home),
            9 => KeyEvent::Special(SpecialKey::End),
            10 => KeyEvent::Special(SpecialKey::PageUp),
            11 => KeyEvent::Special(SpecialKey::PageDown),
            12 => KeyEvent::Special(SpecialKey::F1),
            13 => KeyEvent::Special(SpecialKey::F12),
            14 => KeyEvent::Char('a'),
            15 => KeyEvent::Char('z'),
            16 => KeyEvent::Char('0'),
            17 => KeyEvent::Char(' '),
            18 => KeyEvent::Char('\x7f'),
            _ => KeyEvent::Char('x'),
        };

        engine.process_key(key, action);
    }
});
