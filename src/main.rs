#![no_std]
#![no_main]

use core::cell::RefCell;
use core::fmt;
use brickbreaker::BrickBreaker;

use bsp::entry;
use bsp::hal::{
    clocks::{Clock, init_clocks_and_plls},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use cortex_m::asm::delay;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use defmt::*;
#[allow(unused_imports)]
use defmt::*;
#[allow(unused_imports)]
use defmt_rtt as _f;
use display_interface_spi::SPIInterface;
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, StyledDrawable};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{MODE_0, SpiBus};
use embedded_hal_bus::spi::ExclusiveDevice;
use fire::Fire;
use fugit::{MicrosDurationU32, RateExtU32};
use input_test_small::InputTestSmall;
use mipidsi::models;
use mipidsi::options::Rotation;
use numtoa::NumToA;
#[allow(unused_imports)]
use panic_probe as _;
use runner::Runner;
use sh1106::Builder;
// use sh1106::Builder;
use sh1106::mode::GraphicsMode;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usbd_serial::embedded_io::Write;
use usbd_picotool_reset::PicoToolReset;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use waveshare_rp2040_zero as bsp;
use waveshare_rp2040_zero::hal::spi::{SpiDevice, State, ValidSpiPinout};
use waveshare_rp2040_zero::hal::Timer;
use waveshare_rp2040_zero::hal::timer::{Alarm, Alarm0, Instant};
use waveshare_rp2040_zero::hal::usb::UsbBus;
use waveshare_rp2040_zero::pac::interrupt;
use waveshare_rp2040_zero::pac::Interrupt::TIMER_IRQ_0;
use waveshare_rp2040_zero::{hal, XOSC_CRYSTAL_FREQ};
use waveshare_rp2040_zero::hal::rom_data::reset_to_usb_boot;
use worm::Worm;

use frugger_core::{ButtonInput, FruggerEngine, FruggerGame, FrugInputs};

use crate::mc_inputs::McInputs;

mod mc_inputs;

static mut USB_DEVICE: Option<UsbDevice<UsbBus>> = None;
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_SERIAL: Mutex<RefCell<Option<SerialPort<UsbBus>>>> = Mutex::new(RefCell::new(None));
static mut PICOTOOL: Mutex<RefCell<Option<PicoToolReset<UsbBus>>>> = Mutex::new(RefCell::new(None));

static mut ALARM_0: Option<Mutex<RefCell<Alarm0>>> = None;

macro_rules! log {
    ($($tts:tt)*) => {
        // crate::log_fmt(format_args!($($tts)*))
    }
}
mod driver;


