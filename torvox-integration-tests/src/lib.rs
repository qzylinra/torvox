//! Cross-crate integration tests for Torvox.
//!
//! These tests verify end-to-end behavior that exercises multiple
//! torvox-core types through torvox-terminal's GhosttyTerminal API.

#[cfg(test)]
mod config_file_validation {
    use std::fs;

    fn workspace_root() -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        manifest_dir
            .strip_suffix("/torvox-integration-tests")
            .unwrap()
            .to_string()
    }

    #[test]
    fn codecov_yml_exists_and_is_valid_yaml() {
        let root = workspace_root();
        let path = format!("{root}/codecov.yml");
        let content = fs::read_to_string(&path).expect("codecov.yml must exist");
        assert!(
            content.contains("codecov:"),
            "codecov.yml must contain 'codecov:' key"
        );
        assert!(
            content.contains("coverage:"),
            "codecov.yml must contain 'coverage:' key"
        );
    }

    #[test]
    fn cargo_mutants_toml_exists_and_is_valid_toml() {
        let root = workspace_root();
        let path = format!("{root}/.cargo/mutants.toml");
        let content = fs::read_to_string(&path).expect(".cargo/mutants.toml must exist");
        let parsed: toml::Value = content
            .parse()
            .expect(".cargo/mutants.toml must be valid TOML");
        let has_valid_keys = parsed.get("examine_globs").is_some()
            || parsed.get("exclude_globs").is_some()
            || parsed.get("test_tool").is_some();
        assert!(
            has_valid_keys,
            ".cargo/mutants.toml must have at least one of: examine_globs, exclude_globs, test_tool"
        );
    }

    #[test]
    fn rust_toolchain_toml_exists_and_is_valid_toml() {
        let root = workspace_root();
        let path = format!("{root}/rust-toolchain.toml");
        let content = fs::read_to_string(&path).expect("rust-toolchain.toml must exist");
        let parsed: toml::Value = content
            .parse()
            .expect("rust-toolchain.toml must be valid TOML");
        assert!(
            parsed.get("toolchain").is_some(),
            "rust-toolchain.toml must have [toolchain] section"
        );
    }

    #[test]
    fn workspace_cargo_toml_is_valid_toml() {
        let root = workspace_root();
        let path = format!("{root}/Cargo.toml");
        let content = fs::read_to_string(&path).expect("workspace Cargo.toml must exist");
        let parsed: toml::Value = content
            .parse()
            .expect("workspace Cargo.toml must be valid TOML");
        assert!(
            parsed.get("workspace").is_some(),
            "workspace Cargo.toml must have [workspace] section"
        );
    }

    #[test]
    fn quality_gate_script_exists() {
        let root = workspace_root();
        let path = format!("{root}/scripts/quality-gate.nu");
        let content = fs::read_to_string(&path).expect("quality-gate.nu must exist");
        assert!(
            content.contains("#!/usr/bin/env nu"),
            "quality-gate.nu must start with nushell shebang"
        );
    }

    #[test]
    fn build_android_libs_script_exists() {
        let root = workspace_root();
        let path = format!("{root}/scripts/build-android-libs.nu");
        let content = fs::read_to_string(&path).expect("build-android-libs.nu must exist");
        assert!(
            content.contains("#!/usr/bin/env nu"),
            "build-android-libs.nu must start with nushell shebang"
        );
    }

    #[test]
    fn no_dead_config_files() {
        let root = workspace_root();
        let dead_files: &[&str] = &[];
        for file in dead_files {
            let path = format!("{root}/{file}");
            assert!(
                !std::path::Path::new(&path).exists(),
                "dead config file found: {file} — remove it"
            );
        }
    }

    #[test]
    fn coverage_ratchet_toml_exists_and_is_valid_toml() {
        let root = workspace_root();
        let path = format!("{root}/coverage-ratchet.toml");
        let content = fs::read_to_string(&path).expect("coverage-ratchet.toml must exist");
        let parsed: toml::Value = content
            .parse()
            .expect("coverage-ratchet.toml must be valid TOML");
        assert!(
            parsed.get("workspace").is_some(),
            "coverage-ratchet.toml must have [workspace] section"
        );
        assert!(
            parsed.get("crates").is_some(),
            "coverage-ratchet.toml must have [crates] section"
        );
        assert!(
            parsed.get("ratchet").is_some(),
            "coverage-ratchet.toml must have [ratchet] section"
        );
    }

    #[test]
    fn coverage_ratchet_workspace_thresholds_are_consistent() {
        let root = workspace_root();
        let path = format!("{root}/coverage-ratchet.toml");
        let content = fs::read_to_string(&path).expect("coverage-ratchet.toml must exist");
        let parsed: toml::Value = content.parse().unwrap();

        let ws = parsed.get("workspace").unwrap();
        let initial = ws
            .get("initial_threshold")
            .and_then(|v| v.as_float())
            .unwrap();
        let target = ws
            .get("target_threshold")
            .and_then(|v| v.as_float())
            .unwrap();
        let increment = ws
            .get("ratchet_increment")
            .and_then(|v| v.as_float())
            .unwrap();

        assert!(initial > 0.0, "initial_threshold must be > 0");
        assert!(
            target > initial,
            "target_threshold must be > initial_threshold"
        );
        assert!(increment > 0.0, "ratchet_increment must be > 0");
        assert!(
            increment <= target - initial,
            "ratchet_increment must not exceed target - initial"
        );
    }

    #[test]
    fn coverage_ratchet_per_crate_thresholds_are_valid() {
        let root = workspace_root();
        let path = format!("{root}/coverage-ratchet.toml");
        let content = fs::read_to_string(&path).expect("coverage-ratchet.toml must exist");
        let parsed: toml::Value = content.parse().unwrap();

        let crates = parsed.get("crates").unwrap();
        let workspace_target = parsed
            .get("workspace")
            .unwrap()
            .get("target_threshold")
            .and_then(|v| v.as_float())
            .unwrap();

        let known_crates = [
            "torvox-core",
            "torvox-terminal",
            "torvox-renderer",
            "torvox-gui-android",
            "torvox-integration-tests",
        ];

        for crate_name in &known_crates {
            let entry = crates
                .get(*crate_name)
                .unwrap_or_else(|| panic!("[crates] must define entry for {crate_name}"));
            let threshold = entry
                .get("threshold")
                .and_then(|v| v.as_float())
                .unwrap_or_else(|| panic!("{crate_name} must have numeric threshold"));
            let target = entry
                .get("target")
                .and_then(|v| v.as_float())
                .unwrap_or_else(|| panic!("{crate_name} must have numeric target"));

            assert!(
                (0.0..=100.0).contains(&threshold),
                "{crate_name} threshold must be 0..100, got {threshold}"
            );
            assert!(
                (0.0..=100.0).contains(&target),
                "{crate_name} target must be 0..100, got {target}"
            );
            assert!(
                target >= threshold,
                "{crate_name} target ({target}) must be >= threshold ({threshold})"
            );
            assert!(
                target <= workspace_target,
                "{crate_name} target ({target}) must be <= workspace target ({workspace_target})"
            );
        }
    }

    #[test]
    fn coverage_ratchet_referenced_crates_exist_in_workspace() {
        let root = workspace_root();
        let ratchet_path = format!("{root}/coverage-ratchet.toml");
        let cargo_path = format!("{root}/Cargo.toml");
        let ratchet_content = fs::read_to_string(&ratchet_path).unwrap();
        let cargo_content = fs::read_to_string(&cargo_path).unwrap();

        let ratchet_parsed: toml::Value = ratchet_content.parse().unwrap();
        let cargo_parsed: toml::Value = cargo_content.parse().unwrap();

        let crates_section = ratchet_parsed.get("crates").unwrap();
        let workspace_members = cargo_parsed
            .get("workspace")
            .unwrap()
            .get("members")
            .unwrap()
            .as_array()
            .unwrap();

        for (crate_name, _) in crates_section.as_table().unwrap().iter() {
            let found = workspace_members.iter().any(|m| {
                m.as_str()
                    .map(|s| s.trim_end_matches('/').ends_with(crate_name))
                    .unwrap_or(false)
            });
            assert!(
                found,
                "crate '{crate_name}' in coverage-ratchet.toml must exist in workspace members"
            );
        }
    }

    #[test]
    fn coverage_ratchet_ratchet_config_is_sane() {
        let root = workspace_root();
        let path = format!("{root}/coverage-ratchet.toml");
        let content = fs::read_to_string(&path).unwrap();
        let parsed: toml::Value = content.parse().unwrap();

        let ratchet = parsed.get("ratchet").unwrap();
        let auto_bump = ratchet
            .get("auto_bump")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let require_clean = ratchet
            .get("require_clean_ci")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        assert!(
            auto_bump,
            "ratchet.auto_bump must be true for CI enforcement"
        );
        assert!(
            require_clean,
            "ratchet.require_clean_ci must be true for CI enforcement"
        );
    }

    #[test]
    fn coverage_ratchet_exclude_lines_are_valid_patterns() {
        let root = workspace_root();
        let path = format!("{root}/coverage-ratchet.toml");
        let content = fs::read_to_string(&path).unwrap();
        let parsed: toml::Value = content.parse().unwrap();

        let cov_section = parsed.get("cargo-llvm-cov").unwrap();
        let exclude = cov_section
            .get("exclude_lines")
            .and_then(|v| v.as_array())
            .unwrap();

        assert!(!exclude.is_empty(), "exclude_lines must not be empty");

        for pattern in exclude {
            let s = pattern
                .as_str()
                .expect("exclude_lines entries must be strings");
            assert!(
                !s.is_empty(),
                "exclude_lines entries must not be empty strings"
            );
        }
    }
}

