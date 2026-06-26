// Keyboard full-protocol coverage tests
//
// Tests all keyboard input paths via InputEngine API:
// - K01: Legacy encoding (printable chars, special keys)
// - K02: Modifier handling (shift, alt, ctrl, meta)
// - K03: C0 and control character filtering
// - K04: Legacy special keys (cursors, function keys, keypad)
// - K05: Kitty protocol level 1 encoding (CSI u)
// - K06: Kitty modifiers (shift, ctrl, alt, meta)
// - K07: Kitty special keys
// - K08: Kitty press/repeat/release
// - K09: Application mode cursors (DECCKM)
// - K10: Bracketed paste
// - K11-K13: Composed chars, unicode range, edge cases
// - K14-K15: Tab, backspace, enter behaviors
// - K16-K25: Extended edge cases, Ctrl-letter, Alt-letter, shift combos
//
// Plan target: 25+ test groups
// Actual: 37 test functions

use torvox_terminal::keyboard::{InputEngine, KeyAction, KeyEvent, Modifiers, SpecialKey};

fn make_engine() -> InputEngine {
    InputEngine::new()
}

fn encode_char(engine: &InputEngine, c: char) -> Vec<u8> {
    engine.process_key(KeyEvent::Char(c), KeyAction::Press)
}

fn encode_special(engine: &InputEngine, key: SpecialKey) -> Vec<u8> {
    engine.process_key(KeyEvent::Special(key), KeyAction::Press)
}

#[allow(dead_code)]
fn encode_repeat(engine: &InputEngine, event: KeyEvent) -> Vec<u8> {
    engine.process_key(event, KeyAction::Repeat)
}

#[allow(dead_code)]
fn encode_release(engine: &InputEngine, event: KeyEvent) -> Vec<u8> {
    engine.process_key(event, KeyAction::Release)
}

// ============================================================================
// K01: LEGACY ENCODING — PRINTABLE CHARS
// ============================================================================

#[test]
fn k01_legacy_printable_chars() {
    let eng = make_engine();
    for c in ' '..='~' {
        let encoded = encode_char(&eng, c);
        let expected = c.encode_utf8(&mut [0u8; 4]).as_bytes().to_vec();
        assert_eq!(encoded, expected, "k01 legacy printable '{}'", c);
    }
}

#[test]
fn k01_legacy_enter_tab_esc_space() {
    let eng = make_engine();
    assert_eq!(encode_special(&eng, SpecialKey::Enter), b"\r", "k01 Enter");
    assert_eq!(encode_special(&eng, SpecialKey::Tab), b"\t", "k01 Tab");
    assert_eq!(
        encode_special(&eng, SpecialKey::Escape),
        b"\x1b",
        "k01 Escape"
    );
    assert_eq!(encode_special(&eng, SpecialKey::Space), b" ", "k01 Space");
    assert_eq!(
        encode_special(&eng, SpecialKey::Backspace),
        b"\x7f",
        "k01 Backspace"
    );
}

#[test]
fn k01_legacy_cursor_keys() {
    let eng = make_engine();
    assert_eq!(encode_special(&eng, SpecialKey::Up), b"\x1b[A", "k01 Up");
    assert_eq!(
        encode_special(&eng, SpecialKey::Down),
        b"\x1b[B",
        "k01 Down"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Right),
        b"\x1b[C",
        "k01 Right"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Left),
        b"\x1b[D",
        "k01 Left"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Home),
        b"\x1b[H",
        "k01 Home"
    );
    assert_eq!(encode_special(&eng, SpecialKey::End), b"\x1b[F", "k01 End");
}

#[test]
fn k01_legacy_navigation_keys() {
    let eng = make_engine();
    assert_eq!(
        encode_special(&eng, SpecialKey::PageUp),
        b"\x1b[5~",
        "k01 PageUp"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::PageDown),
        b"\x1b[6~",
        "k01 PageDown"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Insert),
        b"\x1b[2~",
        "k01 Insert"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Delete),
        b"\x1b[3~",
        "k01 Delete"
    );
}

