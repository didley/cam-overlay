use crate::settings::Settings;

// Public domain 8x8 bitmap font (font8x8_basic, printable ASCII 32..=126)
#[rustfmt::skip]
const FONT: [[u8; 8]; 95] = [
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], // 32 ' '
    [0x18,0x3C,0x3C,0x18,0x18,0x00,0x18,0x00], // 33 '!'
    [0x36,0x36,0x00,0x00,0x00,0x00,0x00,0x00], // 34 '"'
    [0x36,0x36,0x7F,0x36,0x7F,0x36,0x36,0x00], // 35 '#'
    [0x0C,0x3E,0x03,0x1E,0x30,0x1F,0x0C,0x00], // 36 '$'
    [0x00,0x63,0x33,0x18,0x0C,0x66,0x63,0x00], // 37 '%'
    [0x1C,0x36,0x1C,0x6E,0x3B,0x33,0x6E,0x00], // 38 '&'
    [0x06,0x06,0x03,0x00,0x00,0x00,0x00,0x00], // 39 '''
    [0x18,0x0C,0x06,0x06,0x06,0x0C,0x18,0x00], // 40 '('
    [0x06,0x0C,0x18,0x18,0x18,0x0C,0x06,0x00], // 41 ')'
    [0x00,0x66,0x3C,0xFF,0x3C,0x66,0x00,0x00], // 42 '*'
    [0x00,0x0C,0x0C,0x3F,0x0C,0x0C,0x00,0x00], // 43 '+'
    [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C,0x06], // 44 ','
    [0x00,0x00,0x00,0x3F,0x00,0x00,0x00,0x00], // 45 '-'
    [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C,0x00], // 46 '.'
    [0x60,0x30,0x18,0x0C,0x06,0x03,0x01,0x00], // 47 '/'
    [0x3E,0x63,0x73,0x7B,0x6F,0x67,0x3E,0x00], // 48 '0'
    [0x0C,0x0E,0x0C,0x0C,0x0C,0x0C,0x3F,0x00], // 49 '1'
    [0x1E,0x33,0x30,0x1C,0x06,0x33,0x3F,0x00], // 50 '2'
    [0x1E,0x33,0x30,0x1C,0x30,0x33,0x1E,0x00], // 51 '3'
    [0x38,0x3C,0x36,0x33,0x7F,0x30,0x78,0x00], // 52 '4'
    [0x3F,0x03,0x1F,0x30,0x30,0x33,0x1E,0x00], // 53 '5'
    [0x1C,0x06,0x03,0x1F,0x33,0x33,0x1E,0x00], // 54 '6'
    [0x3F,0x33,0x30,0x18,0x0C,0x0C,0x0C,0x00], // 55 '7'
    [0x1E,0x33,0x33,0x1E,0x33,0x33,0x1E,0x00], // 56 '8'
    [0x1E,0x33,0x33,0x3E,0x30,0x18,0x0E,0x00], // 57 '9'
    [0x00,0x0C,0x0C,0x00,0x00,0x0C,0x0C,0x00], // 58 ':'
    [0x00,0x0C,0x0C,0x00,0x00,0x0C,0x0C,0x06], // 59 ';'
    [0x18,0x0C,0x06,0x03,0x06,0x0C,0x18,0x00], // 60 '<'
    [0x00,0x00,0x3F,0x00,0x00,0x3F,0x00,0x00], // 61 '='
    [0x06,0x0C,0x18,0x30,0x18,0x0C,0x06,0x00], // 62 '>'
    [0x1E,0x33,0x30,0x18,0x0C,0x00,0x0C,0x00], // 63 '?'
    [0x3E,0x63,0x7B,0x7B,0x7B,0x03,0x1E,0x00], // 64 '@'
    [0x0C,0x1E,0x33,0x33,0x3F,0x33,0x33,0x00], // 65 'A'
    [0x3F,0x66,0x66,0x3E,0x66,0x66,0x3F,0x00], // 66 'B'
    [0x3C,0x66,0x03,0x03,0x03,0x66,0x3C,0x00], // 67 'C'
    [0x1F,0x36,0x66,0x66,0x66,0x36,0x1F,0x00], // 68 'D'
    [0x7F,0x46,0x16,0x1E,0x16,0x46,0x7F,0x00], // 69 'E'
    [0x7F,0x46,0x16,0x1E,0x16,0x06,0x0F,0x00], // 70 'F'
    [0x3C,0x66,0x03,0x03,0x73,0x66,0x7C,0x00], // 71 'G'
    [0x33,0x33,0x33,0x3F,0x33,0x33,0x33,0x00], // 72 'H'
    [0x1E,0x0C,0x0C,0x0C,0x0C,0x0C,0x1E,0x00], // 73 'I'
    [0x78,0x30,0x30,0x30,0x33,0x33,0x1E,0x00], // 74 'J'
    [0x67,0x66,0x36,0x1E,0x36,0x66,0x67,0x00], // 75 'K'
    [0x0F,0x06,0x06,0x06,0x46,0x66,0x7F,0x00], // 76 'L'
    [0x63,0x77,0x7F,0x7F,0x6B,0x63,0x63,0x00], // 77 'M'
    [0x63,0x67,0x6F,0x7B,0x73,0x63,0x63,0x00], // 78 'N'
    [0x1C,0x36,0x63,0x63,0x63,0x36,0x1C,0x00], // 79 'O'
    [0x3F,0x66,0x66,0x3E,0x06,0x06,0x0F,0x00], // 80 'P'
    [0x1E,0x33,0x33,0x33,0x3B,0x1E,0x38,0x00], // 81 'Q'
    [0x3F,0x66,0x66,0x3E,0x36,0x66,0x67,0x00], // 82 'R'
    [0x1E,0x33,0x07,0x0E,0x38,0x33,0x1E,0x00], // 83 'S'
    [0x3F,0x2D,0x0C,0x0C,0x0C,0x0C,0x1E,0x00], // 84 'T'
    [0x33,0x33,0x33,0x33,0x33,0x33,0x3F,0x00], // 85 'U'
    [0x33,0x33,0x33,0x33,0x33,0x1E,0x0C,0x00], // 86 'V'
    [0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00], // 87 'W'
    [0x63,0x63,0x36,0x1C,0x1C,0x36,0x63,0x00], // 88 'X'
    [0x33,0x33,0x33,0x1E,0x0C,0x0C,0x1E,0x00], // 89 'Y'
    [0x7F,0x63,0x31,0x18,0x4C,0x66,0x7F,0x00], // 90 'Z'
    [0x1E,0x06,0x06,0x06,0x06,0x06,0x1E,0x00], // 91 '['
    [0x03,0x06,0x0C,0x18,0x30,0x60,0x40,0x00], // 92 '\'
    [0x1E,0x18,0x18,0x18,0x18,0x18,0x1E,0x00], // 93 ']'
    [0x08,0x1C,0x36,0x63,0x00,0x00,0x00,0x00], // 94 '^'
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xFF], // 95 '_'
    [0x0C,0x0C,0x18,0x00,0x00,0x00,0x00,0x00], // 96 '`'
    [0x00,0x00,0x1E,0x30,0x3E,0x33,0x6E,0x00], // 97 'a'
    [0x07,0x06,0x06,0x3E,0x66,0x66,0x3B,0x00], // 98 'b'
    [0x00,0x00,0x1E,0x33,0x03,0x33,0x1E,0x00], // 99 'c'
    [0x38,0x30,0x30,0x3E,0x33,0x33,0x6E,0x00], // 100 'd'
    [0x00,0x00,0x1E,0x33,0x3F,0x03,0x1E,0x00], // 101 'e'
    [0x1C,0x36,0x06,0x0F,0x06,0x06,0x0F,0x00], // 102 'f'
    [0x00,0x00,0x6E,0x33,0x33,0x3E,0x30,0x1F], // 103 'g'
    [0x07,0x06,0x36,0x6E,0x66,0x66,0x67,0x00], // 104 'h'
    [0x0C,0x00,0x0E,0x0C,0x0C,0x0C,0x1E,0x00], // 105 'i'
    [0x30,0x00,0x30,0x30,0x30,0x33,0x33,0x1E], // 106 'j'
    [0x07,0x06,0x66,0x36,0x1E,0x36,0x67,0x00], // 107 'k'
    [0x0E,0x0C,0x0C,0x0C,0x0C,0x0C,0x1E,0x00], // 108 'l'
    [0x00,0x00,0x33,0x7F,0x7F,0x6B,0x63,0x00], // 109 'm'
    [0x00,0x00,0x1F,0x33,0x33,0x33,0x33,0x00], // 110 'n'
    [0x00,0x00,0x1E,0x33,0x33,0x33,0x1E,0x00], // 111 'o'
    [0x00,0x00,0x3B,0x66,0x66,0x3E,0x06,0x0F], // 112 'p'
    [0x00,0x00,0x6E,0x33,0x33,0x3E,0x30,0x78], // 113 'q'
    [0x00,0x00,0x3B,0x6E,0x66,0x06,0x0F,0x00], // 114 'r'
    [0x00,0x00,0x3E,0x03,0x1E,0x30,0x1F,0x00], // 115 's'
    [0x08,0x0C,0x3E,0x0C,0x0C,0x2C,0x18,0x00], // 116 't'
    [0x00,0x00,0x33,0x33,0x33,0x33,0x6E,0x00], // 117 'u'
    [0x00,0x00,0x33,0x33,0x33,0x1E,0x0C,0x00], // 118 'v'
    [0x00,0x00,0x63,0x6B,0x7F,0x7F,0x36,0x00], // 119 'w'
    [0x00,0x00,0x63,0x36,0x1C,0x36,0x63,0x00], // 120 'x'
    [0x00,0x00,0x33,0x33,0x33,0x3E,0x30,0x1F], // 121 'y'
    [0x00,0x00,0x3F,0x19,0x0C,0x26,0x3F,0x00], // 122 'z'
    [0x38,0x0C,0x0C,0x07,0x0C,0x0C,0x38,0x00], // 123 '{'
    [0x18,0x18,0x18,0x00,0x18,0x18,0x18,0x00], // 124 '|'
    [0x07,0x0C,0x0C,0x38,0x0C,0x0C,0x07,0x00], // 125 '}'
    [0x6E,0x3B,0x00,0x00,0x00,0x00,0x00,0x00], // 126 '~'
];