#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);


    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    // let mut alarm_0 = timer.alarm_0().unwrap();
    // alarm_0.schedule(MicrosDurationU32::millis(8)).unwrap();
    // alarm_0.enable_interrupt();
    //
    // unsafe { ALARM_0 = Some(Mutex::new(RefCell::new(alarm_0))); }

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    //
    // unsafe {
    //     // Enable the interrupt.
    //     NVIC::unmask(TIMER_IRQ_0);
    // }




    // let usb_bus = UsbBusAllocator::new(UsbBus::new(pac.USBCTRL_REGS,
    //                                                pac.USBCTRL_DPRAM, clocks.usb_clock, true, &mut pac.RESETS));
    // unsafe {
    //     USB_BUS = Some(usb_bus);
    // }
    // let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    // let mut serial = SerialPort::new(&usb_bus);
    // let mut picotool: PicoToolReset<_> = PicoToolReset::new(&usb_bus);

    // unsafe {
    //     PICOTOOL = Mutex::new(RefCell::new(Some(picotool)));
    //     // USB_SERIAL = Mutex::new(RefCell::new(Some(serial)));
    // }

    // // TODO only one of the usb devices can be used at once, the serial one needs 0x02, they aren't compatible from what I can see.
    // //  Make an interrupt for each that can be used.
    // let mut usb_device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x2E8A, 0x000A))
    //     .strings(&[StringDescriptors::default()
    //         .manufacturer("RP2040")
    //         .product("Picotool port")
    //         .serial_number("TEST")]).unwrap()
    //     .device_class(0x00)
    //     .build();
    //
    // loop {
    //     usb_device.poll(&mut [&mut picotool]);
    //     delay.delay_ms(8);
    //     log!("aa");
    // }
    // unsafe {
    //     USB_DEVICE = Some(usb_dev);
    // }

    // let mut led_pin = pins.gp5.into_push_pull_output();

    let left_pin = pins.gp15.into_pull_up_input();
    let left = left_pin.as_input();
    let right_pin = pins.gp14.into_pull_up_input();
    let right = right_pin.as_input();

    // let up_pin = pins.gp27.into_pull_up_input();
    let up_pin = pins.gp3.into_pull_up_input();
    let up = up_pin.as_input();
    let down_pin = pins.gp26.into_pull_up_input();
    let down = down_pin.as_input();

    // let a_pin = pins.gp7.into_pull_up_input();
    let a_pin = pins.gp2.into_pull_up_input();
    let a = a_pin.as_input();
    // let b_pin = pins.gp8.into_pull_up_input();
    let b_pin = pins.gp4.into_pull_up_input();
    let b = b_pin.as_input();
    //
    // // turn on the backlight
    // led_pin.set_high().unwrap();
    //
    // let mut rst = pins.gp6.into_push_pull_output();
    // rst.set_high().unwrap();

    // let dc = pins.gp4.into_push_pull_output();
    //
    // let rx = pins.gp0.into_function::<hal::gpio::FunctionSpi>();
    // let tx = pins.gp3.into_function::<hal::gpio::FunctionSpi>();
    // let cs = pins.gp1.into_push_pull_output();
    // let sck = pins.gp2.into_function::<hal::gpio::FunctionSpi>();
    // timer.delay_ms(1000);
    //
    // // let dummy_cs = dummy_pin::DummyPin::new_low();
    // let spi: bsp::hal::spi::Spi::<_, _, _, 8> = bsp::hal::spi::Spi::new(pac.SPI0, (tx, rx, sck));
    // let mut spi = spi.init(&mut pac.RESETS, clocks.peripheral_clock.freq(), 500.MHz(), MODE_0);
    // let baud = spi.set_baudrate(clocks.peripheral_clock.freq(), 63.MHz());
    // log!("Actual baudrate is {baud}");
    // let mut spi_timer = timer.clone();
    //
    // let spi_bus = ExclusiveDevice::new(spi, cs, &mut spi_timer).unwrap();

    let mut display_timer = timer.clone();

    // let di = SPIInterface::new(spi_bus, dc);
    // let mut display = mipidsi::Builder::new(models::ILI9341Rgb565, di)
    //     .display_size(240, 320)
    //     .orientation(mipidsi::options::Orientation { rotation: Rotation::Deg90, mirrored: false })
    //     .reset_pin(rst)
    //     .init(&mut display_timer).unwrap();
    // display.clear(Rgb565::CSS_ROYAL_BLUE).unwrap();

    let sda_pin = pins.gp0.reconfigure();
    let scl_pin = pins.gp1.reconfigure();

    // let mut game = BrickBreaker::new();
    // let mut game = Worm::new(timer.get_counter().ticks());
    // let mut game = InputTest::new();
    // let mut game = Fire::new();
    let mut game = Runner::new(timer.get_counter().ticks());
    let mut game = InputTestSmall::new();
    let target_fps = 1000 / InputTestSmall::TARGET_FPS;

    let mut mc_inputs = McInputs::new(a, b, up, down, left, right);
    let mut frug_inputs = FrugInputs::default();

    let mut buf = [0u8; 20];

    let mut logic_avg = RollingAverage::new();

    let i2c = bsp::hal::i2c::I2C::i2c0(pac.I2C0, sda_pin, scl_pin, 1.MHz(), &mut pac.RESETS, &clocks.system_clock);
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    display.init().unwrap();
    display.flush().unwrap();


    // // For the little 2 colour one
    // let interface = I2CDisplayInterface::new(i2c);
    // let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    // display.init().unwrap();

    log!("test");

    loop {
        let frame_start = timer.get_counter();

        mc_inputs.tick(&mut frug_inputs);

        game.update(&frug_inputs);

        let logic_end = timer.get_counter();
        let logic_time = (logic_end - frame_start).to_millis();

        logic_avg.add(logic_time);

        game.frugger().draw_frame(&mut display);
        display.flush();

        let draw_end = timer.get_counter();
        let draw_time = (draw_end - logic_end).to_millis();
        let total_time = (draw_end - frame_start).to_millis();
        log!("{}", draw_time);
        // log!("Logic: {logic_time} Draw: {draw_time}, Total: {total_time} / {target_fps}");

        // let txt_style = MonoTextStyle::new(&FONT_10X20, if total_time < target_fps { Rgb565::WHITE } else { Rgb565::RED });
        // let rect_style = PrimitiveStyle::with_fill(Rgb565::BLACK);
        // let frame_time = total_time.numtoa_str(10, &mut buf);
        // Rectangle::new(Point::new(0, 0), Size::new(30, 20)).draw_styled(&rect_style, &mut display);
        // let text = Text::new(frame_time, Point::new(0, 15), txt_style);
        // text.draw(&mut display);

        if total_time < target_fps {
            timer.delay_ms((target_fps - logic_time) as u32);
        }
    }
}