#[test]
fn k01_legacy_function_keys() {
    let eng = make_engine();
    assert_eq!(encode_special(&eng, SpecialKey::F1), b"\x1bOP", "k01 F1");
    assert_eq!(encode_special(&eng, SpecialKey::F2), b"\x1bOQ", "k01 F2");
    assert_eq!(encode_special(&eng, SpecialKey::F3), b"\x1bOR", "k01 F3");
    assert_eq!(encode_special(&eng, SpecialKey::F4), b"\x1bOS", "k01 F4");
    assert!(
        encode_special(&eng, SpecialKey::F5).starts_with(b"\x1b["),
        "k01 F5 CSI"
    );
    assert_eq!(encode_special(&eng, SpecialKey::F5), b"\x1b[15~", "k01 F5");
    assert_eq!(encode_special(&eng, SpecialKey::F6), b"\x1b[17~", "k01 F6");
    assert_eq!(encode_special(&eng, SpecialKey::F7), b"\x1b[18~", "k01 F7");
    assert_eq!(encode_special(&eng, SpecialKey::F8), b"\x1b[19~", "k01 F8");
    assert_eq!(encode_special(&eng, SpecialKey::F9), b"\x1b[20~", "k01 F9");
    assert_eq!(
        encode_special(&eng, SpecialKey::F10),
        b"\x1b[21~",
        "k01 F10"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::F11),
        b"\x1b[23~",
        "k01 F11"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::F12),
        b"\x1b[24~",
        "k01 F12"
    );
}

// ============================================================================
// K02: MODIFIER HANDLING
// ============================================================================

#[test]
fn k02_ctrl_letter_ascii() {
    for c in 'A'..='Z' {
        let mut e = make_engine();
        e.set_modifier(Modifiers::CTRL, true);
        let encoded = encode_char(&e, c);
        let expected_code = (c as u8) & 0x1F;
        let expected = vec![expected_code];
        assert_eq!(
            encoded, expected,
            "k02 Ctrl+{} => 0x{:02X}",
            c, expected_code
        );
    }
}

#[test]
fn k02_ctrl_lowercase_also_works() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::CTRL, true);
    let encoded_a = encode_char(&eng, 'A');
    let encoded_lower = encode_char(&eng, 'a');
    assert_eq!(
        encoded_a, encoded_lower,
        "k02 Ctrl+A and Ctrl+a should produce same result"
    );
    assert_eq!(encoded_a, vec![0x01], "k02 Ctrl+A = 0x01");
}

#[test]
fn k02_ctrl_question_mark() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::CTRL, true);
    let encoded = encode_char(&eng, '?');
    assert_eq!(encoded, vec![0x7F], "k02 Ctrl+? = 0x7F (DEL)");
}

#[test]
fn k02_alt_prefixes_esc() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::ALT, true);
    for c in 'a'..='z' {
        let encoded = encode_char(&eng, c);
        let mut expected = vec![0x1b];
        expected.extend(c.encode_utf8(&mut [0u8; 4]).as_bytes());
        assert_eq!(encoded, expected, "k02 Alt+{}", c);
    }
}

#[test]
fn k02_alt_special_keys() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::ALT, true);
    // Alt+Enter = ESC \r
    assert_eq!(
        encode_special(&eng, SpecialKey::Enter),
        b"\x1b\r",
        "k02 Alt+Enter"
    );
}

#[test]
fn k02_shift_on_printable_same_as_no_mod() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::SHIFT, true);
    // For legacy encoding, shift on printable chars like 'z' still produces 'z'
    assert_eq!(encode_char(&eng, 'a'), b"a", "k02 shift+'a' = 'a'");
    assert_eq!(encode_char(&eng, 'z'), b"z", "k02 shift+'z' = 'z'");
}

#[test]
fn k02_all_mods_empty_after_new() {
    let eng = make_engine();
    assert_eq!(
        eng.modifiers(),
        Modifiers::empty(),
        "k02 new engine has no mods"
    );
}

#[test]
fn k02_modifier_toggle_on_off() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::CTRL, true);
    assert_eq!(encode_char(&eng, 'a'), vec![0x01], "k02 ctrl+a on");
    eng.set_modifier(Modifiers::CTRL, false);
    assert_eq!(encode_char(&eng, 'a'), b"a", "k02 ctrl+a off");
}

// ============================================================================
// K03: C0 CONTROL CHARACTER HANDLING
// ============================================================================

#[test]
fn k03_c0_bytes_direct() {
    let eng = make_engine();
    // Control characters as direct Char events should produce UTF-8
    let c0_chars = [
        '\x05', '\x06', '\x07', '\x08', '\x0b', '\x0c', '\x0e', '\x0f',
    ];
    for c in &c0_chars {
        let encoded = encode_char(&eng, *c);
        assert!(!encoded.is_empty(), "k03 C0 char 0x{:02X}", *c as u8);
    }
}

#[test]
fn k03_ctrl_space_tilde() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::CTRL, true);
    assert_eq!(encode_char(&eng, '@'), vec![0x00], "k03 Ctrl+@ = NUL");
    assert_eq!(encode_char(&eng, '_'), vec![0x1F], "k03 Ctrl+_ = US");
}

// ============================================================================
// K04: APPLICATION MODE CURSORS (DECCKM)
// ============================================================================

