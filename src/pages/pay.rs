use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, TextArea};

use crate::api::Api;

use super::{activate, inactivate, Page};

pub struct PayPage<'a> {
    api: &'a mut Api,
}

impl<'a> PayPage<'a> {
    pub fn new(api: &'a mut Api) -> Self {
        Self { api }
    }
}

enum Field {
    Handle,
    Note,
}

#[async_trait]
impl<'a> Page for PayPage<'a> {
    async fn on_input_event(&mut self, _event: Input) -> bool {
        false
    }
    async fn make_progress(&mut self) -> bool {
        false
    }

    fn render(&mut self, f: &mut Frame<CrosstermBackend<StdoutLock>>, area: Rect) {
        let [mut amount, mut handle, mut note] = [
            TextArea::default(),
            TextArea::default(),
            TextArea::default(),
        ];
        {
            amount.set_block(Block::default().borders(Borders::ALL).title("$"));
            handle.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Name, @username, email, phone"),
            );
            note.set_block(Block::default().borders(Borders::ALL).title("Note"));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .margin(2)
            .split(area);

        let mut selected: Field = Field::Handle;

        activate(&mut amount);
        inactivate(&mut handle);
        inactivate(&mut note);

        {
            let a_widget = amount.widget();
            f.render_widget(a_widget, chunks[0]);
            let h_widget = handle.widget();
            f.render_widget(h_widget, chunks[1]);
            let n_widget = note.widget();
            f.render_widget(n_widget, chunks[2]);

            let send_btn_text = Paragraph::new(Text::from("Pay")).alignment(Alignment::Right);
            let req_btn_text = Paragraph::new(Text::from("Request")).alignment(Alignment::Left);

            let btn_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(47),
                        Constraint::Length(6),
                        Constraint::Percentage(47),
                    ]
                    .as_ref(),
                )
                .split(chunks[3]);

            f.render_widget(send_btn_text, btn_layout[0]);
            f.render_widget(req_btn_text, btn_layout[2]);
        }

        let block = Block::default()
            .title("Pay & Request")
            .borders(Borders::ALL);
        f.render_widget(block, area);
    }
}
