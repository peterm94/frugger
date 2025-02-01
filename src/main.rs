#![no_std]
#![no_main]

use bsp::entry;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use bsp::hal::spi::{SpiDevice, State, ValidSpiPinout};
use bsp::hal::usb::UsbBus;
use bsp::hal::Timer;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    watchdog::Watchdog,
};
use bsp::XOSC_CRYSTAL_FREQ;
use defmt::*;
#[allow(unused_imports)]
use defmt::*;
#[allow(unused_imports)]
use defmt_rtt as _f;
use fugit::RateExtU32;
use numtoa::NumToA;
#[allow(unused_imports)]
use panic_probe as _;
use ssd1306::mode::DisplayConfig;
use usb_device::class_prelude::*;
use usbd_serial::embedded_io::Write;
use waveshare_rp2040_zero as bsp;

use frugger_core::{ButtonInput, FruggerEngine, FruggerGame};

mod mc_inputs;

static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS2: Option<UsbBusAllocator<UsbBus>> = None;

macro_rules! log {
    ($($tts:tt)*) => {
        // crate::restarter::log_fmt(format_args!($($tts)*))
    };
}
mod driver;
mod mini_gb;
mod restarter;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

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

    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    unsafe {
        USB_BUS = Some(UsbBusAllocator::new(UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        )));
    }

    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    restarter::register(timer, &bus_ref);

    mini_gb::start(&clocks.system_clock, timer.clone());
}