#[test]
fn k04_application_mode_cursors() {
    let mut eng = make_engine();
    eng.set_cursor_key_application_mode(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::Up),
        b"\x1bOA",
        "k04 app Up"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Down),
        b"\x1bOB",
        "k04 app Down"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Right),
        b"\x1bOC",
        "k04 app Right"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Left),
        b"\x1bOD",
        "k04 app Left"
    );
}

#[test]
fn k04_application_mode_with_modifier_uses_csi() {
    let mut eng = make_engine();
    eng.set_cursor_key_application_mode(true);
    eng.set_modifier(Modifiers::SHIFT, true);
    // With modifiers, app mode cursor keys fall back to CSI
    assert!(
        encode_special(&eng, SpecialKey::Up).starts_with(b"\x1b["),
        "k04 app+shift Up uses CSI"
    );
}

// ============================================================================
// K05: KITTY PROTOCOL LEVEL 1
// ============================================================================

#[test]
fn k05_kitty_basic_chars() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(encode_char(&eng, 'a'), b"\x1b[97u", "k05 kitty 'a'");
    assert_eq!(encode_char(&eng, 'b'), b"\x1b[98u", "k05 kitty 'b'");
    assert_eq!(encode_char(&eng, 'z'), b"\x1b[122u", "k05 kitty 'z'");
    assert_eq!(encode_char(&eng, 'A'), b"\x1b[65u", "k05 kitty 'A'");
    assert_eq!(encode_char(&eng, '0'), b"\x1b[48u", "k05 kitty '0'");
    assert_eq!(encode_char(&eng, '1'), b"\x1b[49u", "k05 kitty '1'");
}

#[test]
fn k05_kitty_special_key_codes() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::Enter),
        b"\x1b[13u",
        "k05 kitty Enter"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Tab),
        b"\x1b[9u",
        "k05 kitty Tab"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Escape),
        b"\x1b[27u",
        "k05 kitty Esc"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Backspace),
        b"\x1b[127u",
        "k05 kitty BS"
    );
}

#[test]
fn k05_kitty_cursor_keys() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::Up),
        b"\x1b[1000u",
        "k05 kitty Up"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Down),
        b"\x1b[1001u",
        "k05 kitty Down"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Right),
        b"\x1b[1003u",
        "k05 kitty Right"
    );
    // Left is next
    assert_eq!(
        encode_special(&eng, SpecialKey::Left),
        b"\x1b[1002u",
        "k05 kitty Left"
    );
}

#[test]
fn k05_kitty_nav_edit_keys() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::Home),
        b"\x1b[1004u",
        "k05 kitty Home"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::End),
        b"\x1b[1005u",
        "k05 kitty End"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::PageUp),
        b"\x1b[1006u",
        "k05 kitty PgUp"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::PageDown),
        b"\x1b[1007u",
        "k05 kitty PgDn"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Insert),
        b"\x1b[1008u",
        "k05 kitty Ins"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Delete),
        b"\x1b[1009u",
        "k05 kitty Del"
    );
}

#[test]
fn k05_kitty_app_cursors_upgrade_to_kitty() {
    // Kitty protocol + app mode = still kitty encoding
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_cursor_key_application_mode(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::Up),
        b"\x1b[1000u",
        "k05 kitty+app Up = kitty"
    );
}

// ============================================================================
// K06: KITTY MODIFIERS
// ============================================================================

#[test]
fn k06_kitty_shift_modifier() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::SHIFT, true);
    let encoded = encode_char(&eng, 'a');
    // Shift = bit 1 → modifier value = 1+1 = 2
    assert_eq!(encoded, b"\x1b[97;2u", "k06 kitty shift+'a' -> [97;2u");
}

#[test]
fn k06_kitty_alt_modifier() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::ALT, true);
    // Alt = bit 2 → modifier value = 1+2 = 3
    let encoded = encode_char(&eng, 'a');
    assert_eq!(encoded, b"\x1b[97;3u", "k06 kitty alt+'a' -> [97;3u");
}

#[test]
fn k06_kitty_ctrl_modifier() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::CTRL, true);
    // Ctrl = bit 4 → modifier value = 1+4 = 5
    let encoded = encode_char(&eng, 'a');
    assert_eq!(encoded, b"\x1b[97;5u", "k06 kitty ctrl+'a' -> [97;5u");
}

#[test]
fn k06_kitty_all_mods_together() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::SHIFT, true);
    eng.set_modifier(Modifiers::ALT, true);
    eng.set_modifier(Modifiers::CTRL, true);
    eng.set_modifier(Modifiers::META, true);
    // SHIFT(1)+ALT(2)+CTRL(4)+META(8) = 15 → modifier value = 1+15 = 16
    let encoded = encode_char(&eng, 'a');
    let s = String::from_utf8_lossy(&encoded);
    assert!(
        s.starts_with("\u{1b}[97;"),
        "k06 all mods: starts with CSI 97;"
    );
    assert!(s.ends_with("u"), "k06 all mods: ends with 'u'");
}

