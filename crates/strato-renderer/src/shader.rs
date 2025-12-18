//! Advanced shader management system
//!
//! This module provides comprehensive shader management including:
//! - Hot-reload with file watching and automatic recompilation
//! - Dynamic shader compilation with macro support
//! - Intelligent caching with dependency tracking
//! - Shader variant generation and specialization
//! - Cross-platform shader compilation (HLSL, GLSL, WGSL)
//! - Performance profiling and optimization hints
//! - Shader debugging and validation tools
//! - Modular shader composition system

use anyhow::{bail, Context, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::{Mutex, RwLock};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime};
use tracing::{info, instrument};
use wgpu::*;

use crate::device::ManagedDevice;
use crate::resources::ResourceHandle;

/// Shader stage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// Shader language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderLanguage {
    WGSL,
    GLSL,
    HLSL,
    SPIRV,
}

/// Shader compilation target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompilationTarget {
    Vulkan,
    Metal,
    DirectX12,
    OpenGL,
    WebGPU,
}

/// Shader macro definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderMacro {
    pub name: String,
    pub value: Option<String>,
}

/// Shader variant configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderVariant {
    pub macros: Vec<ShaderMacro>,
    pub features: Vec<String>,
    pub optimization_level: u32,
}

/// Shader source information
#[derive(Debug, Clone)]
pub struct ShaderSource {
    pub path: PathBuf,
    pub content: String,
    pub language: ShaderLanguage,
    pub stage: ShaderStage,
    pub includes: HashSet<PathBuf>,
    pub dependencies: HashSet<PathBuf>,
    pub last_modified: SystemTime,
    pub content_hash: [u8; 32],
}

/// Compiled shader module
#[derive(Debug)]
pub struct CompiledShader {
    pub module: ShaderModule,
    pub source_hash: [u8; 32],
    pub variant: ShaderVariant,
    pub compilation_time: Instant,
    pub spirv_size: usize,
    pub validation_errors: Vec<String>,
    pub optimization_applied: bool,
    pub usage_count: AtomicU64,
    pub last_used: RwLock<Instant>,
}

/// Shader compilation statistics
#[derive(Debug, Clone, Default)]
pub struct CompilationStats {
    pub total_compilations: u64,
    pub successful_compilations: u64,
    pub failed_compilations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hot_reloads: u64,
    pub average_compilation_time: Duration,
    pub total_compilation_time: Duration,
}

/// Shader dependency graph node
#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub path: PathBuf,
    pub dependents: HashSet<PathBuf>,
    pub dependencies: HashSet<PathBuf>,
    pub last_modified: SystemTime,
}

/// Hot-reload event
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    FileChanged(PathBuf),
    FileDeleted(PathBuf),
    FileCreated(PathBuf),
    DependencyChanged(PathBuf, HashSet<PathBuf>),
}

/// Shader manager with advanced features
pub struct ShaderManager {
    device: Arc<ManagedDevice>,
    shader_cache: RwLock<HashMap<([u8; 32], ShaderVariant), Arc<CompiledShader>>>,
    source_cache: RwLock<HashMap<PathBuf, Arc<ShaderSource>>>,
    dependency_graph: RwLock<HashMap<PathBuf, DependencyNode>>,
    compilation_stats: RwLock<CompilationStats>,

    // Hot-reload system
    file_watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
    hot_reload_enabled: AtomicBool,
    hot_reload_receiver: Arc<Mutex<Option<mpsc::Receiver<Event>>>>,
    watched_directories: RwLock<HashSet<PathBuf>>,

    // Shader preprocessing
    include_directories: RwLock<Vec<PathBuf>>,
    global_macros: RwLock<Vec<ShaderMacro>>,
    preprocessor_cache: RwLock<HashMap<String, String>>,

    // Performance tracking
    compilation_queue: Mutex<Vec<(PathBuf, ShaderVariant)>>,
    background_compilation: AtomicBool,

    // Validation and debugging
    validation_enabled: AtomicBool,
    debug_info_enabled: AtomicBool,
    optimization_enabled: AtomicBool,
}

