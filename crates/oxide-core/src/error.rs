//! Error types for OxideUI framework

use thiserror::Error;

/// Main error type for OxideUI operations
#[derive(Debug, Error)]
pub enum OxideError {
    #[error("Platform error: {0}")]
    Platform(String),
    
    #[error("Renderer error: {0}")]
    Renderer(String),
    
    #[error("Widget error: {0}")]
    Widget(String),
    
    #[error("State management error: {0}")]
    State(String),
    
    #[error("Layout calculation error: {0}")]
    Layout(String),
    
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl OxideError {
    /// Create a platform error from a string
    pub fn platform<S: Into<String>>(msg: S) -> Self {
        Self::Platform(msg.into())
    }
    
    /// Create a renderer error from a string
    pub fn renderer<S: Into<String>>(msg: S) -> Self {
        Self::Renderer(msg.into())
    }
    
    /// Create a widget error from a string
    pub fn widget<S: Into<String>>(msg: S) -> Self {
        Self::Widget(msg.into())
    }
    
    /// Create a state error from a string
    pub fn state<S: Into<String>>(msg: S) -> Self {
        Self::State(msg.into())
    }
    
    /// Create a layout error from a string
    pub fn layout<S: Into<String>>(msg: S) -> Self {
        Self::Layout(msg.into())
    }
    
    /// Create an initialization error from a string
    pub fn initialization<S: Into<String>>(msg: S) -> Self {
        Self::Initialization(msg.into())
    }
    
    /// Create a configuration error from a string
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }
    
    /// Create a not implemented error from a string
    pub fn not_implemented<S: Into<String>>(msg: S) -> Self {
        Self::NotImplemented(msg.into())
    }
    
    /// Create a plugin error from a string
    pub fn plugin_error<S: Into<String>>(msg: S) -> Self {
        Self::PluginError(msg.into())
    }
    
    /// Create an other error from a string
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}

/// Result type alias for OxideUI operations
pub type Result<T> = std::result::Result<T, OxideError>;

/// Alternative result type alias for backward compatibility
pub type OxideResult<T> = Result<T>;