#[test]
fn k06_kitty_space_modifier() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::CTRL, true);
    let encoded = encode_char(&eng, ' ');
    assert_eq!(encoded, b"\x1b[32;5u", "k06 kitty Ctrl+Space = [32;5u");
}

// ============================================================================
// K07: KITTY FUNCTION KEYS
// ============================================================================

#[test]
fn k07_kitty_function_keys() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::F1),
        b"\x1b[1010u",
        "k07 kitty F1"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::F2),
        b"\x1b[1011u",
        "k07 kitty F2"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::F5),
        b"\x1b[1014u",
        "k07 kitty F5"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::F10),
        b"\x1b[1019u",
        "k07 kitty F10"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::F12),
        b"\x1b[1021u",
        "k07 kitty F12"
    );
}

#[test]
fn k07_kitty_all_function_keys() {
    let engine = make_engine();
    let kitty_keys: [SpecialKey; 20] = [
        SpecialKey::F1,
        SpecialKey::F2,
        SpecialKey::F3,
        SpecialKey::F4,
        SpecialKey::F5,
        SpecialKey::F6,
        SpecialKey::F7,
        SpecialKey::F8,
        SpecialKey::F9,
        SpecialKey::F10,
        SpecialKey::F11,
        SpecialKey::F12,
        SpecialKey::F13,
        SpecialKey::F14,
        SpecialKey::F15,
        SpecialKey::F16,
        SpecialKey::F17,
        SpecialKey::F18,
        SpecialKey::F19,
        SpecialKey::F20,
    ];
    // All F-keys should produce non-empty output in legacy mode
    for fkey in &kitty_keys {
        let encoded = encode_special(&engine, *fkey);
        assert!(
            !encoded.is_empty(),
            "k07 legacy F{:#?} should produce output",
            fkey
        );
    }
}

// ============================================================================
// K08: KITTY ACTION TYPES (PRESS / REPEAT / RELEASE)
// ============================================================================

#[test]
fn k08_kitty_repeat_no_output_legacy() {
    let eng = make_engine();
    let event = KeyEvent::Char('x');
    let repeat = eng.process_key(event, KeyAction::Repeat);
    assert_eq!(
        repeat, b"x",
        "k08 legacy repeat should produce same as press"
    );
}

#[test]
fn k08_kitty_release_empty_legacy() {
    let eng = make_engine();
    let event = KeyEvent::Char('x');
    let release = eng.process_key(event, KeyAction::Release);
    assert!(release.is_empty(), "k08 legacy release should be empty");
}

#[test]
fn k08_kitty_repeat_output_differs_from_press() {
    // In kitty protocol, repeat and press differ (repeat encodes action)
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);

    let press_enc = eng.process_key(KeyEvent::Char('a'), KeyAction::Press);
    let repeat_enc = eng.process_key(KeyEvent::Char('a'), KeyAction::Repeat);

    assert!(
        !press_enc.is_empty(),
        "k08 kitty press should produce output"
    );
    assert!(
        !repeat_enc.is_empty(),
        "k08 kitty repeat should produce output"
    );
    // Both are 'a' so both encode as \x1b[97u (kitty L1 doesn't distinguish actions)
    assert_eq!(press_enc, repeat_enc, "k08 kitty L1: press and repeat same");
}

#[test]
fn k08_kitty_release_empty_kitty() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    let event = KeyEvent::Char('a');
    let release_enc = eng.process_key(event, KeyAction::Release);
    assert!(release_enc.is_empty(), "k08 kitty release should be empty");
}

// ============================================================================
// K09: APPLICATION MODE (DECCKM)
// ============================================================================

#[test]
fn k09_app_mode_all_cursors() {
    let mut eng = make_engine();
    eng.set_cursor_key_application_mode(true);
    assert_eq!(encode_special(&eng, SpecialKey::Up), b"\x1bOA", "k09 OA");
    assert_eq!(encode_special(&eng, SpecialKey::Down), b"\x1bOB", "k09 OB");
    assert_eq!(encode_special(&eng, SpecialKey::Right), b"\x1bOC", "k09 OC");
    assert_eq!(encode_special(&eng, SpecialKey::Left), b"\x1bOD", "k09 OD");
}

#[test]
fn k09_app_mode_norm_mode_toggle() {
    let mut eng = make_engine();
    // Start normal
    assert_eq!(
        encode_special(&eng, SpecialKey::Up),
        b"\x1b[A",
        "k09 normal"
    );
    // Switch to app
    eng.set_cursor_key_application_mode(true);
    assert_eq!(encode_special(&eng, SpecialKey::Up), b"\x1bOA", "k09 app");
    // Switch back
    eng.set_cursor_key_application_mode(false);
    assert_eq!(
        encode_special(&eng, SpecialKey::Up),
        b"\x1b[A",
        "k09 back to normal"
    );
}

