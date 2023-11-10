struct GridInfo {
    z: f32,
    r: f32,
    g: f32,
    b: f32,
}

var<push_constant> info: GridInfo;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let tex_coord = vec2<u32>(
        in_vertex_index % 2u,
        ((in_vertex_index + 5u) % 6u) / 3u,
    );

    var out: VertexOutput;
    out.color = vec4<f32>(info.r, info.g, info.b, 1.0);
    out.clip_position = vec4<f32>(
        vec2<f32>(tex_coord) * vec2<f32>(2.0, 2.0) - vec2<f32>(1.0, 1.0),
        info.z, 
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
