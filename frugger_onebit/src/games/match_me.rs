use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use frugger_core::{FrugInputs, FruggerGame, Orientation};
use heapless::Deque;
use rand::prelude::SmallRng;
use rand::SeedableRng;
use crate::OneBit;

struct Tile {
    pos: Point,
    on: bool,
}

struct State {
    rng: SmallRng,
    tiles: [Tile; 3],
    sequence: Deque<u8, 100>,
    ptr: usize,
    timer: u64,
    showing: bool,
}

pub struct MatchMe {
    engine: OneBit,
    state: State,
}

impl MatchMe {
    const RECT: Rectangle = Rectangle::new(Point::zero(), Size::new_equal(16));
    const PRESSED: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 10);
    const OFF: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    pub fn new(rng: u64) -> Self {
        Self {
            engine: OneBit::new(Self::ORIENTATION),
            state: State {
                rng: SmallRng::seed_from_u64(rng),
                tiles: [
                    Tile {
                        pos: Point::new(4, 50),
                        on: false,
                    },
                    Tile {
                        pos: Point::new(24, 70),
                        on: false,
                    },
                    Tile {
                        pos: Point::new(44, 50),
                        on: false,
                    },
                ],
                sequence: Deque::new(),
                ptr: 0,
                timer: 60 * MatchMe::TARGET_FPS,
                showing: true,
            },
        }
    }

    fn draw_tile(tile: &Tile, engine: &mut <MatchMe as FruggerGame>::Engine) {
        Self::RECT
            .translate(tile.pos)
            .draw_styled(if tile.on { &Self::PRESSED } else { &Self::OFF }, engine)
            .unwrap()
    }
}

impl FruggerGame for MatchMe {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Portrait;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        let state = &mut self.state;

        if state.timer == 0 {
        } else {
            state.timer -= 1;
        }

        for tile in &self.state.tiles {
            MatchMe::draw_tile(tile, &mut self.engine);
        }
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}
