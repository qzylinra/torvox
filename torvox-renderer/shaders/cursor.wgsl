// Cursor rendering shader - solid color rectangle

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Uniforms {
    projection: mat4x4<f32>,
    cell_size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) corner: vec2<f32>,
    @location(1) cursor_pos: vec2<f32>,
    @location(2) cursor_size: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform cursor position to screen space
    let screen_pos = (in.cursor_pos + in.corner * 0.5) * uniforms.cell_size;

    // Apply projection matrix
    out.position = uniforms.projection * vec4<f32>(screen_pos, 0.0, 1.0);

    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
