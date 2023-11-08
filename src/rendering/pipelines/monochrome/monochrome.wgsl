struct MonochromeCell {
    x: i32,
    y: i32,
    r: f32,
    g: f32,
    b: f32,
    glyph_index: u32,
}

struct GlyphInfo {
    size: vec2<u32>,
    offset: vec2<i32>,
}

struct PushConstants {
    target_size: vec2<u32>,
    offset: vec2<i32>,
    z: f32,
}

struct HighlightInfo {
    fg: vec3<f32>,
    bg: vec3<f32>,
}

var<push_constant> constants: PushConstants;

@group(0) @binding(0)
var<storage, read> highlights: array<HighlightInfo>;
@group(1) @binding(0)
var glyph_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var glyph_sampler: sampler;
@group(1) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;
@group(2) @binding(0)
var<storage, read> cells: array<MonochromeCell>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_index: u32,
    @location(1) tex_coord: vec2<f32>,
    @location(2) fg: vec3<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let grid_cell = cells[grid_index];
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_info = glyphs[grid_cell.glyph_index];
    let position = vec2<i32>(grid_cell.x, grid_cell.y);

    var out: VertexOutput;
    out.fg = vec3<f32>(grid_cell.r, grid_cell.g, grid_cell.b);
    out.tex_index = grid_cell.glyph_index;
    out.tex_coord = tex_coord;
    out.clip_position = vec4<f32>(
        (
            vec2<f32>(position + constants.offset + glyph_info.offset) + 
            tex_coord * vec2<f32>(glyph_info.size)
        ) / vec2<f32>(constants.target_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        constants.z, 
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSampleLevel(
        glyph_textures[in.tex_index],
        glyph_sampler,
        in.tex_coord,
        0.0
    );
    return vec4<f32>(in.fg, sample.r);
}
