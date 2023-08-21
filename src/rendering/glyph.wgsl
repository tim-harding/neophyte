struct GridCell {
    glyph_index: u32,
    highlight_index: u32,
    position: vec2<i32>,
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
    z: f32,
}

struct HighlightInfo {
    fg: vec3<f32>,
    bg: vec3<f32>,
}

@group(0) @binding(0)
var<storage, read> highlights: array<HighlightInfo>;
@group(1) @binding(0)
var<storage, read> grid_cells: array<GridCell>;
var<push_constant> grid_info: GridInfo;
@group(2) @binding(0)
var glyph_textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1)
var glyph_sampler: sampler;
@group(2) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_index: u32,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec3<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let grid_cell = grid_cells[grid_index];
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_info = glyphs[grid_cell.glyph_index];
    let hl_info = highlights[grid_cell.highlight_index];

    var out: VertexOutput;
    out.color = hl_info.fg;
    out.tex_index = grid_cell.glyph_index;
    out.tex_coord = tex_coord;
    out.clip_position = vec4<f32>(
        (
            vec2<f32>(grid_cell.position) + 
            grid_info.offset +
            tex_coord * vec2<f32>(glyph_info.size) +
            vec2<f32>(0.0, f32(grid_info.baseline))
        ) / vec2<f32>(grid_info.surface_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        grid_info.z, 
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
    return vec4<f32>(in.color, sample.r);
}
