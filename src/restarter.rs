use waveshare_rp2040_zero::pac::interrupt;

use core::cell::RefCell;
use core::fmt;
use core::ops::DerefMut;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use fugit::MicrosDurationU32;
use usb_device::bus::UsbBusAllocator;
use usb_device::device::UsbDevice;
use usb_device::prelude::*;
use usbd_picotool_reset::PicoToolReset;
use usbd_serial::embedded_io::Write;
use usbd_serial::SerialPort;
use waveshare_rp2040_zero::hal::timer::{Alarm, Alarm0};
use waveshare_rp2040_zero::hal::usb::UsbBus;
use waveshare_rp2040_zero::hal::Timer;
use waveshare_rp2040_zero::pac::Interrupt::TIMER_IRQ_0;

static mut USB_SERIAL: Mutex<RefCell<Option<SerialPort<UsbBus>>>> = Mutex::new(RefCell::new(None));

static ALARM_0: Mutex<RefCell<Option<Alarm0>>> = Mutex::new(RefCell::new(None));

static HANDLER: Mutex<RefCell<Option<PicotoolHandler>>> = Mutex::new(RefCell::new(None));

struct PicotoolHandler {
    picotool: PicoToolReset<'static, UsbBus>,
    usb_device: UsbDevice<'static, UsbBus>,
    usb_serial: SerialPort<'static, UsbBus>,
}

pub(crate) fn register(mut timer: Timer, bus_ref: &'static UsbBusAllocator<UsbBus>) {
    let mut picotool: PicoToolReset<_> = PicoToolReset::new(&bus_ref);
    let mut usb_serial = SerialPort::new(&bus_ref);

    let usb_device = UsbDeviceBuilder::new(&bus_ref, UsbVidPid(0x2E8A, 0x000A))
        .strings(&[StringDescriptors::default()
            .manufacturer("RP2040")
            .product("Picotool port")
            .serial_number("TEST")])
        .unwrap()
        .device_class(0x00) // make this 0x02 for the serial stuff to work
        // .composite_with_iads()  -- this doesn't do anything?
        .build();

    let mut alarm_0 = timer.alarm_0().unwrap();
    alarm_0.schedule(MicrosDurationU32::millis(8)).unwrap();
    alarm_0.enable_interrupt();

    cortex_m::interrupt::free(|cs| {
        HANDLER.borrow(cs).replace(Some(PicotoolHandler {
            picotool,
            usb_device,
            usb_serial,
        }));
    });

    cortex_m::interrupt::free(|cs| {
        ALARM_0.borrow(cs).replace(Some(alarm_0));
    });

    unsafe {
        // Enable the interrupt.
        NVIC::unmask(TIMER_IRQ_0);
    }
}

pub fn log_fmt(fmt: fmt::Arguments<'_>) {
    unsafe {
        cortex_m::interrupt::free(|cs| {
            if let Some(handler) = HANDLER.borrow(cs).borrow_mut().deref_mut() {
                let _ = handler.usb_serial.write_fmt(fmt);
                let _ = handler.usb_serial.write("\r\n".as_bytes());
            }
        });
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn TIMER_IRQ_0() {
    cortex_m::interrupt::free(|cs| {
        if let Some(handler) = HANDLER.borrow(cs).borrow_mut().deref_mut() {
            // add serial to the list for it to poll
            handler.usb_device.poll(&mut [&mut handler.picotool]);
        }

        if let Some(alarm) = ALARM_0.borrow(&cs).borrow_mut().deref_mut() {
            alarm.schedule(MicrosDurationU32::millis(8)).unwrap();
            alarm.clear_interrupt();
        }
    });
}
