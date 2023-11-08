struct EmojiCell {
    x: i32,
    y: i32,
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

var<push_constant> constants: PushConstants;

@group(0) @binding(0)
var glyph_textures: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var glyph_sampler: sampler;
@group(0) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;
@group(1) @binding(0)
var<storage, read> cells: array<EmojiCell>;

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
    let emoji_cell = cells[grid_index];
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_info = glyphs[emoji_cell.glyph_index];
    let position = vec2<i32>(emoji_cell.x, emoji_cell.y);

    var out: VertexOutput;
    out.tex_index = emoji_cell.glyph_index;
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
    return sample;
}

