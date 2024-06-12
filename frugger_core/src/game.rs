#[cfg(test)]
mod tests {
    use embedded_graphics::geometry::{Point, Size};
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
    use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window};

    use crate::{Frugger, FruggerColour};

    #[test]
    fn test() {
        let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .build();

        let d1 = display.clone();

        let mut frugger = Frugger::new(FruggerColour::Blue);

        let mut window = Window::new("Test", &output_settings);

        let f_style = PrimitiveStyleBuilder::new()
            .stroke_color(FruggerColour::Red)
            .stroke_width(3)
            .fill_color(FruggerColour::Green)
            .build();

        let mut x = 0;

        loop {
            x = (x + 1) % 320;
            let rec2 = Rectangle::new(Point::new(x, 0), Size::new(10, 10))
                .into_styled(f_style);

            rec2.draw(&mut frugger);

            frugger.draw_frame(&mut display);
            window.update(&display);
        }
    }
}