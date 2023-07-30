use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;

/// The built-in highlight group name was set to use the attributes hl_id
/// defined by a previous hl_attr_define call. This event is not needed to
/// render the grids which use attribute ids directly, but is useful for a UI
/// who want to render its own elements with consistent highlighting.
#[derive(Debug, Clone)]
pub struct HlGroupSet {
    /// The highlight group name
    pub name: String,
    /// The highlight attributes to apply
    pub hl_id: u64,
}

impl HlGroupSet {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            name: parse_string(iter.next()?)?,
            hl_id: parse_u64(iter.next()?)?,
        })
    }
}
