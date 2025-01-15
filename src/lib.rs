#![no_std]
#![doc = include_str!("../README.md")]

/// Preconfigured devices
pub mod devices;

use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

/// A struct to integrate with a new IS31FL3731 powered device.
pub struct IS31FL3731<I2C> {
    /// The i2c bus that is used to interact with the device. See implementation below for the
    /// trait methods required.
    pub i2c: I2C,
    /// The 7-bit i2c slave address of the device. By default on most devices this is `0x74`.
    pub address: u8,
    /// The current frame register in use.
    frame: u8,
}

impl<I2C> IS31FL3731<I2C> {
    /// Creates and sets up a new instance of the IS31FL3731 driver.
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            frame: 0,
        }
    }

    /// Change the slave address to a new 7-bit address. Should be configured before calling
    /// [setup](Self::setup) method.
    pub fn set_address(&mut self, address: u8) {
        self.address = address;
    }
}

impl<I2C, I2cError> IS31FL3731<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    /// Fill the display with a single brightness. The brightness should range from 0 to 255. The reason that blink is an optional is
    /// because you can either set blink to true, set blink to false, or not set blink at all. The
    /// frame is the frame in which the fill should be applied to. Please consult the "General
    /// Description" section on the first page of the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
    /// for more information on frames.
    pub fn fill_blocking(
        &mut self,
        brightness: u8,
        blink: Option<bool>,
        frame: u8,
    ) -> Result<(), I2cError> {
        self.bank_blocking(frame)?;
        let mut payload = [brightness; 25];
        for row in 0..6 {
            payload[0] = addresses::COLOR_OFFSET + row * 24;
            self.i2c.write(self.address, &payload)?;
        }
        if blink.is_some() {
            let data = if blink.unwrap() { 1 } else { 0 } * 0xFF;
            for col in 0..18 {
                self.write_register_blocking(frame, addresses::BLINK_OFFSET + col, data)?;
            }
        }
        Ok(())
    }

    /// Setup the display. Should be called before interacting with the device to ensure proper
    /// functionality. Delay is something that your device's HAL should provide which allows for
    /// the process to sleep for a certain amount of time (in this case 10 MS to perform a reset).
    ///
    /// When you run this function the following steps will occur:
    /// 1. The chip will be told that it's being "reset".
    /// 2. All frames will be cleared.
    /// 3. Audio syncing will be turned off.
    /// 4. The chip will be told that it's being turned back on.
    pub fn setup_blocking(
        &mut self,
        delay: &mut impl DelayNs,
    ) -> Result<(), Error<I2cError>> {
        self.sleep_blocking(true)?;
        delay.delay_ms(10);
        self.mode_blocking(addresses::PICTURE_MODE)?;
        self.frame_blocking(0)?;
        for frame in 0..8 {
            self.fill_blocking(0, Some(false), frame)?;
            for col in 0..18 {
                self.write_register_blocking(frame, addresses::ENABLE_OFFSET + col, 0xFF)?;
            }
        }
        self.audio_sync_blocking(false)?;
        self.sleep_blocking(false)?;
        Ok(())
    }

    /// Set the brightness for a specific LED. Just like the [fill method](Self::fill) the
    /// brightness should range from 0 to 255. If the LED is out of range then the function will
    /// return an error of [InvalidLocation](Error::InvalidLocation).
    pub fn pixel_blocking(&mut self, led: u8, brightness: u8) -> Result<(), Error<I2cError>> {
        if led >= 144 {
            return Err(Error::InvalidLocation(led));
        }
        self.write_register_blocking(self.frame, addresses::COLOR_OFFSET + led, brightness)?;
        Ok(())
    }

    /// Individially assign and updated brightness values for all 144 LEDs at once.
    pub fn all_pixels_blocking(&mut self, buf: &[u8; 144]) -> Result<(), Error<I2cError>> {
        self.bank_blocking(self.frame)?;
        let mut payload = [0; 145];
        payload[0] = addresses::COLOR_OFFSET;
        payload[1..].clone_from_slice(buf);
        self.i2c.write(self.address, &payload)?;
        Ok(())
    }

    /// Set frame ranging from 0 to 8. Please consult the "General Description" section on the
    /// first page of the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
    /// for more information on frames.
    pub fn frame_blocking(&mut self, frame: u8) -> Result<(), Error<I2cError>> {
        if frame > 8 {
            return Err(Error::InvalidLocation(frame));
        }
        self.frame = frame;
        self.write_register_blocking(addresses::CONFIG_BANK, addresses::FRAME, frame)?;
        Ok(())
    }

    /// Send a reset message to the slave device. Delay is something that your device's HAL should
    /// provide which allows for the process to sleep for a certain amount of time (in this case 10
    /// MS to perform a reset).
    pub fn reset_blocking(&mut self, delay: &mut impl DelayNs) -> Result<(), I2cError> {
        self.sleep_blocking(true)?;
        delay.delay_ms(10);
        self.sleep_blocking(false)?;
        Ok(())
    }

    /// Set the device mode. Please consult page 17 and 18 of the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
    /// to learn mode about the different modes.
    pub fn mode_blocking(&mut self, mode: u8) -> Result<(), I2cError> {
        self.write_register_blocking(addresses::CONFIG_BANK, addresses::MODE_REGISTER, mode)?;
        Ok(())
    }

    /// Set the slave device to sync audio
    pub fn audio_sync_blocking(&mut self, yes: bool) -> Result<(), I2cError> {
        self.write_register_blocking(
            addresses::CONFIG_BANK,
            addresses::AUDIOSYNC,
            if yes { 1 } else { 0 },
        )?;
        Ok(())
    }

    /// Set the device to sleep
    pub fn sleep_blocking(&mut self, yes: bool) -> Result<(), I2cError> {
        self.write_register_blocking(
            addresses::CONFIG_BANK,
            addresses::SHUTDOWN,
            if yes { 0 } else { 1 },
        )?;
        Ok(())
    }

    fn write_register_blocking(
        &mut self,
        bank: u8,
        register: u8,
        value: u8,
    ) -> Result<(), I2cError> {
        self.bank_blocking(bank)?;
        self.i2c.write(self.address, &[register, value])?;
        Ok(())
    }

    fn bank_blocking(&mut self, bank: u8) -> Result<(), I2cError> {
        self.i2c
            .write(self.address, &[addresses::BANK_ADDRESS, bank])?;
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<I2C, I2cError> IS31FL3731<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = I2cError>,
{
    /// Fill the display with a single brightness. The brightness should range from 0 to 255. The reason that blink is an optional is
    /// because you can either set blink to true, set blink to false, or not set blink at all. The
    /// frame is the frame in which the fill should be applied to. Please consult the "General
    /// Description" section on the first page of the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
    /// for more information on frames.
    pub async fn fill(
        &mut self,
        brightness: u8,
        blink: Option<bool>,
        frame: u8,
    ) -> Result<(), I2cError> {
        self.bank(frame).await?;
        let mut payload = [brightness; 25];
        for row in 0..6 {
            payload[0] = addresses::COLOR_OFFSET + row * 24;
            self.i2c.write(self.address, &payload).await?;
        }
        if blink.is_some() {
            let data = if blink.unwrap() { 1 } else { 0 } * 0xFF;
            for col in 0..18 {
                self.write_register(frame, addresses::BLINK_OFFSET + col, data)
                    .await?;
            }
        }
        Ok(())
    }

    /// Setup the display. Should be called before interacting with the device to ensure proper
    /// functionality. Delay is something that your device's HAL should provide which allows for
    /// the process to sleep for a certain amount of time (in this case 10 MS to perform a reset).
    ///
    /// When you run this function the following steps will occur:
    /// 1. The chip will be told that it's being "reset".
    /// 2. All frames will be cleared.
    /// 3. Audio syncing will be turned off.
    /// 4. The chip will be told that it's being turned back on.
    pub async fn setup(
        &mut self,
        delay: &mut impl DelayNs,
    ) -> Result<(), Error<I2cError>> {
        self.sleep(true).await?;
        delay.delay_ms(10);
        self.mode(addresses::PICTURE_MODE).await?;
        self.frame(0).await?;
        for frame in 0..8 {
            self.fill(0, Some(false), frame).await?;
            for col in 0..18 {
                self.write_register(frame, addresses::ENABLE_OFFSET + col, 0xFF)
                    .await?;
            }
        }
        self.audio_sync(false).await?;
        self.sleep(false).await?;
        Ok(())
    }

    /// Set the brightness for a specific LED. Just like the [fill method](Self::fill) the
    /// brightness should range from 0 to 255. If the LED is out of range then the function will
    /// return an error of [InvalidLocation](Error::InvalidLocation).
    pub async fn pixel(&mut self, led: u8, brightness: u8) -> Result<(), Error<I2cError>> {
        if led >= 144 {
            return Err(Error::InvalidLocation(led));
        }
        self.write_register(self.frame, addresses::COLOR_OFFSET + led, brightness)
            .await?;
        Ok(())
    }

    /// Individially assign and updated brightness values for all 144 LEDs at once.
    pub async fn all_pixels(&mut self, buf: &[u8; 144]) -> Result<(), Error<I2cError>> {
        self.bank(self.frame).await?;
        let mut payload = [0; 145];
        payload[0] = addresses::COLOR_OFFSET;
        payload[1..].clone_from_slice(buf);
        self.i2c.write(self.address, &payload).await?;
        Ok(())
    }

    /// Set frame ranging from 0 to 8. Please consult the "General Description" section on the
    /// first page of the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
    /// for more information on frames.
    pub async fn frame(&mut self, frame: u8) -> Result<(), Error<I2cError>> {
        if frame > 8 {
            return Err(Error::InvalidLocation(frame));
        }
        self.frame = frame;
        self.write_register(addresses::CONFIG_BANK, addresses::FRAME, frame)
            .await?;
        Ok(())
    }

    /// Send a reset message to the slave device. Delay is something that your device's HAL should
    /// provide which allows for the process to sleep for a certain amount of time (in this case 10
    /// MS to perform a reset).
    pub async fn reset(&mut self, delay: &mut impl DelayNs) -> Result<(), I2cError> {
        self.sleep(true).await?;
        delay.delay_ms(10);
        self.sleep(false).await?;
        Ok(())
    }

    /// Set the device mode. Please consult page 17 and 18 of the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
    /// to learn mode about the different modes.
    pub async fn mode(&mut self, mode: u8) -> Result<(), I2cError> {
        self.write_register(addresses::CONFIG_BANK, addresses::MODE_REGISTER, mode)
            .await?;
        Ok(())
    }

    /// Set the slave device to sync audio
    pub async fn audio_sync(&mut self, yes: bool) -> Result<(), I2cError> {
        self.write_register(
            addresses::CONFIG_BANK,
            addresses::AUDIOSYNC,
            if yes { 1 } else { 0 },
        )
        .await?;
        Ok(())
    }

    /// Set the device to sleep
    pub async fn sleep(&mut self, yes: bool) -> Result<(), I2cError> {
        self.write_register(
            addresses::CONFIG_BANK,
            addresses::SHUTDOWN,
            if yes { 0 } else { 1 },
        )
        .await?;
        Ok(())
    }

    async fn write_register(&mut self, bank: u8, register: u8, value: u8) -> Result<(), I2cError> {
        self.bank(bank).await?;
        self.i2c.write(self.address, &[register, value]).await?;
        Ok(())
    }

    async fn bank(&mut self, bank: u8) -> Result<(), I2cError> {
        self.i2c
            .write(self.address, &[addresses::BANK_ADDRESS, bank])
            .await?;
        Ok(())
    }
}

const GAMMA_TABLE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
    2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 11,
    11, 11, 12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 21, 21, 22, 22,
    23, 23, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 31, 32, 33, 34, 34, 35, 36, 37, 37, 38, 39,
    40, 40, 41, 42, 43, 44, 45, 46, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
    62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 76, 77, 78, 79, 80, 81, 83, 84, 85, 86, 88,
    89, 90, 91, 93, 94, 95, 96, 98, 99, 100, 102, 103, 104, 106, 107, 109, 110, 111, 113, 114, 116,
    117, 119, 120, 121, 123, 124, 126, 128, 129, 131, 132, 134, 135, 137, 138, 140, 142, 143, 145,
    146, 148, 150, 151, 153, 155, 157, 158, 160, 162, 163, 165, 167, 169, 170, 172, 174, 176, 178,
    179, 181, 183, 185, 187, 189, 191, 193, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212, 214,
    216, 218, 220, 222, 224, 227, 229, 231, 233, 235, 237, 239, 241, 244, 246, 248, 250, 252, 255,
];