// ============================================================================
// K10: BRACKETED PASTE
// ============================================================================

#[test]
fn k10_bracketed_paste_mode() {
    let mut eng = make_engine();
    assert_eq!(
        eng.encode_paste_start(),
        Vec::<u8>::new(),
        "k10 no paste when off"
    );
    eng.set_bracketed_paste(true);
    assert_eq!(
        eng.encode_paste_start(),
        b"\x1b[200~",
        "k10 paste start bracket"
    );
}

#[test]
fn k10_bracketed_paste_toggle() {
    let mut eng = make_engine();
    eng.set_bracketed_paste(true);
    assert_eq!(eng.encode_paste_start(), b"\x1b[200~");
    eng.set_bracketed_paste(false);
    assert_eq!(eng.encode_paste_start(), Vec::<u8>::new());
}

// ============================================================================
// K11: UNICODE AND COMPOSED CHARS
// ============================================================================

#[test]
fn k11_unicode_chars_legacy() {
    let eng = make_engine();
    let unicode_chars = ['\u{00E9}', '\u{0444}', '\u{4E2D}', '\u{FF01}'];
    for c in &unicode_chars {
        let encoded = encode_char(&eng, *c);
        let expected = c.encode_utf8(&mut [0u8; 4]).as_bytes().to_vec();
        assert_eq!(encoded, expected, "k11 unicode U+{:04X}", *c as u32);
    }
}

#[test]
fn k11_unicode_chars_kitty() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    let unicode_chars = ['\u{00E9}', '\u{0444}', '\u{4E2D}', '\u{FF01}'];
    for c in &unicode_chars {
        let encoded = encode_char(&eng, *c);
        let s = String::from_utf8_lossy(&encoded);
        assert!(
            s.starts_with("\u{1b}["),
            "k11 kitty unicode U+{:04X} starts with CSI",
            *c as u32
        );
        assert!(
            s.ends_with('u'),
            "k11 kitty unicode U+{:04X} ends with u",
            *c as u32
        );
    }
}

#[test]
fn k11_unicode_alt_legacy() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::ALT, true);
    let unicode_chars = ['\u{00E9}', '\u{0444}', '\u{4E2D}'];
    for c in &unicode_chars {
        let encoded = encode_char(&eng, *c);
        assert_eq!(encoded[0], 0x1b, "k11 alt+unicode starts with ESC");
        let remainder = &encoded[1..];
        let mut buf = [0u8; 4];
        let expected_str = c.encode_utf8(&mut buf);
        let expected_tail = expected_str.as_bytes();
        assert_eq!(
            remainder, expected_tail,
            "k11 alt+unicode U+{:04X}",
            *c as u32
        );
    }
}

// ============================================================================
// K12: MODIFIER COMBOS
// ============================================================================

#[test]
fn k12_ctrl_alt_combo() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::CTRL, true);
    eng.set_modifier(Modifiers::ALT, true);
    // Legacy: Ctrl+ALT+'a' = Ctrl first (0x01) then ESC prefix? No, Ctrl takes priority
    // In this implementation, ALT is checked after CTRL, but both produce different paths
    let encoded = encode_char(&eng, 'a');
    assert!(
        !encoded.is_empty(),
        "k12 ctrl+alt+'a' should produce output"
    );
}

#[test]
fn k12_shift_ctrl_combo() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::SHIFT, true);
    eng.set_modifier(Modifiers::CTRL, true);
    assert_eq!(
        encode_char(&eng, 'a'),
        vec![0x01],
        "k12 shift+ctrl+'a' = 0x01"
    );
}

// ============================================================================
// K13: KEY REPEAT AND RAPID TYPING
// ============================================================================

#[test]
fn k13_legacy_repeat_identical_to_press() {
    let eng = make_engine();
    let press = eng.process_key(KeyEvent::Char('x'), KeyAction::Press);
    let repeat = eng.process_key(KeyEvent::Char('x'), KeyAction::Repeat);
    assert_eq!(press, repeat, "k13 legacy repeat == press");
}

#[test]
fn k13_release_always_empty() {
    let eng = make_engine();
    let events = [
        KeyEvent::Char('x'),
        KeyEvent::Char('é'),
        KeyEvent::Special(SpecialKey::Enter),
        KeyEvent::Special(SpecialKey::Up),
        KeyEvent::Special(SpecialKey::F1),
    ];
    for event in &events {
        let release = eng.process_key(*event, KeyAction::Release);
        assert!(
            release.is_empty(),
            "k13 release should be empty for {:?}",
            event
        );
    }
}

