use embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use frugger_core::{ButtonState, FruggerGame, FrugInputs, Palette, Orientation};
use crate::OneBit;

pub struct InputTestSmall {
    engine: OneBit,
}

impl InputTestSmall {
    pub fn new() -> Self {
        Self {
            engine: OneBit::new(Orientation::Landscape)
        }
    }

    fn draw_styled(button: &ButtonState, rect: &Rectangle, engine: &mut <InputTestSmall as FruggerGame>::Engine) {
        let pressed_style = PrimitiveStyle::with_stroke(BinaryColor::On, 10);
        let held_style = PrimitiveStyle::with_stroke(BinaryColor::On, 5);
        let off_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let released_style = PrimitiveStyle::with_stroke(BinaryColor::On, 8);

        match button {
            ButtonState::PRESSED => { rect.draw_styled(&pressed_style, engine); }
            ButtonState::RELEASED => { rect.draw_styled(&released_style, engine); }
            ButtonState::DOWN => { rect.draw_styled(&held_style, engine); }
            ButtonState::UP => { rect.draw_styled(&off_style, engine); }
        }
    }
}

impl FruggerGame for InputTestSmall {
    const TARGET_FPS: u64 = 60;
    const ORIENTATION: Orientation = Orientation::Landscape;
    type Color = BinaryColor;
    type Engine = OneBit;

    fn update(&mut self, inputs: &FrugInputs) {
        let left = Rectangle::new(Point::new(0, 20), Size::new_equal(10));
        let a = Rectangle::new(Point::new(20, 20), Size::new_equal(10));
        let right = Rectangle::new(Point::new(40, 20), Size::new_equal(10));

        InputTestSmall::draw_styled(&inputs.left, &left, &mut self.engine);
        InputTestSmall::draw_styled(&inputs.a, &a, &mut self.engine);
        InputTestSmall::draw_styled(&inputs.right, &right, &mut self.engine);
    }

    fn frugger(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}
