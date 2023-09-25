use crate::{
    event::mode_info_set::CursorShape, rendering::depth_texture::DepthTexture, ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::include_wgsl;

pub struct CursorBg {
    pipeline: wgpu::RenderPipeline,
    push_constants: PushConstants,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct PushConstants {
    vertex: PushConstantsVertex,
    fragment: PushConstantsFragment,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct PushConstantsVertex {
    position: Vec2<f32>,
    surface_size: Vec2<u32>,
    fill: Vec2<f32>,
    cell_size: Vec2<f32>,
}

impl PushConstantsFragment {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct PushConstantsFragment {
    color: [f32; 4],
}

impl PushConstantsVertex {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

impl CursorBg {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("bg.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cursor pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..PushConstantsVertex::SIZE as u32,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: (PushConstantsVertex::SIZE as u32)
                        ..(PushConstantsVertex::SIZE as u32 + PushConstantsFragment::SIZE as u32),
                },
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cursor render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,
            push_constants: Default::default(),
        }
    }

    pub fn update(&mut self, ui: &Ui, surface_size: Vec2<u32>, cell_size: Vec2<f32>) {
        let mode = &ui.modes[ui.current_mode as usize];
        let fill = mode.cell_percentage.unwrap_or(10) as f32 / 100.0;
        let color = ui
            .highlight_groups
            .get("Cursor")
            .and_then(|hl| ui.highlights.get(hl).unwrap().rgb_attr.background)
            .unwrap_or(ui.default_colors.rgb_fg.unwrap_or_default());
        self.push_constants = PushConstants {
            vertex: PushConstantsVertex {
                position: (ui.position(ui.cursor.grid) + ui.cursor.pos.into()).into(),
                surface_size,
                fill: match mode.cursor_shape.unwrap_or(CursorShape::Block) {
                    CursorShape::Block => Vec2::new(1.0, 1.0),
                    CursorShape::Horizontal => Vec2::new(1.0, fill),
                    CursorShape::Vertical => Vec2::new(fill, 1.0),
                },
                cell_size,
            },
            fragment: PushConstantsFragment {
                color: color.into_linear(),
            },
        };
    }

    pub fn render<'b, 'c, 'a: 'b + 'c>(&'a self, render_pass: &'b mut wgpu::RenderPass<'c>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.push_constants.vertex]),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            PushConstantsVertex::SIZE as u32,
            cast_slice(&[self.push_constants.fragment]),
        );
        render_pass.draw(0..6, 0..1);
    }
}