const SCALE: u32 = 2;
const CHAR_W: u32 = 8 * SCALE;
const CHAR_H: u32 = 8 * SCALE;
const ITEM_HEIGHT: u32 = CHAR_H + 8;
const PAD_X: u32 = 8;
const SEPARATOR_HEIGHT: u32 = 9;

const BG_COLOR: [u8; 4] = [30, 30, 30, 230];
const HOVER_COLOR: [u8; 4] = [60, 60, 60, 230];
const HEADER_COLOR: [u8; 4] = [160, 160, 160, 255];
const TEXT_COLOR: [u8; 4] = [240, 240, 240, 255];
const ACTIVE_COLOR: [u8; 4] = [100, 180, 255, 255];
const SEP_COLOR: [u8; 4] = [80, 80, 80, 230];

#[derive(Clone)]
enum ItemKind {
    Header,
    Separator,
    Radio {
        group: &'static str,
        value: &'static str,
    },
    Toggle,
}

#[derive(Clone)]
struct MenuItem {
    label: &'static str,
    kind: ItemKind,
}

pub enum MenuAction {
    SetZoom(i32),
    SetShape(String),
    SetFit(String),
    ToggleMirror,
}

pub struct AppMenu {
    items: Vec<MenuItem>,
    pub visible: bool,
    pub pos_x: f32,
    pub pos_y: f32,
    pub hover_index: Option<usize>,
    menu_width: u32,
}

