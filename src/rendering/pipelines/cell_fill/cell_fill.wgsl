struct BgCell {
    x: i32,
    y: i32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct GridInfo {
    target_size: vec2<i32>,
    cell_size: vec2<i32>,
    offset: vec2<i32>,
    z: f32,
}

@group(0) @binding(0)
var<storage, read> grid_cells: array<BgCell>;
var<push_constant> grid_info: GridInfo;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let grid_index = in_vertex_index / 6u;
    let grid_cell = grid_cells[grid_index];
    let pos = vec2<i32>(
        grid_cell.x,
        grid_cell.y,
    );
    let tex_coord = vec2<u32>(
        in_vertex_index % 2u,
        ((in_vertex_index + 5u) % 6u) / 3u,
    );

    var out: VertexOutput;
    out.color = vec4<f32>(grid_cell.r, grid_cell.g, grid_cell.b, grid_cell.a);
    out.clip_position = vec4<f32>(
        vec2<f32>(((pos + vec2<i32>(tex_coord)) * vec2<i32>(grid_info.cell_size)) + grid_info.offset) / 
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
