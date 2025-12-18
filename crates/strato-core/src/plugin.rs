//! Plugin system for StratoUI
//!
//! Provides a flexible plugin architecture for extending framework functionality
//! with custom widgets, themes, and behaviors.

use crate::{
    error::{Result, StratoError},
    event::{Event, EventHandler, EventResult},
    theme::Theme,
    widget::Widget,
};
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Minimum StratoUI version required
    pub min_strato_version: String,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginCapability {
    /// Provides custom widgets
    Widgets,
    /// Provides custom themes
    Themes,
    /// Provides event handlers
    EventHandlers,
    /// Provides layout engines
    LayoutEngines,
    /// Provides state management
    StateManagement,
    /// Provides rendering backends
    RenderingBackends,
    /// Provides platform integration
    PlatformIntegration,
}

/// Plugin lifecycle states
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Active,
    Error(String),
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    fn initialize(&mut self, context: &mut PluginContext) -> Result<()>;

    /// Activate the plugin
    fn activate(&mut self, context: &mut PluginContext) -> Result<()>;

    /// Deactivate the plugin
    fn deactivate(&mut self, context: &mut PluginContext) -> Result<()>;

    /// Cleanup plugin resources
    fn cleanup(&mut self, context: &mut PluginContext) -> Result<()>;

    /// Handle plugin events
    fn handle_event(&mut self, _event: &Event, _context: &mut PluginContext) -> EventResult {
        EventResult::Ignored
    }

    /// Get plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get mutable plugin as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Plugin context provides access to framework services
