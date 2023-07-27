use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct SetTitle(Vec<String>);

impl TryFrom<Value> for SetTitle {
    type Error = SetTitleParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let titles: Option<Vec<_>> = match value {
            Value::Array(array) => array
                .into_iter()
                .map(|value| match value {
                    Value::String(s) => s.into_str(),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        let titles = titles.ok_or(SetTitleParseError)?;
        Ok(Self(titles))
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse set_title event")]
pub struct SetTitleParseError;
