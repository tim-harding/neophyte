struct GridInfo {
    target_size: vec2<i32>,
    offset: vec2<i32>,
    grid_width: u32,
    z: f32,
}

struct Line {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    r: f32,
    g: f32,
    b: f32,
}

@group(0) @binding(0)
var<storage, read> lines: array<Line>;
var<push_constant> grid_info: GridInfo;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let line_index = in_vertex_index / 6u;
    let line = lines[line_index];
    let tex_coord = vec2<u32>(
        in_vertex_index % 2u,
        ((in_vertex_index + 5u) % 6u) / 3u,
    );
    let position = vec2<i32>(line.x, line.y);
    let size = vec2<u32>(line.w, line.h);

    var out: VertexOutput;
    out.color = vec4<f32>(line.r, line.g, line.b, 1.0);
    out.clip_position = vec4<f32>(
        vec2<f32>(position + grid_info.offset + vec2<i32>(tex_coord * size)) / 
        vec2<f32>(grid_info.target_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        grid_info.z, 
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
