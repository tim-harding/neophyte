use super::messagepack_ext_types::Window;
use crate::util::{Parse, Values};
use rmpv::Value;

/// Indicates the range of buffer text displayed in the window, as well as the
/// cursor position in the buffer.
#[derive(Debug, Clone)]
pub struct WinViewport {
    /// The grid to update
    pub grid: u64,
    /// The window to update
    pub win: Window,
    /// The first line of the grid to display
    pub topline: u64,
    /// One past the last line of the grid to display. If there are filler lines
    /// past the end, this is One more than the line count of the buffer.
    pub botline: u64,
    /// The line the cursor is on
    pub curline: u64,
    /// The column the cursor is on
    pub curcol: u64,
    /// The line count of the buffer
    pub line_count: u64,
    /// how much the top line of a window moved since win_viewport was last
    /// emitted. It is intended to be used to implement smooth scrolling. For
    /// this purpose it only counts "virtual" or "displayed" lines, so folds
    /// only count as one line. When scrolling more than a full screen it is an
    /// approximate value.
    pub scroll_delta: i64,
}

impl Parse for WinViewport {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
            topline: iter.next()?,
            botline: iter.next()?,
            curline: iter.next()?,
            curcol: iter.next()?,
            line_count: iter.next()?,
            scroll_delta: iter.next()?,
        })
    }
}
