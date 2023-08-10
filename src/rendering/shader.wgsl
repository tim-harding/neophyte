struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct GridCell {
    color: vec3<f32>,
}

struct GlyphInfo {
    atlas_origin: vec2<u32>,
    size: vec2<u32>,
    placement_offset: vec2<i32>,
}

@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var s_texture: sampler;
@group(0) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;
@group(1) @binding(0)
var<storage, read> grid_cells: array<GridCell>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) mul: vec3<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.mul = grid_cells[in_vertex_index / 6u].color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample = textureSample(t_texture, s_texture, in.tex_coords);
    return vec4<f32>(in.mul, sample.r);
}
