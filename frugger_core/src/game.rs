#[cfg(test)]
mod tests {
    use embedded_graphics::geometry::{Point, Size};
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
    use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window};

    use crate::{Frugger, FruggerColour, Palette};

    enum SweetiePalette {
        Black,
        Purple,
        Red,
        Orange,
        Yellow,
        Lime,
        Green,
        Teal,
        NavyBlue,
        DarkBlue,
        Blue,
        LightBlue,
        White,
        LightGrey,
        DarkGrey,
        BlueGrey
    }

    impl Palette for SweetiePalette {
        fn colours() -> [Rgb565; 16] {
            [
                Rgb565::new(0x1a,0x1c,0x2c),
                Rgb565::new(0x5d,0x27,0x5d),
                Rgb565::new(0xb1,0x3e,0x53),
                Rgb565::new(0xef,0x7d,0x57),
                Rgb565::new(0xff,0xcd,0x75),
                Rgb565::new(0xa7,0xf0,0x70),
                Rgb565::new(0x38,0xb7,0x64),
                Rgb565::new(0x25,0x71,0x79),
                Rgb565::new(0x29,0x36,0x6f),
                Rgb565::new(0x3b,0x5d,0xc9),
                Rgb565::new(0x41,0xa6,0xf6),
                Rgb565::new(0x73,0xef,0xf7),
                Rgb565::new(0xf4,0xf4,0xf4),
                Rgb565::new(0x94,0xb0,0xc2),
                Rgb565::new(0x56,0x6c,0x86),
                Rgb565::new(0x33,0x3c,0x57)
            ]
        }

        fn index(&self) -> u8 {
            match self {
                SweetiePalette::Black => 0,
                SweetiePalette::Purple => 1,
                SweetiePalette::Red => 2,
                SweetiePalette::Orange => 3,
                SweetiePalette::Yellow => 4,
                SweetiePalette::Lime => 5,
                SweetiePalette::Green => 6,
                SweetiePalette::Teal => 7,
                SweetiePalette::NavyBlue => 8,
                SweetiePalette::DarkBlue => 9,
                SweetiePalette::Blue => 10,
                SweetiePalette::LightBlue => 11,
                SweetiePalette::White => 12,
                SweetiePalette::LightGrey => 13,
                SweetiePalette::DarkGrey => 14,
                SweetiePalette::BlueGrey => 15,
            }
        }
    }

    #[test]
    fn test() {
        let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .build();

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
            window.update(&frugger);
        }
    }
}