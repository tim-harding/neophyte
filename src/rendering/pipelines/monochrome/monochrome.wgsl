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
    origin: vec2<u32>,
}

struct PushConstants {
    target_size: vec2<u32>,
    offset: vec2<i32>,
    z: f32,
    atlas_size: u32,
}

var<push_constant> constants: PushConstants;

@group(0) @binding(0)
var atlas: texture_2d<f32>;
@group(0) @binding(1)
var glyph_sampler: sampler;
@group(0) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;
@group(1) @binding(0)
var<storage, read> cells: array<MonochromeCell>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) fg: vec3<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let grid_cell = cells[grid_index];
    let corner = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_info = glyphs[grid_cell.glyph_index];
    let position = vec2<i32>(grid_cell.x, grid_cell.y);
    let atlas_dim = f32(constants.atlas_size);
    let atlas_size = vec2<f32>(atlas_dim, atlas_dim);
    let origin_uv = vec2<f32>(glyph_info.origin) / atlas_size;
    let size_uv = vec2<f32>(glyph_info.size) / atlas_size;

    var out: VertexOutput;
    out.tex_coord = origin_uv + size_uv * corner;
    out.clip_position = vec4<f32>(
        (
            vec2<f32>(position + constants.offset + glyph_info.offset) + 
            corner * vec2<f32>(glyph_info.size)
        ) / vec2<f32>(constants.target_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        constants.z, 
        1.0
    );
    out.fg = vec3<f32>(grid_cell.r, grid_cell.g, grid_cell.b);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSampleLevel(
        atlas,
        glyph_sampler,
        in.tex_coord,
        0.0
    );
    return vec4<f32>(in.fg, sample.r);
}
