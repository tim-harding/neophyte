use crate::{event::Anchor, util::Vec2};

#[derive(Debug, Clone)]
pub struct Window {
    pub grid: u64,
    pub start: Vec2,
    pub size: Vec2,
    pub anchor: Option<AnchorInfo>,
    pub focusable: bool,
}

#[derive(Debug, Clone)]
pub struct AnchorInfo {
    pub anchor: Anchor,
    pub grid: u64,
    pub pos: Vec2,
}
