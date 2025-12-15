//! Hot reload and live preview system for StratoUI
//!
//! Provides file watching, code reloading, and live preview capabilities
//! for rapid development and iteration

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[cfg(feature = "hot-reload")]
use notify::{Watcher, RecursiveMode, Event, EventKind, RecommendedWatcher};
#[cfg(feature = "hot-reload")]
use tokio::sync::mpsc;
#[cfg(feature = "hot-reload")]
use tokio_tungstenite::{accept_async, tungstenite::Message};
#[cfg(feature = "hot-reload")]
use futures_util::{SinkExt, StreamExt};

/// File change event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: SystemTime,
}

/// Hot reload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    /// Directories to watch for changes
    pub watch_dirs: Vec<PathBuf>,
    /// File extensions to watch
    pub watch_extensions: Vec<String>,
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Enable hot reload
    pub enabled: bool,
    /// Enable live preview
    pub live_preview: bool,
    /// Preview server port
    pub preview_port: u16,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            watch_dirs: vec![PathBuf::from("src"), PathBuf::from("assets")],
            watch_extensions: vec![
                "rs".to_string(),
                "toml".to_string(),
                "css".to_string(),
                "js".to_string(),
                "html".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "svg".to_string(),
            ],
            debounce_ms: 300,
            enabled: true,
            live_preview: true,
            preview_port: 3000,
        }
    }
}

/// Hot reload event handler
pub trait HotReloadHandler: Send + Sync {
    /// Handle file changes
    fn handle_change(&self, change: &FileChange) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Handle compilation errors
    fn handle_error(&self, error: &str);
    
    /// Handle successful reload
    fn handle_reload_success(&self);
}

/// File watcher for hot reload
#[cfg(feature = "hot-reload")]
pub struct FileWatcher {
    config: HotReloadConfig,
    watcher: Option<RecommendedWatcher>,
    handlers: Arc<RwLock<Vec<Arc<dyn HotReloadHandler>>>>,
    file_cache: Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
    debounce_cache: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
}

#[cfg(feature = "hot-reload")]
impl FileWatcher {
    /// Create a new file watcher
    pub fn new(config: HotReloadConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            watcher: None,
            handlers: Arc::new(RwLock::new(Vec::new())),
            file_cache: Arc::new(RwLock::new(HashMap::new())),
            debounce_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Add a hot reload handler
    pub fn add_handler(&self, handler: Arc<dyn HotReloadHandler>) {
        self.handlers.write().push(handler);
    }

    /// Start watching for file changes
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(());
        }

        let (tx, mut rx) = mpsc::channel(100);
        let handlers = Arc::clone(&self.handlers);
        let file_cache = Arc::clone(&self.file_cache);
        let debounce_cache = Arc::clone(&self.debounce_cache);
        let debounce_duration = Duration::from_millis(self.config.debounce_ms);
        let watch_extensions = self.config.watch_extensions.clone();

        // Create file watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Err(e) = tx.blocking_send(event) {
                        eprintln!("Failed to send file event: {}", e);
                    }
                }
                Err(e) => eprintln!("File watcher error: {}", e),
            }
        })?;

        // Watch configured directories
        for dir in &self.config.watch_dirs {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)?;
                println!("Watching directory: {}", dir.display());
            }
        }

        self.watcher = Some(watcher);

        // Spawn event processing task
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                Self::process_event(
                    event,
                    &handlers,
                    &file_cache,
                    &debounce_cache,
                    debounce_duration,
                    &watch_extensions,
                ).await;
            }
        });

        println!("Hot reload system started");
        Ok(())
    }

    /// Process a file system event
    async fn process_event(
        event: Event,
        handlers: &Arc<RwLock<Vec<Arc<dyn HotReloadHandler>>>>,
        file_cache: &Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
        debounce_cache: &Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
        debounce_duration: Duration,
        watch_extensions: &[String],
    ) {
        let now = SystemTime::now();

        for path in event.paths {
            // Check if file extension is watched
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !watch_extensions.contains(&ext.to_string()) {
                    continue;
                }
            }

            // Debounce file changes
            {
                let mut debounce = debounce_cache.lock();
                if let Some(&last_time) = debounce.get(&path) {
                    if now.duration_since(last_time).unwrap_or_default() < debounce_duration {
                        continue;
                    }
                }
                debounce.insert(path.clone(), now);
            }

            let change_type = match event.kind {
                EventKind::Create(_) => ChangeType::Created,
                EventKind::Modify(_) => ChangeType::Modified,
                EventKind::Remove(_) => ChangeType::Deleted,
                _ => continue,
            };

            let file_change = FileChange {
                path: path.clone(),
                change_type,
                timestamp: now,
            };

            // Update file cache
            match file_change.change_type {
                ChangeType::Created | ChangeType::Modified => {
                    file_cache.write().insert(path.clone(), now);
                }
                ChangeType::Deleted => {
                    file_cache.write().remove(&path);
                }
                _ => {}
            }

            // Notify handlers
            let handlers_read = handlers.read();
            for handler in handlers_read.iter() {
                if let Err(e) = handler.handle_change(&file_change) {
                    handler.handle_error(&format!("Hot reload error: {}", e));
                } else {
                    handler.handle_reload_success();
                }
            }
        }
    }

    /// Stop watching for file changes
    pub fn stop(&mut self) {
        self.watcher = None;
        println!("Hot reload system stopped");
    }
}

