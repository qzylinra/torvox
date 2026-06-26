use torvox_terminal::keyboard::{InputEngine, KeyAction, KeyEvent, Modifiers, SpecialKey};

fn engine() -> InputEngine {
    InputEngine::new()
}

fn encode_char(ch: char, mods: &[(Modifiers, bool)]) -> Vec<u8> {
    let mut eng = engine();
    for (m, pressed) in mods {
        eng.set_modifier(*m, *pressed);
    }
    eng.process_key(KeyEvent::Char(ch), KeyAction::Press)
}

fn encode_special(key: SpecialKey, mods: &[(Modifiers, bool)]) -> Vec<u8> {
    let mut eng = engine();
    for (m, pressed) in mods {
        eng.set_modifier(*m, *pressed);
    }
    eng.process_key(KeyEvent::Special(key), KeyAction::Press)
}

#[test]
fn ctrl_shift_alt_combo_char() {
    let out = encode_char(
        'a',
        &[
            (Modifiers::CTRL, true),
            (Modifiers::SHIFT, true),
            (Modifiers::ALT, true),
        ],
    );
    assert_eq!(out, vec![0x01], "Ctrl should win over Shift+Alt");
}

#[test]
fn shift_ctrl_combo_ctrl_wins() {
    let out = encode_char('z', &[(Modifiers::SHIFT, true), (Modifiers::CTRL, true)]);
    assert_eq!(out, vec![0x1A], "Ctrl+Shift+Z should produce 0x1A");
}

#[test]
fn alt_shift_combo() {
    let out = encode_char('x', &[(Modifiers::ALT, true), (Modifiers::SHIFT, true)]);
    assert!(
        out.starts_with(&[0x1b]),
        "Alt+Shift should produce ESC prefix, got: {:?}",
        out
    );
}

#[test]
fn triple_modifier_ctrl_alt_shift() {
    let out = encode_char(
        'c',
        &[
            (Modifiers::CTRL, true),
            (Modifiers::ALT, true),
            (Modifiers::SHIFT, true),
        ],
    );
    assert_eq!(out, vec![0x03], "Ctrl+Alt+Shift+C should produce 0x03");
}

#[test]
fn ctrl_shift_arrow_up() {
    let out = encode_special(
        SpecialKey::Up,
        &[(Modifiers::CTRL, true), (Modifiers::SHIFT, true)],
    );
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("1;6"),
        "Ctrl+Shift+Up should encode modifier 6, got: {}",
        s
    );
}

#[test]
fn alt_ctrl_arrow_down() {
    let out = encode_special(
        SpecialKey::Down,
        &[(Modifiers::CTRL, true), (Modifiers::ALT, true)],
    );
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("1;7"),
        "Ctrl+Alt+Down should encode modifier 7, got: {}",
        s
    );
}

#[test]
fn shift_ctrl_home() {
    let out = encode_special(
        SpecialKey::Home,
        &[(Modifiers::SHIFT, true), (Modifiers::CTRL, true)],
    );
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("1;6"),
        "Ctrl+Shift+Home should encode modifier 6, got: {}",
        s
    );
}

#[test]
fn modifier_release_produces_empty() {
    let mut eng = engine();
    eng.set_modifier(Modifiers::CTRL, true);
    let out = eng.process_key(KeyEvent::Char('c'), KeyAction::Release);
    assert!(
        out.is_empty(),
        "Release should produce empty, got: {:?}",
        out
    );
}

#[test]
fn repeat_same_as_press() {
    let eng = engine();
    let press = eng.process_key(KeyEvent::Char('a'), KeyAction::Press);
    let repeat = eng.process_key(KeyEvent::Char('a'), KeyAction::Repeat);
    assert_eq!(press, repeat, "Repeat should produce same output as Press");
}

#[test]
fn kitty_ctrl_shift_a() {
    let mut eng = engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::CTRL, true);
    eng.set_modifier(Modifiers::SHIFT, true);
    let out = eng.process_key(KeyEvent::Char('a'), KeyAction::Press);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("97;6"),
        "Kitty Ctrl+Shift+A should have modifier 6, got: {}",
        s
    );
}

#[test]
fn kitty_alt_enter() {
    let mut eng = engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::ALT, true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("13;3"),
        "Kitty Alt+Enter should have modifier 3, got: {}",
        s
    );
}

#[test]
fn kitty_release_empty() {
    let mut eng = engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::CTRL, true);
    let out = eng.process_key(KeyEvent::Char('a'), KeyAction::Release);
    assert!(
        out.is_empty(),
        "Kitty release should produce empty, got: {:?}",
        out
    );
}

#[test]
fn cursor_app_mode_up() {
    let mut eng = engine();
    eng.set_cursor_key_application_mode(true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("OA"),
        "App cursor mode should produce SS3, got: {}",
        s
    );
}

#[test]
fn cursor_app_mode_with_ctrl_falls_back_to_csi() {
    let mut eng = engine();
    eng.set_cursor_key_application_mode(true);
    eng.set_modifier(Modifiers::CTRL, true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("[1;5"),
        "App cursor mode+Ctrl should use CSI, got: {}",
        s
    );
}

