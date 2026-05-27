pub mod atlas;
pub mod pipeline;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("wgpu surface error: {0}")]
    Surface(String),
    #[error("shader compilation failed: {0}")]
    Shader(String),
    #[error("atlas full")]
    AtlasFull,
}
