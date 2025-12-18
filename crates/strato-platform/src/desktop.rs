//! Desktop platform implementation

use crate::event_loop::CustomEvent;
use crate::window::WindowInner;
use crate::{Platform, PlatformError, Window, WindowBuilder, WindowId};
use std::collections::HashMap;
use std::sync::Arc;
use strato_core::event::Event;

/// Desktop platform implementation
pub struct DesktopPlatform {
    event_loop: Option<winit::event_loop::EventLoop<CustomEvent>>,
    windows: HashMap<WindowId, Arc<winit::window::Window>>,
    next_window_id: WindowId,
}

impl DesktopPlatform {
    /// Create a new desktop platform
    pub fn new() -> Self {
        Self {
            event_loop: Some(
                winit::event_loop::EventLoopBuilder::with_user_event()
                    .build()
                    .expect("Failed to create event loop"),
            ),
            windows: HashMap::new(),
            next_window_id: 0,
        }
    }
}

impl Platform for DesktopPlatform {
    fn init() -> Result<Self, PlatformError> {
        Ok(Self::new())
    }

    fn create_window(&mut self, builder: WindowBuilder) -> Result<Window, PlatformError> {
        let event_loop = self
            .event_loop
            .as_ref()
            .ok_or_else(|| PlatformError::EventLoop("Event loop not available".to_string()))?;

        let winit_window = builder
            .build_winit(event_loop)
            .map_err(|e| PlatformError::WindowCreation(e.to_string()))?;

        let window_arc = Arc::new(winit_window);
        let window_id = self.next_window_id;
        self.next_window_id += 1;

        self.windows.insert(window_id, window_arc.clone());

        Ok(Window {
            id: window_id,
            inner: WindowInner::Desktop(window_arc),
        })
    }

    fn run_event_loop(
        &mut self,
        mut callback: Box<dyn FnMut(Event) + 'static>,
    ) -> Result<(), PlatformError> {
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| PlatformError::EventLoop("Event loop already taken".to_string()))?;

        use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};

        let mut cursor_position = winit::dpi::PhysicalPosition::new(0.0, 0.0);
        let mut scale_factor = 1.0;

        let _ = event_loop.run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);

            match event {
                WinitEvent::WindowEvent { event, .. } => match event {
                    WinitWindowEvent::CursorMoved {
                        position,
                        device_id,
                        ..
                    } => {
                        cursor_position = position;
                        if let Some(strato_event) = crate::event_loop::convert_window_event(
                            WinitWindowEvent::CursorMoved {
                                position,
                                device_id,
                            },
                            cursor_position,
                            scale_factor,
                        ) {
                            callback(strato_event);
                        }
                    }
                    WinitWindowEvent::ScaleFactorChanged {
                        scale_factor: sf,
                        inner_size_writer,
                    } => {
                        scale_factor = sf;
                        if let Some(strato_event) = crate::event_loop::convert_window_event(
                            WinitWindowEvent::ScaleFactorChanged {
                                scale_factor: sf,
                                inner_size_writer,
                            },
                            cursor_position,
                            scale_factor,
                        ) {
                            callback(strato_event);
                        }
                    }
                    _ => {
                        if let Some(strato_event) = crate::event_loop::convert_window_event(
                            event,
                            cursor_position,
                            scale_factor,
                        ) {
                            callback(strato_event);
                        }
                    }
                },
                WinitEvent::AboutToWait => {
                    // All events have been processed
                }
                _ => {}
            }
        });

        Ok(())
    }

    fn request_redraw(&self, window_id: WindowId) {
        if let Some(window) = self.windows.get(&window_id) {
            window.request_redraw();
        }
    }

    fn exit(&mut self) {
        std::process::exit(0);
    }
}
