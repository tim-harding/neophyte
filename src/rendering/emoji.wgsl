struct EmojiCell {
    position: vec2<i32>,
    glyph_index: u32,
}

struct GlyphInfo {
    // The dimensions of the glyph texture
    size: vec2<u32>,
}

// TODO: Maybe store these as f32 to avoid casting in the shader
struct GridInfo {
    surface_size: vec2<u32>,
    cell_size: vec2<u32>,
    offset: vec2<f32>,
    grid_width: u32,
    baseline: u32,
}

struct HighlightInfo {
    fg: vec3<f32>,
    bg: vec3<f32>,
}

var<push_constant> grid_info: GridInfo;

@group(0) @binding(0)
var glyph_textures: binding_array<texture_2d<vec4<f32>>>;
@group(0) @binding(1)
var glyph_sampler: sampler;
@group(0) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;
@group(1) @binding(0)
var<storage, read> emoji_cells: array<EmojiCell>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_index: u32,
    @location(1) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let emoji_cell = emoji_cells[grid_index];
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_info = glyphs[emoji_cell.glyph_index];
    let hl_info = highlights[emoji_cell.highlight_index];

    var out: VertexOutput;
    out.tex_index = emoji_cell.glyph_index;
    out.tex_coord = tex_coord;
    out.clip_position = vec4<f32>(
        (
            vec2<f32>(emoji_cell.position) + 
            grid_info.offset +
            tex_coord * vec2<f32>(glyph_info.size) +
            vec2<f32>(0.0, f32(grid_info.baseline))
        ) / vec2<f32>(grid_info.surface_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        0.0, 
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSampleLevel(
        glyph_textures[in.tex_index],
        glyph_sampler,
        in.tex_coord,
        0.0
    );
}
