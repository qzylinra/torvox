uniffi::setup_scaffolding!();

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct BridgeCell {
    pub char_code: u32,
    pub fg: u32,
    pub bg: u32,
}
