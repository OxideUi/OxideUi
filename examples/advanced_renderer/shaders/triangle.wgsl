// Advanced triangle shader demonstrating the shader management system
// This shader supports hot-reload and automatic recompilation

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// Vertex shader
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

// Fragment shader with animated color effect
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple color animation based on position
    let animated_color = input.color * (sin(input.clip_position.x * 0.01) * 0.5 + 0.5);
    return vec4<f32>(animated_color, 1.0);
}