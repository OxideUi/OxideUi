//! Grid widget for 2D layout
use crate::widget::{generate_id, Widget, WidgetId};
use std::any::Any;
use strato_core::{
    event::{Event, EventResult},
    layout::{Constraints, Layout, Size},
};
use strato_renderer::batch::RenderBatch;

/// Unit for grid tracks (rows/columns)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridUnit {
    /// Fixed size in pixels
    Pixel(f32),
    /// Fraction of available space (fr)
    Fraction(f32),
    /// Auto size (fits content)
    Auto,
}

/// Grid widget for 2D layout
#[derive(Debug)]
pub struct Grid {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    rows: Vec<GridUnit>,
    cols: Vec<GridUnit>,
    row_gap: f32,
    col_gap: f32,
    // Store layout results for rendering
    cached_child_layouts: Vec<Layout>,
}

impl Grid {
    /// Create a new grid
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            children: Vec::new(),
            rows: Vec::new(),
            cols: Vec::new(),
            row_gap: 0.0,
            col_gap: 0.0,
            cached_child_layouts: Vec::new(),
        }
    }

    /// Set columns template
    pub fn columns(mut self, cols: Vec<GridUnit>) -> Self {
        self.cols = cols;
        self
    }

    /// Set rows template
    pub fn rows(mut self, rows: Vec<GridUnit>) -> Self {
        self.rows = rows;
        self
    }

    /// Set gap between rows
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
    }

    /// Set gap between columns
    pub fn col_gap(mut self, gap: f32) -> Self {
        self.col_gap = gap;
        self
    }

    /// Add children
    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    /// Add a single child
    pub fn child(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }
}

impl Widget for Grid {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        // If no columns defined, default to 1 column auto
        if self.cols.is_empty() {
            self.cols.push(GridUnit::Auto);
        }
        // If no rows defined, we will implicitly add auto rows as needed

        let num_cols = self.cols.len();
        let num_children = self.children.len();
        let implicit_rows_needed = (num_children as f32 / num_cols as f32).ceil() as usize;

        // Final rows list including implicit ones
        let mut final_rows = self.rows.clone();
        while final_rows.len() < implicit_rows_needed {
            final_rows.push(GridUnit::Auto);
        }
        let num_rows = final_rows.len();

        // 1. Calculate available space
        let available_width = constraints.max_width;
        let available_height = constraints.max_height;

        // 2. Resolve Track Sizes (simplistic implementation)
        // First pass: Calculate fixed and auto sizes
        let mut col_widths = vec![0.0; num_cols];
        let mut row_heights = vec![0.0; num_rows];

        // Helper to get child at (row, col)
        let get_child_idx = |r, c| r * num_cols + c;

        // Measure AUTO tracks
        for r in 0..num_rows {
            for c in 0..num_cols {
                let idx = get_child_idx(r, c);
                if idx >= self.children.len() {
                    continue;
                }

                let is_col_auto = matches!(self.cols[c], GridUnit::Auto);
                let is_row_auto = matches!(final_rows[r], GridUnit::Auto);

                if is_col_auto || is_row_auto {
                    // Measure content
                    // TODO: This is naive. True grid layout is complex.
                    // We measure with loose constraints to get content size.
                    let measure_constraints = Constraints::loose(available_width, available_height);
                    let size = self.children[idx].layout(measure_constraints);

                    if is_col_auto {
                        col_widths[c] = f32::max(col_widths[c], size.width);
                    }
                    if is_row_auto {
                        row_heights[r] = f32::max(row_heights[r], size.height);
                    }
                }
            }
        }

        // Measure FIXED tracks
        for (c, unit) in self.cols.iter().enumerate() {
            if let GridUnit::Pixel(px) = unit {
                col_widths[c] = *px;
            }
        }
        for (r, unit) in final_rows.iter().enumerate() {
            if let GridUnit::Pixel(px) = unit {
                row_heights[r] = *px;
            }
        }

