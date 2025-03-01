use crate::games::input_test::InputTestSmall;
use crate::games::match_me::MatchMe;
use crate::games::racer::Racer;
use crate::games::runner::Runner;
use crate::games::triangle_jump::Jump;
use crate::games::worm::SmolWorm;
use crate::hi_score::HiScore;
use crate::{OneBit, Signal};
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_7X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;
use frugger_core::{FrugInputs, FruggerGame, Orientation};

pub enum Game {
    Scores(HiScore),
    InputTest(InputTestSmall),
    MatchMe(MatchMe),
    Racer(Racer),
    Runner(Runner),
    TriangleJump(Jump),
    Worm(SmolWorm),
}

#[derive(Clone)]
pub enum SaveOffset {
    TriangleScores = 0,
}

impl FruggerGame for Game {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        match self {
            Game::Scores(game) => game.update(inputs),
            Game::InputTest(game) => game.update(inputs),
            Game::MatchMe(game) => game.update(inputs),
            Game::Racer(game) => game.update(inputs),
            Game::Runner(game) => game.update(inputs),
            Game::TriangleJump(game) => game.update(inputs),
            Game::Worm(game) => game.update(inputs),
        }
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        match self {
            Game::Scores(game) => game.frugger(),
            Game::InputTest(game) => game.frugger(),
            Game::MatchMe(game) => game.frugger(),
            Game::Racer(game) => game.frugger(),
            Game::Runner(game) => game.frugger(),
            Game::TriangleJump(game) => game.frugger(),
            Game::Worm(game) => game.frugger(),
        }
    }
}
pub struct Menu {
    engine: OneBit,
    curr_game: Option<Game>,
    game_changed: bool,
    selection: u8,
    ticks: u64,
    pause_start: u64,
    pub load: fn() -> [u8; 1024],
    pub save: fn(usize, [u8; 32]),
}

impl Menu {
    pub fn new(load: fn() -> [u8; 1024], save: fn(usize, [u8; 32])) -> Self {
        Self {
            engine: OneBit::new(Self::ORIENTATION),
            selection: 0,
            game_changed: false,
            curr_game: None,
            ticks: 0,
            pause_start: 0,
            load,
            save,
        }
    }
}

impl FruggerGame for Menu {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        self.ticks = self.ticks.wrapping_add(1);

        if inputs.left.down() && inputs.right.down() {
            self.pause_start += 1;
            if self.pause_start == 120 {
                // Force a full screen redraw
                self.game_changed = true;
                self.curr_game = None;
                return;
            }
        } else {
            self.pause_start = 0;
        }

        if let Some(signal) = self
            .curr_game
            .as_mut()
            .and_then(|game| game.frugger().signal.clone())
        {
            match signal {
                Signal::Save { save_offset, score } => {
                    let offset = save_offset as usize * 32;
                    self.curr_game = Some(Game::Scores(HiScore::new(
                        &(self.load)()[offset..offset + 32],
                        score,
                        self.save,
                    )));

                    self.game_changed = true;
                    return;
                }
            }
        }

        if let Some(game) = &mut self.curr_game {
            game.update(inputs);
            return;
        }

        // Inputs
        if inputs.right.pressed() {
            self.selection = (self.selection + 1) % 4;
        } else if inputs.left.pressed() {
            self.selection = (self.selection + 3) % 4;
        } else if inputs.a.pressed() {
            // start the game
            self.curr_game = match self.selection {
                0 => Some(Game::TriangleJump(Jump::new(self.ticks))),
                1 => Some(Game::Worm(SmolWorm::new(self.ticks))),
                2 => Some(Game::Racer(Racer::new(self.ticks))),
                3 => Some(Game::MatchMe(MatchMe::new(self.ticks))),
                _ => None,
            };

            self.game_changed = true;
            return;
        }

        // Draw menu
        let txt_style = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);

        let mut text = Text::new("Jump", Point::new(32, 30), txt_style);
        text.text_style.alignment = Alignment::Center;
        text.draw(&mut self.engine).unwrap();

        let mut text = Text::new("Worm", Point::new(32, 45), txt_style);
        text.text_style.alignment = Alignment::Center;
        text.draw(&mut self.engine).unwrap();

        let mut text = Text::new("Racer", Point::new(32, 60), txt_style);
        text.text_style.alignment = Alignment::Center;
        text.draw(&mut self.engine).unwrap();

        let mut text = Text::new("Match", Point::new(32, 75), txt_style);
        text.text_style.alignment = Alignment::Center;
        text.draw(&mut self.engine).unwrap();

        let mut sel = Rectangle::new(Point::new(1, 18), Size::new(62, 17));
        sel.translate_mut(Point::new(0, (self.selection * 15) as i32));

        sel.draw_styled(
            &PrimitiveStyle::with_stroke(BinaryColor::On, 1),
            &mut self.engine,
        )
        .unwrap()
    }

    fn frugger(&mut self) -> &mut Self::Engine {

        let engine = if let Some(game) = &mut self.curr_game {
            game.frugger()
        } else {
            &mut self.engine
        };

        if self.game_changed {
            self.game_changed = false;
            engine.clear_buffer()
        }

        engine
    }
}
