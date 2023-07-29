use super::util::{parse_array, parse_string};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct SetTitle(Vec<String>);

impl SetTitle {
    pub fn parse(value: Value) -> Option<Self> {
        let titles: Option<Vec<_>> = parse_array(value)?.into_iter().map(parse_string).collect();
        Some(Self(titles?))
    }
}
