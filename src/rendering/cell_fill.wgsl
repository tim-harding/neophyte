struct GridInfo {
    target_size: vec2<u32>,
    cell_size: vec2<u32>,
    offset: vec2<f32>,
    grid_width: u32,
    z: f32,
}

struct HighlightInfo {
    fg: vec4<f32>,
    bg: vec4<f32>,
}

@group(0) @binding(0)
var<storage, read> highlights: array<HighlightInfo>;
@group(1) @binding(0)
var<storage, read> grid_cells: array<u32>;
var<push_constant> grid_info: GridInfo;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(2) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let highlight_index = grid_cells[grid_index];
    let hl_info = highlights[highlight_index];
    let pos = vec2<u32>(
        grid_index % grid_info.grid_width, 
        grid_index / grid_info.grid_width
    );
    let tex_coord = vec2<u32>(
        in_vertex_index % 2u,
        ((in_vertex_index + 5u) % 6u) / 3u,
    );

    var out: VertexOutput;
    out.color = hl_info.bg;
    out.clip_position = vec4<f32>(
        (vec2<f32>((pos + tex_coord) * grid_info.cell_size ) + grid_info.offset) / 
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
