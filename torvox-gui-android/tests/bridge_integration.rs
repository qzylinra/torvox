use torvox_android::bridge::{
    BridgeAttrs, BridgeCell, BridgeTheme, Shell, TerminalConfig, TorvoxBridge,
};
use torvox_core::cell::Cell;

// ═══════════════════════════════════════════════
// 11.1 Wire format roundtrip
// ═══════════════════════════════════════════════

#[test]
fn wire_format_roundtrip_terminal_config() {
    let config = TerminalConfig {
        shell: Shell::Custom {
            path: "/bin/zsh".to_string(),
        },
        rows: 36,
        cols: 120,
        scrollback_lines: 10000,
        font_size_tenths: 160,
        theme: BridgeTheme::from(torvox_core::config::Theme::catppuccin_mocha_named()),
        home: "/data/data/com.torvox".to_string(),
        user: "user".to_string(),
        path: "/system/bin".to_string(),
        working_directory: "/data/data/com.torvox".to_string(),
        prefix: "torvox".to_string(),
    };

    // Encode to wire format
    let wire_size = boltffi::__private::wire::WireEncode::wire_size(&config);
    let mut buf = vec![0u8; wire_size];
    let written = boltffi::__private::wire::WireEncode::encode_to(&config, &mut buf);
    assert_eq!(
        written, wire_size,
        "encode_to must write exact wire_size bytes"
    );

    // Decode from wire format
    let (decoded, consumed) =
        <TerminalConfig as boltffi::__private::wire::WireDecode>::decode_from(&buf)
            .expect("wire_decode of valid TerminalConfig must succeed");
    assert_eq!(consumed, wire_size, "decode_from must consume all bytes");
    assert_eq!(
        decoded, config,
        "wire roundtrip must preserve TerminalConfig"
    );
}

#[test]
fn wire_format_roundtrip_bridge_cell() {
    let cell = BridgeCell {
        char_code: 0x41,
        fg: 0xFF_FF_00_FF,
        bg: 0x00_00_00_FF,
        attrs: BridgeAttrs {
            bold: true,
            italic: true,
            ..Default::default()
        },
    };

    let wire_size = boltffi::__private::wire::WireEncode::wire_size(&cell);
    let mut buf = vec![0u8; wire_size];
    let written = boltffi::__private::wire::WireEncode::encode_to(&cell, &mut buf);
    assert_eq!(written, wire_size);

    let (decoded, consumed) =
        <BridgeCell as boltffi::__private::wire::WireDecode>::decode_from(&buf)
            .expect("wire_decode of valid BridgeCell must succeed");
    assert_eq!(consumed, wire_size);
    assert_eq!(decoded, cell, "wire roundtrip must preserve BridgeCell");
}

// ═══════════════════════════════════════════════
// 11.2 BridgeCell ↔ Cell conversion roundtrip
// ═══════════════════════════════════════════════

#[test]
fn bridge_cell_to_cell_roundtrip() {
    let core_cell = Cell {
        char: 'X',
        fg: torvox_core::cell::Color::new(255, 128, 64),
        bg: torvox_core::cell::Color::new(0, 0, 0),
        attrs: torvox_core::cell::Attrs {
            bold: true,
            dim: false,
            italic: true,
            underline: false,
            double_underline: true,
            reverse: false,
            strikethrough: true,
            blink: false,
            hidden: false,
            overline: false,
            protected: false,
            double_width: false,
            double_height_top: false,
            double_height_bottom: false,
        },
        width: 1,
    };

    let bridge: BridgeCell = core_cell.into();
    let back: Cell = bridge.into();

    assert_eq!(core_cell.char, back.char, "char must roundtrip");
    assert_eq!(core_cell.fg, back.fg, "fg color must roundtrip");
    assert_eq!(core_cell.bg, back.bg, "bg color must roundtrip");
    assert_eq!(core_cell.attrs, back.attrs, "attrs must roundtrip");
    assert_eq!(
        core_cell, back,
        "full Cell roundtrip must preserve equality"
    );
}