impl AppMenu {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                label: "Zoom",
                kind: ItemKind::Header,
            },
            MenuItem {
                label: "1x",
                kind: ItemKind::Radio {
                    group: "zoom",
                    value: "1",
                },
            },
            MenuItem {
                label: "1.5x",
                kind: ItemKind::Radio {
                    group: "zoom",
                    value: "2",
                },
            },
            MenuItem {
                label: "2x",
                kind: ItemKind::Radio {
                    group: "zoom",
                    value: "3",
                },
            },
            MenuItem {
                label: "",
                kind: ItemKind::Separator,
            },
            MenuItem {
                label: "Shape",
                kind: ItemKind::Header,
            },
            MenuItem {
                label: "Circle",
                kind: ItemKind::Radio {
                    group: "shape",
                    value: "circle",
                },
            },
            MenuItem {
                label: "Rounded Rect",
                kind: ItemKind::Radio {
                    group: "shape",
                    value: "rounded-rect",
                },
            },
            MenuItem {
                label: "",
                kind: ItemKind::Separator,
            },
            MenuItem {
                label: "Scale",
                kind: ItemKind::Header,
            },
            MenuItem {
                label: "Crop",
                kind: ItemKind::Radio {
                    group: "fit",
                    value: "cover",
                },
            },
            MenuItem {
                label: "Fit",
                kind: ItemKind::Radio {
                    group: "fit",
                    value: "contain",
                },
            },
            MenuItem {
                label: "Stretch",
                kind: ItemKind::Radio {
                    group: "fit",
                    value: "fill",
                },
            },
            MenuItem {
                label: "",
                kind: ItemKind::Separator,
            },
            MenuItem {
                label: "Mirror",
                kind: ItemKind::Toggle,
            },
        ];

        Self {
            items,
            visible: false,
            pos_x: 0.0,
            pos_y: 0.0,
            hover_index: None,
            menu_width: 200,
        }
    }

    pub fn show(&mut self, x: f32, y: f32) {
        self.visible = true;
        self.pos_x = x;
        self.pos_y = y;
        self.hover_index = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.hover_index = None;
    }

    fn item_y_offset(&self, index: usize) -> u32 {
        let mut y = 0u32;
        for i in 0..index {
            y += match self.items[i].kind {
                ItemKind::Separator => SEPARATOR_HEIGHT,
                _ => ITEM_HEIGHT,
            };
        }
        y
    }

    pub fn total_height(&self) -> u32 {
        self.item_y_offset(self.items.len())
    }

    pub fn update_hover(&mut self, screen_x: f32, screen_y: f32) {
        if !self.visible {
            return;
        }
        let rx = screen_x - self.pos_x;
        let ry = screen_y - self.pos_y;
        if rx < 0.0 || rx >= self.menu_width as f32 || ry < 0.0 || ry >= self.total_height() as f32
        {
            self.hover_index = None;
            return;
        }

        let mut y = 0u32;
        for (i, item) in self.items.iter().enumerate() {
            let h = match item.kind {
                ItemKind::Separator => SEPARATOR_HEIGHT,
                _ => ITEM_HEIGHT,
            };
            if (ry as u32) >= y && (ry as u32) < y + h {
                match item.kind {
                    ItemKind::Header | ItemKind::Separator => self.hover_index = None,
                    _ => self.hover_index = Some(i),
                }
                return;
            }
            y += h;
        }
        self.hover_index = None;
    }

    pub fn click(&self, screen_x: f32, screen_y: f32, _settings: &Settings) -> Option<MenuAction> {
        if !self.visible {
            return None;
        }
        let rx = screen_x - self.pos_x;
        let ry = screen_y - self.pos_y;
        if rx < 0.0 || rx >= self.menu_width as f32 || ry < 0.0 || ry >= self.total_height() as f32
        {
            return None;
        }

        let mut y = 0u32;
        for item in &self.items {
            let h = match item.kind {
                ItemKind::Separator => SEPARATOR_HEIGHT,
                _ => ITEM_HEIGHT,
            };
            if (ry as u32) >= y && (ry as u32) < y + h {
                return match &item.kind {
                    ItemKind::Radio { group, value } => match *group {
                        "zoom" => Some(MenuAction::SetZoom(value.parse().unwrap_or(1))),
                        "shape" => Some(MenuAction::SetShape(value.to_string())),
                        "fit" => Some(MenuAction::SetFit(value.to_string())),
                        _ => None,
                    },
                    ItemKind::Toggle => Some(MenuAction::ToggleMirror),
                    _ => None,
                };
            }
            y += h;
        }
        None
    }

    /// Renders the menu to an RGBA pixel buffer. Returns (pixels, width, height).
    pub fn render_pixels(&self, settings: &Settings) -> (Vec<u8>, u32, u32) {
        let w = self.menu_width;
        let h = self.total_height();
        let mut buf = vec![0u8; (w * h * 4) as usize];

        // Fill background
        fill_rect(&mut buf, w, 0, 0, w, h, BG_COLOR);

        let mut y = 0u32;
        for (i, item) in self.items.iter().enumerate() {
            match &item.kind {
                ItemKind::Separator => {
                    let mid = y + SEPARATOR_HEIGHT / 2;
                    fill_rect(&mut buf, w, PAD_X, mid, w - PAD_X * 2, 1, SEP_COLOR);
                    y += SEPARATOR_HEIGHT;
                }
                ItemKind::Header => {
                    draw_text(&mut buf, w, PAD_X, y + 4, item.label, HEADER_COLOR);
                    y += ITEM_HEIGHT;
                }
                ItemKind::Radio { group, value } => {
                    // Hover highlight
                    if self.hover_index == Some(i) {
                        fill_rect(&mut buf, w, 0, y, w, ITEM_HEIGHT, HOVER_COLOR);
                    }

                    let is_active = match *group {
                        "zoom" => {
                            settings.zoom_level.to_string() == *value
                        }
                        "shape" => settings.shape == *value,
                        "fit" => settings.fit_mode == *value,
                        _ => false,
                    };

                    let color = if is_active { ACTIVE_COLOR } else { TEXT_COLOR };
                    let prefix = if is_active { "> " } else { "  " };
                    let text = format!("{}{}", prefix, item.label);
                    draw_text(&mut buf, w, PAD_X, y + 4, &text, color);
                    y += ITEM_HEIGHT;
                }
                ItemKind::Toggle => {
                    if self.hover_index == Some(i) {
                        fill_rect(&mut buf, w, 0, y, w, ITEM_HEIGHT, HOVER_COLOR);
                    }
                    let color = if settings.flipped {
                        ACTIVE_COLOR
                    } else {
                        TEXT_COLOR
                    };
                    let prefix = if settings.flipped { "> " } else { "  " };
                    let text = format!("{}{}", prefix, item.label);
                    draw_text(&mut buf, w, PAD_X, y + 4, &text, color);
                    y += ITEM_HEIGHT;
                }
            }
        }

        (buf, w, h)
    }
}

