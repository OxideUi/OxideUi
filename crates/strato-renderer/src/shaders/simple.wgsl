// Simple shader for 2D rendering with texture support
// Supports both solid colors and textured rendering

// Uniform buffer for projection matrix
struct Uniforms {
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Texture and sampler for text/image rendering (optional)
@group(0) @binding(1)
var texture: texture_2d<f32>;

@group(0) @binding(2)
var texture_sampler: sampler;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

// Vertex shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture at UV coordinates
    // If UV is (0,0), use solid color (for non-textured geometry)
    // Otherwise, modulate texture with vertex color
    var tex_color = textureSample(texture, texture_sampler, in.uv);
    
    // Mix texture and vertex color
    // If texture is white (1,1,1,1) or UV is at origin, use vertex color
    // Otherwise blend texture with color
    return tex_color * in.color;
}
