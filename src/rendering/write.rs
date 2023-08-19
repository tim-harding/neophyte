use super::{font, grid, highlights, read::ReadStateUpdates, State};
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
};

pub struct WriteState {
    pub grid: grid::Write,
    pub font: font::Write,
    pub highlights: highlights::Write,
}

impl WriteState {
    // TODO: Should only rebuild the pipeline as the result of a resize
    pub fn updates(
        &mut self,
        ui: Ui,
        state: &State,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
    ) -> ReadStateUpdates {
        ReadStateUpdates {
            grid: self.grid.updates(state, &ui, fonts, font_cache),
            highlights: self.highlights.updates(&ui, &state),
            font: self.font.updates(state, font_cache),
        }
    }
}
