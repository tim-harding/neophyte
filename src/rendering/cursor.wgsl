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

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        (info.vertex.position + tex_coord * info.vertex.fill)
        * info.vertex.cell_size
        / vec2<f32>(info.vertex.surface_size) 
        * vec2<f32>(2.0, -2.0) 
        + vec2<f32>(-1.0, 1.0),
        1.0,
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return info.fragment.color;
}

