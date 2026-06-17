//! A simple Driver for the Waveshare 1.54" (C) E-Ink Display via SPI

use embedded_hal::{delay::*, digital::*, spi::SpiDevice};

#[cfg(feature = "graphics")]
use crate::color::QuadColor;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};

/// Width of epd1in54 in pixels
pub const WIDTH: u32 = 200;
/// Height of epd1in54 in pixels
pub const HEIGHT: u32 = 200;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: QuadColor = QuadColor::White;
const IS_BUSY_LOW: bool = false;
const NUM_DISPLAY_BITS: u32 = WIDTH / 4 * HEIGHT;
const SINGLE_BYTE_WRITE: bool = false;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 1in54g EPD
#[cfg(feature = "graphics")]
pub type Display1in54g = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) * 2 },
    QuadColor,
>;

/// Epd1in54g driver
pub struct Epd1in54g<SPI, BUSY, DC, RST, DELAY> {
    interface: DisplayInterface<SPI, BUSY, DC, RST, DELAY, SINGLE_BYTE_WRITE>,
    color: QuadColor,
}

impl<SPI, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, BUSY, DC, RST, DELAY>
    for Epd1in54g<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Based on Reference Program Code from:
        // https://files.waveshare.com/wiki/1.54inch%20e-Paper%20Module%20(G)/1.54inch_e-Paper_(G).pdf
        // and:
        // https://github.com/waveshareteam/e-Paper/blob/86aa9932f471a50157cf02fafadc2c1b4a965449/E-paper_Separate_Program/1in54_e-Paper_G/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd1in54g.py
        self.interface.reset(delay, 10_000, 2_000);

        // Ref sequence
        // 0x4d (unknown)
        // 0x00 (panel setting)
        // 0x06 (booster soft start)
        // 0x50 (vcom and data interval)
        // 0x61 (resolution)
        // 0xe9 (unknown)
        // 0x30 (pll)
        // 0x04 (power on)

        // Unknown command (4D)
        self.cmd_with_data(spi, Command::UnknownInit1, &[0x78])?;

        // set the panel settings
        self.cmd_with_data(spi, Command::PanelSetting, &[0x0f, 0x29])?;

        // start the booster
        self.cmd_with_data(
            spi,
            Command::BoosterSoftStart,
            &[0x0D, 0x12, 0x30, 0x20, 0x19, 0x2A, 0x22],
        )?;

        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x37])?;

        // set resolution
        self.send_resolution(spi)?;

        // Unknown 0xe9
        self.cmd_with_data(spi, Command::UnkonwnInit2, &[0x01])?;
        self.cmd_with_data(spi, Command::PllControl, &[0x08])?;

        // Only in the python code, but not in C?
        // (These appear to be the fast LUT commands. Should probably note them for that impl)
        // self.cmd_with_data(spi, Command::UnknownInit3, &[0x02])?;
        // self.cmd_with_data(spi, Command::UnknownInit4, &[0x5D])?;
        // self.cmd_with_data(spi, Command::UnknownInit5, &[0x00])?;

        // power on
        self.command(spi, Command::PowerOn)?;
        delay.delay_us(5000);
        self.wait_until_idle(spi, delay)?;

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, BUSY, DC, RST, DELAY>
    for Epd1in54g<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    type DisplayColor = QuadColor;
    fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd1in54g { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;

        self.cmd_with_data(spi, Command::PowerOff, &[0x00])?;
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0xa5])?;

        Ok(())
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn set_background_color(&mut self, color: QuadColor) {
        self.color = color;
    }

    fn background_color(&self) -> &QuadColor {
        &self.color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::DataStartTransmission1, buffer)?;

        Ok(())
    }

    #[allow(unused)]
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        // TODO: this exists in the documented command set, so we should implement it.
        unimplemented!()
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::DisplayRefresh, &[0x00])?;
        self.wait_until_idle(spi, delay)?;

        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)?;

        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        let color = QuadColor::colors_byte(DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_COLOR);

        // Clear the black
        self.command(spi, Command::DataStartTransmission1)?;
        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

        Ok(())
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _delay: &mut DELAY,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        Ok(())
    }

    fn wait_until_idle(&mut self, _spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.wait_until_idle(delay, IS_BUSY_LOW);
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> Epd1in54g<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn send_data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.data(spi, data)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::ResolutionSetting)?;

        // // | D7 | D6 | D5 | D4 | D3 | D2 |      D1 |      D0 |
        // // |  - |  - |  - |  - |  - |  - | HRES[9] | HRES[8] |
        // self.send_data(spi, &[((w >> 8) as u8) & 0b0000_0011])?;
        // // | D7 | D6 | D5 | D4 | D3 | D2 | D1 | D0 |
        // // |         HRES[7:2]           |  0 |  0 |
        // self.send_data(spi, &[(w as u8) & 0b1111_1100])?;
        // // | D7 | D6 | D5 | D4 | D3 | D2 |      D1 |      D0 |
        // // |  - |  - |  - |  - |  - |  - | VRES[9] | VRES[8] |
        // self.send_data(spi, &[((h >> 8) as u8) & 0b0000_0011])?;
        // // | D7 | D6 | D5 | D4 | D3 | D2 | D1 |      D0 |
        // // |                  VRES[7:0]                 |
        // self.send_data(spi, &[(h as u8)])?;
        // Send it as the original did, to avoid potential differences
        self.send_data(spi, &[
            (w / 256) as u8,
            (w % 256) as u8,
            (h / 256) as u8,
            (h % 256) as u8
        ])?;

        Ok(())
    }
}
