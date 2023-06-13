use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_textarea::Input;

use crate::api::Api;

use super::{centered_rect, Page};

pub struct ErrorPage<'a, 'b> {
    api: &'a mut Api,
    msg: &'b str,
}

impl<'a, 'b> ErrorPage<'a, 'b> {
    pub fn new(api: &'a mut Api, msg: &'b str) -> Self {
        Self { api, msg }
    }
}

#[async_trait]
impl<'a, 'b> Page for ErrorPage<'a, 'b> {
    async fn on_input_event(&mut self, _event: Input) -> bool {
        // on any keystroke, exit
        true
    }

    async fn make_progress(&mut self) -> bool {
        false
    }

    fn render(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        let popup = centered_rect(50, 30, area);
        f.render_widget(Clear, popup);

        let text = Paragraph::new(vec![Spans::from(Span::styled(
            self.msg,
            Style::default().fg(tui::style::Color::Red),
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ERROR")
                .style(Style::default().bg(tui::style::Color::White)),
        )
        .alignment(tui::layout::Alignment::Center);

        f.render_widget(text, popup);
    }
}