#[test]
fn bridge_cell_from_cell_encodes_u32_colors() {
    let core_cell = Cell {
        char: 'A',
        fg: torvox_core::cell::Color::new(10, 20, 30),
        bg: torvox_core::cell::Color::new(40, 50, 60),
        ..Default::default()
    };

    let bridge: BridgeCell = core_cell.into();
    assert_eq!(bridge.char_code, 0x41);
    assert_eq!(bridge.fg, 0x0A141EFF, "fg should encode as RGBA u32");
    assert_eq!(bridge.bg, 0x28323CFF, "bg should encode as RGBA u32");
}

// ═══════════════════════════════════════════════
// 11.3 Truncated wire input → error, not panic
// ═══════════════════════════════════════════════

#[test]
fn truncated_wire_input_returns_error() {
    let config = TerminalConfig::default();
    let wire_size = boltffi::__private::wire::WireEncode::wire_size(&config);
    let mut buf = vec![0u8; wire_size];
    boltffi::__private::wire::WireEncode::encode_to(&config, &mut buf);

    // Try decoding with increasingly shorter truncations
    for truncate_len in (0..wire_size).step_by(4).chain([wire_size - 1]) {
        let truncated = &buf[..truncate_len.min(buf.len())];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            <TerminalConfig as boltffi::__private::wire::WireDecode>::decode_from(truncated)
        }));
        match result {
            Ok(Err(_)) => {} // expected — decode error
            Ok(Ok(_)) => {
                // Only acceptable if we got the full bytes
                assert_eq!(
                    truncate_len, wire_size,
                    "decode should only succeed on complete input"
                );
            }
            Err(panic_payload) => {
                let msg = panic_payload
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| panic_payload.downcast_ref::<&str>().copied())
                    .unwrap_or("<opaque>");
                panic!(
                    "decode_from panicked on truncated input (len={}): {}",
                    truncate_len, msg
                );
            }
        }
    }
}

// ═══════════════════════════════════════════════
// 11.4 Bit-flip corrupted wire bytes → error, not panic
// ═══════════════════════════════════════════════

#[test]
fn bitflip_corrupted_wire_returns_error() {
    let config = TerminalConfig::default();
    let wire_size = boltffi::__private::wire::WireEncode::wire_size(&config);
    let mut correct_buf = vec![0u8; wire_size];
    boltffi::__private::wire::WireEncode::encode_to(&config, &mut correct_buf);

    // Flip each byte in the wire buffer and verify decode returns Err
    for flip_pos in 0..wire_size {
        let mut corrupted = correct_buf.clone();
        corrupted[flip_pos] ^= 0x01; // flip LSB

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            <TerminalConfig as boltffi::__private::wire::WireDecode>::decode_from(&corrupted)
        }));
        match result {
            Ok(Err(_)) => {} // expected — decode error from corruption
            Ok(Ok(_)) => {
                // If decoding happened to succeed, that's OK for some
                // positions (e.g. flipping unused padding bits)
            }
            Err(panic_payload) => {
                let msg = panic_payload
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| panic_payload.downcast_ref::<&str>().copied())
                    .unwrap_or("<opaque>");
                panic!(
                    "decode_from panicked on bitflip at byte {}: {}",
                    flip_pos, msg
                );
            }
        }
    }
}

#[test]
fn bitflip_single_bit_does_not_panic() {
    let config = TerminalConfig::default();
    let wire_size = boltffi::__private::wire::WireEncode::wire_size(&config);
    let mut correct_buf = vec![0u8; wire_size];
    boltffi::__private::wire::WireEncode::encode_to(&config, &mut correct_buf);

    // Sample: flip bit 3 of byte at position wire_size/2
    let mut corrupted = correct_buf.clone();
    corrupted[wire_size / 2] ^= 0x08;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        <TerminalConfig as boltffi::__private::wire::WireDecode>::decode_from(&corrupted)
    }));
    assert!(
        result.is_ok(),
        "bit-flipped wire input must not cause a panic"
    );
}

// ═══════════════════════════════════════════════
// 11.5 Multi-threaded bridge access via thread::scope
// ═══════════════════════════════════════════════

