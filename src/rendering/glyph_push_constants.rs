use bytemuck::{Pod, Zeroable, checked::cast_slice};
use neophyte_linalg::PixelVec;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GlyphPushConstants {
    pub target_size: PixelVec<i32>,
    pub offset: PixelVec<i32>,
    pub z: f32,
    pub atlas_size: i32,
}

impl GlyphPushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;

    pub fn set(self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[self]));
    }
}
