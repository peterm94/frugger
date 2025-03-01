use crate::OneBit;
use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;
use frugger_core::{FrugInputs, FruggerGame, Orientation};
use heapless::Deque;
use libm::{cosf, roundf, sinf, sqrt};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Clone)]
struct Pos(f32, f32);

impl Pos {
    fn point(&self) -> Point {
        Point::new(roundf(self.0) as _, roundf(self.1) as _)
    }
}

struct GameState {
    apple: Circle,
    segments: Deque<Pos, 1000>,
    dir: f32,
    speed: f32,
    rng: SmallRng,
    game_over: bool,
}

pub struct SmolWorm {
    engine: OneBit,
    state: GameState,
}

impl SmolWorm {
    pub fn new(rng: u64) -> Self {
        let mut segments = Deque::new();
        segments.push_back(Pos(32.0, 64.0));

        let mut worm = Self {
            engine: OneBit::new(Self::ORIENTATION),
            state: GameState {
                apple: Circle::new(Point::new(20, 100), 4),
                segments,
                dir: 0.0,
                speed: 0.5,
                rng: SmallRng::seed_from_u64(rng),
                game_over: false,
            },
        };

        worm.add_head();
        worm.add_head();
        worm.add_head();
        worm.add_head();
        worm.add_head();
        worm.add_head();
        worm.add_head();
        worm.add_head();
        worm
    }

    fn add_head(&mut self) -> Pos {
        let head = self.state.segments.front().unwrap();
        let move_x = self.state.speed * cosf(self.state.dir);
        let move_y = self.state.speed * sinf(self.state.dir);
        let mut new_head = Pos(move_x + head.0, move_y + head.1);

        new_head.0 = (new_head.0 + 64.0) % 64.0;
        new_head.1 = (new_head.1 + 128.0) % 128.0;

        self.state.segments.push_front(new_head.clone());
        new_head
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

    const APPLE_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);
    const WORM_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);
}

trait Distance {
    fn distance(&self, other: &Point) -> f32;
}
impl Distance for Point {
    fn distance(&self, other: &Point) -> f32 {
        let dx = (self.x - other.x).pow(2);
        let dy = (self.y - other.y).pow(2);
        sqrt((dx + dy) as f64) as f32
    }
}

impl FruggerGame for SmolWorm {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        // inputs
        if inputs.left.down() {
            self.state.dir -= 0.2 * (self.state.speed / 2.0);
        }

        if inputs.right.down() {
            self.state.dir += 0.2 * (self.state.speed / 2.0);
        }

        let head = if !self.state.game_over {
            &self.add_head()
        } else {
            if self.state.segments.len() == 0 {
                Self::draw_text("GAME\nOVER", &mut self.engine);
                return;
            }

            self.state.segments.front().unwrap()
        };
        let head = head.point();

        // Pad the collision box a bit
        if self.state.apple.center().distance(&head) < 3.5 {
            // Don't remove the tail, move the apple
            self.add_head();
            self.add_head();
            self.state.apple.top_left = Point::new(
                self.state.rng.gen_range(2..62),
                self.state.rng.gen_range(2..126),
            );
            self.state.speed += 0.035;
        } else {
            // Remove the tail so we stay the same length
            self.state.segments.pop_back();
        }

        for (i, seg) in self.state.segments.iter().enumerate() {
            let point = seg.point();

            // Check for collisions
            if i > 10 && point.distance(&head) <= 2.0 {
                self.state.game_over = true;
            }
            Circle::with_center(point, 2)
                .draw_styled(&Self::WORM_STYLE, &mut self.engine)
                .unwrap();
        }

        self.state
            .apple
            .draw_styled(&Self::APPLE_STYLE, &mut self.engine)
            .unwrap();
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}