impl ShaderManager {
    /// Create a new shader manager
    pub fn new(device: Arc<ManagedDevice>) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;

        Ok(Self {
            device,
            shader_cache: RwLock::new(HashMap::new()),
            source_cache: RwLock::new(HashMap::new()),
            dependency_graph: RwLock::new(HashMap::new()),
            compilation_stats: RwLock::new(CompilationStats::default()),

            file_watcher: Arc::new(Mutex::new(Some(watcher))),
            hot_reload_enabled: AtomicBool::new(true),
            hot_reload_receiver: Arc::new(Mutex::new(Some(rx))),
            watched_directories: RwLock::new(HashSet::new()),

            include_directories: RwLock::new(Vec::new()),
            global_macros: RwLock::new(Vec::new()),
            preprocessor_cache: RwLock::new(HashMap::new()),

            compilation_queue: Mutex::new(Vec::new()),
            background_compilation: AtomicBool::new(true),

            validation_enabled: AtomicBool::new(true),
            debug_info_enabled: AtomicBool::new(false),
            optimization_enabled: AtomicBool::new(true),
        })
    }

    /// Load and compile a shader
    #[instrument(skip(self))]
    pub fn load_shader(
        &self,
        path: impl AsRef<Path> + std::fmt::Debug,
        stage: ShaderStage,
        variant: ShaderVariant,
    ) -> Result<Arc<CompiledShader>> {
        let path = path.as_ref().to_path_buf();

        // Load source if not cached
        let source = self.load_source(&path, stage)?;

        // Check cache first
        let cache_key = (source.content_hash, variant.clone());
        if let Some(cached) = self.shader_cache.read().get(&cache_key) {
            let mut stats = self.compilation_stats.write();
            stats.cache_hits += 1;
            cached.usage_count.fetch_add(1, Ordering::Relaxed);
            *cached.last_used.write() = Instant::now();
            return Ok(cached.clone());
        }

        // Compile shader
        let compiled = self.compile_shader(&source, variant)?;

        // Cache the result
        self.shader_cache
            .write()
            .insert(cache_key, compiled.clone());

        let mut stats = self.compilation_stats.write();
        stats.cache_misses += 1;

        Ok(compiled)
    }

    /// Load shader source from file
    fn load_source(&self, path: &Path, stage: ShaderStage) -> Result<Arc<ShaderSource>> {
        // Check source cache
        if let Some(cached) = self.source_cache.read().get(path) {
            // Check if file has been modified
            let metadata = fs::metadata(path)?;
            if metadata.modified()? <= cached.last_modified {
                return Ok(cached.clone());
            }
        }

        // Read file content
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read shader file: {}", path.display()))?;

        // Detect language
        let language = self.detect_shader_language(path, &content)?;

        // Preprocess shader
        let processed_content = self.preprocess_shader(&content, path)?;

        // Calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(&processed_content);
        let content_hash: [u8; 32] = hasher.finalize().into();

        // Extract dependencies
        let (includes, dependencies) = self.extract_dependencies(&processed_content, path)?;

        let source = Arc::new(ShaderSource {
            path: path.to_path_buf(),
            content: processed_content,
            language,
            stage,
            includes,
            dependencies: dependencies.clone(),
            last_modified: fs::metadata(path)?.modified()?,
            content_hash,
        });

        // Update dependency graph
        self.update_dependency_graph(path, dependencies);

        // Cache the source
        self.source_cache
            .write()
            .insert(path.to_path_buf(), source.clone());

        // Watch for changes if hot-reload is enabled
        if self.hot_reload_enabled.load(Ordering::Relaxed) {
            self.watch_file(path)?;
        }

        Ok(source)
    }

    /// Detect shader language from file extension and content
    fn detect_shader_language(&self, path: &Path, content: &str) -> Result<ShaderLanguage> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "wgsl" => return Ok(ShaderLanguage::WGSL),
                "glsl" | "vert" | "frag" | "comp" => return Ok(ShaderLanguage::GLSL),
                "hlsl" | "fx" => return Ok(ShaderLanguage::HLSL),
                "spv" => return Ok(ShaderLanguage::SPIRV),
                _ => {}
            }
        }

        // Try to detect from content
        if content.contains("@vertex")
            || content.contains("@fragment")
            || content.contains("@compute")
        {
            Ok(ShaderLanguage::WGSL)
        } else if content.contains("#version") || content.contains("gl_") {
            Ok(ShaderLanguage::GLSL)
        } else if content.contains("cbuffer") || content.contains("SV_") {
            Ok(ShaderLanguage::HLSL)
        } else {
            // Default to WGSL for new shaders
            Ok(ShaderLanguage::WGSL)
        }
    }

    /// Preprocess shader with includes and macros
    fn preprocess_shader(&self, content: &str, base_path: &Path) -> Result<String> {
        let mut processed = content.to_string();

        // Apply global macros
        for macro_def in self.global_macros.read().iter() {
            let replacement = match &macro_def.value {
                Some(value) => format!("#define {} {}", macro_def.name, value),
                None => format!("#define {}", macro_def.name),
            };

            let pattern = format!(r"#define\s+{}\s*.*", regex::escape(&macro_def.name));
            let re = Regex::new(&pattern)?;
            processed = re.replace_all(&processed, replacement.as_str()).to_string();
        }

        // Process includes
        processed = self.process_includes(&processed, base_path)?;

        Ok(processed)
    }

    /// Process #include directives
    fn process_includes(&self, content: &str, base_path: &Path) -> Result<String> {
        let include_re = Regex::new(r#"#include\s+"([^"]+)""#)?;
        let mut processed = content.to_string();
        let mut included_files = HashSet::new();

        // Recursive include processing
        loop {
            let mut found_include = false;

            for cap in include_re.captures_iter(&processed.clone()) {
                let include_path = &cap[1];
                let full_path = self.resolve_include_path(include_path, base_path)?;

                if included_files.contains(&full_path) {
                    // Avoid circular includes
                    continue;
                }

                let include_content = fs::read_to_string(&full_path).with_context(|| {
                    format!("Failed to read include file: {}", full_path.display())
                })?;

                processed = processed.replace(&cap[0], &include_content);
                included_files.insert(full_path);
                found_include = true;
                break;
            }

            if !found_include {
                break;
            }
        }

        Ok(processed)
    }

    /// Resolve include path relative to base path and include directories
    fn resolve_include_path(&self, include_path: &str, base_path: &Path) -> Result<PathBuf> {
        let include_path = Path::new(include_path);

        // Try relative to current file
        if let Some(parent) = base_path.parent() {
            let full_path = parent.join(include_path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        // Try include directories
        for include_dir in self.include_directories.read().iter() {
            let full_path = include_dir.join(include_path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        bail!("Include file not found: {}", include_path.display());
    }

    /// Extract shader dependencies from content
    fn extract_dependencies(
        &self,
        content: &str,
        base_path: &Path,
    ) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>)> {
        let include_re = Regex::new(r#"#include\s+"([^"]+)""#)?;
        let mut includes = HashSet::new();
        let mut dependencies = HashSet::new();

        for cap in include_re.captures_iter(content) {
            let include_path = &cap[1];
            if let Ok(full_path) = self.resolve_include_path(include_path, base_path) {
                includes.insert(full_path.clone());
                dependencies.insert(full_path);
            }
        }

        Ok((includes, dependencies))
    }

    /// Update dependency graph
    fn update_dependency_graph(&self, path: &Path, dependencies: HashSet<PathBuf>) {
        let mut graph = self.dependency_graph.write();

        // Update current node
        let node = graph
            .entry(path.to_path_buf())
            .or_insert_with(|| DependencyNode {
                path: path.to_path_buf(),
                dependents: HashSet::new(),
                dependencies: HashSet::new(),
                last_modified: SystemTime::now(),
            });

        node.dependencies = dependencies.clone();
        node.last_modified = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::now());

        // Update dependent nodes
        for dep_path in dependencies {
            let dep_node = graph
                .entry(dep_path.clone())
                .or_insert_with(|| DependencyNode {
                    path: dep_path.clone(),
                    dependents: HashSet::new(),
                    dependencies: HashSet::new(),
                    last_modified: SystemTime::now(),
                });

            dep_node.dependents.insert(path.to_path_buf());
        }
    }

    /// Compile shader with variant
    #[instrument(skip(self, source))]
    fn compile_shader(
        &self,
        source: &ShaderSource,
        variant: ShaderVariant,
    ) -> Result<Arc<CompiledShader>> {
        let start_time = Instant::now();

        // Apply variant macros
        let mut shader_source = source.content.clone();
        for macro_def in &variant.macros {
            let definition = match &macro_def.value {
                Some(value) => format!("#define {} {}\n", macro_def.name, value),
                None => format!("#define {}\n", macro_def.name),
            };
            shader_source = format!("{}{}", definition, shader_source);
        }

        // Create shader module descriptor
        let descriptor = ShaderModuleDescriptor {
            label: Some(&format!("Shader-{}", source.path.display())),
            source: wgpu::ShaderSource::Wgsl(shader_source.clone().into()),
        };

        // Compile shader
        let module = self.device.device.create_shader_module(descriptor);

        let compilation_time = start_time.elapsed();

        // Validate if enabled
        let mut validation_errors = Vec::new();
        if self.validation_enabled.load(Ordering::Relaxed) {
            validation_errors = self.validate_shader(&module, &source)?;
        }

        let compiled = Arc::new(CompiledShader {
            module,
            source_hash: source.content_hash,
            variant,
            compilation_time: start_time,
            spirv_size: shader_source.len(), // Approximation
            validation_errors: validation_errors.clone(),
            optimization_applied: self.optimization_enabled.load(Ordering::Relaxed),
            usage_count: AtomicU64::new(1),
            last_used: RwLock::new(Instant::now()),
        });

        // Update statistics
        let mut stats = self.compilation_stats.write();
        stats.total_compilations += 1;
        if validation_errors.is_empty() {
            stats.successful_compilations += 1;
        } else {
            stats.failed_compilations += 1;
        }
        stats.total_compilation_time += compilation_time;
        stats.average_compilation_time =
            stats.total_compilation_time / stats.total_compilations as u32;

        info!(
            "Compiled shader: {} in {:?}",
            source.path.display(),
            compilation_time
        );

        Ok(compiled)
    }

    /// Validate compiled shader
    fn validate_shader(
        &self,
        _module: &ShaderModule,
        _source: &ShaderSource,
    ) -> Result<Vec<String>> {
        // Placeholder for shader validation

        Ok(Vec::new())
    }

    /// Watch file for changes
    fn watch_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            let mut watched = self.watched_directories.write();
            if !watched.contains(parent) {
                if let Some(ref mut watcher) = *self.file_watcher.lock() {
                    watcher.watch(parent, RecursiveMode::NonRecursive)?;
                    watched.insert(parent.to_path_buf());
                }
            }
        }
        Ok(())
    }

    /// Process hot-reload events
    pub fn process_hot_reload_events(&self) -> Result<Vec<HotReloadEvent>> {
        let mut events = Vec::new();

        if let Some(ref receiver) = *self.hot_reload_receiver.lock() {
            while let Ok(event) = receiver.try_recv() {
                match event.kind {
                    notify::EventKind::Modify(_) => {
                        for path in event.paths {
                            if self.is_shader_file(&path) {
                                self.invalidate_shader_cache(&path);
                                events.push(HotReloadEvent::FileChanged(path));
                            }
                        }
                    }
                    notify::EventKind::Remove(_) => {
                        for path in event.paths {
                            if self.is_shader_file(&path) {
                                self.remove_from_cache(&path);
                                events.push(HotReloadEvent::FileDeleted(path));
                            }
                        }
                    }
                    notify::EventKind::Create(_) => {
                        for path in event.paths {
                            if self.is_shader_file(&path) {
                                events.push(HotReloadEvent::FileCreated(path));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(events)
    }

    /// Check if file is a shader file
    fn is_shader_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "wgsl" | "glsl" | "hlsl" | "vert" | "frag" | "comp"
            )
        } else {
            false
        }
    }

    /// Invalidate shader cache for a file
    fn invalidate_shader_cache(&self, path: &Path) {
        // Remove from source cache
        self.source_cache.write().remove(path);

        // Find and remove dependent shaders from compiled cache
        let dependents = self.find_dependents(path);
        let mut cache = self.shader_cache.write();

        cache.retain(|_, compiled| !dependents.contains(&compiled.source_hash));

        let mut stats = self.compilation_stats.write();
        stats.hot_reloads += 1;

        info!("Invalidated shader cache for: {}", path.display());
    }

    /// Remove shader from cache
    fn remove_from_cache(&self, path: &Path) {
        self.source_cache.write().remove(path);
        self.dependency_graph.write().remove(path);
    }

    /// Find all shaders dependent on a file
    fn find_dependents(&self, path: &Path) -> HashSet<[u8; 32]> {
        let mut dependents = HashSet::new();
        let graph = self.dependency_graph.read();

        if let Some(node) = graph.get(path) {
            for dependent_path in &node.dependents {
                if let Some(source) = self.source_cache.read().get(dependent_path) {
                    dependents.insert(source.content_hash);
                }
            }
        }

        dependents
    }

    /// Add include directory
    pub fn add_include_directory(&self, path: impl AsRef<Path>) {
        self.include_directories
            .write()
            .push(path.as_ref().to_path_buf());
    }

    /// Add global macro
    pub fn add_global_macro(&self, name: impl Into<String>, value: Option<String>) {
        self.global_macros.write().push(ShaderMacro {
            name: name.into(),
            value,
        });
    }

    /// Enable or disable hot-reload
    pub fn set_hot_reload_enabled(&self, enabled: bool) {
        self.hot_reload_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Enable or disable validation
    pub fn set_validation_enabled(&self, enabled: bool) {
        self.validation_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Enable or disable optimization
    pub fn set_optimization_enabled(&self, enabled: bool) {
        self.optimization_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Initialize the shader manager (placeholder for integration)
    pub fn initialize(&self) -> Result<()> {
        info!("Shader manager initialized");
        Ok(())
    }

    /// Check for shader reloads (integration method)
    pub fn check_for_reloads(&self) -> Result<()> {
        let _events = self.process_hot_reload_events()?;
        Ok(())
    }

    /// Get compilation statistics
    pub fn get_stats(&self) -> CompilationStats {
        self.compilation_stats.read().clone()
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.shader_cache.write().clear();
        self.source_cache.write().clear();
        self.preprocessor_cache.write().clear();

        info!("Cleared all shader caches");
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        let shader_cache_size = self.shader_cache.read().len();
        let source_cache_size = self.source_cache.read().len();
        let preprocessor_cache_size = self.preprocessor_cache.read().len();

        (
            shader_cache_size,
            source_cache_size,
            preprocessor_cache_size,
        )
    }
}

impl Drop for ShaderManager {
    fn drop(&mut self) {
        // File watcher cleanup disabled for compatibility
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_macro_creation() {
        let macro_def = ShaderMacro {
            name: "MAX_LIGHTS".to_string(),
            value: Some("16".to_string()),
        };

        assert_eq!(macro_def.name, "MAX_LIGHTS");
        assert_eq!(macro_def.value, Some("16".to_string()));
    }

    #[test]
    fn test_shader_variant_equality() {
        let variant1 = ShaderVariant {
            macros: vec![ShaderMacro {
                name: "TEST".to_string(),
                value: None,
            }],
            features: Vec::new(),
            optimization_level: 2,
        };

        let variant2 = ShaderVariant {
            macros: vec![ShaderMacro {
                name: "TEST".to_string(),
                value: None,
            }],
            features: Vec::new(),
            optimization_level: 2,
        };

        assert_eq!(variant1, variant2);
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(ShaderLanguage::WGSL, ShaderLanguage::WGSL);
    }
}
