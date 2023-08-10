struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// @group(0) @binding(2)
// var<uniform> fg: array<vec3<f32>>;

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
    // out.mul = fg[in_vertex_index / 6u];
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Uniforms
@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var s_texture: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample = textureSample(t_texture, s_texture, in.tex_coords);
    return vec4<f32>(in.mul, sample.r);
}
