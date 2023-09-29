struct VertexInfo {
    position: vec2<f32>,
    target_size: vec2<u32>,
    fill: vec2<f32>,
    cell_size: vec2<f32>,
}

struct FragmentInfo {
    fg: vec3<f32>,
    bg: vec3<f32>,
}

struct Info {
    vertex: VertexInfo,
    fragment: FragmentInfo,
}

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;
var<push_constant> info: Info;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn rev_y(v: vec2<f32>) -> vec2<f32> {
    return v * vec2<f32>(1.0, -1.0) + vec2<f32>(0.0, 1.0);
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let corner = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );

    let fill = rev_y(rev_y(corner) * info.vertex.fill);
    let target_size = vec2<f32>(info.vertex.target_size);

    var out: VertexOutput;
    out.uv = (info.vertex.position + fill * info.vertex.cell_size) / target_size;
    out.clip_position = vec4<f32>(
        out.uv
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
        tex,
        tex_sampler,
        in.uv,
        0.0
    );
    let color = mix(info.fragment.fg, info.fragment.bg, sample.a);
    return vec4<f32>(color, 1.0);
}