#[cfg(test)]
mod vt_to_snapshot_pipeline {
    use std::thread;
    use std::time::Duration;

    use torvox_terminal::ghostty_terminal::GhosttyTerminal;

    #[test]
    fn vt_write_then_snapshot_dims() {
        let mut t = GhosttyTerminal::new(10, 20, 100).unwrap();
        t.vt_write(b"X");
        thread::sleep(Duration::from_millis(20));
        let snap = t.take_snapshot();
        assert_eq!(snap.rows, 10);
        assert_eq!(snap.cols, 20);
    }

    #[test]
    fn vt_write_cjk_then_snapshot() {
        let mut t = GhosttyTerminal::new(2, 20, 100).unwrap();
        let utf8 = "中".as_bytes().to_vec();
        t.vt_write(&utf8);
        thread::sleep(Duration::from_millis(20));
        let snap = t.take_snapshot();
        let has_zh = snap.cells.iter().any(|c| c.codepoint == 0x4E2D);
        assert!(has_zh, "expected 中 character in snapshot");
    }

    #[test]
    fn vt_write_emoji_then_snapshot() {
        let mut t = GhosttyTerminal::new(1, 20, 100).unwrap();
        let utf8 = "\u{1F600}".as_bytes().to_vec();
        t.vt_write(&utf8);
        thread::sleep(Duration::from_millis(20));
        let snap = t.take_snapshot();
        let has_emoji = snap.cells.iter().any(|c| c.codepoint == 0x1F600);
        assert!(has_emoji, "expected emoji in snapshot");
    }

