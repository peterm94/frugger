#![cfg_attr(not(test), no_std)]

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use frugger_core::{Frugger, Palette};
use heapless::Vec;

#[derive(Default)]
struct Ball {
    pos: Point,
    ang: u16,
    spd: u16,
}

impl Ball {
    const RADIUS: u16 = 6;
}

struct Brick {
    col: Palette,
    rect: Rectangle,
}

impl Brick {
    const WIDTH: u32 = 30;
    const HEIGHT: u32 = 10;
}

#[derive(Default)]
struct GameState {
    bricks: Vec<Brick, 70>,
    pad_pos: Point,
    ball: Ball,
}

impl GameState {
    const PAD_WIDTH: u16 = 100;
    const PAD_HEIGHT: u16 = 10;
    const PAD_SPEED: u16 = 10;
}


struct BrickBreaker {
    frugger: Frugger,
    state: GameState,
}

impl BrickBreaker {
    fn new() -> Self {
        let mut bricks: Vec<Brick, 70> = Vec::new();

        let cols = [Palette::Purple, Palette::Red, Palette::Orange, Palette::Yellow, Palette::Lime, Palette::Green, Palette::Teal];

        for i in 0..10 {
            for j in 0..7 {
                bricks.push(Brick {
                    rect: Rectangle::new(Point::new(i * 32 + 1, j * 12 + 1), Size::new(Brick::WIDTH, Brick::HEIGHT)),
                    col: cols[j as usize],
                }).ok();
            }
        }


        Self {
            frugger: Frugger::new(Palette::BlueGrey),
            state: GameState {
                bricks,
                ..Default::default()
            },
        }
    }

    fn update<T>(&mut self, display: &mut T) where T: DrawTarget<Color=Rgb565> {
        for brick in &self.state.bricks {
            let style = PrimitiveStyle::with_stroke(brick.col, 1);
            brick.rect.draw_styled(&style, &mut self.frugger);
        }

        self.frugger.draw_frame(display);
    }
}


#[cfg(test)]
mod tests {
    use embedded_graphics::geometry::Size;
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window};

    use crate::BrickBreaker;

    #[test]
    fn run() {
        let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .build();

        let mut window = Window::new("Brickbreaker", &output_settings);

        let mut brickbreaker = BrickBreaker::new();

        'game: loop {
            brickbreaker.update(&mut display);
            window.update(&display);

            for event in window.events() {
                match event {
                    SimulatorEvent::KeyUp { .. } => {}
                    SimulatorEvent::KeyDown { .. } => {}
                    SimulatorEvent::MouseButtonUp { .. } => {}
                    SimulatorEvent::MouseButtonDown { .. } => {}
                    SimulatorEvent::MouseWheel { .. } => {}
                    SimulatorEvent::MouseMove { .. } => {}
                    SimulatorEvent::Quit => break 'game
                }
            }
        }
    }
}