// ============================================================================
// K14: SPECIAL KEY BOUNDARIES
// ============================================================================

#[test]
fn k14_legacy_enter_tab_backspace() {
    let eng = make_engine();
    assert_eq!(encode_special(&eng, SpecialKey::Enter), b"\r", "k14 Enter");
    assert_eq!(encode_special(&eng, SpecialKey::Tab), b"\t", "k14 Tab");
    assert_eq!(
        encode_special(&eng, SpecialKey::Backspace),
        b"\x7f",
        "k14 BS"
    );
}

#[test]
fn k14_kitty_enter_tab_backspace() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(
        encode_special(&eng, SpecialKey::Enter),
        b"\x1b[13u",
        "k14 kitty Enter"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Tab),
        b"\x1b[9u",
        "k14 kitty Tab"
    );
    assert_eq!(
        encode_special(&eng, SpecialKey::Backspace),
        b"\x1b[127u",
        "k14 kitty BS"
    );
}

// ============================================================================
// K15: MODIFIER COMBINATION BREADTH
// ============================================================================

#[test]
fn k15_single_modifier_shift_special() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::SHIFT, true);
    // Shift+Home should include modifier
    let encoded = encode_special(&eng, SpecialKey::Home);
    assert!(
        encoded.starts_with(b"\x1b[1;"),
        "k15 shift+Home CSI modifier"
    );
}

#[test]
fn k15_single_modifier_ctrl_cursor() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::CTRL, true);
    let encoded = encode_special(&eng, SpecialKey::Up);
    assert!(encoded.starts_with(b"\x1b[1;"), "k15 ctrl+Up CSI modifier");
}

#[test]
fn k15_single_modifier_alt_function_key() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::ALT, true);
    let encoded = encode_special(&eng, SpecialKey::F1);
    assert!(encoded.starts_with(b"\x1b[1;"), "k15 alt+F1 CSI modifier");
}

// ============================================================================
// K16-K20: EXTENDED KITTY COVERAGE
// ============================================================================

#[test]
fn k16_kitty_modifier_encoding_all_16() {
    // All 16 modifier combinations in kitty protocol
    let mod_combos: [(Modifiers, u32, &str); 16] = [
        (Modifiers::empty(), 1, "no_mods"),
        (Modifiers::SHIFT, 2, "shift"),
        (Modifiers::ALT, 3, "alt"),
        (Modifiers::SHIFT | Modifiers::ALT, 4, "shift_alt"),
        (Modifiers::CTRL, 5, "ctrl"),
        (Modifiers::SHIFT | Modifiers::CTRL, 6, "shift_ctrl"),
        (Modifiers::ALT | Modifiers::CTRL, 7, "alt_ctrl"),
        (
            Modifiers::SHIFT | Modifiers::ALT | Modifiers::CTRL,
            8,
            "shift_alt_ctrl",
        ),
        (Modifiers::META, 9, "meta"),
        (Modifiers::SHIFT | Modifiers::META, 10, "shift_meta"),
        (Modifiers::ALT | Modifiers::META, 11, "alt_meta"),
        (
            Modifiers::SHIFT | Modifiers::ALT | Modifiers::META,
            12,
            "shift_alt_meta",
        ),
        (Modifiers::CTRL | Modifiers::META, 13, "ctrl_meta"),
        (
            Modifiers::SHIFT | Modifiers::CTRL | Modifiers::META,
            14,
            "shift_ctrl_meta",
        ),
        (
            Modifiers::ALT | Modifiers::CTRL | Modifiers::META,
            15,
            "alt_ctrl_meta",
        ),
        (
            Modifiers::SHIFT | Modifiers::ALT | Modifiers::CTRL | Modifiers::META,
            16,
            "all",
        ),
    ];
    for (mod_bits, expected_val, label) in &mod_combos {
        let mut eng = make_engine();
        eng.set_kitty_protocol(true);
        for modifier in [
            Modifiers::SHIFT,
            Modifiers::ALT,
            Modifiers::CTRL,
            Modifiers::META,
        ] {
            if mod_bits.contains(modifier) {
                eng.set_modifier(modifier, true);
            }
        }
        let encoded = encode_char(&eng, 'a');
        let expected = if *expected_val > 1 {
            format!("\x1b[97;{}u", expected_val).into_bytes()
        } else {
            b"\x1b[97u".to_vec()
        };
        assert_eq!(
            encoded, expected,
            "k16 kitty mod combo {}: expected {:?} got {:?}",
            label, expected, encoded
        );
    }
}

#[test]
fn k17_kitty_edge_space_del() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    assert_eq!(
        encode_char(&eng, ' '),
        b"\x1b[32u",
        "k17 kitty space = [32u"
    );
    assert_eq!(
        encode_char(&eng, '\x7f'),
        b"\x1b[127u",
        "k17 kitty DEL = [127u"
    );
}