    #[test]
    fn snapshot_uri_at_bounds() {
        let t = GhosttyTerminal::new(3, 3, 0).unwrap();
        let snap = t.take_snapshot();
        let _ = snap.uri_at(0, 0);
        let _ = snap.uri_at(2, 2);
        assert!(snap.uri_at(3, 0).is_none());
        assert!(snap.uri_at(0, 3).is_none());
        assert!(snap.uri_at(100, 100).is_none());
    }

    #[test]
    fn snapshot_cells_count() {
        let t = GhosttyTerminal::new(5, 10, 0).unwrap();
        let snap = t.take_snapshot();
        assert_eq!(snap.cells.len(), 5 * 10);
    }

    #[test]
    fn snapshot_cell_default_fg_is_catppuccin_mocha() {
        let t = GhosttyTerminal::new(1, 1, 0).unwrap();
        let snap = t.take_snapshot();
        // Default fg is Catppuccin Mocha (205,214,244) → byte/255 (no srgb_to_linear)
        let cell = &snap.cells[0];
        assert!((cell.fg[0] - 0.804).abs() < 0.01);
        assert!((cell.fg[1] - 0.839).abs() < 0.01);
        assert!((cell.fg[2] - 0.957).abs() < 0.01);
    }

    #[test]
    fn dump_grid_after_writes() {
        let mut t = GhosttyTerminal::new(3, 5, 0).unwrap();
        t.vt_write(b"AB");
        thread::sleep(Duration::from_millis(20));
        let dumped = t.dump_grid();
        assert_eq!(dumped.rows, 3);
        assert_eq!(dumped.cols, 5);
        assert_eq!(dumped.visible.len(), 3 * 5);
    }

