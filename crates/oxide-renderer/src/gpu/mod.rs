//! GPU rendering pipeline modules
//!
//! Modular, testable GPU rendering system built on wgpu

// BLOCCO 1: Device Setup ✅
pub mod device;

// BLOCCO 2: Surface Configuration ✅
pub mod surface;

// BLOCCO 3: Shader Compilation ✅
pub mod shader_mgr;

// BLOCCO 4: Buffer Management ✅
pub mod buffer_mgr;

// BLOCCO 5: Pipeline Creation ✅
pub mod pipeline_mgr;

// BLOCCO 6: Render Pass ✅
pub mod render_pass_mgr;

// BLOCCO 7: Drawing System ✅
pub mod drawing;

// Re-exports
pub use device::DeviceManager;
pub use surface::SurfaceManager;
pub use shader_mgr::ShaderManager;
pub use buffer_mgr::{BufferManager, SimpleVertex};
pub use pipeline_mgr::PipelineManager;
pub use render_pass_mgr::RenderPassManager;
pub use drawing::DrawingSystem;