#[test]
fn keypad_app_mode_enter() {
    let mut eng = engine();
    eng.set_keypad_application_mode(true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
    assert_eq!(out, b"\r", "Keypad app mode Enter should still be CR");
}

#[test]
fn bracketed_paste_start_end() {
    let mut eng = engine();
    eng.set_bracketed_paste(true);
    assert_eq!(eng.encode_paste_start(), b"\x1b[200~");
    assert_eq!(eng.encode_paste_end(), b"\x1b[201~");
}

#[test]
fn bracketed_paste_disabled_returns_empty() {
    let mut eng = engine();
    eng.set_bracketed_paste(false);
    assert!(eng.encode_paste_start().is_empty());
    assert!(eng.encode_paste_end().is_empty());
}

#[test]
fn mouse_sgr_press_encoding() {
    let eng = engine();
    let out = eng.encode_mouse_press(0, 5, 10, Modifiers::empty());
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.starts_with("\x1b[<"),
        "SGR mouse press should start with ESC[<, got: {}",
        s
    );
    assert!(
        s.contains("M"),
        "SGR mouse press should end with M, got: {}",
        s
    );
}

#[test]
fn mouse_sgr_release_encoding() {
    let eng = engine();
    let out = eng.encode_mouse_release(0, 5, 10, Modifiers::empty());
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.starts_with("\x1b[<"),
        "SGR mouse release should start with ESC[<, got: {}",
        s
    );
    assert!(
        s.ends_with("m"),
        "SGR mouse release should end with m, got: {}",
        s
    );
}

#[test]
fn backspace_bs_mode() {
    let mut eng = engine();
    eng.set_backspace_byte(0x08);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
    assert_eq!(out, vec![0x08], "BS mode backspace should be 0x08");
}

#[test]
fn backspace_del_mode() {
    let mut eng = engine();
    eng.set_backspace_byte(0x7f);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
    assert_eq!(out, vec![0x7f], "DEL mode backspace should be 0x7f");
}

#[test]
fn alt_backspace_del_mode() {
    let mut eng = engine();
    eng.set_backspace_byte(0x7f);
    eng.set_modifier(Modifiers::ALT, true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
    assert_eq!(out, vec![0x1b, 0x7f], "Alt+Backspace should be ESC+DEL");
}

#[test]
fn alt_backspace_bs_mode() {
    let mut eng = engine();
    eng.set_backspace_byte(0x08);
    eng.set_modifier(Modifiers::ALT, true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
    assert_eq!(
        out,
        vec![0x1b, 0x08],
        "Alt+Backspace BS mode should be ESC+BS"
    );
}

#[test]
fn kitty_function_keys_with_modifier() {
    let mut eng = engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::SHIFT, true);
    let out = eng.process_key(KeyEvent::Special(SpecialKey::F5), KeyAction::Press);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains(";2"),
        "Kitty Shift+F5 should have modifier 2, got: {}",
        s
    );
}

#[test]
fn legacy_f1_no_modifier() {
    let out = encode_special(SpecialKey::F1, &[]);
    assert_eq!(out, b"\x1bOP", "F1 should be SS3 P");
}

#[test]
fn legacy_f5_no_modifier() {
    let out = encode_special(SpecialKey::F5, &[]);
    assert_eq!(out, b"\x1b[15~", "F5 should be CSI 15~");
}

#[test]
fn shift_tab_encodes_csi_z() {
    let out = encode_special(SpecialKey::Tab, &[(Modifiers::SHIFT, true)]);
    assert_eq!(out, b"\x1b[Z", "Shift+Tab should be CSI Z");
}

#[test]
fn no_modifier_tab_is_bare() {
    let out = encode_special(SpecialKey::Tab, &[]);
    assert_eq!(out, b"\t", "Tab without modifier should be bare 0x09");
}

#[test]
fn modifiers_bitwise_combine() {
    let m = Modifiers::SHIFT | Modifiers::CTRL;
    assert!(m.contains(Modifiers::SHIFT));
    assert!(m.contains(Modifiers::CTRL));
    assert!(!m.contains(Modifiers::ALT));
}

#[test]
fn modifiers_clear() {
    let mut eng = engine();
    eng.set_modifier(Modifiers::CTRL, true);
    assert!(eng.modifiers().contains(Modifiers::CTRL));
    eng.set_modifier(Modifiers::CTRL, false);
    assert!(!eng.modifiers().contains(Modifiers::CTRL));
}

#[test]
fn kitty_ctrl_unicode_passes_through() {
    let mut eng = engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::CTRL, true);
    let out = eng.process_key(KeyEvent::Char('\u{20ac}'), KeyAction::Press);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("8364"),
        "Kitty Ctrl+Euro should use codepoint 8364, got: {}",
        s
    );
}

#[test]
fn legacy_ctrl_unicode_passes_through() {
    let out = encode_char('\u{20ac}', &[(Modifiers::CTRL, true)]);
    let s = String::from_utf8_lossy(&out);
    assert!(
        s.contains("\u{20ac}"),
        "Legacy Ctrl+Euro should pass UTF-8 through, got: {:?}",
        s
    );
}
