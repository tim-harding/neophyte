struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct Fg {
    color: vec3<f32>,
    index: u32,
}

@group(0) @binding(0)
var textures: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var samplers: binding_array<sampler>;
@group(0) @binding(2)
var<storage, read> fg: array<vec3<Fg>>;

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
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    let x = fg[in_vertex_index / 6u];
    out.color = x.color;
    out.tex_index = x.index;
    out.tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32((index_vertex_index - 1u) / 3u),
    );
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSampleLevel(
        textures[in.index],
        samplers[in.index],
        fragment.tex_coord,
        0.0
    );
    var sample = textureSample(t_texture, s_texture, in.tex_coords);
    return vec4<f32>(in.color, sample.r);
}
