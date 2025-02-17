use crate::util::SM;
use crate::OneBit;
use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::{Alignment, Text};
use frugger_core::{FrugInputs, FruggerGame, Orientation};
use heapless::Vec;
use numtoa::NumToA;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

struct State {
    rng: SmallRng,
    tiles: [Point; 3],
    sequence: Vec<u8, 100>,
    ptr: usize,
    timer: u64,
    showing: bool,
}

pub struct MatchMe {
    engine: OneBit,
    state: State,
    sm: SM<State>,
}

impl MatchMe {
    const RECT: Rectangle = Rectangle::new(Point::zero(), Size::new_equal(16));
    const PRESSED: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 4);
    const OFF: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    fn draw_blank(tiles: &[Point; 3], engine: &mut OneBit) {
        for tile in tiles {
            MatchMe::draw_tile(tile, false, engine);
        }
    }

    pub fn new(rng: u64) -> Self {
        let mut sm = SM::new();

        // Start timer, give it a bit
        sm.add(
            0,
            |state: &mut State, inputs: &FrugInputs, engine: &mut OneBit| {
                state.timer -= 1;
                Self::draw_text("REPEAT\nPATTERN", engine);
                MatchMe::draw_blank(&state.tiles, engine);
                return if state.timer == 0 { 1 } else { 0 };
            },
        );

        // Play it
        sm.add(
            1,
            |state: &mut State, inputs: &FrugInputs, engine: &mut OneBit| {
                // Set timer for how long to show the current state
                if state.timer == 0 {
                    state.timer = 30;
                }

                state.timer -= 1;

                // If timer runs out, go to next item in sequence
                if state.timer == 0 {
                    state.ptr += 1;

                    // At the end, move to input mode and reset the pointer
                    if state.sequence.len() == state.ptr {
                        state.ptr = 0;
                        return 2;
                    } else {
                        return 3;
                    }
                }

                Self::draw_step(state.ptr, engine);
                for (i, tile) in state.tiles.iter().enumerate() {
                    MatchMe::draw_tile(tile, i == state.sequence[state.ptr] as usize, engine);
                }
                return 1;
            },
        );

        // User turn
        sm.add(
            2,
            |state: &mut State, inputs: &FrugInputs, engine: &mut OneBit| {
                MatchMe::draw_tile(&state.tiles[0], inputs.left.down(), engine);
                MatchMe::draw_tile(&state.tiles[1], inputs.a.down(), engine);
                MatchMe::draw_tile(&state.tiles[2], inputs.right.down(), engine);

                let req = state.sequence[state.ptr] as usize;
                Self::draw_step(state.ptr, engine);

                if inputs.left.pressed() {
                    if req == 0 {
                        // Correct, move pointer
                        state.ptr += 1;
                    } else {
                        return 5;
                    }
                } else if inputs.a.pressed() {
                    if req == 1 {
                        // Correct, move pointer
                        state.ptr += 1;
                    } else {
                        return 5;
                    }
                } else if inputs.right.pressed() {
                    if req == 2 {
                        // Correct, move pointer
                        state.ptr += 1;
                    } else {
                        return 5;
                    }
                }

                if state.sequence.len() == state.ptr {
                    // Correct! add to sequence, start again
                    state.sequence.push(state.rng.gen_range(0..=2));
                    state.ptr = 0;
                    return 4;
                }

                return 2;
            },
        );

        sm.add(
            3,
            |state: &mut State, inputs: &FrugInputs, engine: &mut OneBit| {
                Self::draw_blank(&state.tiles, engine);
                Self::draw_step(state.ptr, engine);
                if state.timer == 0 {
                    state.timer = 5;
                }

                state.timer -= 1;

                if state.timer == 0 {
                    return 1;
                }

                3
            },
        );

        sm.add(
            4,
            |state: &mut State, inputs: &FrugInputs, engine: &mut OneBit| {
                MatchMe::draw_tile(&state.tiles[0], inputs.left.down(), engine);
                MatchMe::draw_tile(&state.tiles[1], inputs.a.down(), engine);
                MatchMe::draw_tile(&state.tiles[2], inputs.right.down(), engine);

                Self::draw_text("PASS", engine);
                if state.timer == 0 {
                    state.timer = 60;
                }

                state.timer -= 1;

                if state.timer == 0 {
                    return 1;
                }

                4
            },
        );

        // loser
        sm.add(
            5,
            |state: &mut State, inputs: &FrugInputs, engine: &mut OneBit| {
                Self::draw_text("LOSER", engine);
                5
            },
        );

        let mut rng = SmallRng::seed_from_u64(rng);
        let mut sequence = Vec::new();
        sequence.push(rng.gen_range(0..=2));

        Self {
            engine: OneBit::new(Self::ORIENTATION),
            state: State {
                rng,
                tiles: [Point::new(4, 50), Point::new(24, 70), Point::new(44, 50)],
                sequence,
                ptr: 0,
                timer: 2 * MatchMe::TARGET_FPS,
                showing: true,
            },
            sm,
        }
    }

    fn draw_tile(tile: &Point, active: bool, engine: &mut <MatchMe as FruggerGame>::Engine) {
        Self::RECT
            .translate(*tile)
            .draw_styled(if active { &Self::PRESSED } else { &Self::OFF }, engine)
            .unwrap()
    }

    fn draw_step(step: usize, engine: &mut OneBit) {
        let mut buf = [0u8; 11];
        let step_str = (step + 1).numtoa_str(10, &mut buf);
        Self::draw_text(step_str, engine);
    }
    fn draw_text(content: &str, engine: &mut OneBit) {
        let mut text = Text::new(
            content,
            Point::new(32, 32),
            MonoTextStyle::new(&FONT_8X13, BinaryColor::On),
        );
        text.text_style.alignment = Alignment::Center;
        text.draw(engine);
    }
}

impl FruggerGame for MatchMe {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        self.sm.tick(&mut self.state, inputs, &mut self.engine);
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}
