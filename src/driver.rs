use core::fmt::Debug;

use display_interface::{DataFormat, WriteOnlyDataCommand};
use display_interface::DataFormat::U16BEIter;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

use crate::driver::SpiCommand::*;

#[repr(u8)]
enum SpiCommand {
    SoftwareReset = 0x01,
    SleepOut = 0x11,
    DisplayOn = 0x29,
    DisplayOff = 0x28,
    ColumnAddressSet = 0x2A,
    PageAddressSet = 0x2B,
    MemoryWrite = 0x2C,
    MemoryAccessControl = 0x36,
    PowerControl1 = 0xC0,
    PowerControl2 = 0xC1,
    PowerControlA = 0xCB,
    PowerControlB = 0xCF,
    PowerOnSeqControl = 0xED,
    DriverTimingControlA = 0xE8,
    DriverTimingControlB = 0xEA,
    ColorSet = 0x2D,
    NormalModeOn = 0x13,
    PixelFormatSet = 0x3A,
    PumpRatioControl = 0xF7,
    VCOMC1 = 0xC5,
    VCOMC2 = 0xC7,
    FrameRateControl = 0xB1,
    DisplayFunctionControl = 0xB6,
}

#[repr(u8)]
pub enum Orientation {
    Portrait = 0x40 | 0x08,
    PortraitFlipped = 0x80 | 0x08,
    Landscape = 0x20 | 0x08,
    LandscapeFlipped = 0x40 | 0x80 | 0x20 | 0x08,
}

impl Into<u8> for SpiCommand {
    fn into(self) -> u8 {
        self as u8
    }
}

pub struct Driver<SPI, RESET, DELAY> {
    spi: SPI,
    rst: RESET,
    timer: DELAY,
}

const PURPLE: u16 = 0x780F;

impl<SPI, RESET, DELAY> Driver<SPI, RESET, DELAY> where SPI: WriteOnlyDataCommand, RESET: OutputPin, DELAY: DelayNs {
    pub fn new(spi: SPI, rst: RESET, timer: DELAY) -> Self {
        Self { spi, rst, timer }
    }

    // pub fn init(&mut self) {
    //     self.rst.set_high().check();
    //     self.timer.delay_ms(120);
    //     self.rst.set_low().check();
    //     self.timer.delay_ms(120);
    //     self.rst.set_high().check();
    //     self.timer.delay_ms(120);
    //
    //     // following https://github.com/Bodmer/TFT_eSPI/blob/master/TFT_Drivers/ILI9341_Init.h#L127
    //
    //     self.cmd(PowerControlB);
    //     self.spi.send_data(DataFormat::U8(&[0x00, 0xC1, 0x30])).check();
    //
    //     self.cmd(PowerOnSeqControl);
    //     self.spi.send_data(DataFormat::U8(&[0x64, 0x03, 0x12, 0x81])).check();
    //
    //     self.cmd(DriverTimingControlA);
    //     self.spi.send_data(DataFormat::U8(&[0x85, 0x00, 0x78])).check();
    //
    //     self.cmd(PowerControlA);
    //     self.spi.send_data(DataFormat::U8(&[0x39, 0x2C, 0x00, 0x32, 0x02])).check();
    //
    //     self.cmd(PumpRatioControl);
    //     self.spi.send_data(DataFormat::U8(&[0x20])).check();
    //
    //     self.cmd(DriverTimingControlB);
    //     self.spi.send_data(DataFormat::U8(&[0x00, 0x00])).check();
    //
    //     self.cmd(PowerControl1);
    //     self.spi.send_data(DataFormat::U8(&[0x10])).check();
    //
    //     self.cmd(PowerControl2);
    //     self.spi.send_data(DataFormat::U8(&[0x00])).check();
    //
    //     self.cmd(VCOMC1);
    //     self.spi.send_data(DataFormat::U8(&[0x30, 0x30])).check();
    //
    //     self.cmd(VCOMC2);
    //     self.spi.send_data(DataFormat::U8(&[0xB7])).check();
    //
    //     self.cmd(PixelFormatSet);
    //     self.spi.send_data(DataFormat::U8(&[0x55])).check();
    //
    //     self.cmd(MemoryAccessControl);
    //     self.spi.send_data(DataFormat::U8(&[0x08])).check(); // rotation 0, portrait mode
    //
    //     self.cmd(FrameRateControl);
    //     self.spi.send_data(DataFormat::U8(&[0x00, 0x1A])).check();
    //
    //     self.cmd(DisplayFunctionControl);
    //     self.spi.send_data(DataFormat::U8(&[0x08, 0x82, 0x27])).check();
    //
    //     // gamma stuff from line 188...
    //
    //     self.cmd(PageAddressSet);
    //     // 0 - 319
    //     self.spi.send_data(DataFormat::U8(&[0x00, 0x00, 0x01, 0x3F])).check();
    //
    //     self.cmd(ColumnAddressSet);
    //     // 0 - 239
    //     self.spi.send_data(DataFormat::U8(&[0x00, 0x00, 0x00, 0xEF])).check();
    //
    //     self.cmd(SleepOut);
    //     self.sleep(120);
    //     self.cmd(DisplayOn);
    //
    //     self.sleep(120);
    //
    //     let data = [0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F];
    //
    //     self.cmd(MemoryWrite);
    //     self.spi.send_data(DataFormat::U8(&data)).check();
    // }

