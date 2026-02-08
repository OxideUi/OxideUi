//! Error types for StratoUI framework

use std::collections::HashMap;
use thiserror::Error;

/// Context information for errors to aid in debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation that was being performed when the error occurred
    pub operation: String,
    /// Component or module where the error occurred
    pub component: String,
    /// Additional contextual data
    pub metadata: HashMap<String, String>,
    /// Stack trace or call path if available
    pub call_path: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            component: component.into(),
            metadata: HashMap::new(),
            call_path: None,
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add call path information
    pub fn with_call_path(mut self, path: impl Into<String>) -> Self {
        self.call_path = Some(path.into());
        self
    }

    /// Format context for logging
    pub fn format_for_log(&self) -> String {
        let mut parts = vec![
            format!("operation={}", self.operation),
            format!("component={}", self.component),
        ];

        if !self.metadata.is_empty() {
            let metadata_str = self
                .metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("metadata=[{}]", metadata_str));
        }

        if let Some(ref path) = self.call_path {
            parts.push(format!("call_path={}", path));
        }

        parts.join(", ")
    }
}

/// Main error type for StratoUI operations
#[derive(Debug, Error)]
pub enum StratoError {
    #[error("Platform error: {message}")]
    Platform {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Renderer error: {message}")]
    Renderer {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Widget error: {message}")]
    Widget {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("State management error: {message}")]
    State {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Layout calculation error: {message}")]
    Layout {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Initialization error: {message}")]
    Initialization {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not implemented: {message}")]
    NotImplemented {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Plugin error: {message}")]
    PluginError {
        message: String,
        context: Option<ErrorContext>,
    },

    #[error("Other error: {message}")]
    Other {
        message: String,
        context: Option<ErrorContext>,
    },
}

impl StratoError {
    /// Create a platform error with context
    pub fn platform_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Platform {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a renderer error with context
    pub fn renderer_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Renderer {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a widget error with context
    pub fn widget_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Widget {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a state error with context
    pub fn state_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::State {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a layout error with context
    pub fn layout_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Layout {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create an initialization error with context
    pub fn initialization_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Initialization {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a configuration error with context
    pub fn configuration_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Configuration {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a plugin error with context
    pub fn plugin_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::PluginError {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create an other error with context
    pub fn other_with_context<S: Into<String>>(msg: S, context: ErrorContext) -> Self {
        Self::Other {
            message: msg.into(),
            context: Some(context),
        }
    }

    /// Create a platform error from a string
    pub fn platform<S: Into<String>>(msg: S) -> Self {
        Self::Platform {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a renderer error from a string
    pub fn renderer<S: Into<String>>(msg: S) -> Self {
        Self::Renderer {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a widget error from a string
    pub fn widget<S: Into<String>>(msg: S) -> Self {
        Self::Widget {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a state error from a string
    pub fn state<S: Into<String>>(msg: S) -> Self {
        Self::State {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a layout error from a string
    pub fn layout<S: Into<String>>(msg: S) -> Self {
        Self::Layout {
            message: msg.into(),
            context: None,
        }
    }

    /// Create an initialization error from a string
    pub fn initialization<S: Into<String>>(msg: S) -> Self {
        Self::Initialization {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a configuration error from a string
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a not implemented error from a string
    pub fn not_implemented<S: Into<String>>(msg: S) -> Self {
        Self::NotImplemented {
            message: msg.into(),
            context: None,
        }
    }

    /// Create a plugin error from a string
    pub fn plugin<S: Into<String>>(msg: S) -> Self {
        Self::PluginError {
            message: msg.into(),
            context: None,
        }
    }

    /// Create an other error from a string
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other {
            message: msg.into(),
            context: None,
        }
    }

    /// Get the error context if available
    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            Self::Platform { context, .. }
            | Self::Renderer { context, .. }
            | Self::Widget { context, .. }
            | Self::State { context, .. }
            | Self::Layout { context, .. }
            | Self::Initialization { context, .. }
            | Self::Configuration { context, .. }
            | Self::NotImplemented { context, .. }
            | Self::PluginError { context, .. }
            | Self::Other { context, .. } => context.as_ref(),
            Self::Io(_) => None,
        }
    }

    /// Format error with context for logging
    pub fn format_for_log(&self) -> String {
        let base_msg = self.to_string();
        if let Some(context) = self.context() {
            format!("{} [{}]", base_msg, context.format_for_log())
        } else {
            base_msg
        }
    }
}

/// Result type alias for StratoUI operations
pub type Result<T> = std::result::Result<T, StratoError>;

/// Alternative result type alias for backward compatibility
pub type StratoResult<T> = Result<T>;

// =============================================================================
// Taffy Layout Engine Error Types
// =============================================================================

/// Layout errors from Taffy engine.
///
/// These errors are NON-RECOVERABLE without a fallback layout.
/// The `TaffyLayoutManager` will attempt to use cached layouts when these occur.
///
/// # Error Recovery
///
/// - If `last_valid_layout` exists: use cached layout (log warning)
/// - If no cache: propagate error to caller (log error)
#[derive(Debug, Clone, thiserror::Error)]
pub enum TaffyLayoutError {
    /// Window/container size is invalid (zero, negative, or infinite).
    #[error("Invalid window size: {width}x{height}")]
    InvalidWindowSize { width: f32, height: f32 },

    /// Taffy computation failed internally.
    #[error("Layout computation failed: {reason}")]
    ComputationFailed { reason: String },

    /// The layout tree structure is corrupted.
    #[error("Layout tree is corrupted")]
    CorruptedTree,

    /// Error from Taffy library itself.
    #[error("Taffy error: {0}")]
    TaffyError(String),

    /// Failed to build node for widget.
    #[error("Failed to build layout node for widget")]
    NodeBuildFailed,
}

/// Rendering errors from Taffy-based layout.
///
/// These errors are RECOVERABLE - skip the widget and continue rendering.
///
/// # Recovery Strategy
///
/// Log warning, skip rendering the problematic widget, continue with siblings.
#[derive(Debug, thiserror::Error)]
pub enum TaffyRenderError {
    /// Viewport coordinates are invalid after validation.
    #[error("Invalid viewport: {0:?}")]
    InvalidViewport(crate::validated_rect::ValidatedRect),

    /// GPU/rendering backend error.
    #[error("GPU error: {0}")]
    GpuError(String),

    /// Required resource (texture, font, etc.) not found.
    #[error("Missing resource: {resource_id}")]
    MissingResource { resource_id: String },
}

/// Validation errors caught before layout computation.
///
/// These are PREVENTIVE errors - they indicate invalid widget configuration
/// and should be caught during development/testing.
///
/// # Common Causes
///
/// - Negative gaps or padding
/// - Invalid button dimensions
/// - NaN/Infinite values in style properties
#[derive(Debug, Clone, thiserror::Error)]
pub enum TaffyValidationError {
    /// Gap value is negative.
    #[error("Negative gap value: {0}")]
    NegativeGap(f32),

    /// Padding contains invalid values.
    #[error("Invalid padding configuration")]
    InvalidPadding,

    /// A value is not finite (NaN or Infinity).
    #[error("Non-finite value in layout configuration")]
    NonFiniteValue,

    /// Width or height is negative.
    #[error("Negative dimension: width={width}, height={height}")]
    NegativeDimension { width: f32, height: f32 },

    /// Button size is invalid (zero or negative).
    #[error("Invalid button size: {width}x{height}")]
    InvalidButtonSize { width: f32, height: f32 },

    /// Border radius is invalid.
    #[error("Invalid border radius: {0}")]
    InvalidBorderRadius(f32),
}

impl From<taffy::TaffyError> for TaffyLayoutError {
    fn from(err: taffy::TaffyError) -> Self {
        TaffyLayoutError::TaffyError(format!("{:?}", err))
    }
}

/// Result type for Taffy layout operations.
pub type TaffyLayoutResult<T> = std::result::Result<T, TaffyLayoutError>;

/// Result type for Taffy render operations.
pub type TaffyRenderResult<T> = std::result::Result<T, TaffyRenderError>;

/// Result type for Taffy validation operations.
pub type TaffyValidationResult<T> = std::result::Result<T, TaffyValidationError>;
