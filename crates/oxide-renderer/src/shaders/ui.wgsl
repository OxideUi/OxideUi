// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) flags: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var text_texture: texture_2d<f32>;

@group(0) @binding(2)
var text_sampler: sampler;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.clip_position = uniforms.projection * vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Check if this is a text vertex (using tex_coords to determine)
    // For debug rectangles and solid shapes, tex_coords will be (0,0)
    if (in.tex_coords.x != 0.0 || in.tex_coords.y != 0.0) {
        // Text rendering: sample from texture atlas and multiply by vertex color
        let text_sample = textureSample(text_texture, text_sampler, in.tex_coords);
        // Use the alpha from texture for text transparency
        return vec4<f32>(in.color.rgb, in.color.a * text_sample.a);
    } else {
        // Solid color rendering (rectangles, shapes, etc.)
        return in.color;
    }
}