        // Measure FRACTION tracks
        let used_width: f32 =
            col_widths.iter().sum::<f32>() + (num_cols.saturating_sub(1) as f32 * self.col_gap);
        let remaining_width = (available_width - used_width).max(0.0);
        let total_col_fr: f32 = self.cols.iter().fold(0.0, |acc, u| {
            if let GridUnit::Fraction(fr) = u {
                acc + fr
            } else {
                acc
            }
        });

        if total_col_fr > 0.0 {
            for (c, unit) in self.cols.iter().enumerate() {
                if let GridUnit::Fraction(fr) = unit {
                    col_widths[c] = (fr / total_col_fr) * remaining_width;
                }
            }
        }

        // For rows, we often don't have a fixed height container, so fractions might be tricky.
        // If we have infinite height constraint, fractions might resolve to 0 or behave like Auto.
        // Here we assume if height is constrained, we distribute.
        if available_height.is_finite() {
            let used_height: f32 = row_heights.iter().sum::<f32>()
                + (num_rows.saturating_sub(1) as f32 * self.row_gap);
            let remaining_height = (available_height - used_height).max(0.0);
            let total_row_fr: f32 = final_rows.iter().fold(0.0, |acc, u| {
                if let GridUnit::Fraction(fr) = u {
                    acc + fr
                } else {
                    acc
                }
            });

            if total_row_fr > 0.0 {
                for (r, unit) in final_rows.iter().enumerate() {
                    if let GridUnit::Fraction(fr) = unit {
                        row_heights[r] = (fr / total_row_fr) * remaining_height;
                    }
                }
            }
        }
        // If height is infinite, treat fractions as auto or 0?
        // For now, let's treat as 0 or maybe min size. In real CSS grid they collapse to content if height is indefinite.
        // We leave them as 0 if not calculated above, unless we implement content-based minimums for fr tracks.

        // 3. Position Children and Re-layout with precise constraints
        self.cached_child_layouts.clear();
        let mut total_width = 0.0f32;
        let mut total_height = 0.0f32;

        let mut current_y = 0.0;
        for r in 0..num_rows {
            let mut current_x = 0.0;
            let row_h = row_heights[r];

            for c in 0..num_cols {
                let idx = get_child_idx(r, c);
                let col_w = col_widths[c];

                if idx < self.children.len() {
                    let cell_x = current_x;
                    let cell_y = current_y;

                    // Re-layout child with exact cell size
                    // We force the child to fit the cell? Or align it?
                    // Typically grid items stretch to fill cell unless aligned.
                    // We'll enforce loose constraints up to cell size, but tight might be better for stretch.
                    // Let's use tight for compatibility with "stretch" default behavior.
                    let cell_constraints = Constraints::tight(col_w, row_h);
                    // Note: If row_h is 0 (e.g. empty fr track), this hides the child.

                    self.children[idx].layout(cell_constraints);

                    self.cached_child_layouts.push(Layout::new(
                        glam::Vec2::new(cell_x, cell_y),
                        Size::new(col_w, row_h),
                    ));
                }

                current_x += col_w + self.col_gap;
            }

            total_width = total_width.max(current_x - self.col_gap); // remove last gap
            current_y += row_h + self.row_gap;
        }
        total_height = current_y - self.row_gap; // remove last gap

        Size::new(total_width, total_height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        for (i, child) in self.children.iter().enumerate() {
            if let Some(child_layout) = self.cached_child_layouts.get(i) {
                let absolute_layout =
                    Layout::new(layout.position + child_layout.position, child_layout.size);
                child.render(batch, absolute_layout);
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        for child in &mut self.children {
            if child.handle_event(event) == EventResult::Handled {
                return EventResult::Handled;
            }
        }
        EventResult::Ignored
    }

    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn children_mut<'a>(&'a mut self) -> Vec<&'a mut (dyn Widget + 'a)> {
        self.children
            .iter_mut()
            .map(|c| c.as_mut() as &'a mut (dyn Widget + 'a))
            .collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Grid {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
            rows: self.rows.clone(),
            cols: self.cols.clone(),
            row_gap: self.row_gap,
            col_gap: self.col_gap,
            cached_child_layouts: Vec::new(),
        })
    }
}
