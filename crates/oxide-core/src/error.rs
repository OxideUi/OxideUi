//! Error types for OxideUI framework

use thiserror::Error;
use std::collections::HashMap;

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
            let metadata_str = self.metadata
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

/// Main error type for OxideUI operations
#[derive(Debug, Error)]
pub enum OxideError {
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

impl OxideError {
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
            Self::Platform { context, .. } |
            Self::Renderer { context, .. } |
            Self::Widget { context, .. } |
            Self::State { context, .. } |
            Self::Layout { context, .. } |
            Self::Initialization { context, .. } |
            Self::Configuration { context, .. } |
            Self::NotImplemented { context, .. } |
            Self::PluginError { context, .. } |
            Self::Other { context, .. } => context.as_ref(),
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

/// Result type alias for OxideUI operations
pub type Result<T> = std::result::Result<T, OxideError>;

/// Alternative result type alias for backward compatibility
pub type OxideResult<T> = Result<T>;