/// Live preview server for hot reload
#[cfg(feature = "hot-reload")]
pub struct LivePreviewServer {
    config: HotReloadConfig,
    clients: Arc<RwLock<Vec<mpsc::UnboundedSender<String>>>>,
}

#[cfg(feature = "hot-reload")]
impl LivePreviewServer {
    /// Create a new live preview server
    pub fn new(config: HotReloadConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the live preview server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.live_preview {
            return Ok(());
        }

        let clients = Arc::clone(&self.clients);
        let port = self.config.preview_port;

        tokio::spawn(async move {
            Self::run_server(port, clients).await;
        });

        println!("Live preview server started on port {}", port);
        Ok(())
    }

    /// Run the WebSocket server
    async fn run_server(
        port: u16,
        clients: Arc<RwLock<Vec<mpsc::UnboundedSender<String>>>>,
    ) {
        use tokio::net::TcpListener;
        use tokio_tungstenite::{accept_async, tungstenite::Message};
        use futures_util::{SinkExt, StreamExt};

        let addr = format!("127.0.0.1:{}", port);
        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to bind to {}: {}", addr, e);
                return;
            }
        };

        while let Ok((stream, _)) = listener.accept().await {
            let clients = Arc::clone(&clients);
            
            tokio::spawn(async move {
                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        eprintln!("WebSocket connection error: {}", e);
                        return;
                    }
                };

                let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                let (tx, mut rx) = mpsc::unbounded_channel();

                // Add client to list
                clients.write().push(tx);

                // Handle incoming messages
                let clients_clone = Arc::clone(&clients);
                tokio::spawn(async move {
                    while let Some(msg) = ws_receiver.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                println!("Received: {}", text);
                            }
                            Ok(Message::Close(_)) => break,
                            Err(e) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                    
                    // Remove client when disconnected
                    clients_clone.write().clear(); // Simplified cleanup
                });

                // Send messages to client
                while let Some(message) = rx.recv().await {
                    if let Err(e) = ws_sender.send(Message::Text(message)).await {
                        eprintln!("Failed to send message: {}", e);
                        break;
                    }
                }
            });
        }
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: &str) {
        let clients = self.clients.read();
        for client in clients.iter() {
            let _ = client.send(message.to_string());
        }
    }

    /// Notify clients of a file change
    pub fn notify_change(&self, change: &FileChange) {
        let message = serde_json::json!({
            "type": "file_change",
            "path": change.path,
            "change_type": format!("{:?}", change.change_type),
            "timestamp": change.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs()
        });

        self.broadcast(&message.to_string());
    }

    /// Notify clients of a reload
    pub fn notify_reload(&self) {
        let message = serde_json::json!({
            "type": "reload",
            "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs()
        });

        self.broadcast(&message.to_string());
    }
}

/// Hot reload manager that coordinates file watching and live preview
#[cfg(feature = "hot-reload")]
pub struct HotReloadManager {
    file_watcher: FileWatcher,
    preview_server: LivePreviewServer,
    config: HotReloadConfig,
}

#[cfg(feature = "hot-reload")]
impl HotReloadManager {
    /// Create a new hot reload manager
    pub fn new(config: HotReloadConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let file_watcher = FileWatcher::new(config.clone())?;
        let preview_server = LivePreviewServer::new(config.clone());

        Ok(Self {
            file_watcher,
            preview_server,
            config,
        })
    }

