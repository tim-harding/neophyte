use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Scroll a grid region. This is semantically unrelated to editor scrolling,
/// rather this is an optimized way to say "copy these screen cells".
///
/// The following diagrams show what happens per scroll direction. "==="
/// represents the SR (scroll region) boundaries. "---" represents the moved
/// rectangles. Note that dst and src share a common region.
///
/// If rows is bigger than 0, move a rectangle in the SR up, this can happen
/// while scrolling down.
///
/// +-------------------------+
/// | (clipped above SR)      |            ^
/// |=========================| dst_top    |
/// | dst (still in SR)       |            |
/// +-------------------------+ src_top    |
/// | src (moved up) and dst  |            |
/// |-------------------------| dst_bot    |
/// | src (invalid)           |            |
/// +=========================+ src_bot
///
/// If rows is less than zero, move a rectangle in the SR down, this can happen
/// while scrolling up.
///
/// +=========================+ src_top
/// | src (invalid)           |            |
/// |------------------------ | dst_top    |
/// | src (moved down) and dst|            |
/// +-------------------------+ src_bot    |
/// | dst (still in SR)       |            |
/// |=========================| dst_bot    |
/// | (clipped below SR)      |            v
/// +-------------------------+
///
/// The scrolled-in area will be filled using ui-event-grid_line directly after
/// the scroll event. The UI thus doesn't need to clear this area as part of
/// handling the scroll event.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GridScroll {
    /// The grid to scroll
    pub grid: u32,
    /// Top border of the scroll region
    pub top: u16,
    /// Bottom border of the scroll region, exclusive
    pub bot: u16,
    /// Left border of the scroll region
    pub left: u16,
    /// Right border of the scroll region, exclusive
    pub right: u16,
    /// The number of rows to scroll by. Positive moves the region up, negative
    /// moves it down.
    pub rows: i32,
    /// Always zero in this version of Neovim. Reserved for future use.
    pub cols: i32,
}

impl Parse for GridScroll {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            top: iter.next()?,
            bot: iter.next()?,
            left: iter.next()?,
            right: iter.next()?,
            rows: iter.next()?,
            cols: iter.next()?,
        })
    }
}
