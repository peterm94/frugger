use crate::OneBit;
use core::fmt::Write;
use embedded_graphics::mono_font::ascii::{FONT_6X12, FONT_8X13};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};

use frugger_core::{FrugInputs, FruggerGame, Orientation};

pub struct HiScore {
    engine: OneBit,
    score_table: heapless::Vec<(heapless::String<3>, u16), 5>,
    new_score: u16,
    new_name: [u8; 3],
    curr_idx: usize,
}

impl FruggerGame for HiScore {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        // Render scores
        Self::draw_header("HI\nSCORES", &mut self.engine);
        self.draw_scores();
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}

const NAME_LUT: [&'static str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

impl HiScore {
    pub fn new(loaded: &[u8], new_score: u16) -> Self {
        let mut score_table = heapless::Vec::new();

        // 128 bytes for storage
        // Each record is 1 byte (bool, entry exists) + 3 bytes (name) + 2 bytes (score)
        // We will have up to 5 entries which will use 30 / 128 bytes of the storage for the game
        for i in 0..5 {
            let start = i * 6;
            let end = start + 6;
            let score_slice = &loaded[start..end];

            // blank entry
            if score_slice[0] == 0 {
                continue;
            }

            let mut name_str = heapless::String::<3>::new();
            name_str.push((score_slice[1] + 65) as char).unwrap();
            name_str.push((score_slice[2] + 65) as char).unwrap();
            name_str.push((score_slice[3] + 65) as char).unwrap();

            let score = u16::from_le_bytes(score_slice[4..6].try_into().unwrap());
            score_table.push((name_str, score));
        }

        Self {
            engine: OneBit::new(Self::ORIENTATION),
            score_table,
            new_score,
            new_name: [0; 3],
            curr_idx: 0,
        }
    }

    fn draw_scores(&mut self) {
        for (line, (name, score)) in self.score_table.iter().enumerate() {
            let mut name_text = Text::new(
                name,
                Point::new(3, (50 + (line * 10)) as _),
                MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
            );
            name_text.text_style.alignment = Alignment::Left;
            name_text.draw(&mut self.engine);

            let mut score_text = heapless::String::<11>::new();
            write!(&mut score_text, "{}", score).unwrap();

            let mut score_text = Text::new(
                &score_text,
                Point::new(64 - 3, (50 + (line * 10)) as _),
                MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
            );
            score_text.text_style.alignment = Alignment::Right;
            score_text.draw(&mut self.engine);
        }
    }

    fn draw_header(content: &str, engine: &mut OneBit) {
        let mut text = Text::new(
            content,
            Point::new(32, 20),
            MonoTextStyle::new(&FONT_8X13, BinaryColor::On),
        );
        text.text_style.alignment = Alignment::Center;
        text.draw(engine);
    }
}
