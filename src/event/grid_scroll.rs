use super::util::{parse_array, parse_i64, parse_u64};
use nvim_rs::Value;

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
#[derive(Debug, Clone, Copy)]
pub struct GridScroll {
    /// The grid to scroll
    pub grid: u64,
    /// Top border of the scroll region
    pub top: u64,
    /// Bottom border of the scroll region, exclusive
    pub bot: u64,
    /// Left border of the scroll region
    pub left: u64,
    /// Right border of the scroll region, exclusive
    pub right: u64,
    /// The number of rows to scroll by. Positive moves the region up, negative
    /// moves it down.
    pub rows: i64,
    /// Always zero in this version of Neovim. Reserved for future use.
    pub cols: u64,
}

impl GridScroll {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            grid: parse_u64(iter.next()?)?,
            top: parse_u64(iter.next()?)?,
            bot: parse_u64(iter.next()?)?,
            left: parse_u64(iter.next()?)?,
            right: parse_u64(iter.next()?)?,
            rows: parse_i64(iter.next()?)?,
            cols: parse_u64(iter.next()?)?,
        })
    }
}