    pub fn init2(&mut self) {
        self.rst.set_low().check();
        self.timer.delay_ms(1);
        self.rst.set_high().check();
        self.timer.delay_ms(5);

        self.cmd_only(SoftwareReset);
        self.sleep(120);

        self.cmd(MemoryAccessControl, &[0x20 | 0x08]);

        self.cmd(PixelFormatSet, &[0x55]);

        self.cmd_only(SleepOut);
        self.sleep(5);

        self.cmd_only(DisplayOn);
        //
        // self.cmd(PageAddressSet);
        // // 0 - 319
        // self.spi.send_data(DataFormat::U8(&[0x00, 0x00, 0x00, 0x0F])).check();
        //
        // self.cmd(ColumnAddressSet);
        // // 0 - 239
        // self.spi.send_data(DataFormat::U8(&[0x00, 0x00, 0x00, 0x01])).check();
        //
        // let data = [0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F,
        //     0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F,
        //     0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F, 0x78, 0x0F,
        //     0x78, 0x0F];
        //
        // self.cmd(MemoryWrite);
        // self.spi.send_data(DataFormat::U8(&data)).check();
    }

    fn cmd_only(&mut self, cmd: SpiCommand) {
        self.spi.send_commands(DataFormat::U8(&[cmd as u8])).check();
    }

    fn cmd(&mut self, cmd: SpiCommand, args: &[u8]) {
        self.cmd_only(cmd);
        self.spi.send_data(DataFormat::U8(args));
    }

    fn sleep(&mut self, ms: u32) {
        self.timer.delay_ms(ms);
    }

    pub fn orient(&mut self, orientation: Orientation) {
        self.cmd(MemoryAccessControl, &[orientation as u8])
    }

    pub fn draw_raw_iter<I: IntoIterator<Item=u16>>(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        data: I,
    ) {
        self.set_window(x0, y0, x1, y1);
        self.write_iter(data)
    }

    fn write_iter<I: IntoIterator<Item=u16>>(&mut self, data: I) {
        self.cmd(MemoryWrite, &[]);
        self.spi.send_data(U16BEIter(&mut data.into_iter()));
    }

    // fn write_slice(&mut self, data: &[u16]) {
    //     self.command(crate::ili::Command::MemoryWrite, &[])?;
    //     self.interface.send_data(DataFormat::U16(data))
    // }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) {
        self.cmd(ColumnAddressSet, &[
            (x0 >> 8) as u8,
            (x0 & 0xff) as u8,
            (x1 >> 8) as u8,
            (x1 & 0xff) as u8,
        ]);

        self.cmd(PageAddressSet, &[
            (y0 >> 8) as u8,
            (y0 & 0xff) as u8,
            (y1 >> 8) as u8,
            (y1 & 0xff) as u8,
        ]);
    }
}


trait LogError<T, E> {
    fn check(self) -> Option<T>;
}

impl<T, E> LogError<T, E> for Result<T, E> where E: Debug {
    fn check(self) -> Option<T> {
        match self {
            Ok(val) => {
                log!("no error");
                Some(val)
            }
            Err(err) => {
                log!("Error: {:?}", err);
                None
            }
        }
    }
}