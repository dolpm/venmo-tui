use tui::{
    style::Color,
    widgets::{
        canvas::{Canvas, Context, Painter, Shape},
        Block, Borders,
    },
};

struct QrCode {
    data: Vec<Vec<bool>>,
}

impl Shape for QrCode {
    fn draw(&self, painter: &mut Painter) {
        for (y, v) in self.data.iter().enumerate() {
            for (x, b) in v.iter().enumerate() {
                if *b {
                    painter.paint(x, y, Color::LightBlue);
                } else {
                    painter.paint(x, y, Color::White);
                }
            }
        }
    }
}

trait Double {
    fn grow(&mut self, by: usize);
}

trait AddMargin {
    fn add_margin(&mut self);
}
impl AddMargin for QrCode {
    fn add_margin(&mut self) {
        let mut new_size = (self.data.len() as f64 * 1.1) as usize;
        if new_size % 2 != 0 {
            new_size += 1;
        }

        let mut new_buffer = vec![vec![false; new_size + 1]; new_size + 1];

        let margin_width = (new_size - self.data.len()) / 2;

        for (y, v) in new_buffer.iter_mut().enumerate() {
            for (x, cur) in v.iter_mut().enumerate() {
                let is_in_margin = y <= margin_width
                    || y >= new_size - margin_width
                    || x <= margin_width
                    || x >= new_size - margin_width;

                if is_in_margin {
                    *cur = false;
                    continue;
                }

                *cur = self.data[y - margin_width - 1][x - margin_width - 1];
            }
        }

        self.data = new_buffer;
    }
}

impl Double for QrCode {
    fn grow(&mut self, by: usize) {
        let mut new_buffer = vec![vec![false; self.data.len() * by]; self.data.len() * by];

        for (y, v) in self.data.iter().enumerate() {
            for (x, &to_set) in v.iter().enumerate() {
                let y = y * by;
                let x = x * by;
                if to_set {
                    for i in 0..by {
                        for j in 0..by {
                            new_buffer[y + i][x + j] = to_set;
                        }
                    }
                }
            }
        }

        self.data = new_buffer;
    }
}

pub fn generate(input: &str) -> Canvas<impl Fn(&mut Context)> {
    let data = Box::leak(Box::new(
        qrcode_generator::to_matrix(input, qrcode_generator::QrCodeEcc::Low).unwrap(),
    ));

    let mut code = QrCode { data: data.clone() };
    code.add_margin();
    code.grow(4);

    let size = code.data.len() as f64;

    Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Venmo me!"))
        .x_bounds([-size, size])
        .y_bounds([-size, size])
        .paint(move |ctx| {
            ctx.draw(&code);
        })
}