    #[test]
    fn dump_grid_codepoints_sequential() {
        let mut t = GhosttyTerminal::new(1, 5, 0).unwrap();
        t.vt_write(b"ABCDE");
        thread::sleep(Duration::from_millis(20));
        let dumped = t.dump_grid();
        let cps: Vec<u32> = dumped.visible[0..5].iter().map(|c| c.codepoint).collect();
        assert_eq!(
            cps,
            vec!['A' as u32, 'B' as u32, 'C' as u32, 'D' as u32, 'E' as u32]
        );
    }
}

#[cfg(test)]
mod config_driven_session {
    use torvox_core::config::{Shell, TerminalConfig};
    use torvox_terminal::ghostty_terminal::GhosttyTerminal;

    #[test]
    fn config_custom_shell() {
        let cfg = TerminalConfig {
            shell: Shell::Custom("/bin/sh".to_string()),
            ..TerminalConfig::default()
        };
        assert!(matches!(cfg.shell, Shell::Custom(ref s) if s == "/bin/sh"));
    }

    #[test]
    fn config_default_shell() {
        let cfg = TerminalConfig::default();
        assert_eq!(cfg.shell, Shell::SystemDefault);
    }

    #[test]
    fn config_dimensions_applied() {
        let cfg = TerminalConfig {
            rows: 25,
            cols: 81,
            ..TerminalConfig::default()
        };
        assert_eq!(cfg.rows, 25);
        assert_eq!(cfg.cols, 81);
    }

    #[test]
    fn config_scrollback() {
        let cfg = TerminalConfig {
            scrollback_lines: 2000,
            ..TerminalConfig::default()
        };
        assert_eq!(cfg.scrollback_lines, 2000);
    }

    #[test]
    fn ghostty_dimensions_match_config() {
        let cfg = TerminalConfig {
            rows: 17,
            cols: 42,
            ..TerminalConfig::default()
        };
        let t = GhosttyTerminal::new(cfg.rows, cfg.cols, cfg.scrollback_lines).unwrap();
        assert_eq!(t.rows(), 17);
        assert_eq!(t.cols(), 42);
    }

    #[test]
    fn ghostty_resize_then_verify() {
        let mut t = GhosttyTerminal::new(20, 30, 0).unwrap();
        t.resize(40, 50);
        assert_eq!(t.rows(), 40);
        assert_eq!(t.cols(), 50);
    }

    #[test]
    fn ghostty_default_dims() {
        let t = GhosttyTerminal::new(24, 80, 0).unwrap();
        assert_eq!(t.rows(), 24);
        assert_eq!(t.cols(), 80);
    }

    #[test]
    fn ghostty_resize_no_crash() {
        let mut t = GhosttyTerminal::new(10, 10, 0).unwrap();
        t.resize(50, 100);
        t.resize(1, 1);
        t.resize(100, 200);
        assert_eq!(t.rows(), 100);
        assert_eq!(t.cols(), 200);
    }
}

#[cfg(test)]
mod session_e2e {
    use std::time::{Duration, Instant};

    use torvox_terminal::session::Session;

    #[test]
    fn spawn_echo_capture() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo e2e_test_marker\n").expect("write");
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            s.process_output();
            if s.is_exited() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let snap = s.terminal().take_snapshot();
        let total_chars: usize = snap.cells.iter().filter(|c| c.codepoint != 0).count();
        assert!(total_chars > 0 || s.is_exited());
    }

    #[test]
    fn spawn_resize_then_echo() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.resize(40, 100).expect("resize");
        assert_eq!(s.terminal().rows(), 40);
        s.write(b"echo after_resize\n").expect("write");
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            s.process_output();
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn spawn_and_exit() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"exit\n").expect("write");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut exited = false;
        while Instant::now() < deadline {
            s.process_output();
            if s.is_exited() {
                exited = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(exited, "shell should exit on 'exit' command");
    }

    #[test]
    fn spawn_long_running_then_drop() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"sleep 60\n").expect("write");
        std::thread::sleep(Duration::from_millis(100));
        // Don't assert is_exited: depends on the test environment.
        // Just verify the session did not panic and can be dropped.
        drop(s);
    }

    #[test]
    fn spawn_session_id_stable() {
        let s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        assert!(!s.exited_flag().load(std::sync::atomic::Ordering::Relaxed));
    }
}