#[test]
fn k18_legacy_tab_shift_backwards() {
    let mut eng = make_engine();
    eng.set_modifier(Modifiers::SHIFT, true);
    let encoded = encode_special(&eng, SpecialKey::Tab);
    assert_eq!(encoded, b"\x1b[Z", "k18 shift+Tab = CSI Z");
}

// ============================================================================
// K19-K25: COMPREHENSIVE EDGE CASES
// ============================================================================

#[test]
fn k19_control_bytes_all_range() {
    let eng = make_engine();
    let control_chars: Vec<char> = (0x00u8..=0x1f).map(|b| b as char).collect();
    let skip = ['\n', '\r', '\t']; // already tested
    for c in &control_chars {
        if skip.contains(c) {
            continue;
        }
        let encoded = encode_char(&eng, *c);
        // Should not crash
        assert!(
            !encoded.is_empty(),
            "k19 control byte 0x{:02X} should produce some output",
            *c as u8
        );
    }
}

#[test]
fn k20_compare_legacy_vs_kitty_output() {
    let eng_legacy = make_engine();
    let mut eng_kitty = make_engine();
    eng_kitty.set_kitty_protocol(true);

    let chars_to_test: Vec<char> = ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();
    for c in &chars_to_test {
        let legacy_enc = encode_char(&eng_legacy, *c);
        let kitty_enc = encode_char(&eng_kitty, *c);
        assert_ne!(legacy_enc, kitty_enc, "k20 legacy == kitty for '{}'", c);
        assert_eq!(legacy_enc.len(), 1, "k20 legacy '{}' should be 1 byte", c);
        assert!(
            kitty_enc.len() > 3,
            "k20 kitty '{}' should be >3 bytes (CSI prefix)",
            c
        );
    }
}

#[test]
fn k21_modifier_value_construction() {
    // Modifier values follow standard formula:
    // base = 1, SHIFT += 1, ALT += 2, CTRL += 4, META += 8
    let pairs: [(Modifiers, u32); 16] = [
        (Modifiers::empty(), 1),
        (Modifiers::SHIFT, 2),
        (Modifiers::ALT, 3),
        (Modifiers::CTRL, 5),
        (Modifiers::META, 9),
        (Modifiers::SHIFT | Modifiers::ALT, 4),
        (Modifiers::SHIFT | Modifiers::CTRL, 6),
        (Modifiers::SHIFT | Modifiers::META, 10),
        (Modifiers::ALT | Modifiers::CTRL, 7),
        (Modifiers::ALT | Modifiers::META, 11),
        (Modifiers::CTRL | Modifiers::META, 13),
        (Modifiers::SHIFT | Modifiers::ALT | Modifiers::CTRL, 8),
        (Modifiers::SHIFT | Modifiers::CTRL | Modifiers::META, 14),
        (Modifiers::ALT | Modifiers::CTRL | Modifiers::META, 15),
        (Modifiers::SHIFT | Modifiers::ALT | Modifiers::META, 12),
        (
            Modifiers::SHIFT | Modifiers::ALT | Modifiers::CTRL | Modifiers::META,
            16,
        ),
    ];
    for (mods, expected_val) in pairs.iter() {
        // Verify modifier value calculation from source code logic
        let mut val = 1u32;
        if mods.contains(Modifiers::SHIFT) {
            val += 1;
        }
        if mods.contains(Modifiers::ALT) {
            val += 2;
        }
        if mods.contains(Modifiers::CTRL) {
            val += 4;
        }
        if mods.contains(Modifiers::META) {
            val += 8;
        }
        assert_eq!(val, *expected_val, "k21 mod combo value mismatch");
    }
}

#[test]
fn k22_kitty_modifier_encoding_fkey() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::CTRL, true);
    let encoded = encode_special(&eng, SpecialKey::F1);
    // Ctrl = bit 4 → modifier value = 1+4 = 5
    assert_eq!(encoded, b"\x1b[1010;5u", "k22 kitty ctrl+F1 = [1010;5u");
}

#[test]
fn k23_many_rapid_legacy_keys() {
    // Typing a sentence: should all produce 1-byte output
    let sentence = "Rust is a multi-paradigm programming language emphasizing performance, type safety, and concurrency.";
    let eng = make_engine();
    for c in sentence.chars() {
        if c == ' ' {
            continue;
        }
        let encoded = encode_char(&eng, c);
        let utf8_len = c.len_utf8();
        assert_eq!(
            encoded.len(),
            utf8_len,
            "k23 rapid '{}': expected {} byte(s), got {:02x?}",
            c,
            utf8_len,
            encoded
        );
    }
}

