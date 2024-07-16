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
    Enable3G = 0xF2,
    GammaSet = 0x26,
    PosGammaCorrection = 0xE0,
    NegGammaCorrection = 0xE1,
    WriteBrightness = 0x51,
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

    pub fn init_tft_espi(&mut self) {
        self.rst.set_low().check();
        self.timer.delay_ms(1);
        self.rst.set_high().check();
        self.timer.delay_ms(5);


        // following https://github.com/Bodmer/TFT_eSPI/blob/master/TFT_Drivers/ILI9341_Init.h#L127

        self.cmd(PowerControlB, &[0x00, 0xC1, 0x30]);

        self.cmd(PowerOnSeqControl, &[0x64, 0x03, 0x12, 0x81]);

        self.cmd(DriverTimingControlA, &[0x85, 0x00, 0x78]);

        self.cmd(PowerControlA, &[0x39, 0x2C, 0x00, 0x34, 0x02]);

        self.cmd(PumpRatioControl, &[0x20]);

        self.cmd(DriverTimingControlB, &[0x00, 0x00]);

        // This messes with it.
        // self.cmd(PowerControl1, &[0x10]);

        self.cmd(PowerControl2, &[0x00]);

        self.cmd(VCOMC1, &[0x30, 0x30]);

        self.cmd(VCOMC2, &[0xB7]);

        self.cmd(PixelFormatSet, &[0x55]);

        self.cmd(MemoryAccessControl, &[0x08]);

        self.cmd(FrameRateControl, &[0x00, 0x1A]);

        self.cmd(DisplayFunctionControl, &[0x08, 0x82, 0x27]);

        // gamma stuff from line 188...
        self.cmd_raw(0xF2, &[0x00]);
        self.cmd_raw(0x26, &[0x01]);
        self.cmd_raw(0xE0, &[0x0F, 0x2A, 0x28, 0x08, 0x0E, 0x08, 0x54, 0xA9, 0x43, 0x0A, 0x0F, 0x00, 0x00, 0x00, 0x00]);
        self.cmd_raw(0xE1, &[0x00, 0x15, 0x17, 0x07, 0x11, 0x06, 0x2B, 0x56, 0x3C, 0x05, 0x10, 0x0F, 0x3F, 0x3F, 0x0F]);

        // 0 - 319
        self.cmd(PageAddressSet, &[0x00, 0x00, 0x01, 0x3F]);

        // 0 - 239
        self.cmd(ColumnAddressSet, &[0x00, 0x00, 0x00, 0xEF]);

        self.cmd_only(SleepOut);
        self.sleep(120);
        self.cmd_only(DisplayOn);

        self.sleep(120);
    }

    pub fn init_fbcp(&mut self) {
        // following https://github.com/juj/fbcp-ili9341/blob/master/ili9341.cpp

        self.rst.set_low().check();
        self.sleep(120);
        self.rst.set_high().check();
        self.sleep(120);

        self.cmd_only(SoftwareReset);
        self.sleep(5);
        self.cmd_only(DisplayOff);
        self.cmd(PowerControlA, &[0x39, 0x2C, 0x00, 0x34, 0x02]);
        self.cmd(PowerControlB, &[0x00, 0xC1, 0x30]);
        self.cmd(DriverTimingControlA, &[0x85, 0x00, 0x78]);
        self.cmd(DriverTimingControlB, &[0x00, 0x00]);
        self.cmd(PowerOnSeqControl, &[0x64, 0x03, 0x12, 0x81]);
        self.cmd(PumpRatioControl, &[0x20]);

        self.cmd(PowerControl1, &[0x23]);
        self.cmd(PowerControl2, &[0x10]);
        self.cmd(VCOMC1, &[0x3e, 0x28]);
        self.cmd(VCOMC2, &[0x86]);

        self.cmd(FrameRateControl, &[0x00, 0x10]);
        self.cmd(DisplayFunctionControl, &[0x08, 0x82, 0x27]);
        self.cmd(Enable3G, &[0x02]);
        self.cmd(GammaSet, &[0x01]);
        self.cmd(PosGammaCorrection, &[0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00]);
        self.cmd(NegGammaCorrection, &[0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F]);

        self.cmd_only(SleepOut);
        self.sleep(120)
    }

    pub fn init2(&mut self) {
        // 85ms full screen draw
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
        self.spi.send_data(DataFormat::U8(args)).check();
    }

    fn cmd_raw(&mut self, cmd: u8, args: &[u8]) {
        self.spi.send_commands(DataFormat::U8(&[cmd])).check();
        self.spi.send_data(DataFormat::U8(args)).check();
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
                // log!("no error");
                Some(val)
            }
            Err(err) => {
                log!("Error: {:?}", err);
                None
            }
        }
    }
}