    /// Start the hot reload system
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(());
        }

        // Start preview server
        self.preview_server.start().await?;

        // Add preview server as a handler
        let preview_handler = PreviewHandler::new(self.preview_server.clients.clone());
        self.file_watcher.add_handler(Arc::new(preview_handler));

        // Start file watcher
        self.file_watcher.start().await?;

        println!("Hot reload manager started");
        Ok(())
    }

    /// Stop the hot reload system
    pub fn stop(&mut self) {
        self.file_watcher.stop();
        println!("Hot reload manager stopped");
    }

    /// Get the preview server
    pub fn preview_server(&self) -> &LivePreviewServer {
        &self.preview_server
    }
}

/// Handler that integrates with the live preview server
#[cfg(feature = "hot-reload")]
struct PreviewHandler {
    clients: Arc<RwLock<Vec<mpsc::UnboundedSender<String>>>>,
}

#[cfg(feature = "hot-reload")]
impl PreviewHandler {
    fn new(clients: Arc<RwLock<Vec<mpsc::UnboundedSender<String>>>>) -> Self {
        Self { clients }
    }
}

#[cfg(feature = "hot-reload")]
impl HotReloadHandler for PreviewHandler {
    fn handle_change(&self, change: &FileChange) -> Result<(), Box<dyn std::error::Error>> {
        let message = serde_json::json!({
            "type": "file_change",
            "path": change.path,
            "change_type": format!("{:?}", change.change_type),
            "timestamp": change.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs()
        });

        let clients = self.clients.read();
        for client in clients.iter() {
            let _ = client.send(message.to_string());
        }

        Ok(())
    }

    fn handle_error(&self, error: &str) {
        let message = serde_json::json!({
            "type": "error",
            "message": error,
            "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs()
        });

        let clients = self.clients.read();
        for client in clients.iter() {
            let _ = client.send(message.to_string());
        }
    }

    fn handle_reload_success(&self) {
        let message = serde_json::json!({
            "type": "reload_success",
            "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_secs()
        });

        let clients = self.clients.read();
        for client in clients.iter() {
            let _ = client.send(message.to_string());
        }
    }
}

/// Utility functions for hot reload
pub mod utils {
    use super::*;

    /// Check if a file should be watched based on extension
    pub fn should_watch_file(path: &Path, extensions: &[String]) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            extensions.contains(&ext.to_string())
        } else {
            false
        }
    }

    /// Get the relative path from a base directory
    pub fn get_relative_path(path: &Path, base: &Path) -> Option<PathBuf> {
        path.strip_prefix(base).ok().map(|p| p.to_path_buf())
    }

    /// Check if a path is in any of the watched directories
    pub fn is_in_watched_dirs(path: &Path, watch_dirs: &[PathBuf]) -> bool {
        watch_dirs.iter().any(|dir| path.starts_with(dir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_config() {
        let config = HotReloadConfig::default();
        assert!(config.enabled);
        assert!(config.live_preview);
        assert_eq!(config.preview_port, 3000);
        assert_eq!(config.debounce_ms, 300);
    }

    #[test]
    fn test_file_change() {
        let change = FileChange {
            path: PathBuf::from("test.rs"),
            change_type: ChangeType::Modified,
            timestamp: SystemTime::now(),
        };

        assert_eq!(change.path, PathBuf::from("test.rs"));
        assert_eq!(change.change_type, ChangeType::Modified);
    }

    #[test]
    fn test_should_watch_file() {
        let extensions = vec!["rs".to_string(), "toml".to_string()];
        
        assert!(utils::should_watch_file(Path::new("main.rs"), &extensions));
        assert!(utils::should_watch_file(Path::new("Cargo.toml"), &extensions));
        assert!(!utils::should_watch_file(Path::new("README.md"), &extensions));
    }

    #[tokio::test]
    #[cfg(feature = "hot-reload")]
    async fn test_file_watcher_creation() {
        let config = HotReloadConfig::default();
        let watcher = FileWatcher::new(config);
        assert!(watcher.is_ok());
    }

    #[test]
    #[cfg(feature = "hot-reload")]
    fn test_live_preview_server_creation() {
        let config = HotReloadConfig::default();
        let server = LivePreviewServer::new(config);
        assert_eq!(server.clients.read().len(), 0);
    }

    #[tokio::test]
    #[cfg(feature = "hot-reload")]
    async fn test_hot_reload_manager() {
        let mut config = HotReloadConfig::default();
        config.enabled = false; // Disable for testing
        
        let manager = HotReloadManager::new(config);
        assert!(manager.is_ok());
    }
}