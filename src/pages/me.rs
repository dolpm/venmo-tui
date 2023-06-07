use std::io::StdoutLock;

use async_trait::async_trait;
use tui::{backend::CrosstermBackend, layout::Rect, Frame};
use tui_textarea::Input;

use crate::api::Api;

use super::{qr, Page};

pub struct MePage<'a> {
    first_name: &'a str,
    handle: &'a str,
}

impl<'a> MePage<'a> {
    pub fn new(api: &'a mut Api) -> Self {
        Self {
            handle: &api.identity.as_ref().unwrap().handle,
            first_name: &api.identity.as_ref().unwrap().display_name,
        }
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
        let uri = format!("https://account.venmo.com/u/{}", self.handle);
        let canvas = qr::generate(&uri);
        f.render_widget(canvas, area);
    }
}
