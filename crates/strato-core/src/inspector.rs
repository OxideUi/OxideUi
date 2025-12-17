use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use parking_lot::RwLock;

use crate::state::StateId;
use crate::types::Rect;
use crate::widget::WidgetId;

/// Configuration for the runtime inspector.
#[derive(Debug, Clone)]
pub struct InspectorConfig {
    /// Whether the inspector is allowed to capture runtime information.
    pub enabled: bool,
    /// Whether layout bounds should be captured on every frame.
    pub capture_layout: bool,
    /// Whether state mutations should be snapshotted.
    pub capture_state: bool,
    /// Whether performance timelines should be tracked.
    pub capture_performance: bool,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(debug_assertions),
            capture_layout: true,
            capture_state: true,
            capture_performance: true,
        }
    }
}

/// A single component record in the captured hierarchy.
#[derive(Debug, Clone)]
pub struct ComponentNodeSnapshot {
    pub id: WidgetId,
    pub name: String,
    pub depth: usize,
    pub props: HashMap<String, String>,
    pub state: HashMap<String, String>,
}

/// Captured layout box for a widget.
#[derive(Debug, Clone, Copy)]
pub struct LayoutBoxSnapshot {
    pub widget_id: WidgetId,
    pub bounds: Rect,
}

/// Captured state change metadata.
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub state_id: StateId,
    pub detail: String,
    pub recorded_at: SystemTime,
}

/// Performance timeline entry (per frame).
#[derive(Debug, Clone)]
pub struct FrameTimelineSnapshot {
    pub frame_id: u64,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub notes: Option<String>,
}

/// Complete snapshot of inspector data for rendering in the overlay.
#[derive(Debug, Clone)]
pub struct InspectorSnapshot {
    pub components: Vec<ComponentNodeSnapshot>,
    pub layout_boxes: Vec<LayoutBoxSnapshot>,
    pub state_snapshots: Vec<StateSnapshot>,
    pub frame_timelines: Vec<FrameTimelineSnapshot>,
}

impl Default for InspectorSnapshot {
    fn default() -> Self {
        Self {
            components: Vec::new(),
            layout_boxes: Vec::new(),
            state_snapshots: Vec::new(),
            frame_timelines: Vec::new(),
        }
    }
}

/// Runtime inspector for StratoSDK that aggregates data from multiple layers.
pub struct Inspector {
    config: RwLock<InspectorConfig>,
    enabled: AtomicBool,
    components: RwLock<Vec<ComponentNodeSnapshot>>,
    layout_boxes: RwLock<Vec<LayoutBoxSnapshot>>,
    state_snapshots: RwLock<HashMap<StateId, StateSnapshot>>,
    frame_timelines: RwLock<Vec<FrameTimelineSnapshot>>,
}

impl Inspector {
    fn new() -> Self {
        let config = InspectorConfig::default();
        Self {
            enabled: AtomicBool::new(config.enabled),
            components: RwLock::new(Vec::new()),
            layout_boxes: RwLock::new(Vec::new()),
            state_snapshots: RwLock::new(HashMap::new()),
            frame_timelines: RwLock::new(Vec::new()),
            config: RwLock::new(config),
        }
    }

    /// Update the inspector configuration at runtime.
    pub fn configure(&self, config: InspectorConfig) {
        *self.config.write() = config.clone();
        self.enabled.store(config.enabled, Ordering::Relaxed);
    }

    /// Get the current configuration.
    pub fn config(&self) -> InspectorConfig {
        self.config.read().clone()
    }

    /// Check if the inspector is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable the inspector without replacing the full configuration.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        self.config.write().enabled = enabled;
    }

    /// Toggle inspector visibility.
    pub fn toggle(&self) -> bool {
        let next = !self.is_enabled();
        self.set_enabled(next);
        next
    }

    /// Reset transient per-frame information so the overlay always shows the latest data.
    pub fn begin_frame(&self) {
        self.layout_boxes.write().clear();
        self.components.write().clear();
    }

    /// Replace the captured widget hierarchy for the current frame.
    pub fn record_component_tree(&self, nodes: Vec<ComponentNodeSnapshot>) {
        if !self.is_enabled() {
            return;
        }
        *self.components.write() = nodes;
    }

    /// Record a layout box for a widget.
    pub fn record_layout_box(&self, snapshot: LayoutBoxSnapshot) {
        if !self.is_enabled() || !self.config().capture_layout {
            return;
        }
        self.layout_boxes.write().push(snapshot);
    }

    /// Record a state mutation/snapshot.
    pub fn record_state_snapshot(&self, state_id: StateId, detail: impl Into<String>) {
        if !self.is_enabled() || !self.config().capture_state {
            return;
        }

        self.state_snapshots.write().insert(
            state_id,
            StateSnapshot {
                state_id,
                detail: detail.into(),
                recorded_at: SystemTime::now(),
            },
        );
    }

    /// Record a per-frame performance timeline entry.
    pub fn record_frame_timeline(
        &self,
        frame_id: u64,
        cpu_time: Duration,
        gpu_time: Duration,
        notes: Option<String>,
    ) {
        if !self.is_enabled() || !self.config().capture_performance {
            return;
        }

        self.frame_timelines.write().push(FrameTimelineSnapshot {
            frame_id,
            cpu_time_ms: (cpu_time.as_secs_f64() * 1000.0) as f32,
            gpu_time_ms: (gpu_time.as_secs_f64() * 1000.0) as f32,
            notes,
        });
    }

    /// Get the full snapshot used by the inspector overlay widget.
    pub fn snapshot(&self) -> InspectorSnapshot {
        InspectorSnapshot {
            components: self.components.read().clone(),
            layout_boxes: self.layout_boxes.read().clone(),
            state_snapshots: self.state_snapshots.read().values().cloned().collect(),
            frame_timelines: self.frame_timelines.read().clone(),
        }
    }
}

static INSPECTOR: OnceLock<Inspector> = OnceLock::new();

/// Access the global inspector instance used by all layers.
pub fn inspector() -> &'static Inspector {
    INSPECTOR.get_or_init(Inspector::new)
}