pub fn log_fmt(fmt: fmt::Arguments<'_>) {
    unsafe {
        cortex_m::interrupt::free(|cs| {
            let serial = USB_SERIAL.borrow(&cs);
            let mut s2 = serial.borrow_mut();
            let s2 = s2.as_mut().unwrap();

            let _ = s2.write_fmt(fmt);
            let _ = s2.write("\r\n".as_bytes());
        });
    }
}

pub struct RollingAverage {
    window: [u64; 10],
    index: usize,
    sum: u64,
}

struct Bencher {
    timer: Timer,
    last: Instant,
}

impl Bencher {
    fn new(timer: Timer) -> Self {
        Self { timer, last: timer.get_counter() }
    }

    fn start(&mut self) {
        self.last = self.timer.get_counter();
    }

    fn cp(&mut self, msg: &str) {
        let end = self.timer.get_counter();
        let time = (end - self.last).to_millis();
        log!("{msg}: {time}ms");
        self.last = self.timer.get_counter();
    }
}

impl RollingAverage {
    pub fn new() -> Self {
        RollingAverage {
            window: [0; 10],
            index: 0,
            sum: 0,
        }
    }

    pub fn add(&mut self, val: u64) {
        self.sum = self.sum - self.window[self.index] + val;
        self.window[self.index] = val;
        self.index = (self.index + 1) % 10;
    }

    pub fn average(&self) -> u64 {
        self.sum / 10
    }
}

// #[allow(non_snake_case)]
// #[interrupt]
// unsafe fn TIMER_IRQ_0() {
//     cortex_m::interrupt::free(|cs| {
//         let usb_dev = USB_DEVICE.as_mut().unwrap();
//         // let serial = USB_SERIAL.borrow(&cs);
//         // let mut s2 = serial.borrow_mut();
//         // let s2 = s2.as_mut().unwrap();
//
//         let picotool = PICOTOOL.borrow(&cs);
//         let mut s2 = picotool.borrow_mut();
//         let s2 = s2.as_mut().unwrap();
//         usb_dev.poll(&mut [s2]);
//
//         let mut alarm = ALARM_0.as_mut().unwrap().borrow(&cs).borrow_mut();
//         alarm.clear_interrupt();
//         alarm.schedule(MicrosDurationU32::millis(8)).unwrap();
//     });
// }