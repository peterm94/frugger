use std::process::exit;
use std::thread::sleep;
use std::time::{Duration, Instant};

use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use frugger_core::{ButtonInput, ButtonState, FrugInputs, FruggerEngine, FruggerGame, Orientation};

use sdl2::keyboard::Keycode;

struct WindowRef(Window);

pub struct FruggerSim<T, C>
where
    T: FruggerGame,
    C: PixelColor,
{
    display: SimulatorDisplay<C>,
    window: WindowRef,
    game: Box<T>,
    inputs: FrugInputs,
    tick_speed: u128,
}

impl WindowRef {
    fn set_button_state(event: SimulatorEvent, button: &mut ButtonState) {
        match event {
            SimulatorEvent::KeyUp { .. } => match button {
                ButtonState::DOWN | ButtonState::PRESSED => *button = ButtonState::RELEASED,
                _ => {}
            },
            SimulatorEvent::KeyDown { .. } => match button {
                ButtonState::RELEASED | ButtonState::UP => *button = ButtonState::PRESSED,
                _ => {}
            },
            _ => {}
        }
    }
}

impl<T> FruggerSim<T, Rgb565>
where
    T: FruggerGame<Color = Rgb565>,
{
    pub fn new_rgb(game: Box<T>) -> Self {
        let display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .build();

        let mut window = Window::new("FruggerSim", &output_settings);
        window.update(&display);

        Self {
            display,
            window: WindowRef(window),
            game,
            tick_speed: 1000 / T::TARGET_FPS as u128,
            inputs: FrugInputs::default(),
        }
    }
}

impl<T, C> FruggerSim<T, C>
where
    T: FruggerGame<Color = C>,
    C: PixelColor + From<Rgb888>,
    Rgb888: From<C>,
{
    pub fn run(&mut self) {
        loop {
            let start = Instant::now();

            // Update inputs
            self.window.tick(&mut self.inputs);

            // Update game
            self.game.update(&self.inputs);

            // Update display
            self.game.frugger().draw_frame(&mut self.display);

            self.window.0.update(&self.display);

            let elapsed = Instant::now().duration_since(start).as_millis();
            if elapsed < self.tick_speed {
                sleep(Duration::from_millis((self.tick_speed - elapsed) as u64))
            }
        }
    }
}

impl<T> FruggerSim<T, BinaryColor>
where
    T: FruggerGame<Color = BinaryColor>,
{
    pub fn new_binary(game: Box<T>) -> Self {
        let display = match T::ORIENTATION {
            Orientation::Landscape => SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64)),
            Orientation::Portrait => SimulatorDisplay::<BinaryColor>::new(Size::new(64, 128)),
        };

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .build();

        let mut window = Window::new("FruggerSim", &output_settings);
        window.update(&display);

        Self {
            display,
            window: WindowRef(window),
            game,
            tick_speed: 1000 / T::TARGET_FPS as u128,
            inputs: FrugInputs::default(),
        }
    }
}

impl ButtonInput for WindowRef {
    fn tick(&mut self, inputs: &mut FrugInputs) {
        for button in [
            &mut inputs.a,
            &mut inputs.b,
            &mut inputs.up,
            &mut inputs.down,
            &mut inputs.left,
            &mut inputs.right,
        ] {
            match button {
                ButtonState::PRESSED => *button = ButtonState::DOWN,
                ButtonState::RELEASED => *button = ButtonState::UP,
                ButtonState::DOWN => {}
                ButtonState::UP => {}
            }
        }

        for event in self.0.events() {
            match event {
                SimulatorEvent::KeyUp { keycode, .. } | SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::A => Self::set_button_state(event, &mut inputs.left),
                        Keycode::D => Self::set_button_state(event, &mut inputs.right),
                        Keycode::W => Self::set_button_state(event, &mut inputs.up),
                        Keycode::S => Self::set_button_state(event, &mut inputs.down),
                        Keycode::J => Self::set_button_state(event, &mut inputs.a),
                        Keycode::K => Self::set_button_state(event, &mut inputs.b),
                        _ => {}
                    }
                }
                SimulatorEvent::Quit => exit(0),
                _ => {}
            }
        }
    }
}
