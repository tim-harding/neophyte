use crate::util::vec2::Vec2;
use bytemuck::{checked::cast_slice, Pod, Zeroable};
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GlyphPushConstants {
    pub target_size: Vec2<u32>,
    pub offset: Vec2<i32>,
    pub z: f32,
}

impl GlyphPushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;

    pub fn set(self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[self]));
    }
}