pub struct PluginContext {
    /// Widget registry
    pub widget_registry: Arc<RwLock<WidgetRegistry>>,
    /// Theme registry
    pub theme_registry: Arc<RwLock<ThemeRegistry>>,
    /// Event system
    pub event_handlers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
    /// Plugin data storage
    pub data_store: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new() -> Self {
        Self {
            widget_registry: Arc::new(RwLock::new(WidgetRegistry::new())),
            theme_registry: Arc::new(RwLock::new(ThemeRegistry::new())),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
            data_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store plugin data
    pub fn store_data<T: Any + Send + Sync>(&self, key: String, data: T) {
        if let Ok(mut store) = self.data_store.write() {
            store.insert(key, Box::new(data));
        }
    }

    /// Retrieve plugin data
    pub fn get_data<T: Any + Send + Sync>(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        let store = self.data_store.read().ok()?;
        store.get(key)?.downcast_ref::<T>().cloned()
    }

    /// Register a widget factory
    pub fn register_widget<W: Widget + 'static>(&self, name: String, factory: WidgetFactory) {
        if let Ok(mut registry) = self.widget_registry.write() {
            registry.register(name, factory);
        }
    }

    /// Register a theme
    pub fn register_theme(&self, name: String, theme: Theme) {
        if let Ok(mut registry) = self.theme_registry.write() {
            registry.register(name, theme);
        }
    }

    /// Add an event handler
    pub fn add_event_handler(&self, handler: Box<dyn EventHandler>) {
        if let Ok(mut handlers) = self.event_handlers.write() {
            handlers.push(handler);
        }
    }
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget factory function type
pub type WidgetFactory = Box<dyn Fn() -> Box<dyn Widget> + Send + Sync>;

/// Widget registry for plugin-provided widgets
pub struct WidgetRegistry {
    factories: HashMap<String, WidgetFactory>,
}

impl WidgetRegistry {
    /// Create a new widget registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a widget factory
    pub fn register(&mut self, name: String, factory: WidgetFactory) {
        self.factories.insert(name, factory);
    }

    /// Create a widget by name
    pub fn create_widget(&self, name: &str) -> Option<Box<dyn Widget>> {
        self.factories.get(name).map(|factory| factory())
    }

    /// Get all registered widget names
    pub fn get_widget_names(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// Check if a widget is registered
    pub fn has_widget(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme registry for plugin-provided themes
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
}

impl ThemeRegistry {
    /// Create a new theme registry
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
        }
    }

    /// Register a theme
    pub fn register(&mut self, name: String, theme: Theme) {
        self.themes.insert(name, theme);
    }

    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Get all registered theme names
    pub fn get_theme_names(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Check if a theme is registered
    pub fn has_theme(&self, name: &str) -> bool {
        self.themes.contains_key(name)
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin manager handles loading, activation, and lifecycle of plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_states: HashMap<String, PluginState>,
    context: PluginContext,
    load_order: Vec<String>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_states: HashMap::new(),
            context: PluginContext::new(),
            load_order: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.metadata().name.clone();

        // Check for duplicate names
        if self.plugins.contains_key(&name) {
            return Err(StratoError::PluginError {
                message: format!("Plugin '{}' is already registered", name),
                context: None,
            });
        }

        // Validate dependencies
        self.validate_dependencies(plugin.metadata())?;

        self.plugin_states
            .insert(name.clone(), PluginState::Unloaded);
        self.plugins.insert(name.clone(), plugin);
        self.load_order.push(name);

        Ok(())
    }

    /// Load a plugin
    pub fn load_plugin(&mut self, name: &str) -> Result<()> {
        if !self.plugins.contains_key(name) {
            return Err(StratoError::PluginError {
                message: format!("Plugin '{}' not found", name),
                context: None,
            });
        }

        // Check current state
        if let Some(state) = self.plugin_states.get(name) {
            match state {
                PluginState::Loaded | PluginState::Active => {
                    return Ok(()); // Already loaded
                }
                PluginState::Loading => {
                    return Err(StratoError::PluginError {
                        message: format!("Plugin '{}' is already loading", name),
                        context: None,
                    });
                }
                _ => {}
            }
        }

        // Set loading state
        self.plugin_states
            .insert(name.to_string(), PluginState::Loading);

        // Load dependencies first
        let dependencies = self.plugins[name].metadata().dependencies.clone();
        for dep in dependencies {
            self.load_plugin(&dep)?;
        }

        // Initialize plugin
        match self.plugins.get_mut(name) {
            Some(plugin) => match plugin.initialize(&mut self.context) {
                Ok(()) => {
                    self.plugin_states
                        .insert(name.to_string(), PluginState::Loaded);
                    tracing::info!("Plugin '{}' loaded successfully", name);
                }
                Err(e) => {
                    let error_msg = format!("Failed to initialize plugin '{}': {}", name, e);
                    self.plugin_states
                        .insert(name.to_string(), PluginState::Error(error_msg.clone()));
                    return Err(StratoError::PluginError {
                        message: error_msg,
                        context: None,
                    });
                }
            },
            None => {
                return Err(StratoError::PluginError {
                    message: format!("Plugin '{}' not found", name),
                    context: None,
                });
            }
        }

        Ok(())
    }

    /// Activate a plugin
    pub fn activate_plugin(&mut self, name: &str) -> Result<()> {
        // Ensure plugin is loaded
        self.load_plugin(name)?;

        // Check current state
        if let Some(PluginState::Active) = self.plugin_states.get(name) {
            return Ok(()); // Already active
        }

        // Activate plugin
        match self.plugins.get_mut(name) {
            Some(plugin) => match plugin.activate(&mut self.context) {
                Ok(()) => {
                    self.plugin_states
                        .insert(name.to_string(), PluginState::Active);
                    tracing::info!("Plugin '{}' activated successfully", name);
                }
                Err(e) => {
                    let error_msg = format!("Failed to activate plugin '{}': {}", name, e);
                    self.plugin_states
                        .insert(name.to_string(), PluginState::Error(error_msg.clone()));
                    return Err(StratoError::PluginError {
                        message: error_msg,
                        context: None,
                    });
                }
            },
            None => {
                return Err(StratoError::PluginError {
                    message: format!("Plugin '{}' not found", name),
                    context: None,
                });
            }
        }

        Ok(())
    }

    /// Deactivate a plugin
    pub fn deactivate_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(PluginState::Active) = self.plugin_states.get(name) {
            match self.plugins.get_mut(name) {
                Some(plugin) => {
                    plugin.deactivate(&mut self.context)?;
                    self.plugin_states
                        .insert(name.to_string(), PluginState::Loaded);
                    tracing::info!("Plugin '{}' deactivated", name);
                }
                None => {
                    return Err(StratoError::PluginError {
                        message: format!("Plugin '{}' not found", name),
                        context: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        // Deactivate first if active
        self.deactivate_plugin(name)?;

        // Cleanup plugin
        match self.plugins.get_mut(name) {
            Some(plugin) => {
                plugin.cleanup(&mut self.context)?;
                self.plugin_states
                    .insert(name.to_string(), PluginState::Unloaded);
                tracing::info!("Plugin '{}' unloaded", name);
            }
            None => {
                return Err(StratoError::PluginError {
                    message: format!("Plugin '{}' not found", name),
                    context: None,
                });
            }
        }

        Ok(())
    }

    /// Load all registered plugins
    pub fn load_all_plugins(&mut self) -> Result<()> {
        let plugin_names: Vec<String> = self.load_order.clone();

        for name in plugin_names {
            if let Err(e) = self.load_plugin(&name) {
                tracing::error!("Failed to load plugin '{}': {}", name, e);
                // Continue loading other plugins
            }
        }

        Ok(())
    }

    /// Activate all loaded plugins
    pub fn activate_all_plugins(&mut self) -> Result<()> {
        let plugin_names: Vec<String> = self.load_order.clone();

        for name in plugin_names {
            if let Some(PluginState::Loaded) = self.plugin_states.get(&name) {
                if let Err(e) = self.activate_plugin(&name) {
                    tracing::error!("Failed to activate plugin '{}': {}", name, e);
                    // Continue activating other plugins
                }
            }
        }

        Ok(())
    }

    /// Get plugin state
    pub fn get_plugin_state(&self, name: &str) -> Option<&PluginState> {
        self.plugin_states.get(name)
    }

    /// Get all plugin names
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, name: &str) -> Option<&PluginMetadata> {
        self.plugins.get(name).map(|p| p.metadata())
    }

    /// Get plugin context
    pub fn get_context(&self) -> &PluginContext {
        &self.context
    }

    /// Get mutable plugin context
    pub fn get_context_mut(&mut self) -> &mut PluginContext {
        &mut self.context
    }

    /// Handle event through all active plugins
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();

        for name in plugin_names {
            if let Some(PluginState::Active) = self.plugin_states.get(&name) {
                if let Some(plugin) = self.plugins.get_mut(&name) {
                    match plugin.handle_event(event, &mut self.context) {
                        EventResult::Handled => return EventResult::Handled,
                        EventResult::Ignored => continue,
                    }
                }
            }
        }

        EventResult::Ignored
    }

    /// Validate plugin dependencies
    fn validate_dependencies(&self, metadata: &PluginMetadata) -> Result<()> {
        for dep in &metadata.dependencies {
            if !self.plugins.contains_key(dep) {
                return Err(StratoError::PluginError {
                    message: format!(
                        "Plugin '{}' depends on '{}' which is not registered",
                        metadata.name, dep
                    ),
                    context: None,
                });
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for creating plugin metadata
#[macro_export]
macro_rules! plugin_metadata {
    (
        name: $name:expr,
        version: $version:expr,
        description: $description:expr,
        author: $author:expr,
        $(dependencies: [$($dep:expr),*],)?
        $(min_strato_version: $min_version:expr,)?
        $(capabilities: [$($cap:expr),*],)?
    ) => {
        $crate::plugin::PluginMetadata {
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            author: $author.to_string(),
            dependencies: vec![$($($dep.to_string()),*)?],
            min_strato_version: plugin_metadata!(@min_version $($min_version)?).to_string(),
            capabilities: vec![$($($cap),*)?],
        }
    };
    (@min_version) => { env!("CARGO_PKG_VERSION") };
    (@min_version $version:expr) => { $version };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        metadata: PluginMetadata,
        initialized: bool,
        active: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                metadata: PluginMetadata {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    description: "Test plugin".to_string(),
                    author: "Test Author".to_string(),
                    dependencies: vec![],
                    min_strato_version: "0.1.0".to_string(),
                    capabilities: vec![PluginCapability::Widgets],
                },
                initialized: false,
                active: false,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn initialize(&mut self, _context: &mut PluginContext) -> Result<()> {
            self.initialized = true;
            Ok(())
        }

        fn activate(&mut self, _context: &mut PluginContext) -> Result<()> {
            self.active = true;
            Ok(())
        }

        fn deactivate(&mut self, _context: &mut PluginContext) -> Result<()> {
            self.active = false;
            Ok(())
        }

        fn cleanup(&mut self, _context: &mut PluginContext) -> Result<()> {
            self.initialized = false;
            self.active = false;
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.get_plugin_names().len(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));

        assert!(manager.register_plugin(plugin).is_ok());
        assert_eq!(manager.get_plugin_names().len(), 1);
        assert!(manager.get_plugin_names().contains(&"test".to_string()));
    }

    #[test]
    fn test_plugin_loading() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));

        manager.register_plugin(plugin).unwrap();
        assert!(manager.load_plugin("test").is_ok());

        match manager.get_plugin_state("test") {
            Some(PluginState::Loaded) => {}
            _ => panic!("Plugin should be loaded"),
        }
    }

    #[test]
    fn test_plugin_activation() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));

        manager.register_plugin(plugin).unwrap();
        assert!(manager.activate_plugin("test").is_ok());

        match manager.get_plugin_state("test") {
            Some(PluginState::Active) => {}
            _ => panic!("Plugin should be active"),
        }
    }

    #[test]
    fn test_widget_registry() {
        let mut registry = WidgetRegistry::new();

        // This would normally be a real widget factory
        let factory: WidgetFactory = Box::new(|| {
            // Return a mock widget
            unimplemented!("Mock widget factory")
        });

        registry.register("test_widget".to_string(), factory);
        assert!(registry.has_widget("test_widget"));
        assert_eq!(registry.get_widget_names().len(), 1);
    }

    #[test]
    fn test_plugin_metadata_macro() {
        let metadata = plugin_metadata! {
            name: "test_plugin",
            version: "1.0.0",
            description: "A test plugin",
            author: "Test Author",
            dependencies: ["dep1", "dep2"],
            capabilities: [PluginCapability::Widgets, PluginCapability::Themes],
        };

        assert_eq!(metadata.name, "test_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.dependencies.len(), 2);
        assert_eq!(metadata.capabilities.len(), 2);
    }
}