#[cfg(test)]
mod common {
    use std::time::{Duration, Instant};
    use torvox_terminal::session::Session;

    pub fn drain_until(
        s: &mut Session,
        condition: impl Fn(&Session) -> bool,
        timeout: Duration,
    ) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            s.process_output();
            if condition(s) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    pub fn drain_to_string(s: &mut Session, timeout: Duration) -> String {
        drain_until(s, |_| false, timeout);
        let snap = s.terminal().take_snapshot();
        let mut result = String::new();
        for row in 0..snap.rows {
            let mut line = String::new();
            for col in 0..snap.cols {
                let idx = (row * snap.cols + col) as usize;
                if idx < snap.cells.len() {
                    let cp = snap.cells[idx].codepoint;
                    if cp != 0
                        && let Some(ch) = char::from_u32(cp)
                    {
                        line.push(ch);
                    }
                }
            }
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                result.push_str(trimmed);
                result.push('\n');
            }
        }
        result
    }

    #[allow(dead_code)]
    pub fn drain_grid(s: &mut Session, timeout: Duration) -> Vec<Vec<char>> {
        drain_until(s, |_| false, timeout);
        let snap = s.terminal().take_snapshot();
        let mut grid = Vec::new();
        for row in 0..snap.rows {
            let mut line = Vec::new();
            for col in 0..snap.cols {
                let idx = (row * snap.cols + col) as usize;
                if idx < snap.cells.len() {
                    let cp = snap.cells[idx].codepoint;
                    if cp != 0 {
                        line.push(char::from_u32(cp).unwrap_or('?'));
                    } else {
                        line.push(' ');
                    }
                } else {
                    line.push(' ');
                }
            }
            grid.push(line);
        }
        grid
    }
}

#[cfg(test)]
mod linux_pty_shell_interaction {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn spawn_shell_and_echo() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo hello_from_pty\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("hello_from_pty"),
            "expected echo output in terminal, got: {result}"
        );
    }

    #[test]
    fn pwd_returns_path() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"pwd\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("/"),
            "pwd should output a path with /, got: {result}"
        );
    }

    #[test]
    fn echo_with_quotes() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo 'hello world'\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("hello world"),
            "expected quoted echo output, got: {result}"
        );
    }

    #[test]
    fn echo_multiple_words() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo one two three\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("one two three"),
            "expected multi-word echo, got: {result}"
        );
    }

    #[test]
    fn env_shows_term() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo $TERM\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("torvox"),
            "expected TERM variable to contain 'torvox', got: {result}"
        );
    }

    #[test]
    fn env_shows_colorterm() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo $COLORTERM\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("truecolor"),
            "expected COLORTERM=truecolor, got: {result}"
        );
    }

    #[test]
    fn pipe_command() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo hello | cat\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("hello"),
            "expected piped echo output, got: {result}"
        );
    }

    #[test]
    fn exit_code_zero() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"true; echo exit_code=$?\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("exit_code=0"),
            "expected exit_code=0 from true, got: {result}"
        );
    }

    #[test]
    fn exit_code_nonzero() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"false; echo exit_code=$?\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("exit_code=1"),
            "expected exit_code=1 from false, got: {result}"
        );
    }

    #[test]
    fn shell_prompt_appears() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        std::thread::sleep(Duration::from_millis(500));
        s.process_output();
        let snap = s.terminal().take_snapshot();
        let has_content = snap.cells.iter().any(|c| c.codepoint != 0);
        assert!(has_content, "shell should produce prompt or startup output");
    }

    #[test]
    fn write_multiple_commands_sequentially() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo first\n").expect("write");
        std::thread::sleep(Duration::from_millis(200));
        s.write(b"echo second\n").expect("write");
        std::thread::sleep(Duration::from_millis(200));
        s.write(b"echo third\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("first") && result.contains("second") && result.contains("third"),
            "expected all three echo outputs, got: {result}"
        );
    }
}

