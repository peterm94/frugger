use crate::OneBit;
use core::fmt::Write;
use core::str::FromStr;
use embedded_graphics::mono_font::ascii::{FONT_6X12, FONT_8X13};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use frugger_core::{FrugInputs, FruggerGame, Orientation};
use heapless::{String, Vec};

struct State {
    score_table: Vec<(String<3>, u16), 5>,
    new_score: u16,
    new_score_line: usize,
    new_name: [u8; 3],
    curr_idx: usize,
    frame: usize,
    save_fn: fn(usize, [u8; 32]),
}

pub struct HiScore {
    engine: OneBit,
    state: State,
}

impl FruggerGame for HiScore {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        self.state.frame += 1;

        let state = &mut self.state;
        let engine = &mut self.engine;

        // No high score, not interactive.
        // Render scores
        Self::draw_header("HI\nSCORES", engine);
        Self::draw_scores(state, engine);
        Self::draw_edit(state, engine);

        // Nothing to do
        if state.new_score_line == 10 {
            return;
        }

        // exit early, already done
        if state.curr_idx == 3 {
            return;
        }

        if inputs.left.pressed() {
            let curr = state.new_name[state.curr_idx] + 26;
            state.new_name[state.curr_idx] = (curr - 1) % 26;
        }
        if inputs.right.pressed() {
            let curr = state.new_name[state.curr_idx];
            state.new_name[state.curr_idx] = (curr + 1) % 26;
        }
        if inputs.a.pressed() {
            state.curr_idx += 1;

            if state.curr_idx == 3 {
                // Save
                let name = &mut state.score_table[state.new_score_line].0;
                name.clear();
                name.push((state.new_name[0] + 65) as char);
                name.push((state.new_name[1] + 65) as char);
                name.push((state.new_name[2] + 65) as char);
                (state.save_fn)(0, score_to_data(&state.score_table));
            }
        }
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}

impl HiScore {
    pub fn new(loaded: &[u8], new_score: u16, save_fn: fn(usize, [u8; 32])) -> Self {
        let mut score_table = Vec::new();

        let mut new_score_line = 10;

        // 128 bytes for storage
        // Each record is 6 bytes (bool, entry exists) + 3 bytes (name) + 2 bytes (score)
        // We will have up to 5 entries which will use 30 / 128 bytes of the storage for the game
        for i in 0..5 {
            let start = i * 6;
            let end = start + 6;
            let score_slice = &loaded[start..end];

            // blank entry, we can put our hi score in here
            if score_slice[0] == 0 {
                new_score_line = i;
                let default_name = String::<3>::from_str("   ").unwrap();
                score_table.push((default_name, new_score)).unwrap();
                break;
            }

            let mut name_str = String::<3>::new();
            name_str.push((score_slice[1] + 65) as char).unwrap();
            name_str.push((score_slice[2] + 65) as char).unwrap();
            name_str.push((score_slice[3] + 65) as char).unwrap();

            let score = u16::from_le_bytes(score_slice[4..6].try_into().unwrap());

            // We need to slot the new score in here
            if new_score_line > 5 && new_score > score {
                new_score_line = i;
                let default_name = String::<3>::new();
                score_table.push((default_name, new_score)).unwrap();

                // Drop the last score if this takes it's place
                if i == 4 {
                    break;
                }
            }

            score_table.push((name_str, score));
        }

        Self {
            engine: OneBit::new(Self::ORIENTATION),
            state: State {
                score_table,
                new_score,
                new_score_line,
                new_name: [0, 0, 0],
                curr_idx: 0,
                frame: 0,
                save_fn,
            },
        }
    }
    fn draw_edit(state: &State, engine: &mut OneBit) {
        if state.new_score_line == 10 {
            return;
        }

        let (_, score) = &state.score_table[state.new_score_line];
        let mut tmp = heapless::String::<3>::new();

        for (i, char) in state.new_name.iter().enumerate() {
            if i == state.curr_idx && state.frame % 30 > 15 {
                tmp.push(' ').unwrap();
            } else {
                tmp.push((char + 65) as char).unwrap();
            }
        }

        let mut name_text = Text::new(
            tmp.as_str(),
            Point::new(3, (50 + (state.new_score_line * 10)) as _),
            MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
        );
        name_text.text_style.alignment = Alignment::Left;
        name_text.draw(engine);

        // let mut score_text = heapless::String::<11>::new();
        // write!(&mut score_text, "{}", score).unwrap();
        //
        // let mut score_text = Text::new(
        //     &score_text,
        //     Point::new(64 - 3, (50 + (state.new_score_line * 10)) as _),
        //     MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
        // );
        // score_text.text_style.alignment = Alignment::Right;
        // score_text.draw(engine);
    }
    fn draw_scores(state: &State, engine: &mut OneBit) {
        for (line, (name, score)) in state.score_table.iter().enumerate() {
            // if state.new_score_line == line {
            //     continue;
            // }
            let mut name_text = Text::new(
                name,
                Point::new(3, (50 + (line * 10)) as _),
                MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
            );
            name_text.text_style.alignment = Alignment::Left;
            name_text.draw(engine);

            let mut score_text = heapless::String::<11>::new();
            write!(&mut score_text, "{}", score).unwrap();

            let mut score_text = Text::new(
                &score_text,
                Point::new(64 - 3, (50 + (line * 10)) as _),
                MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
            );
            score_text.text_style.alignment = Alignment::Right;
            score_text.draw(engine);
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

fn score_to_data(scores: &Vec<(String<3>, u16), 5>) -> [u8; 32] {
    let mut buf = [0u8; 32];

    for (i, (name, score)) in scores.iter().enumerate() {
        let offset = i * 6;
        buf[offset] = u8::MAX;
        let name = name.as_bytes();
        buf[offset + 1] = name[0] - 65;
        buf[offset + 2] = name[1] - 65;
        buf[offset + 3] = name[2] - 65;
        buf[offset + 4] = score.to_le_bytes()[0];
        buf[offset + 5] = score.to_le_bytes()[1];
    }

    buf
}