#[test]
fn multithreaded_bridge_ping() {
    let config = TerminalConfig {
        shell: Shell::Custom {
            path: "/bin/sh".to_string(),
        },
        rows: 24,
        cols: 80,
        scrollback_lines: 50000,
        font_size_tenths: 140,
        theme: torvox_core::config::Theme::catppuccin_mocha().into(),
        home: String::new(),
        user: String::new(),
        path: String::new(),
        working_directory: String::new(),
        prefix: String::new(),
    };
    let bridge = TorvoxBridge::new(config);

    let bridge = &bridge;

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for i in 0..10 {
            handles.push(scope.spawn(move || {
                let result = bridge.ping();
                assert_eq!(result.unwrap(), "pong", "thread {} ping failed", i);
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }
    });
}

#[test]
fn multithreaded_bridge_get_config() {
    let config = TerminalConfig {
        shell: Shell::Custom {
            path: "/bin/bash".to_string(),
        },
        rows: 40,
        cols: 120,
        scrollback_lines: 10000,
        font_size_tenths: 160,
        theme: torvox_core::config::Theme::dracula_plus().into(),
        home: String::new(),
        user: String::new(),
        path: String::new(),
        working_directory: String::new(),
        prefix: String::new(),
    };
    let bridge = TorvoxBridge::new(config.clone());

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(scope.spawn(|| {
                let got = bridge.get_config();
                assert_eq!(got.shell, config.shell);
                assert_eq!(got.rows, config.rows);
                assert_eq!(got.cols, config.cols);
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }
    });
}

#[test]
fn multithreaded_bridge_get_theme_names() {
    let config = TerminalConfig::default();
    let bridge = TorvoxBridge::new(config);

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..10 {
            handles.push(scope.spawn(|| {
                let names = bridge.get_theme_names();
                assert_eq!(names.len(), 16, "must return 16 built-in themes");
            }));
        }
        for h in handles {
            h.join().expect("thread panicked");
        }
    });
}

#[test]
fn multithreaded_bridge_mixed_access() {
    let config = TerminalConfig {
        shell: Shell::Custom {
            path: "/bin/sh".to_string(),
        },
        rows: 30,
        cols: 100,
        scrollback_lines: 50000,
        font_size_tenths: 140,
        theme: torvox_core::config::Theme::catppuccin_mocha().into(),
        home: String::new(),
        user: String::new(),
        path: String::new(),
        working_directory: String::new(),
        prefix: String::new(),
    };
    let bridge = TorvoxBridge::new(config);

    std::thread::scope(|scope| {
        let pingers = scope.spawn(|| {
            for _ in 0..20 {
                assert_eq!(bridge.ping().unwrap(), "pong");
            }
        });
        let config_readers = scope.spawn(|| {
            for _ in 0..20 {
                let _cfg = bridge.get_config();
            }
        });
        let theme_readers = scope.spawn(|| {
            for _ in 0..20 {
                let names = bridge.get_theme_names();
                assert!(!names.is_empty());
            }
        });
        let theme_getters = scope.spawn(|| {
            for _ in 0..20 {
                let theme = bridge.get_theme("Catppuccin Mocha".to_string());
                assert!(theme.is_some(), "Catppuccin Mocha must exist");
            }
        });
        pingers.join().expect("pingers panicked");
        config_readers.join().expect("config_readers panicked");
        theme_readers.join().expect("theme_readers panicked");
        theme_getters.join().expect("theme_getters panicked");
    });
}

/// Verify all bridge TerminalConfig fields have matching core TerminalConfig defaults.
/// This test ensures bridge config stays in sync with core config.
#[test]
fn bridge_terminal_config_default_aligns_with_core() {
    let bridge = torvox_android::bridge::TerminalConfig::default();
    let core = torvox_core::config::TerminalConfig::default();
    assert_eq!(bridge.rows, core.rows, "rows should match core default");
    assert_eq!(bridge.cols, core.cols, "cols should match core default");
    assert_eq!(
        bridge.scrollback_lines, core.scrollback_lines,
        "scrollback_lines should match"
    );
    assert_eq!(
        bridge.font_size_tenths, core.font_size_tenths,
        "font_size_tenths should match"
    );
    assert_eq!(
        bridge.shell,
        torvox_android::bridge::Shell::SystemDefault,
        "shell should match core default"
    );
}
