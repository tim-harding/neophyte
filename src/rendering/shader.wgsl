struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct GridCell {
    color: vec4<f32>,
    glyph_index: vec4<u32>,
}

struct GlyphInfo {
    // The dimensions of the glyph texture
    size: vec2<u32>,
    // Displacement from the glyph position origin
    placement_offset: vec2<i32>,
}

// TODO: Maybe store these as f32 to avoid casting in the shader
struct GridInfo {
    // The dimensions of the texture we're drawing to
    surface_size: vec2<u32>,
    // The dimensions of the Neovim grid
    grid_size: vec2<u32>,
    // The dimensions of a single glyph. (font_height, advance)
    glyph_size: vec2<u32>,
}

@group(0) @binding(0)
var glyph_textures: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var glyph_sampler: sampler;
@group(0) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;
@group(1) @binding(0)
var<storage, read> grid_cells: array<GridCell>;
var<push_constant> grid_info: GridInfo;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_index: u32,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec3<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    model: VertexInput,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let grid_cell = grid_cells[grid_index];
    let glyph_index = grid_cell.glyph_index.r;
    let grid_coord = vec2<f32>(
        f32(grid_index % grid_info.grid_size.x),
        f32(grid_index / grid_info.grid_size.x),
    );
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_info = glyphs[glyph_index];

    var out: VertexOutput;
    out.color = grid_cell.color.rgb;
    out.tex_index = glyph_index;
    out.tex_coord = tex_coord;
    out.clip_position = vec4<f32>(
        (
            grid_coord * vec2<f32>(grid_info.glyph_size) + 
            tex_coord * vec2<f32>(glyph_info.size) +
            vec2<f32>(glyph_info.placement_offset) * vec2<f32>(1.0, -1.0) +
            vec2<f32>(0.0, 24.0)
        ) / vec2<f32>(grid_info.surface_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        0.0, 
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
