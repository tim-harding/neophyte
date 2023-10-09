struct GridInfo {
    target_size: vec2<u32>,
    cell_size: vec2<u32>,
    offset: vec2<i32>,
    grid_width: u32,
    z: f32,
}

struct HighlightInfo {
    fg: vec4<f32>,
    bg: vec4<f32>,
}

struct Line {
    position: vec2<i32>,
    size: vec2<u32>,
    highlight_index: u32,
}

@group(0) @binding(0)
var<storage, read> highlights: array<HighlightInfo>;
@group(1) @binding(0)
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
    let hl_info = highlights[line.highlight_index];
    let pos = vec2<u32>(
        line_index % grid_info.grid_width, 
        line_index / grid_info.grid_width
    );
    let tex_coord = vec2<u32>(
        in_vertex_index % 2u,
        ((in_vertex_index + 5u) % 6u) / 3u,
    );

    var out: VertexOutput;
    out.color = hl_info.fg;
    out.clip_position = vec4<f32>(
        vec2<f32>(line.position + vec2<i32>(tex_coord * line.size)) / 
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