fn fill_rect(buf: &mut [u8], buf_w: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for row in y..y + h {
        for col in x..x + w {
            if col < buf_w {
                let idx = ((row * buf_w + col) * 4) as usize;
                if idx + 3 < buf.len() {
                    buf[idx] = color[0];
                    buf[idx + 1] = color[1];
                    buf[idx + 2] = color[2];
                    buf[idx + 3] = color[3];
                }
            }
        }
    }
}

fn draw_text(buf: &mut [u8], buf_w: u32, x: u32, y: u32, text: &str, color: [u8; 4]) {
    for (ci, ch) in text.chars().enumerate() {
        let idx = ch as usize;
        if idx < 32 || idx > 126 {
            continue;
        }
        let glyph = &FONT[idx - 32];
        let cx = x + (ci as u32) * CHAR_W;

        for row in 0..8u32 {
            let bits = glyph[row as usize];
            for col in 0..8u32 {
                if bits & (1 << col) != 0 {
                    // Scale up
                    for sy in 0..SCALE {
                        for sx in 0..SCALE {
                            let px = cx + col * SCALE + sx;
                            let py = y + row * SCALE + sy;
                            let idx = ((py * buf_w + px) * 4) as usize;
                            if idx + 3 < buf.len() {
                                buf[idx] = color[0];
                                buf[idx + 1] = color[1];
                                buf[idx + 2] = color[2];
                                buf[idx + 3] = color[3];
                            }
                        }
                    }
                }
            }
        }
    }
}
