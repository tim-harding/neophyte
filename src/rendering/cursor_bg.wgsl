struct VertexInfo {
    position: vec2<f32>,
    surface_size: vec2<u32>,
    fill: vec2<f32>,
    cell_size: vec2<f32>,
}

struct FragmentInfo {
    color: vec4<f32>,
}

struct Info {
    vertex: VertexInfo,
    fragment: FragmentInfo,
}

var<push_constant> info: Info;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

fn rev_y(v: vec2<f32>) -> vec2<f32> {
    return v * vec2<f32>(1.0, -1.0) + vec2<f32>(0.0, 1.0);
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let uv = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );

    let fill = rev_y(rev_y(uv) * info.vertex.fill);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        (info.vertex.position + fill)
        * info.vertex.cell_size
        / vec2<f32>(info.vertex.surface_size) 
        * vec2<f32>(2.0, -2.0) 
        + vec2<f32>(-1.0, 1.0),
        0.0,
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return info.fragment.color;
}

