use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, BorderType},
    Frame,
};
use tui_textarea::Input;

use crate::api::Api;

use super::{pay::PayPage, qr, Page};

pub struct MePage<'a> {
    api: &'a mut Api,
}

impl<'a> MePage<'a> {
    pub fn new(api: &'a mut Api) -> Self {
        Self { api }
    }
}

#[async_trait]
impl<'a> Page for MePage<'a> {
    async fn on_input_event(&mut self, _event: Input) -> bool {
        false
    }
    async fn make_progress(&mut self) -> bool {
        false
    }

    fn render(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Length(30)].as_ref())
            .split(area);

        {
            let left_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(10)].as_ref())
                .split(inner_layout[0]);

            let text = Paragraph::new(vec![
                Spans::from(Span::styled(
                    format!("{}", self.api.identity.as_ref().unwrap().display_name),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Spans::from(Span::styled(
                    format!("@{}", self.api.identity.as_ref().unwrap().handle),
                    Style::default(),
                )),
                Spans::from(Span::styled("", Style::default())),
                Spans::from(Span::styled(
                    format!(
                        "Balance: ${}",
                        self.api
                            .identity
                            .as_ref()
                            .unwrap()
                            .balance
                            .user_balance
                            .value
                    ),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ])
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double))
            .alignment(tui::layout::Alignment::Center);

            f.render_widget(text, left_layout[0]);

            let mut payment = PayPage::new(self.api);

            payment.render(f, left_layout[1]);

        }

        let uri = format!(
            "https://account.venmo.com/u/{}",
            self.api.identity.as_ref().unwrap().handle
        );
        let canvas = qr::generate(&uri);

        f.render_widget(canvas, inner_layout[1]);
    }
}
