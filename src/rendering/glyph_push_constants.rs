use crate::util::vec2::PixelVec;
use bytemuck::{checked::cast_slice, Pod, Zeroable};
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GlyphPushConstants {
    pub target_size: PixelVec<u32>,
    pub offset: PixelVec<i32>,
    pub z: f32,
    pub atlas_size: u32,
}

impl GlyphPushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;

    pub fn set(self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[self]));
    }
}