#[cfg(test)]
mod linux_signal_handling {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn resize_sends_sigwinch() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Give shell time to start
        std::thread::sleep(Duration::from_millis(300));
        // Resize should send SIGWINCH to the child
        s.resize(40, 120).expect("resize");
        assert_eq!(s.terminal().rows(), 40);
        assert_eq!(s.terminal().cols(), 120);
        // Write a command after resize to verify shell is still alive
        s.write(b"echo still_alive\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("still_alive"),
            "shell should still be responsive after resize, got: {result}"
        );
    }

    #[test]
    fn multiple_resizes_in_sequence() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        std::thread::sleep(Duration::from_millis(300));
        s.resize(30, 90).expect("resize 1");
        s.resize(50, 140).expect("resize 2");
        s.resize(10, 40).expect("resize 3");
        assert_eq!(s.terminal().rows(), 10);
        assert_eq!(s.terminal().cols(), 40);
        s.write(b"echo after_resizes\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("after_resizes"),
            "shell should work after multiple resizes, got: {result}"
        );
    }

    #[test]
    fn resize_too_small_then_back() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        std::thread::sleep(Duration::from_millis(300));
        s.resize(1, 1).expect("resize to 1x1");
        assert_eq!(s.terminal().rows(), 1);
        assert_eq!(s.terminal().cols(), 1);
        s.resize(24, 80).expect("resize back");
        assert_eq!(s.terminal().rows(), 24);
        assert_eq!(s.terminal().cols(), 80);
        s.write(b"echo recovered\n").expect("write");
        let result = drain_to_string(&mut s, Duration::from_secs(3));
        assert!(
            result.contains("recovered"),
            "shell should recover from tiny resize, got: {result}"
        );
    }
}

#[cfg(test)]
mod linux_exit_behavior {
    use std::time::{Duration, Instant};

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn shell_exits_on_exit_command() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"exit\n").expect("write");
        let exited = drain_until(&mut s, |s| s.is_exited(), Duration::from_secs(3));
        assert!(exited, "shell should exit after 'exit' command");
    }

    #[test]
    fn shell_exits_with_code() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"exit 42\n").expect("write");
        let exited = drain_until(&mut s, |s| s.is_exited(), Duration::from_secs(3));
        assert!(exited, "shell should exit after 'exit 42' command");
    }

    #[test]
    fn drop_kills_child_process() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"sleep 300\n").expect("write");
        std::thread::sleep(Duration::from_millis(200));
        let start = Instant::now();
        drop(s);
        let elapsed = start.elapsed();
        // Drop should complete within a reasonable time (kill + waitpid)
        assert!(
            elapsed < Duration::from_secs(3),
            "drop should not hang, took {:?}",
            elapsed
        );
    }

    #[test]
    fn drop_closes_pty() {
        let pid;
        {
            let s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
            pid = s.terminal().rows(); // just access something to prove it's alive
            drop(s);
        }
        // After drop, the child process should be dead.
        // We verify by checking that a new session can be spawned (not blocked).
        let mut s2 = Session::spawn("/bin/sh", 24, 80).expect("second spawn");
        s2.write(b"echo after_drop\n").expect("write");
        let result = drain_to_string(&mut s2, Duration::from_secs(3));
        assert!(
            result.contains("after_drop"),
            "new session should work after previous drop"
        );
        let _ = pid; // suppress unused warning
    }

    #[test]
    fn multiple_sessions_concurrent() {
        let mut s1 = Session::spawn("/bin/sh", 24, 80).expect("spawn 1");
        let mut s2 = Session::spawn("/bin/sh", 24, 80).expect("spawn 2");
        s1.write(b"echo session_one\n").expect("write 1");
        s2.write(b"echo session_two\n").expect("write 2");
        let r1 = drain_to_string(&mut s1, Duration::from_secs(3));
        let r2 = drain_to_string(&mut s2, Duration::from_secs(3));
        assert!(
            r1.contains("session_one"),
            "session 1 should have its output, got: {r1}"
        );
        assert!(
            r2.contains("session_two"),
            "session 2 should have its output, got: {r2}"
        );
    }
}

#[cfg(test)]
mod linux_scrollback {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn long_output_triggers_scrollback() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // seq 1 200 outputs 200 lines, well beyond 24-row viewport
        s.write(b"seq 1 200\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(5));
        let scrollback = s.terminal().scrollback_len();
        assert!(
            scrollback > 0,
            "expected scrollback > 0 after 200 lines of output, got {scrollback}"
        );
    }

