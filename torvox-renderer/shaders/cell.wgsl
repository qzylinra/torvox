struct Uniforms {
    projection: mat4x4<f32>,
    cell_size: vec2<f32>,
    atlas_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var atlas_texture: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) fg: vec4<f32>,
    @location(2) bg: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) instance_pos: vec2<f32>,
    @location(1) instance_uv_offset: vec2<f32>,
    @location(2) instance_uv_size: vec2<f32>,
    @location(3) instance_fg: vec4<f32>,
    @location(4) instance_bg: vec4<f32>,
    @location(5) instance_flags: f32,
) -> VertexOutput {
    let corner = array(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
    );

    let cell_origin = instance_pos * uniforms.cell_size;
    let half_cell = uniforms.cell_size * 0.5;
    let world_pos = cell_origin + half_cell + corner[vertex_index] * half_cell;
    let clip_pos = uniforms.projection * vec4<f32>(world_pos, 0.0, 1.0);

    var output: VertexOutput;
    output.position = clip_pos;
    let uv_corner = corner[vertex_index] * 0.5 + vec2<f32>(0.5);
    output.uv = instance_uv_offset + uv_corner * instance_uv_size;
    output.fg = instance_fg;
    output.bg = instance_bg;

    return output;
}

@fragment
fn fs_main(
    @location(0) uv: vec2<f32>,
    @location(1) fg: vec4<f32>,
    @location(2) bg: vec4<f32>,
) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas_texture, atlas_sampler, uv);
    let glyph_alpha = texel.a;
    let effective_alpha = glyph_alpha * fg.a;
    return mix(bg, vec4<f32>(fg.rgb, 1.0), effective_alpha);
}
