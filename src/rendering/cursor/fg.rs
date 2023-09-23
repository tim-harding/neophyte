use crate::{
    rendering::depth_texture::DepthTexture,
    text::{
        cache::{CacheValue, FontCache, GlyphKind},
        fonts::{FontStyle, Fonts},
    },
    ui::Ui,
    util::{srgb, vec2::Vec2},
};
use bytemuck::{cast_slice, Pod, Zeroable};
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
};
use wgpu::include_wgsl;

pub struct CursorFg {
    pipeline: wgpu::RenderPipeline,
    push_constants: PushConstants,
    glyphs: Vec<u32>,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct PushConstants {
    vertex: PushConstantsVertex,
    fragment: PushConstantsFragment,
}

impl PushConstants {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct PushConstantsVertex {
    position: Vec2<u32>,
    surface_size: Vec2<u32>,
    glyphs: [u32; 8],
}

impl PushConstantsVertex {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct PushConstantsFragment {
    color: [f32; 3],
}

impl PushConstantsFragment {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

impl CursorFg {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        glyph_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("fg.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cursor fg pipeline layout"),
            bind_group_layouts: &[glyph_bind_group_layout],
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
            label: Some("Cursor fg render pipeline"),
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
                depth_write_enabled: false,
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
            glyphs: vec![],
        }
    }

    pub fn update(
        &mut self,
        ui: &Ui,
        surface_size: Vec2<u32>,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        let color = ui
            .highlight_groups
            .get("Cursor")
            .and_then(|hl| ui.highlights.get(hl).unwrap().rgb_attr.foreground)
            .unwrap_or(ui.default_colors.rgb_bg.unwrap_or_default());
        let grid = ui.grid(ui.cursor.grid);
        let cell = grid.get(ui.cursor.pos);
        self.glyphs.clear();
        let style = if let Some(hl) = ui.highlights.get(&cell.highlight) {
            FontStyle::new(
                hl.rgb_attr.bold.unwrap_or_default(),
                hl.rgb_attr.italic.unwrap_or_default(),
            )
        } else {
            FontStyle::default()
        };

        let mut parser = Parser::new(
            Script::Latin,
            cell.text.chars().map(|c| Token {
                ch: c,
                offset: 0,
                len: 0,
                info: c.into(),
                data: cell.highlight as u32,
            }),
        );

        let mut cluster = CharCluster::new();
        if !parser.next(&mut cluster) {
            todo!()
        }

        let mut best_font = None;
        for font_info in fonts.iter() {
            if let Some(font) = font_info.style_or_regular(style) {
                match cluster.map(|c| font.charmap().map(c)) {
                    Status::Discard => {}
                    Status::Keep => best_font = Some(font),
                    Status::Complete => {
                        best_font = Some(font);
                        break;
                    }
                }
            }
        }

        if let Some(font) = best_font {
            let mut shaper = shape_context
                .builder(font.as_ref())
                .size(fonts.size() as f32)
                .script(Script::Arabic)
                .build();
            shaper.add_cluster(&cluster);
            let metrics = font.metrics(fonts.size());
            shaper.shape_with(|glyph_cluster| {
                for glyph in glyph_cluster.glyphs {
                    let CacheValue { index, kind } =
                        match font_cache.get(font.as_ref(), metrics.em, glyph.id, style) {
                            Some(glyph) => glyph,
                            None => {
                                continue;
                            }
                        };
                    let glyph_index = index as u32;

                    match kind {
                        GlyphKind::Monochrome => {
                            let offset = font_cache.monochrome.offset[index];
                            let position = offset * Vec2::new(1, -1)
                                + Vec2::new(
                                    (glyph.x * metrics.scale_factor).round() as i32
                                        + ui.cursor.pos.x as i32,
                                    (glyph.y * metrics.scale_factor
                                        + (ui.cursor.pos.y as u32 * metrics.cell_size_px.y
                                            + metrics.em_px)
                                            as f32)
                                        .round() as i32,
                                );
                            self.glyphs.push(glyph_index);
                        }
                        GlyphKind::Emoji => return,
                    };
                }
            });
        }

        self.push_constants = PushConstants {
            vertex: PushConstantsVertex {
                position: ui.cursor.pos.into(),
                surface_size,
                glyphs: [0u32; 8],
            },
            fragment: PushConstantsFragment {
                color: [srgb(color.r()), srgb(color.g()), srgb(color.b())],
            },
        };

        for (i, &glyph) in self.glyphs.iter().take(8).enumerate() {
            self.push_constants.vertex.glyphs[i] = glyph;
        }
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
        render_pass.draw(0..self.glyphs.len() as u32 * 6, 0..1);
    }
}
