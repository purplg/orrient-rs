use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};

pub struct Textbox<'a> {
    style: Style,
    block: Option<Block<'a>>,
    content: &'a str,
}

impl<'a> Textbox<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            style: Style::default(),
            block: None,
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
}

impl<'a> Widget for Textbox<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let text_box_area = match self.block.take() {
            Some(block) => {
                let inner_area = block.inner(area);
                block.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_box_area.width < 1 || text_box_area.height < 1 {
            return;
        }

        buf.set_stringn(
            text_box_area.x,
            text_box_area.y,
            self.content,
            text_box_area.width as usize,
            self.style,
        );
    }
}
