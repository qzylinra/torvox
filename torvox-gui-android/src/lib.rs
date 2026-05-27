uniffi::setup_scaffolding!();

pub mod commands;

pub struct BridgeState {
    pub terminal_pid: i32,
}
