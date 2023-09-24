struct PushConstantsVertex {
    position: vec2<u32>,
    surface_size: vec2<u32>,
    glyphs: array<u32, 8>,
}

struct PushConstantsFragment {
    color: vec3<f32>,
}

struct PushConstants {
    vertex: PushConstantsVertex,
    fragment: PushConstantsFragment,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_index: u32,
    @location(1) tex_coord: vec2<f32>,
}

struct GlyphInfo {
    size: vec2<u32>,
}

struct HighlightInfo {
    fg: vec3<f32>,
    bg: vec3<f32>,
}

var<push_constant> push_constants: PushConstants;

@group(0) @binding(0)
var<storage, read> highlights: array<HighlightInfo>;
@group(1) @binding(0)
var glyph_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var glyph_sampler: sampler;
@group(1) @binding(2)
var<storage, read> glyphs: array<GlyphInfo>;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let glyph_index = push_constants.vertex.glyphs[in_vertex_index / 6u];
    let glyph_info = glyphs[glyph_index];

    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        (vec2<f32>(push_constants.vertex.position) + tex_coord * vec2<f32>(glyph_info.size))
        / vec2<f32>(push_constants.vertex.surface_size) 
        * vec2<f32>(2.0, -2.0) 
        + vec2<f32>(-1.0, 1.0),
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
    return vec4<f32>(push_constants.fragment.color, sample.r);
}
