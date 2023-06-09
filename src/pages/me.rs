use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use tui_textarea::Input;

use crate::api::Api;

use super::{pay::PayPage, qr, Page};

pub struct MePage<'a> {
    display_name: String,
    handle: String,
    balance: f32,
    pay_page: PayPage<'a>,
}

impl<'a> MePage<'a> {
    pub fn new(api: &'a mut Api) -> Self {
        Self {
            handle: api.identity.as_ref().unwrap().handle.clone(),
            display_name: api.identity.as_ref().unwrap().display_name.clone(),
            balance: api.identity.as_ref().unwrap().balance.user_balance.value,
            pay_page: PayPage::new(api),
        }
    }
}

#[async_trait]
impl<'a> Page for MePage<'a> {
    async fn on_input_event(&mut self, event: Input) -> bool {
        self.pay_page.on_input_event(event).await
    }

    async fn make_progress(&mut self) -> bool {
        self.pay_page.make_progress().await
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
                    format!("{}", self.display_name),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Spans::from(Span::styled(format!("@{}", self.handle), Style::default())),
                Spans::from(Span::styled("", Style::default())),
                Spans::from(Span::styled(
                    format!("Balance: ${}", self.balance),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double),
            )
            .alignment(tui::layout::Alignment::Center);

            f.render_widget(text, left_layout[0]);

            self.pay_page.render(f, left_layout[1]);
        }

        let uri = format!("https://account.venmo.com/u/{}", self.handle);
        let canvas = qr::generate(&uri);

        f.render_widget(canvas, inner_layout[1]);
    }
}