pub fn gamma(val: u8) -> u8 {
    GAMMA_TABLE[val as usize]
}

/// See the [data sheet](https://www.lumissil.com/assets/pdf/core/IS31FL3731_DS.pdf)
/// for more information on registers.
pub mod addresses {
    pub const MODE_REGISTER: u8 = 0x00;
    pub const FRAME: u8 = 0x01;
    pub const AUTOPLAY1: u8 = 0x02;
    pub const AUTOPLAY2: u8 = 0x03;
    pub const BLINK: u8 = 0x05;
    pub const AUDIOSYNC: u8 = 0x06;
    pub const BREATH1: u8 = 0x08;
    pub const BREATH2: u8 = 0x09;
    pub const SHUTDOWN: u8 = 0x0A;
    pub const GAIN: u8 = 0x0B;
    pub const ADC: u8 = 0x0C;

    pub const CONFIG_BANK: u8 = 0x0B;
    pub const BANK_ADDRESS: u8 = 0xFD;

    pub const PICTURE_MODE: u8 = 0x00;
    pub const AUTOPLAY_MODE: u8 = 0x08;
    pub const AUDIOPLAY_MODE: u8 = 0x18;

    pub const ENABLE_OFFSET: u8 = 0x00;
    pub const BLINK_OFFSET: u8 = 0x12;
    pub const COLOR_OFFSET: u8 = 0x24;
}

#[derive(Clone, Copy, Debug)]
pub enum Error<I2cError> {
    I2cError(I2cError),
    InvalidLocation(u8),
    InvalidFrame(u8),
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::I2cError(error)
    }
}
