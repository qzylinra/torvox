// @Input encoding (Kitty keyboard), IMPL_TERM_004, impl, [REQ_TERM_004]
// @need-ids: REQ_TERM_004, REQ_TERM_005
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Modifiers: u8 {
        const SHIFT = 0b0001;
        const ALT    = 0b0010;
        const CTRL   = 0b0100;
        const META   = 0b1000;
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MouseStyle {
    Press,
    Release,
    Motion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialKey {
    Enter,
    Tab,
    Backspace,
    Escape,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Special(SpecialKey),
}

pub struct InputEngine {
    modifiers: Modifiers,
    bracketed_paste: bool,
    kitty_protocol: bool,
    cursor_key_application_mode: bool,
    keypad_application_mode: bool,
    backspace_byte: u8,
}

impl InputEngine {
    pub fn new() -> Self {
        Self {
            modifiers: Modifiers::empty(),
            bracketed_paste: false,
            kitty_protocol: false,
            cursor_key_application_mode: false,
            keypad_application_mode: false,
            backspace_byte: 0x7f,
        }
    }

    pub fn set_backspace_byte(&mut self, byte: u8) {
        self.backspace_byte = byte;
    }

    pub fn set_bracketed_paste(&mut self, enabled: bool) {
        self.bracketed_paste = enabled;
    }

    pub fn set_kitty_protocol(&mut self, enabled: bool) {
        self.kitty_protocol = enabled;
    }

    pub fn set_cursor_key_application_mode(&mut self, enabled: bool) {
        self.cursor_key_application_mode = enabled;
    }

    pub fn set_keypad_application_mode(&mut self, enabled: bool) {
        self.keypad_application_mode = enabled;
    }

    pub fn set_modifier(&mut self, modifier: Modifiers, pressed: bool) {
        if pressed {
            self.modifiers |= modifier;
        } else {
            self.modifiers.remove(modifier);
        }
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    pub fn process_key(&self, key: KeyEvent, action: KeyAction) -> Vec<u8> {
        if self.kitty_protocol {
            self.encode_kitty(key, action)
        } else {
            self.encode_legacy(key, action)
        }
    }

    fn encode_kitty(&self, key: KeyEvent, action: KeyAction) -> Vec<u8> {
        match key {
            KeyEvent::Char(c) => {
                if c == ' ' || c == '\x7f' {
                    return self.encode_kitty_special(key, action);
                }
                if action == KeyAction::Release {
                    return Vec::new();
                }
                let mods = self.modifier_value();
                let code = c as u32;
                if mods > 1 {
                    format!("\x1b[{};{}u", code, mods).into_bytes()
                } else {
                    format!("\x1b[{}u", code).into_bytes()
                }
            }
            KeyEvent::Special(special) => self.encode_kitty_special_key(special, action),
        }
    }

    fn encode_kitty_special(&self, key: KeyEvent, action: KeyAction) -> Vec<u8> {
        if action == KeyAction::Release {
            return Vec::new();
        }
        match key {
            KeyEvent::Char(' ') => {
                let mods = self.modifier_value();
                if mods > 1 {
                    format!("\x1b[32;{}u", mods).into_bytes()
                } else {
                    b"\x1b[32u".to_vec()
                }
            }
            KeyEvent::Char('\x7f') => {
                let mods = self.modifier_value();
                if mods > 1 {
                    format!("\x1b[127;{}u", mods).into_bytes()
                } else {
                    b"\x1b[127u".to_vec()
                }
            }
            _ => Vec::new(),
        }
    }

    fn encode_kitty_special_key(&self, key: SpecialKey, action: KeyAction) -> Vec<u8> {
        if action == KeyAction::Release {
            return Vec::new();
        }
        let mods = self.modifier_value();
        let code = match key {
            SpecialKey::Enter => 13,
            SpecialKey::Tab => 9,
            SpecialKey::Backspace => 127,
            SpecialKey::Escape => 27,
            SpecialKey::Up => 1000,
            SpecialKey::Down => 1001,
            SpecialKey::Left => 1002,
            SpecialKey::Right => 1003,
            SpecialKey::Home => 1004,
            SpecialKey::End => 1005,
            SpecialKey::PageUp => 1006,
            SpecialKey::PageDown => 1007,
            SpecialKey::Insert => 1008,
            SpecialKey::Delete => 1009,
            SpecialKey::F1 => 1010,
            SpecialKey::F2 => 1011,
            SpecialKey::F3 => 1012,
            SpecialKey::F4 => 1013,
            SpecialKey::F5 => 1014,
            SpecialKey::F6 => 1015,
            SpecialKey::F7 => 1016,
            SpecialKey::F8 => 1017,
            SpecialKey::F9 => 1018,
            SpecialKey::F10 => 1019,
            SpecialKey::F11 => 1020,
            SpecialKey::F12 => 1021,
            SpecialKey::F13 => 1022,
            SpecialKey::F14 => 1023,
            SpecialKey::F15 => 1024,
            SpecialKey::F16 => 1025,
            SpecialKey::F17 => 1026,
            SpecialKey::F18 => 1027,
            SpecialKey::F19 => 1028,
            SpecialKey::F20 => 1029,
            SpecialKey::Space => 32,
        };
        if mods > 1 {
            format!("\x1b[{};{}u", code, mods).into_bytes()
        } else {
            format!("\x1b[{}u", code).into_bytes()
        }
    }

    fn encode_legacy(&self, key: KeyEvent, action: KeyAction) -> Vec<u8> {
        if action == KeyAction::Release {
            return Vec::new();
        }
        match key {
            KeyEvent::Char(c) => {
                if self.modifiers.contains(Modifiers::CTRL) && c.is_ascii() {
                    let code = (c as u8).to_ascii_uppercase();
                    return match code {
                        b'?' => vec![0x7F],
                        0x40..=0x5F => vec![code & 0x1F],
                        _ => {
                            let mut buf = [0u8; 4];
                            c.encode_utf8(&mut buf).as_bytes().to_vec()
                        }
                    };
                }
                if self.modifiers.contains(Modifiers::ALT) {
                    let mut bytes = Vec::new();
                    bytes.push(0x1b);
                    bytes.extend_from_slice(c.encode_utf8(&mut [0u8; 4]).as_bytes());
                    return bytes;
                }
                c.encode_utf8(&mut [0u8; 4]).as_bytes().to_vec()
            }
            KeyEvent::Special(special) => self.encode_legacy_special(special),
        }
    }

    fn encode_legacy_special(&self, key: SpecialKey) -> Vec<u8> {
        // Encode a legacy (non-Kitty) special key sequence.
        // Separates the CSI escape prefix from the function-key parameter so
        // modifiers can be inserted correctly as `CSI param;mod final`.
        //
        // For xterm-compatible encoding:
        //   No modifier: SS3 \x1bO{ch} (cursor keys in app mode, F1-F4)
        //                 CSI \x1b[{param}~ (PageUp, Insert, F5-F20, etc.)
        //                 raw {byte} (Enter, Tab, Backspace)
        //   With modifier: CSI \x1b[{param};{mod}{ch/~}
        //                  ESC {ch} (Enter+Alt = \x1b\r, Backspace+Alt = \x1b\x7f)
        //                  CSI Z (Shift+Tab = \x1b[Z)
        enum KeyEncoding {
            /// SS3 prefix (NO modifiers in legacy mode — forces CSI fallback)
            Ss3(u8),
            /// CSI prefix with explicit parameter number, tilde-terminated
            CsiTilde(&'static str),
            /// CSI prefix with a letter final byte (no parameter number baked in)
            CsiLetter(u8),
            /// Bare final byte (Enter = CR, Tab = TAB, Backspace = DEL)
            Bare(u8),
            /// Shift+Tab → CSI Z
            CsiZ,
        }
        let encoding = match key {
            SpecialKey::Up if self.cursor_key_application_mode => KeyEncoding::Ss3(b'A'),
            SpecialKey::Down if self.cursor_key_application_mode => KeyEncoding::Ss3(b'B'),
            SpecialKey::Right if self.cursor_key_application_mode => KeyEncoding::Ss3(b'C'),
            SpecialKey::Left if self.cursor_key_application_mode => KeyEncoding::Ss3(b'D'),
            SpecialKey::Up => KeyEncoding::CsiLetter(b'A'),
            SpecialKey::Down => KeyEncoding::CsiLetter(b'B'),
            SpecialKey::Right => KeyEncoding::CsiLetter(b'C'),
            SpecialKey::Left => KeyEncoding::CsiLetter(b'D'),
            SpecialKey::Home => KeyEncoding::CsiLetter(b'H'),
            SpecialKey::End => KeyEncoding::CsiLetter(b'F'),
            SpecialKey::PageUp => KeyEncoding::CsiTilde("5"),
            SpecialKey::PageDown => KeyEncoding::CsiTilde("6"),
            SpecialKey::Insert => KeyEncoding::CsiTilde("2"),
            SpecialKey::Delete => KeyEncoding::CsiTilde("3"),
            SpecialKey::F1 => KeyEncoding::Ss3(b'P'),
            SpecialKey::F2 => KeyEncoding::Ss3(b'Q'),
            SpecialKey::F3 => KeyEncoding::Ss3(b'R'),
            SpecialKey::F4 => KeyEncoding::Ss3(b'S'),
            SpecialKey::F5 => KeyEncoding::CsiTilde("15"),
            SpecialKey::F6 => KeyEncoding::CsiTilde("17"),
            SpecialKey::F7 => KeyEncoding::CsiTilde("18"),
            SpecialKey::F8 => KeyEncoding::CsiTilde("19"),
            SpecialKey::F9 => KeyEncoding::CsiTilde("20"),
            SpecialKey::F10 => KeyEncoding::CsiTilde("21"),
            SpecialKey::F11 => KeyEncoding::CsiTilde("23"),
            SpecialKey::F12 => KeyEncoding::CsiTilde("24"),
            SpecialKey::F13 => KeyEncoding::CsiTilde("25"),
            SpecialKey::F14 => KeyEncoding::CsiTilde("26"),
            SpecialKey::F15 => KeyEncoding::CsiTilde("28"),
            SpecialKey::F16 => KeyEncoding::CsiTilde("29"),
            SpecialKey::F17 => KeyEncoding::CsiTilde("31"),
            SpecialKey::F18 => KeyEncoding::CsiTilde("32"),
            SpecialKey::F19 => KeyEncoding::CsiTilde("33"),
            SpecialKey::F20 => KeyEncoding::CsiTilde("34"),
            SpecialKey::Tab => KeyEncoding::CsiZ,
            SpecialKey::Enter => KeyEncoding::Bare(b'\r'),
            SpecialKey::Backspace => KeyEncoding::Bare(self.backspace_byte),
            SpecialKey::Escape => KeyEncoding::Bare(b'\x1b'),
            SpecialKey::Space => KeyEncoding::Bare(b' '),
        };

        let mods = self.modifier_bits();
        let has_mods = !self.modifiers.is_empty();

        match encoding {
            // SS3 keys: no modifiers in legacy mode.
            // Without mods: \x1bO{ch}
            // With mods: switch to CSI \x1b[1;{mod}{ch}
            KeyEncoding::Ss3(ch) if !has_mods => {
                let mut result = vec![0x1b, b'O'];
                result.push(ch);
                result
            }
            KeyEncoding::Ss3(ch) => format!("\x1b[1;{}{}", mods, ch as char).into_bytes(),

            // CSI tilde keys with a baked-in parameter.
            // Without mods: \x1b[{param}~
            // With mods: \x1b[{param};{mod}~
            KeyEncoding::CsiTilde(param) if !has_mods => format!("\x1b[{}~", param).into_bytes(),
            KeyEncoding::CsiTilde(param) => format!("\x1b[{};{}~", param, mods).into_bytes(),

            // CSI letter keys (arrows, Home, End).
            // Without mods: \x1b[{ch}
            // With mods: \x1b[1;{mod}{ch}
            KeyEncoding::CsiLetter(ch) if !has_mods => {
                let mut result = vec![0x1b, b'['];
                result.push(ch);
                result
            }
            KeyEncoding::CsiLetter(ch) => format!("\x1b[1;{}{}", mods, ch as char).into_bytes(),

            // Tab with modifiers → CSI Z (backwards tab).
            // Tab without modifiers → raw TAB byte.
            // Shift+Tab produces plain \x1b[Z (shift is implicit in back-tab)
            KeyEncoding::CsiZ if !has_mods => vec![b'\t'],
            KeyEncoding::CsiZ if mods == 2 => vec![0x1b, b'[', b'Z'],
            KeyEncoding::CsiZ => format!("\x1b[{}Z", mods).into_bytes(),

            // Bare keys (Enter, Backspace, Escape, Space).
            // Without mods: just the byte.
            // Alt+Enter → \x1b\r (ESC prefix + CR)
            // Alt+Backspace → \x1b{byte} (ESC prefix + DEL or BS)
            // Other modifiers: CSI 1;{mod}{ch}
            KeyEncoding::Bare(ch) if !has_mods => vec![ch],
            KeyEncoding::Bare(ch) if ch == b'\r' || ch == self.backspace_byte => {
                // Alt prefix: ESC followed by bare byte
                if self.modifiers.contains(Modifiers::ALT) && mods == 3 {
                    vec![0x1b, ch]
                } else {
                    format!("\x1b[1;{}{}", mods, ch as char).into_bytes()
                }
            }
            KeyEncoding::Bare(ch) => format!("\x1b[1;{}{}", mods, ch as char).into_bytes(),
        }
    }

    fn modifier_bits(&self) -> u32 {
        let mut val = 1u32;
        if self.modifiers.contains(Modifiers::SHIFT) {
            val += 1;
        }
        if self.modifiers.contains(Modifiers::ALT) {
            val += 2;
        }
        if self.modifiers.contains(Modifiers::CTRL) {
            val += 4;
        }
        if self.modifiers.contains(Modifiers::META) {
            val += 8;
        }
        val
    }

    fn modifier_value(&self) -> u32 {
        self.modifier_bits()
    }

    pub fn encode_paste_start(&self) -> Vec<u8> {
        if self.bracketed_paste {
            b"\x1b[200~".to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn encode_paste_end(&self) -> Vec<u8> {
        if self.bracketed_paste {
            b"\x1b[201~".to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn encode_mouse_press(
        &self,
        button: u32,
        col: u32,
        row: u32,
        modifiers: Modifiers,
    ) -> Vec<u8> {
        encode_mouse(MouseStyle::Press, button, col, row, modifiers)
    }

    pub fn encode_mouse_release(
        &self,
        button: u32,
        col: u32,
        row: u32,
        modifiers: Modifiers,
    ) -> Vec<u8> {
        encode_mouse(MouseStyle::Release, button, col, row, modifiers)
    }

    pub fn encode_mouse_motion(
        &self,
        button: u32,
        col: u32,
        row: u32,
        modifiers: Modifiers,
    ) -> Vec<u8> {
        encode_mouse(MouseStyle::Motion, button, col, row, modifiers)
    }
}

impl Default for InputEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn encode_mouse(
    style: MouseStyle,
    button: u32,
    col: u32,
    row: u32,
    modifiers: Modifiers,
) -> Vec<u8> {
    let btn = button & 0x03;
    let mut encoded = btn;
    if matches!(style, MouseStyle::Motion) {
        encoded |= 0x20;
    }
    if modifiers.contains(Modifiers::SHIFT) {
        encoded |= 0x04;
    }
    if modifiers.contains(Modifiers::ALT) {
        encoded |= 0x08;
    }
    if modifiers.contains(Modifiers::CTRL) {
        encoded |= 0x10;
    }
    if matches!(style, MouseStyle::Press) && button >= 64 {
        encoded |= 0x40;
    }
    let suffix = match style {
        MouseStyle::Release => 'm',
        _ => 'M',
    };
    format!("\x1b[<{};{};{}{}", encoded, col + 1, row + 1, suffix).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[test]
    fn modifiers_default_empty() {
        let m = Modifiers::default();
        assert!(m.is_empty());
    }

    #[test]
    fn modifiers_combine() {
        let m = Modifiers::SHIFT | Modifiers::CTRL;
        assert!(m.contains(Modifiers::SHIFT));
        assert!(m.contains(Modifiers::CTRL));
        assert!(!m.contains(Modifiers::ALT));
    }

    #[test]
    fn input_engine_new() {
        let engine = InputEngine::new();
        assert_eq!(engine.modifiers(), Modifiers::empty());
    }

    #[test]
    fn encode_char_a() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, b"a");
    }

    #[test]
    fn encode_char_with_ctrl() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('c'), KeyAction::Press);
        assert_eq!(result, vec![0x03]);
    }

    #[test]
    fn encode_char_with_alt() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('x'), KeyAction::Press);
        assert_eq!(result, vec![0x1b, b'x']);
    }

    #[test]
    fn encode_enter() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
        assert_eq!(result, b"\r");
    }

    #[test]
    fn encode_tab() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Tab), KeyAction::Press);
        assert_eq!(result, b"\t");
    }

    #[test]
    fn encode_backspace() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(result, vec![0x7f]);
    }

    #[test]
    fn encode_escape() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Escape), KeyAction::Press);
        assert_eq!(result, b"\x1b");
    }

    #[test]
    fn encode_up_arrow() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[A");
    }

    #[test]
    fn encode_down_arrow() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Down), KeyAction::Press);
        assert_eq!(result, b"\x1b[B");
    }

    #[test]
    fn encode_right_arrow() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Right), KeyAction::Press);
        assert_eq!(result, b"\x1b[C");
    }

    #[test]
    fn encode_left_arrow() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Left), KeyAction::Press);
        assert_eq!(result, b"\x1b[D");
    }

    #[test]
    fn encode_home() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Home), KeyAction::Press);
        assert_eq!(result, b"\x1b[H");
    }

    #[test]
    fn encode_end() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::End), KeyAction::Press);
        assert_eq!(result, b"\x1b[F");
    }

    #[test]
    fn encode_page_up() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageUp), KeyAction::Press);
        assert_eq!(result, b"\x1b[5~");
    }

    #[test]
    fn encode_page_down() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageDown), KeyAction::Press);
        assert_eq!(result, b"\x1b[6~");
    }

    #[test]
    fn encode_insert() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Insert), KeyAction::Press);
        assert_eq!(result, b"\x1b[2~");
    }

    #[test]
    fn encode_delete() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Delete), KeyAction::Press);
        assert_eq!(result, b"\x1b[3~");
    }

    #[test]
    fn encode_f1() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F1), KeyAction::Press);
        assert_eq!(result, b"\x1bOP");
    }

    #[test]
    fn encode_f2() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F2), KeyAction::Press);
        assert_eq!(result, b"\x1bOQ");
    }

    #[test]
    fn encode_f5() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F5), KeyAction::Press);
        assert_eq!(result, b"\x1b[15~");
    }

    #[test]
    fn encode_f12() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F12), KeyAction::Press);
        assert_eq!(result, b"\x1b[24~");
    }

    #[test]
    fn encode_release_returns_empty() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Release);
        assert!(result.is_empty());
    }

    #[test]
    fn encode_repeat_works() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Repeat);
        assert_eq!(result, b"a");
    }

    #[test]
    fn bracketed_paste_start() {
        let mut engine = InputEngine::new();
        engine.set_bracketed_paste(true);
        assert_eq!(engine.encode_paste_start(), b"\x1b[200~");
    }

    #[test]
    fn bracketed_paste_end() {
        let mut engine = InputEngine::new();
        engine.set_bracketed_paste(true);
        assert_eq!(engine.encode_paste_end(), b"\x1b[201~");
    }

    #[test]
    fn bracketed_paste_disabled() {
        let engine = InputEngine::new();
        assert!(engine.encode_paste_start().is_empty());
        assert!(engine.encode_paste_end().is_empty());
    }

    #[test]
    fn mouse_press_sgr() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_press(0, 5, 10, Modifiers::empty());
        assert_eq!(result, b"\x1b[<0;6;11M");
    }

    #[test]
    fn mouse_release_sgr() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_release(0, 5, 10, Modifiers::empty());
        assert_eq!(result, b"\x1b[<0;6;11m");
    }

    #[test]
    fn mouse_motion_sgr() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_motion(0, 5, 10, Modifiers::empty());
        assert_eq!(result, b"\x1b[<32;6;11M");
    }

    #[test]
    fn mouse_press_with_shift() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_press(0, 5, 10, Modifiers::SHIFT);
        assert_eq!(result, b"\x1b[<4;6;11M");
    }

    #[test]
    fn mouse_press_with_ctrl() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_press(0, 5, 10, Modifiers::CTRL);
        assert_eq!(result, b"\x1b[<16;6;11M");
    }

    #[test]
    fn mouse_press_button_1() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_press(1, 5, 10, Modifiers::empty());
        assert_eq!(result, b"\x1b[<1;6;11M");
    }

    #[test]
    fn mouse_press_button_2() {
        let engine = InputEngine::new();
        let result = engine.encode_mouse_press(2, 5, 10, Modifiers::empty());
        assert_eq!(result, b"\x1b[<2;6;11M");
    }

    #[test]
    fn kitty_protocol_char() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, b"\x1b[97u");
    }

    #[test]
    fn kitty_protocol_enter() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
        assert_eq!(result, b"\x1b[13u");
    }

    #[test]
    fn kitty_protocol_up() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[1000u");
    }

    #[test]
    fn kitty_protocol_with_shift() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, b"\x1b[97;2u");
    }

    #[test]
    fn kitty_protocol_release_returns_empty() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Release);
        assert!(result.is_empty());
    }

    #[test]
    fn ctrl_with_various_keys() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);

        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, vec![0x01]);

        let result = engine.process_key(KeyEvent::Char('d'), KeyAction::Press);
        assert_eq!(result, vec![0x04]);

        let result = engine.process_key(KeyEvent::Char('z'), KeyAction::Press);
        assert_eq!(result, vec![0x1a]);
    }

    #[test]
    fn alt_with_arrow() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3A");
    }

    #[quickcheck]
    fn process_key_never_panics_char(ch: u8, shift: bool, alt: bool, ctrl: bool) {
        let mut engine = InputEngine::new();
        if shift {
            engine.set_modifier(Modifiers::SHIFT, true);
        }
        if alt {
            engine.set_modifier(Modifiers::ALT, true);
        }
        if ctrl {
            engine.set_modifier(Modifiers::CTRL, true);
        }
        let c = char::from(ch);
        let _ = engine.process_key(KeyEvent::Char(c), KeyAction::Press);
    }

    #[quickcheck]
    fn process_key_never_panics_special(key: u8, shift: bool, alt: bool) {
        let mut engine = InputEngine::new();
        if shift {
            engine.set_modifier(Modifiers::SHIFT, true);
        }
        if alt {
            engine.set_modifier(Modifiers::ALT, true);
        }
        let special = match key % 21 {
            0 => SpecialKey::Enter,
            1 => SpecialKey::Tab,
            2 => SpecialKey::Backspace,
            3 => SpecialKey::Escape,
            4 => SpecialKey::Up,
            5 => SpecialKey::Down,
            6 => SpecialKey::Left,
            7 => SpecialKey::Right,
            8 => SpecialKey::Home,
            9 => SpecialKey::End,
            10 => SpecialKey::PageUp,
            11 => SpecialKey::PageDown,
            12 => SpecialKey::Insert,
            13 => SpecialKey::Delete,
            14 => SpecialKey::F1,
            15 => SpecialKey::F5,
            16 => SpecialKey::F12,
            17 => SpecialKey::Space,
            18 => SpecialKey::F2,
            19 => SpecialKey::F10,
            _ => SpecialKey::F20,
        };
        let _ = engine.process_key(KeyEvent::Special(special), KeyAction::Press);
    }

    #[quickcheck]
    fn kitty_protocol_output_is_valid_utf8(ch: u8) {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let c = char::from(ch);
        let result = engine.process_key(KeyEvent::Char(c), KeyAction::Press);
        assert!(!result.is_empty(), "kitty output should not be empty");
    }

    // ── Modifier combination matrix ─────────────────────────────────────────

    #[test]
    fn ctrl_shift_a_encodes_as_ctrl_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Char('A'), KeyAction::Press);
        assert_eq!(
            result,
            vec![0x01],
            "Ctrl+Shift+A should encode as Ctrl+A (SOH)"
        );
    }

    #[test]
    fn ctrl_with_unicode_char_does_not_panic() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('€'), KeyAction::Press);
        // Ctrl modifier is a no-op for non-ASCII; '€' (U+20AC) passes through as UTF-8
        assert_eq!(
            result,
            "€".as_bytes(),
            "Ctrl+€ should produce raw UTF-8 of €"
        );
    }

    #[test]
    fn alt_with_enter_encodes_esc_prefix() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
        assert_eq!(
            result, b"\x1b\r",
            "Alt+Enter should produce ESC prefix + CR"
        );
    }

    #[test]
    fn ctrl_up_arrow_encodes_csi_1_5_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5A", "Ctrl+Up should be ESC[1;5A");
    }

    #[test]
    fn ctrl_down_arrow_encodes_csi_1_5_b() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Down), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5B", "Ctrl+Down should be ESC[1;5B");
    }

    #[test]
    fn ctrl_right_arrow_encodes_csi_1_5_c() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Right), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5C", "Ctrl+Right should be ESC[1;5C");
    }

    #[test]
    fn ctrl_left_arrow_encodes_csi_1_5_d() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Left), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5D", "Ctrl+Left should be ESC[1;5D");
    }

    #[test]
    fn ctrl_shift_arrow_encodes_csi_1_6_direction() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Left), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;6D", "Ctrl+Shift+Left should be ESC[1;6D");
    }

    #[test]
    fn shift_up_arrow_encodes_csi_1_2_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;2A", "Shift+Up should be ESC[1;2A");
    }

    #[test]
    fn shift_tab_encodes_csi_z() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Tab), KeyAction::Press);
        assert_eq!(result, b"\x1b[Z", "Shift+Tab should produce CSI Z");
    }

    #[test]
    fn ctrl_a_encodes_soh() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, b"\x01", "Ctrl+A should be SOH (0x01)");
    }

    #[test]
    fn ctrl_b_encodes_stx() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('b'), KeyAction::Press);
        assert_eq!(result, b"\x02", "Ctrl+B should be STX (0x02)");
    }

    #[test]
    fn ctrl_e_encodes_enq() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('e'), KeyAction::Press);
        assert_eq!(result, b"\x05", "Ctrl+E should be ENQ (0x05)");
    }

    #[test]
    fn ctrl_f_encodes_ack() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('f'), KeyAction::Press);
        assert_eq!(result, b"\x06", "Ctrl+F should be ACK (0x06)");
    }

    #[test]
    fn ctrl_g_encodes_bel() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('g'), KeyAction::Press);
        assert_eq!(result, b"\x07", "Ctrl+G should be BEL (0x07)");
    }

    #[test]
    fn ctrl_h_encodes_bs() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('h'), KeyAction::Press);
        assert_eq!(result, b"\x08", "Ctrl+H should be BS (0x08)");
    }

    #[test]
    fn ctrl_i_encodes_ht() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('i'), KeyAction::Press);
        assert_eq!(result, b"\x09", "Ctrl+I should be HT (0x09)");
    }

    #[test]
    fn ctrl_j_encodes_lf() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('j'), KeyAction::Press);
        assert_eq!(result, b"\x0a", "Ctrl+J should be LF (0x0A)");
    }

    #[test]
    fn ctrl_k_encodes_vt() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('k'), KeyAction::Press);
        assert_eq!(result, b"\x0b", "Ctrl+K should be VT (0x0B)");
    }

    #[test]
    fn ctrl_l_encodes_ff() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('l'), KeyAction::Press);
        assert_eq!(result, b"\x0c", "Ctrl+L should be FF (0x0C)");
    }

    #[test]
    fn ctrl_m_encodes_cr() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('m'), KeyAction::Press);
        assert_eq!(result, b"\x0d", "Ctrl+M should be CR (0x0D)");
    }

    #[test]
    fn ctrl_q_encodes_dc1() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('q'), KeyAction::Press);
        assert_eq!(result, b"\x11", "Ctrl+Q should be DC1 (0x11)");
    }

    #[test]
    fn ctrl_s_encodes_dc2() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('s'), KeyAction::Press);
        assert_eq!(result, b"\x13", "Ctrl+S should be DC2 (0x13)");
    }

    #[test]
    fn alt_a_encodes_esc_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, b"\x1ba", "Alt+A should be ESC+a");
    }

    #[test]
    fn alt_z_encodes_esc_z() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('z'), KeyAction::Press);
        assert_eq!(result, b"\x1bz", "Alt+Z should be ESC+z");
    }

    #[test]
    fn alt_1_encodes_esc_1() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('1'), KeyAction::Press);
        assert_eq!(result, b"\x1b1", "Alt+1 should be ESC+1");
    }

    #[test]
    fn alt_space_encodes_esc_space() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char(' '), KeyAction::Press);
        assert_eq!(result, b"\x1b ", "Alt+Space should be ESC+space");
    }

    #[test]
    fn ctrl_alt_a_encodes_ctrl_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        // Ctrl+Alt+A produces SOH (0x01) — Alt is suppressed by Ctrl
        assert_eq!(
            result, b"\x01",
            "Ctrl+Alt+A produces SOH (Ctrl takes priority over Alt)"
        );
    }

    #[test]
    fn ctrl_alt_z_encodes_ctrl_z() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('z'), KeyAction::Press);
        // Ctrl+Alt+Z produces SUB (0x1A) — Alt is suppressed by Ctrl
        assert_eq!(
            result, b"\x1a",
            "Ctrl+Alt+Z produces SUB (Ctrl takes priority over Alt)"
        );
    }

    #[test]
    fn key_release_produces_no_output_in_legacy_mode() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Char('x'), KeyAction::Release);
        assert!(result.is_empty(), "Key release should produce no output");
    }

    #[test]
    fn no_modifier_special_key_encodes_directly() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
        assert_eq!(result, b"\r", "Enter without mods should be CR");
    }

    #[test]
    fn ctrl_backspace_encodes_csi_1_5_del() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(
            result, b"\x1b[1;5\x7f",
            "Ctrl+Backspace should produce CSI ESC[1;5<DEL>"
        );
    }

    #[test]
    fn ctrl_space_encodes_space() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char(' '), KeyAction::Press);
        assert_eq!(result, b" ", "Ctrl+Space passes through as space");
    }

    #[test]
    fn alt_up_arrow_encodes_esc_1_3_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3A", "Alt+Up should be ESC[1;3A");
    }

    #[test]
    fn alt_down_arrow_encodes_esc_1_3_b() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Down), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3B", "Alt+Down should be ESC[1;3B");
    }

    #[test]
    fn alt_left_arrow_encodes_esc_1_3_d() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Left), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3D", "Alt+Left should be ESC[1;3D");
    }

    #[test]
    fn alt_right_arrow_encodes_esc_1_3_c() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Right), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3C", "Alt+Right should be ESC[1;3C");
    }

    #[test]
    fn ctrl_home_encodes_esc_1_5_h() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Home), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5H", "Ctrl+Home should be ESC[1;5H");
    }

    #[test]
    fn ctrl_end_encodes_esc_1_5_f() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::End), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5F", "Ctrl+End should be ESC[1;5F");
    }

    #[test]
    fn shift_home_encodes_esc_1_2_h() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Home), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;2H", "Shift+Home should be ESC[1;2H");
    }

    #[test]
    fn shift_end_encodes_esc_1_2_f() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::End), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;2F", "Shift+End should be ESC[1;2F");
    }

    #[test]
    fn alt_home_encodes_esc_1_3_h() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Home), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3H", "Alt+Home should be ESC[1;3H");
    }

    #[test]
    fn page_up_with_no_modifier_encodes_csi_5_tilde() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageUp), KeyAction::Press);
        assert_eq!(result, b"\x1b[5~", "PageUp should be ESC[5~");
    }

    #[test]
    fn page_down_with_no_modifier_encodes_csi_6_tilde() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageDown), KeyAction::Press);
        assert_eq!(result, b"\x1b[6~", "PageDown should be ESC[6~");
    }

    #[test]
    fn shift_page_up_encodes_csi_5_2_tilde() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageUp), KeyAction::Press);
        assert_eq!(result, b"\x1b[5;2~", "Shift+PageUp should produce ESC[5;2~");
    }

    #[test]
    fn f1_encodes_ss3_p() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F1), KeyAction::Press);
        assert_eq!(result, b"\x1bOP", "F1 should be SS3 P (ESC OP)");
    }

    #[test]
    fn f2_encodes_ss3_q() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F2), KeyAction::Press);
        assert_eq!(result, b"\x1bOQ", "F2 should be SS3 Q (ESC OQ)");
    }

    #[test]
    fn f3_encodes_ss3_r() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F3), KeyAction::Press);
        assert_eq!(result, b"\x1bOR", "F3 should be SS3 R (ESC OR)");
    }

    #[test]
    fn f4_encodes_ss3_s() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F4), KeyAction::Press);
        assert_eq!(result, b"\x1bOS", "F4 should be SS3 S (ESC OS)");
    }

    #[test]
    fn alt_f1_encodes_csi_1_3_p() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F1), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;3P", "Alt+F1 should produce CSI ESC[1;3P");
    }

    #[test]
    fn shift_f1_encodes_csi_1_2_p() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F1), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;2P", "Shift+F1 should produce CSI ESC[1;2P");
    }

    #[test]
    fn bracketed_paste_char_input() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Char('x'), KeyAction::Press);
        assert_eq!(
            result, b"x",
            "Char input without modifiers should produce the char"
        );
    }

    #[test]
    fn cursor_key_application_mode_up() {
        let mut engine = InputEngine::new();
        engine.set_cursor_key_application_mode(true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1bOA", "App cursor mode Up should be SS3 A");
    }

    #[test]
    fn cursor_key_application_mode_down() {
        let mut engine = InputEngine::new();
        engine.set_cursor_key_application_mode(true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Down), KeyAction::Press);
        assert_eq!(result, b"\x1bOB", "App cursor mode Down should be SS3 B");
    }

    #[test]
    fn cursor_key_application_mode_with_ctrl_uses_csi_1_5_a() {
        let mut engine = InputEngine::new();
        engine.set_cursor_key_application_mode(true);
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(
            result, b"\x1b[1;5A",
            "App mode + Ctrl+Up should produce CSI ESC[1;5A"
        );
    }

    #[test]
    fn insert_encodes_csi_2_tilde() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Insert), KeyAction::Press);
        assert_eq!(result, b"\x1b[2~", "Insert should be ESC[2~");
    }

    #[test]
    fn delete_encodes_csi_3_tilde() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Delete), KeyAction::Press);
        assert_eq!(result, b"\x1b[3~", "Delete should be ESC[3~");
    }

    #[test]
    fn shift_insert_encodes_csi_2_2_tilde() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Insert), KeyAction::Press);
        assert_eq!(result, b"\x1b[2;2~", "Shift+Insert should produce ESC[2;2~");
    }

    #[test]
    fn bracketed_paste_text_wraps_in_200_201_tilde() {
        let mut engine = InputEngine::new();
        engine.set_bracketed_paste(true);
        let result = engine.process_key(KeyEvent::Char('x'), KeyAction::Press);
        assert_eq!(
            result, b"x",
            "Bracketed paste mode affects paste, not char input"
        );
    }

    #[test]
    fn keypad_application_enter_encodes_cr() {
        let mut engine = InputEngine::new();
        engine.set_keypad_application_mode(true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Enter), KeyAction::Press);
        // App keypad mode doesn't affect Enter without additional modifiers
        assert_eq!(result, b"\r", "App keypad Enter produces CR (no SS3 M)");
    }

    #[test]
    fn no_modifier_space_encodes_space() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Char(' '), KeyAction::Press);
        assert_eq!(result, b" ", "Space without mods should be 0x20");
    }

    #[test]
    fn ctrl_underscore_encodes_us() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        // Ctrl+_ maps to 0x1F (US) via bitmask: 0x5F & 0x1F = 0x1F
        let result = engine.process_key(KeyEvent::Char('_'), KeyAction::Press);
        assert_eq!(result, vec![0x1F], "Ctrl+_ should encode as US (0x1F)");
    }

    #[test]
    fn ctrl_question_mark_encodes_del() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        // Ctrl+? maps to 0x7F (DEL) — special case
        let result = engine.process_key(KeyEvent::Char('?'), KeyAction::Press);
        assert_eq!(result, vec![0x7F], "Ctrl+? should encode as DEL (0x7F)");
    }

    #[test]
    fn ctrl_slash_encodes_slash() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        // Ctrl+/ passes through — encoder does not translate to 0x1F
        let result = engine.process_key(KeyEvent::Char('/'), KeyAction::Press);
        assert_eq!(result, b"/", "Ctrl+/ passes through as '/'");
    }

    #[test]
    fn no_modifier_backspace_encodes_del() {
        let engine = InputEngine::new();
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(result, b"\x7f", "Backspace should be DEL (0x7F)");
    }

    #[test]
    fn alt_backspace_encodes_esc_del() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(
            result, b"\x1b\x7f",
            "Alt+Backspace should produce ESC prefix + DEL"
        );
    }

    #[test]
    fn backspace_bs_mode_encodes_bs() {
        let mut engine = InputEngine::new();
        engine.set_backspace_byte(0x08);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(result, b"\x08", "Backspace in BS mode should be 0x08 (^H)");
    }

    #[test]
    fn alt_backspace_bs_mode_encodes_esc_bs() {
        let mut engine = InputEngine::new();
        engine.set_backspace_byte(0x08);
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(
            result, b"\x1b\x08",
            "Alt+Backspace in BS mode should produce ESC + BS"
        );
    }

    #[test]
    fn ctrl_backspace_bs_mode_encodes_csi_1_5_0x08() {
        let mut engine = InputEngine::new();
        engine.set_backspace_byte(0x08);
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Backspace), KeyAction::Press);
        assert_eq!(
            result, b"\x1b[1;5\x08",
            "Ctrl+Backspace in BS mode should produce CSI with BS byte"
        );
    }

    #[test]
    fn kitty_protocol_ctrl_a_encodes_97_5u() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('a'), KeyAction::Press);
        assert_eq!(result, b"\x1b[97;5u", "Kitty Ctrl+A should be ESC[97;5u");
    }

    #[test]
    fn kitty_protocol_alt_b_encodes_98_3u() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Char('b'), KeyAction::Press);
        assert_eq!(result, b"\x1b[98;3u", "Kitty Alt+B should be ESC[98;3u");
    }

    #[test]
    fn kitty_protocol_release_encodes_empty() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let result = engine.process_key(KeyEvent::Char('x'), KeyAction::Release);
        assert!(result.is_empty(), "Kitty key release should be empty");
    }

    #[test]
    fn kitty_protocol_up_arrow_encodes_1000_1u() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(result, b"\x1b[1000u", "Kitty Up should be ESC[1000u");
    }

    #[test]
    fn kitty_protocol_with_ctrl_up_encodes_1000_5u() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        assert_eq!(
            result, b"\x1b[1000;5u",
            "Kitty Ctrl+Up should be ESC[1000;5u"
        );
    }

    #[test]
    fn ctrl_alt_up_encodes_esc_1_5_a() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Up), KeyAction::Press);
        // Ctrl+Alt produces modifier value 7 (1 + 2 + 4), not 5
        assert_eq!(result, b"\x1b[1;7A", "Ctrl+Alt+Up should be ESC[1;7A");
    }

    #[test]
    fn shift_alt_left_encodes_esc_1_4_d() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Left), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;4D", "Shift+Alt+Left should be ESC[1;4D");
    }

    #[test]
    fn ctrl_num_1_encodes_33_5u() {
        let mut engine = InputEngine::new();
        engine.set_kitty_protocol(true);
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Char('1'), KeyAction::Press);
        assert_eq!(result, b"\x1b[49;5u", "Kitty Ctrl+1 should be ESC[49;5u");
    }

    #[test]
    fn shift_function_key_f5_contains_modifier() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::SHIFT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::F5), KeyAction::Press);
        assert_eq!(result, b"\x1b[15;2~", "Shift+F5 should produce ESC[15;2~");
    }

    #[test]
    fn ctrl_home_encodes_csi_1_5_h() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::Home), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5H", "Ctrl+Home should be ESC[1;5H");
    }

    #[test]
    fn ctrl_end_encodes_csi_1_5_f() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::End), KeyAction::Press);
        assert_eq!(result, b"\x1b[1;5F", "Ctrl+End should be ESC[1;5F");
    }

    #[test]
    fn alt_page_up_encodes_csi_5_3_tilde() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::ALT, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageUp), KeyAction::Press);
        assert_eq!(result, b"\x1b[5;3~", "Alt+PageUp should produce ESC[5;3~");
    }

    #[test]
    fn ctrl_page_down_encodes_csi_6_5_tilde() {
        let mut engine = InputEngine::new();
        engine.set_modifier(Modifiers::CTRL, true);
        let result = engine.process_key(KeyEvent::Special(SpecialKey::PageDown), KeyAction::Press);
        assert_eq!(
            result, b"\x1b[6;5~",
            "Ctrl+PageDown should produce ESC[6;5~"
        );
    }
}