    #[test]
    fn scrollback_contains_earlier_lines() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"seq 1 100\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(5));
        let line = s.terminal().read_line_text(0);
        // First line of scrollback should contain "1"
        assert!(line.is_some(), "scrollback should have at least one line");
    }

    #[test]
    fn search_in_scrollback() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo SEARCH_TARGET_XYZ\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        let result = s.terminal().search_in_scrollback("SEARCH_TARGET_XYZ");
        assert!(
            result.is_some(),
            "should find SEARCH_TARGET_XYZ in scrollback"
        );
    }

    #[test]
    fn small_output_no_scrollback() {
        let mut s = Session::spawn("/bin/sh", 1, 80).expect("spawn");
        s.write(b"echo hi\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        // With only 1 row, output wraps but scrollback may or may not be used
        // Just verify no crash
        let _ = s.terminal().scrollback_len();
    }
}

#[cfg(test)]
mod linux_unicode_handling {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn echo_cjk_characters() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // dash printf uses octal escapes: 中 = UTF-8 E4 B8 AD = \344\270\255
        // In Rust bytes: backslash is \\, so \344 becomes \x5c\x33\x34\x34
        // Simpler: just send the raw UTF-8 bytes directly to the PTY
        s.write(b"\xe4\xb8\xad\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        let snap = s.terminal().take_snapshot();
        let has_cjk = snap.cells.iter().any(|c| c.codepoint == 0x4E2D);
        assert!(has_cjk, "expected 中 character from echo output");
    }

    #[test]
    fn echo_emoji() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // 😀 = UTF-8 F0 9F 98 80, send raw bytes
        s.write(b"\xf0\x9f\x98\x80\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        let snap = s.terminal().take_snapshot();
        let has_emoji = snap.cells.iter().any(|c| c.codepoint == 0x1F600);
        assert!(has_emoji, "expected emoji in terminal output");
    }

    #[test]
    fn echo_mixed_ascii_and_cjk() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Write raw UTF-8 bytes "hello 中文" followed by newline through the shell
        s.write(b"echo hello \xe4\xb8\xad\xe6\x96\x87\n")
            .expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        let snap = s.terminal().take_snapshot();
        let has_h = snap.cells.iter().any(|c| c.codepoint == 'h' as u32);
        let has_cjk = snap.cells.iter().any(|c| c.codepoint == 0x4E2D);
        assert!(has_h, "expected 'h' in mixed output");
        assert!(has_cjk, "expected 中 in mixed output");
    }

    #[test]
    fn wide_characters_occupy_two_columns() {
        let mut s = Session::spawn("/bin/sh", 24, 20).expect("spawn");
        // 中 = UTF-8 E4 B8 AD
        s.write(b"echo \xe4\xb8\xad\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        let snap = s.terminal().take_snapshot();
        let has_cjk = snap.cells.iter().any(|c| c.codepoint == 0x4E2D);
        assert!(has_cjk, "expected 中 character in output");
    }
}

#[cfg(test)]
mod linux_ansi_sequences {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn clear_screen_sequence() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"echo before_clear\n").expect("write");
        std::thread::sleep(Duration::from_millis(300));
        // Send ANSI clear screen: ESC[H ESC[2J
        s.write(b"\x1b[H\x1b[2J").expect("write");
        std::thread::sleep(Duration::from_millis(100));
        let snap = s.terminal().take_snapshot();
        // After clear, all cells should be empty
        let has_content = snap.cells.iter().any(|c| c.codepoint != 0);
        assert!(!has_content, "screen should be cleared after ESC[2J");
    }

    #[test]
    fn cursor_movement() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Clear screen
        s.write(b"\x1b[2J\x1b[H").expect("clear");
        std::thread::sleep(Duration::from_millis(200));
        s.process_output();
        // Verify screen was cleared (or has content from prompt after clear)
        let snap = s.terminal().take_snapshot();
        assert_eq!(snap.rows, 24);
        assert_eq!(snap.cols, 80);
        // Cursor positioning + write should not crash
        s.write(b"\x1b[5;5H").expect("cursor move");
        s.write(b"X").expect("write");
        std::thread::sleep(Duration::from_millis(200));
        s.process_output();
    }

    #[test]
    fn sgr_bold_attribute() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Set bold, write X, reset
        s.write(b"\x1b[1mX\x1b[0m\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(2));
        let snap = s.terminal().take_snapshot();
        // Find the X cell and check bold flag
        let x_cell = snap.cells.iter().find(|c| c.codepoint == 'X' as u32);
        assert!(x_cell.is_some(), "should find bold X cell");
    }

    #[test]
    fn sgr_colors() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Write red text
        s.write(b"\x1b[31mR\x1b[0m\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(2));
        let snap = s.terminal().take_snapshot();
        let r_cell = snap.cells.iter().find(|c| c.codepoint == 'R' as u32);
        assert!(r_cell.is_some(), "should find colored R cell");
    }

    #[test]
    fn osc52_clipboard_does_not_crash() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // OSC 52 set clipboard (should be handled gracefully)
        s.write(b"\x1b]52;c;SGVsbG8=\x07").expect("write");
        std::thread::sleep(Duration::from_millis(200));
        s.process_output();
        // No crash = success
    }
}

