use tui::{
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

pub struct Checkbox<'a> {
    style: Style,
    block: Option<Block<'a>>,
    content: &'a str,
    unchecked_symbol: Option<&'a str>,
    checked_symbol: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> Checkbox<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            style: Style::default(),
            block: None,
            unchecked_symbol: None,
            checked_symbol: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn checked_symbol(mut self, symbol: &'a str) -> Self {
        self.checked_symbol = Some(symbol);
        self
    }

    pub fn unchecked_symbol(mut self, symbol: &'a str) -> Self {
        self.unchecked_symbol = Some(symbol);
        self
    }
}

#[derive(Default)]
pub struct CheckboxState {
    checked: bool,
}

impl CheckboxState {
    pub fn checked(&self) -> bool {
        self.checked
    }

    /// Toggles the checkbox and returns the new state
    pub fn toggle(&mut self) -> bool {
        self.checked = !self.checked;
        self.checked
    }
}

impl StatefulWidget for Checkbox<'_> {
    type State = CheckboxState;

    fn render(
        mut self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        buf.set_style(area, self.style);

        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if area.width < 1 || area.height < 1 {
            return;
        }

        let symbol = if state.checked() {
            self.checked_symbol.unwrap_or("[X] ")
        } else {
            self.unchecked_symbol.unwrap_or("[ ] ")
        };

        buf.set_stringn(
            area.x,
            area.y,
            format!("{}{}", symbol, self.content),
            area.width as usize,
            self.style,
        );
    }
}

impl Widget for Checkbox<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        <Self as StatefulWidget>::render(self, area, buf, &mut CheckboxState::default());
    }
}
