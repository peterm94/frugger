use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable, Triangle};
use frugger_core::{FruggerGame, FrugInputs, Orientation};
use libm::roundf;
use crate::OneBit;

#[derive(Clone, Default)]
struct Pos(f32, f32);

impl Pos {
    fn point(&self) -> Point {
        Point::new(roundf(self.0) as _, roundf(self.1) as _)
    }
}

struct State {
    grounded: bool,
    pos: Pos,
    vel: f32,
    triangles: heapless::Deque<f32, 5>,
    speed: f32,
}

pub struct Runner {
    engine: OneBit,
    state: State,
}

impl Runner {
    pub fn new(rng: u64) -> Self {
        Self {
            engine: OneBit::new(Orientation::Landscape),
            state: State {
                pos: Pos(10.0, Self::GROUND),
                grounded: true,
                vel: 0.0,
                triangles: heapless::Deque::new(),
                speed: 1.0,
            },
        }
    }

    const FILLED: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);
    const GRAVITY: f32 = 0.06;
    const GROUND: f32 = 50.0;
}

impl FruggerGame for Runner {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Landscape;
    type Color = BinaryColor;
    type Engine = OneBit;


    fn update(&mut self, inputs: &FrugInputs) {
        if inputs.a.pressed() && self.state.pos.1 == Self::GROUND {
            self.state.vel = -2.0;
        }

        // Gravity
        if self.state.pos.1 < Self::GROUND {
            self.state.vel += Self::GRAVITY;
        }

        // Apply velocity
        self.state.pos.1 += self.state.vel;

        // Reset on ground
        if self.state.pos.1 > Self::GROUND {
            self.state.pos.1 = Self::GROUND;
            self.state.vel = 0.0;
        }

        // remove off screen triangles
        if let Some(x) = self.state.triangles.front() {
            if x < &-20.0 {
                self.state.triangles.pop_front();
            }
        }

        // make new ones
        if self.state.triangles.is_empty() {
            self.state.triangles.push_back(140.0);
        }

        // move and render triangles
        for pos in self.state.triangles.iter_mut() {
            *pos -= self.state.speed;
            let tri = Triangle::new(Point::new(roundf(*pos) as i32, (Self::GROUND + 5.0) as i32),
                                    Point::new((roundf(*pos) + 10.0) as i32, (Self::GROUND + 5.0) as i32),
                                    Point::new((roundf(*pos) + 5.0) as i32, (Self::GROUND - 5.0) as i32));

            tri.draw_styled(&Self::FILLED, &mut self.engine);
        }


        // Draw player
        Circle::with_center(self.state.pos.point(), 10).draw_styled(&Self::FILLED, &mut self.engine);
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}