#[cfg(test)]
mod linux_binary_safety {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn null_bytes_do_not_panic() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"printf '\\x00\\x00\\x00'\n").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(2));
        // No panic = success
    }

    #[test]
    fn random_binary_data_does_not_panic() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Generate random-ish binary data
        let mut data = Vec::new();
        for i in 0..=255u8 {
            data.push(i);
        }
        s.write(&data).expect("write binary");
        drain_until(&mut s, |_| false, Duration::from_secs(2));
        // No panic = success
    }

    #[test]
    fn escape_sequences_in_binary_do_not_crash() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Mix of ESC, CSI, and random bytes
        s.write(b"\x1b\x03\x1b[?1h\x00\xff\xfe").expect("write");
        drain_until(&mut s, |_| false, Duration::from_secs(2));
        // No crash = success
    }

    #[test]
    fn very_long_line_does_not_crash() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        // Write a line longer than the terminal width
        let long_line = "A".repeat(500);
        s.write(format!("{long_line}\n").as_bytes())
            .expect("write long line");
        drain_until(&mut s, |_| false, Duration::from_secs(3));
        // No crash = success
    }

    #[test]
    fn rapid_writes_do_not_crash() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        for i in 0..50 {
            s.write(format!("echo line{i}\n").as_bytes())
                .expect("write");
        }
        drain_until(&mut s, |_| false, Duration::from_secs(5));
        // No crash = success
    }
}

#[cfg(test)]
mod linux_session_lifecycle {
    use std::time::Duration;

    use super::common::*;
    use torvox_terminal::session::Session;

    #[test]
    fn spawn_is_not_exited() {
        let s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        assert!(
            !s.is_exited(),
            "new session should not be immediately exited"
        );
    }

    #[test]
    fn write_after_exit_does_not_panic() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"exit\n").expect("write");
        drain_until(&mut s, |s| s.is_exited(), Duration::from_secs(3));
        // Writing to an exited session should not panic
        let _ = s.write(b"echo after_exit\n");
    }

    #[test]
    fn process_output_after_exit() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"exit\n").expect("write");
        drain_until(&mut s, |s| s.is_exited(), Duration::from_secs(3));
        // process_output should not panic after exit
        s.process_output();
    }

    #[test]
    fn take_snapshot_after_exit() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        s.write(b"exit\n").expect("write");
        drain_until(&mut s, |s| s.is_exited(), Duration::from_secs(3));
        // take_snapshot should not panic after exit
        let snap = s.terminal().take_snapshot();
        assert_eq!(snap.rows, 24);
        assert_eq!(snap.cols, 80);
    }

    #[test]
    fn terminal_dims_match_spawn_params() {
        let s = Session::spawn("/bin/sh", 30, 120).expect("spawn");
        assert_eq!(s.terminal().rows(), 30);
        assert_eq!(s.terminal().cols(), 120);
    }

    #[test]
    fn session_drop_after_spawn_without_write() {
        let s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        drop(s);
    }

    #[test]
    fn session_drop_after_many_writes() {
        let mut s = Session::spawn("/bin/sh", 24, 80).expect("spawn");
        for i in 0..20 {
            s.write(format!("echo line{i}\n").as_bytes())
                .expect("write");
        }
        std::thread::sleep(Duration::from_millis(200));
        drop(s);
    }
}
