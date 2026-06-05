//! Kani formal verification proofs for torvox-core.
//!
//! Run with: `cargo kani --manifest-path torvox-core/kani/Cargo.toml`
//!
//! These proofs verify invariants that unit tests cannot prove in general:
//!  - Color construction preserves inputs exactly
//!  - Color equality is reflexive
//!  - TerminalConfig default values are documented
//!  - Shell equality works correctly
//!
//! NOTE: Grid proofs (grid_new_never_panics, grid_resize_preserves_at_least_min_dim)
//! are omitted because CBMC cannot handle the state explosion from nested
//! Vec<Line(Vec<Cell>> allocation — even at 5×5 grid dimensions.
//! These are covered by quickcheck property tests with 10K+ random inputs.

#[cfg(kani)]
mod color_proofs {
    use torvox_core::cell::Color;

    /// Proof: Color construction preserves its inputs exactly.
    #[kani::proof]
    fn color_construction_preserves_inputs() {
        let r: u8 = kani::any();
        let g: u8 = kani::any();
        let b: u8 = kani::any();
        let a: u8 = kani::any();
        let c = Color { r, g, b, a };
        assert!(c.r == r);
        assert!(c.g == g);
        assert!(c.b == b);
        assert!(c.a == a);
    }

    /// Proof: Color equality is reflexive.
    #[kani::proof]
    fn color_equality_reflexive() {
        let r: u8 = kani::any();
        let g: u8 = kani::any();
        let b: u8 = kani::any();
        let a: u8 = kani::any();
        let c = Color { r, g, b, a };
        assert!(c == c);
    }
}

#[cfg(kani)]
mod config_proofs {
    use torvox_core::config::{Shell, TerminalConfig};

    /// Proof: TerminalConfig::default has the documented values.
    #[kani::proof]
    fn terminal_config_default_values() {
        let cfg = TerminalConfig::default();
        assert!(cfg.rows == 24);
        assert!(cfg.cols == 80);
        assert!(cfg.font_size_tenths == 140);
    }

    /// Proof: Shell equality with SystemDefault.
    #[kani::proof]
    fn shell_default_eq() {
        let a = Shell::SystemDefault;
        let b = Shell::SystemDefault;
        assert!(a == b);
    }
}

#[cfg(not(kani))]
pub fn _placeholder() {}