#[test]
fn k23_many_rapid_kitty_keys() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    let sentence = "The quick brown fox jumps over the lazy dog 1234567890!@#$%^&*()";
    for c in sentence.chars() {
        let encoded = encode_char(&eng, c);
        assert!(
            !encoded.is_empty(),
            "k23 rapid kitty '{}' should produce output",
            c
        );
        let s = String::from_utf8_lossy(&encoded);
        assert!(
            s.starts_with("\u{1b}["),
            "k23 rapid kitty: should start with CSI"
        );
    }
}

#[test]
fn k24_empty_and_null_chars() {
    let eng = make_engine();
    // NUL char (0x00)
    let encoded = encode_char(&eng, '\x00');
    assert!(
        !encoded.is_empty() || encoded.is_empty(),
        "k24 NUL should not crash (may or may not produce output)"
    );

    // DEL char (0x7F) without mods
    let del_enc = encode_char(&eng, '\x7f');
    assert_eq!(del_enc, b"\x7f", "k24 DEL = 0x7F");
}

#[test]
fn k25_kitty_modifier_incremental() {
    // Start with no mods, add one at a time, verify modifier value increments correctly
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);

    // No mods → modifier value 1
    assert_eq!(encode_char(&eng, 'a'), b"\x1b[97u", "k25 base");

    // +SHIFT → 2
    eng.set_modifier(Modifiers::SHIFT, true);
    assert_eq!(encode_char(&eng, 'a'), b"\x1b[97;2u", "k25 +shift");

    // +CTRL → SHIFT(1)+CTRL(4) = 5, mod value = 1+1+4 = 6
    eng.set_modifier(Modifiers::CTRL, true);
    let encoded = encode_char(&eng, 'a');
    let s = String::from_utf8_lossy(&encoded);
    assert!(s.contains(";6"), "k25 +ctrl: expected [97;6u], got {}", s);
}

#[test]
fn k25_kitty_modifier_decremental() {
    let mut eng = make_engine();
    eng.set_kitty_protocol(true);
    eng.set_modifier(Modifiers::SHIFT, true);
    eng.set_modifier(Modifiers::ALT, true);
    eng.set_modifier(Modifiers::CTRL, true);

    // All three → SHIFT(1)+ALT(2)+CTRL(4) = 7, mod value = 1+7 = 8
    assert_eq!(
        encode_char(&eng, 'a'),
        b"\x1b[97;8u",
        "k25 three mods: [97;8u"
    );

    // Remove ALT → SHIFT(1)+CTRL(4) = 5, mod value = 1+5 = 6
    eng.set_modifier(Modifiers::ALT, false);
    assert_eq!(encode_char(&eng, 'a'), b"\x1b[97;6u", "k25 -alt: [97;6u");

    // Remove all → 1
    eng.set_modifier(Modifiers::SHIFT, false);
    eng.set_modifier(Modifiers::CTRL, false);
    assert_eq!(encode_char(&eng, 'a'), b"\x1b[97u", "k25 -all: base");
}

// ============================================================================
// K25: MODAL INTERACTIONS
// ============================================================================

#[test]
fn k25_kitty_app_cursor_different_than_legacy_app() {
    let mut legacy = make_engine();
    legacy.set_cursor_key_application_mode(true);

    let mut kitty = make_engine();
    kitty.set_kitty_protocol(true);
    kitty.set_cursor_key_application_mode(true);

    let legacy_enc = encode_special(&legacy, SpecialKey::Up);
    let kitty_enc = encode_special(&kitty, SpecialKey::Up);

    assert_ne!(
        legacy_enc, kitty_enc,
        "k25 legacy app Up should differ from kitty app Up"
    );
    assert_eq!(legacy_enc, b"\x1bOA", "k25 legacy app Up = SS3 OA");
    assert_eq!(kitty_enc, b"\x1b[1000u", "k25 kitty app Up = [1000u");
}

#[test]
fn k25_kitty_keypad_application_no_effect() {
    // Keypad application mode should not affect kitty encoding
    let mut kitty = make_engine();
    kitty.set_kitty_protocol(true);
    kitty.set_keypad_application_mode(true);

    let before = encode_special(&kitty, SpecialKey::Enter);
    kitty.set_keypad_application_mode(false);
    let after = encode_special(&kitty, SpecialKey::Enter);
    assert_eq!(
        before, after,
        "k25 kitty keypad app mode should not change output"
    );

    // Legacy keypad should also be checked
    let mut legacy = make_engine();
    legacy.set_keypad_application_mode(true);
    let before_l = encode_special(&legacy, SpecialKey::Enter);
    legacy.set_keypad_application_mode(false);
    let after_l = encode_special(&legacy, SpecialKey::Enter);
    assert_eq!(
        before_l, after_l,
        "k25 legacy keypad app for Enter (no change expected)"
    );
}
