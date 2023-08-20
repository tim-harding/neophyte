struct GridCell {
    x: i32,
    y: i32,
    highlight_index: u32,
    width: u32,
}

struct GridInfo {
    surface_size: vec2<u32>,
    cell_size: vec2<u32>,
    offset: vec2<f32>,
    grid_width: u32,
    baseline: u32,
}

struct HighlightInfo {
    fg: vec4<f32>,
    bg: vec4<f32>,
}

@group(0) @binding(0)
var<storage, read> highlights: array<HighlightInfo>;
@group(1) @binding(0)
var<storage, read> grid_cells: array<GridCell>;
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
    let grid_cell = grid_cells[grid_index];
    let hl_info = highlights[grid_cell.highlight_index];
    let tex_coord = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );

    var out: VertexOutput;
    out.color = hl_info.bg;
    out.clip_position = vec4<f32>(
        (
            vec2<f32>(f32(grid_cell.x), f32(grid_cell.y)) + 
            grid_info.offset +
            tex_coord * 
            vec2<f32>(f32(grid_cell.width), f32(grid_info.cell_size.y))
        ) / vec2<f32>(grid_info.surface_size) * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0),
        0.0, 
        1.0
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}
