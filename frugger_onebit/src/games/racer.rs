use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle, StyledDrawable, Triangle};
use frugger_core::{FrugInputs, FruggerGame, Orientation};
use libm::roundf;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use crate::OneBit;

#[derive(Clone, Default)]
struct Pos(f32, f32);

impl Pos {
    fn point(&self) -> Point {
        Point::new(roundf(self.0) as _, roundf(self.1) as _)
    }
}
struct State {
    walls: heapless::Vec<((Pos, Pos), (Pos, Pos)), 20>,
    player_pos: Pos,
    player_vel: f32,
    rng: SmallRng,
    road_min: f32, // score: u32,
}

pub struct Racer {
    engine: OneBit,
    state: State,
}

impl Racer {
    pub fn new(rng: u64) -> Self {
        Self {
            engine: OneBit::new(Self::ORIENTATION),
            state: State {
                walls: heapless::Vec::from_slice(&[(
                    (Pos(10.0, 128.0), Pos(10.0, -200.0)),
                    (Pos(54.0, 128.0), Pos(54.0, -200.0)),
                )])
                .unwrap(),
                player_pos: Pos(32.0, 115.0),
                player_vel: 0.0,
                rng: SmallRng::seed_from_u64(rng),
                road_min: 20.0,
            },
        }
    }

    const WALL_STYLE: PrimitiveStyle<BinaryColor> =
        PrimitiveStyle::with_stroke(BinaryColor::On, 2);
    const PLAYER_STYLE: PrimitiveStyle<BinaryColor> =
        PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    const LEFT: f32 = -1.2;
    const RIGHT: f32 = 1.2;

    const PLAYER: Triangle = Triangle::new(Point::new(0, 0), Point::new(-4, 10), Point::new(4, 10));
    const PLAYER_L: Triangle =
        Triangle::new(Point::new(-1, 0), Point::new(-3, 11), Point::new(5, 9));
    const PLAYER_R: Triangle =
        Triangle::new(Point::new(1, 0), Point::new(-5, 9), Point::new(3, 11));
}

impl FruggerGame for Racer {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        // Inputs
        let sprite = if inputs.left.down() {
            self.state.player_pos.0 += Self::LEFT;
            Self::PLAYER_L
        } else if inputs.right.down() {
            self.state.player_pos.0 += Self::RIGHT;
            Self::PLAYER_R
        } else {
            Self::PLAYER
        };

        // Shift everything to move the screen up
        let move_amt = 2.0;
        if move_amt > 0.0 {
            self.state.walls.iter_mut().for_each(
                |((Pos(_, y), Pos(_, y2)), (Pos(_, y3), Pos(_, y4)))| {
                    *y += move_amt;
                    *y2 += move_amt;
                    *y3 += move_amt;
                    *y4 += move_amt;
                },
            );
        }

        // Draw walls
        self.state.walls.iter().for_each(|(w1, w2)| {
            Line::new(w1.0.point(), w1.1.point())
                .draw_styled(&Self::WALL_STYLE, &mut self.engine)
                .unwrap();
            Line::new(w2.0.point(), w2.1.point())
                .draw_styled(&Self::WALL_STYLE, &mut self.engine)
                .unwrap();
        });

        // Draw player
        sprite
            .translate(self.state.player_pos.point())
            // .translate_mut(Point::new(0, -10))
            .draw_styled(&Self::PLAYER_STYLE, &mut self.engine)
            .unwrap();


        // Draw score
        // Text::with_text_style(self.state.score.to_string().as_str(), Point::zero(), )

        // Clean old walls and spawn new ones
        self.state.walls.retain(|(w1, _)| w1.1 .1 < 129.0);

        while self.state.walls.len() < 15 {
            let (last_left, last_right) = self.state.walls.last().unwrap();

            let ny = roundf(last_left.1 .1 - self.state.rng.gen_range(50.0..100.0));
            let start1 = last_left.1.clone();
            let start2 = last_right.1.clone();

            let n1 = self.state.rng.gen_range(1.0..64.0 - self.state.road_min);
            let n2 = self.state.rng.gen_range(n1 + self.state.road_min..64.0);

            self.state
                .walls
                .push(((start1, Pos(n1, ny)), (start2, Pos(n2, ny))));
        }
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}
