struct VertexInfo {
    transform: mat3x3<f32>,
    fill: vec2<f32>,
    cursor_size: vec2<f32>,
}

struct FragmentInfo {
    fg: vec3<f32>,
    bg: vec3<f32>,
    size: vec2<f32>,
    speed: f32,
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
    @location(1) corner: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.corner = vec2<f32>(
        f32(in_vertex_index % 2u),
        f32(((in_vertex_index + 5u) % 6u) / 3u),
    );
    let affine = info.vertex.transform * vec3<f32>(out.corner, 1.0);

    var fill = out.corner * info.vertex.fill;
    fill.y = fill.y + 1.0 - info.vertex.fill.y;
    out.uv = vec2<f32>(affine.x / affine.z, affine.y / affine.z) + fill * info.vertex.cursor_size;
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

    let sz = max(info.fragment.size.x, info.fragment.size.y);
    let rect = 1. - max(abs(in.corner.x - 0.5), abs(in.corner.y - 0.5)) * 2.;
    let round = 1. - length(in.corner - vec2<f32>(0.5, 0.5)) * 2.;
    let l1 = round * sz / (info.fragment.speed / 4. + 1.);
    let l2 = rect * sz;
    let t = min(1., max(0., info.fragment.speed / 4.));
    let l = max(0., min(1., mix(l2, l1, t)));
    let color = mix(info.fragment.fg, info.fragment.bg, sample.a);
    return vec4<f32>(color, l);
}

