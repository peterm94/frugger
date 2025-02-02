use crate::{OneBit};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable, Triangle};
use frugger_core::{FrugInputs, FruggerGame, Orientation};
use libm::roundf;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Clone, Default)]
struct Pos(f32, f32);

impl Pos {
    fn point(&self) -> Point {
        Point::new(roundf(self.0) as _, roundf(self.1) as _)
    }
}
struct State {
    platforms: heapless::Vec<Pos, 100>,
    player_pos: Pos,
    player_vel: f32,
    rng: SmallRng,
    score: u32,
}

pub struct Jump {
    engine: OneBit,
    state: State,
}

impl Jump {
    pub fn new(rng: u64) -> Self {
        Self {
            engine: OneBit::new(Self::ORIENTATION),
            state: State {
                platforms: heapless::Vec::from_slice(&[
                    Pos(32.0, Jump::GROUND),
                    Pos(16.0, Jump::GROUND),
                    Pos(48.0, Jump::GROUND),
                ])
                .unwrap(),
                player_pos: Pos(32.0, 120.0),
                player_vel: 0.0,
                rng: SmallRng::seed_from_u64(rng),
                score: 0,
            },
        }
    }

    const PLAYER_STYLE: PrimitiveStyle<BinaryColor> =
        PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    const LEFT: f32 = 1.0;
    const RIGHT: f32 = -1.0;

    const PLAT_WIDTH: u32 = 10;
    const PLAT_HALF_W: f32 = Self::PLAT_WIDTH as f32 / 2.0 + 5.0;
    const MAX_VEL: f32 = 1.0;
    const GROUND: f32 = 122.0;
    const GRAVITY: f32 = 0.06;

    const PLATFORM: Rectangle = Rectangle::new(Point::zero(), Size::new(Self::PLAT_WIDTH, 3));
    const PLAYER: Triangle = Triangle::new(Point::new(0, 0), Point::new(-3, 10), Point::new(3, 10));

    const MAX_DIST: f32 = 30.0;
}

impl FruggerGame for Jump {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        // Inputs
        if inputs.a.down() {
            self.state.player_pos.0 += Self::LEFT;
        } else if inputs.b.down() {
            self.state.player_pos.0 += Self::RIGHT;
        }

        // Screen edges
        self.state.player_pos.0 += 64.0;
        self.state.player_pos.0 %= 64.0;

        // Check if on top of a platform
        let player = &self.state.player_pos;

        if self.state.player_vel > 0.0 {
            for Pos(x, y) in &self.state.platforms {
                if (*y >= player.1 && *y < player.1 + 2.0)
                    && (*x < player.0 + Jump::PLAT_HALF_W && *x > player.0 - Jump::PLAT_HALF_W)
                {
                    self.state.player_vel = -2.0;
                    break;
                }
            }
        }

        // Gravity - always applies
        self.state.player_vel += Self::GRAVITY;

        // Apply velocity
        self.state.player_pos.1 += self.state.player_vel;

        // Shift everything to move the screen up
        let move_amt = 64.0 - self.state.player_pos.1;
        if move_amt > 0.0 {
            self.state
                .platforms
                .iter_mut()
                .for_each(|Pos(_, y)| *y += move_amt);
            self.state.player_pos.1 += move_amt;
            self.state.score += move_amt as u32;
        }

        // Draw platforms
        self.state.platforms.iter().for_each(|platform| {
            Jump::PLATFORM
                .translate(platform.point())
                .translate_mut(Point::new(-5, 0))
                .draw_styled(&Self::PLAYER_STYLE, &mut self.engine)
                .unwrap();
        });

        // Draw player
        Jump::PLAYER
            .translate(self.state.player_pos.point())
            .translate_mut(Point::new(0, -10))
            .draw_styled(&Self::PLAYER_STYLE, &mut self.engine)
            .unwrap();

        // Draw score
        // Text::with_text_style(self.state.score.to_string().as_str(), Point::zero(), )

        // Clean up oob platforms and spawn new ones
        self.state.platforms.retain(|platform| platform.1 < 129.0);

        while self.state.platforms.len() < 10 {
            let last = self.state.platforms.last().unwrap();
            let nx = self.state.rng.gen_range(5.0..64.0 - 5.0);
            let ny = self.state.rng.gen_range(0.0..Jump::MAX_DIST);
            self.state.platforms.push(Pos(nx, last.1 - ny));
        }
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}
