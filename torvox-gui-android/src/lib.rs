pub mod bridge;
#[cfg(target_os = "android")]
pub mod jni_bridge;
pub mod surface;

#[cfg(not(target_os = "android"))]
pub mod mock_surface;

pub use bridge::{BridgeCell, TerminalConfig, TerminalError, TerminalEvent, TorvoxBridge};
pub use surface::AndroidSurface;
