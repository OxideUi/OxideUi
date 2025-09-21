// === STRUCTS & BINDINGS ===

struct Uniforms {
    projection: mat4x4<f32>,
    // You could also add `resolution: vec2<f32>` for shaders needing the window size
}

struct VertexInput {
    @location(0) position: vec2<f32>,   // 2D position is sufficient
    @location(1) color: vec4<f32>,      // Primary color
    @location(2) uv: vec2<f32>,         // UV coordinates or geometric data
    @location(3) params: vec4<f32>,     // Additional parameters (e.g., corner radius)
    @location(4) flags: u32,            // Bitmask for the rendering type
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) flags: u32, // Use 'flat' for flags; they don't need interpolation
    @location(1) @interpolate(perspective) color: vec4<f32>,
    @location(2) @interpolate(perspective) uv: vec2<f32>,
    @location(3) @interpolate(perspective) params: vec4<f32>,
    @location(4) @interpolate(perspective) world_position: vec2<f32>, // Original position in pixels for geometric calculations
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var main_sampler: sampler;
@group(0) @binding(2) var main_texture: texture_2d<f32>; // Texture Atlas for text, icons, etc.

// === BITMASK FLAGS ===
// (These should also be defined as constants in your Rust code)
const FLAG_TYPE_SOLID: u32        = 0u;
const FLAG_TYPE_TEXTURED: u32     = 1u;
const FLAG_TYPE_SDF_TEXT: u32     = 2u;
const FLAG_TYPE_ROUNDED_RECT: u32 = 3u;
const FLAG_TYPE_MASK: u32         = 3u; // Mask to extract the type (first 2 bits)


// === HELPER FUNCTIONS ===

// Renders a rounded rectangle using a Signed Distance Function
fn sdf_rounded_box(point: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(point) - size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}


// === VERTEX SHADER ===

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.world_position = in.position;
    out.flags = in.flags;
    out.color = in.color;
    out.uv = in.uv;
    out.params = in.params;
    return out;
}


// === FRAGMENT SHADER ===

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let render_type = in.flags & FLAG_TYPE_MASK;
    var final_color: vec4<f32>;

    switch (render_type) {
        // --- Path 1: Solid color shapes ---
        case FLAG_TYPE_SOLID: {
            final_color = in.color;
        }

        // --- Path 2: Simple textured shapes (images, icons) ---
        case FLAG_TYPE_TEXTURED: {
            let texture_color = textureSample(main_texture, main_sampler, in.uv);
            final_color = in.color * texture_color;
        }

        // --- Path 3: Text rendered with Signed Distance Fields (SDF) ---
        case FLAG_TYPE_SDF_TEXT: {
            // Sample the distance value from the glyph atlas (stored in the 'r' channel)
            let distance = textureSample(main_texture, main_sampler, in.uv).r;
            // `smoothstep` creates a soft, antialiased edge
            // `in.params.x` can hold the font's edge "width" or "smoothing" factor
            let edge_width = in.params.x; 
            let alpha = smoothstep(0.5 - edge_width, 0.5 + edge_width, distance);
            final_color = vec4<f32>(in.color.rgb, in.color.a * alpha);
        }

        // --- Path 4: Rounded rectangles ---
        case FLAG_TYPE_ROUNDED_RECT: {
            // Necessary parameters are passed via vertex fields
            let rect_size = in.params.xy;
            let corner_radius = in.params.z;
            
            // Calculate the distance from the rounded rectangle's border
            let dist = sdf_rounded_box(in.world_position - rect_size * 0.5, rect_size * 0.5, corner_radius);

            // `smoothstep` provides high-quality antialiasing
            // A value of 1.0 represents a 1-pixel feather.
            let alpha = 1.0 - smoothstep(0.0, 1.0, dist);
            final_color = vec4<f32>(in.color.rgb, in.color.a * alpha);
        }

        default: {
            // Fallback color for debugging (e.g., magenta)
            final_color = vec4<f32>(1.0, 0.0, 1.0, 1.0);
        }
    }

    // Handle pre-multiplied alpha if your blend mode requires it
    final_color.r *= final_color.a;
    final_color.g *= final_color.a;
    final_color.b *= final_color.a;

    return final_color;
}