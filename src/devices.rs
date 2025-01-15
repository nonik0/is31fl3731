#[allow(unused_imports)]
use crate::{Error, IS31FL3731};
#[allow(unused_imports)]
use embedded_hal::delay::DelayNs;
#[allow(unused_imports)]
use embedded_hal::i2c::I2c;

#[cfg(feature = "charlie_bonnet")]
pub struct CharlieBonnet<I2C> {
    pub device: IS31FL3731<I2C>,
}
#[cfg(feature = "charlie_wing")]
pub struct CharlieWing<I2C> {
    pub device: IS31FL3731<I2C>,
}
#[cfg(feature = "keybow_2040")]
pub struct Keybow2040<I2C> {
    pub device: IS31FL3731<I2C>,
}
#[cfg(feature = "led_shim")]
pub struct LEDShim<I2C> {
    pub device: IS31FL3731<I2C>,
}
#[cfg(feature = "matrix")]
pub struct Matrix<I2C> {
    pub device: IS31FL3731<I2C>,
}
#[cfg(feature = "rgb_matrix_5x5")]
pub struct RGBMatrix5x5<I2C> {
    pub device: IS31FL3731<I2C>,
}
#[cfg(feature = "scroll_phat_hd")]
pub struct ScrollPhatHD<I2C> {
    pub device: IS31FL3731<I2C>,
}

#[cfg(feature = "charlie_bonnet")]
impl<I2C, I2cError> CharlieBonnet<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn configure(i2c: I2C) -> IS31FL3731<I2C> {
        IS31FL3731 {
            i2c,
            address: 0x74,
            frame: 0,
        }
    }

    pub fn calc_pixel(x: u8, y: u8) -> Result<u8, Error<I2cError>> {
        if x > 16 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 8 {
            return Err(Error::InvalidLocation(y));
        }
        Ok(if x >= 8 {
            (x - 6) * 16 - (y + 1)
        } else {
            (x + 1) * 16 + (7 - y)
        })
    }
}

#[cfg(feature = "charlie_wing")]
impl<I2C, I2cError> CharlieWing<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn configure(i2c: I2C) -> IS31FL3731<I2C> {
        IS31FL3731 {
            i2c,
            address: 0x74,
            frame: 0,
        }
    }

    pub fn calc_pixel(x: u8, y: u8) -> Result<u8, Error<I2cError>> {
        if x > 15 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 7 {
            return Err(Error::InvalidLocation(y));
        }
        let mut x = x;
        let mut y = y;
        if x > 7 {
            x -= 15;
            y += 8;
        } else {
            y = 7 - y
        }
        Ok(x * 16 + y)
    }
}

#[cfg(feature = "keybow_2040")]
impl<I2C> Keybow2040<I2C> {
    pub fn configure(i2c: I2C) -> Self {
        Self {
            device: IS31FL3731 {
                i2c,
                address: 0x74,
                frame: 0,
            },
        }
    }

    pub fn calc_pixel<E>(x: u8, y: u8) -> Result<u8, Error<E>> {
        if x > 16 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 3 {
            return Err(Error::InvalidLocation(y));
        }
        let lookup = [
            [120, 88, 104],
            [136, 40, 72],
            [112, 80, 96],
            [128, 32, 64],
            [121, 89, 105],
            [137, 41, 73],
            [113, 81, 97],
            [129, 33, 65],
            [122, 90, 106],
            [138, 25, 74],
            [114, 82, 98],
            [130, 17, 66],
            [123, 91, 107],
            [139, 26, 75],
            [115, 83, 99],
            [131, 18, 67],
        ];
        Ok(lookup[x as usize][y as usize])
    }
}

#[cfg(feature = "keybow_2040")]
impl<I2C, I2cError> Keybow2040<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn pixel_rgb_blocking(
        &mut self,
        x: u8,
        y: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), Error<I2cError>> {
        let x = (4 * (3 - x)) + y;
        self.device.pixel_blocking(Self::calc_pixel(x, 0)?, r)?;
        self.device.pixel_blocking(Self::calc_pixel(x, 1)?, g)?;
        self.device.pixel_blocking(Self::calc_pixel(x, 2)?, b)?;
        Ok(())
    }
}

#[cfg(all(feature = "keybow_2040", feature = "async"))]
impl<I2C, I2cError> Keybow2040<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = I2cError>,
{
    pub async fn pixel_rgb(
        &mut self,
        x: u8,
        y: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), Error<I2cError>> {
        let x = (4 * (3 - x)) + y;
        self.device.pixel(Self::calc_pixel(x, 0)?, r).await?;
        self.device.pixel(Self::calc_pixel(x, 1)?, g).await?;
        self.device.pixel(Self::calc_pixel(x, 2)?, b).await?;
        Ok(())
    }
}

#[cfg(feature = "led_shim")]
impl<I2C> LEDShim<I2C> {
    pub fn configure(i2c: I2C) -> Self {
        Self {
            device: IS31FL3731 {
                i2c,
                address: 0x75,
                frame: 0,
            },
        }
    }

    pub fn calc_pixel<E>(x: u8, y: u8) -> Result<u8, Error<E>> {
        if x > 28 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 3 {
            return Err(Error::InvalidLocation(y));
        }
        if y == 0 {
            if x < 7 {
                return Ok(118 - x);
            }
            if x < 15 {
                return Ok(141 - x);
            }
            if x < 21 {
                return Ok(106 + x);
            }
            if x == 21 {
                return Ok(14);
            }
            return Ok(x - 14);
        }
        if y == 1 {
            if x < 2 {
                return Ok(69 - x);
            }
            if x < 7 {
                return Ok(86 - x);
            }
            if x < 12 {
                return Ok(28 - x);
            }
            if x < 14 {
                return Ok(45 - x);
            }
            if x == 14 {
                return Ok(47);
            }
            if x == 15 {
                return Ok(41);
            }
            if x < 21 {
                return Ok(x + 9);
            }
            if x == 21 {
                return Ok(95);
            }
            if x < 26 {
                return Ok(x + 67);
            }
            return Ok(x + 50);
        }

        if x == 0 {
            return Ok(85);
        }
        if x < 7 {
            return Ok(102 - x);
        }
        if x < 11 {
            return Ok(44 - x);
        }
        if x == 14 {
            return Ok(63);
        }
        if x < 17 {
            return Ok(42 + x);
        }
        if x < 21 {
            return Ok(x + 25);
        }
        if x == 21 {
            return Ok(111);
        }
        if x < 27 {
            return Ok(x + 83);
        }

        Ok(93)
    }
}

#[cfg(feature = "led_shim")]
impl<I2C, I2cError> LEDShim<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn pixel_rgb_blocking(
        &mut self,
        x: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), Error<I2cError>> {
        self.device.pixel_blocking(Self::calc_pixel(x, 0)?, r)?;
        self.device.pixel_blocking(Self::calc_pixel(x, 1)?, g)?;
        self.device.pixel_blocking(Self::calc_pixel(x, 2)?, b)?;
        Ok(())
    }
}

#[cfg(all(feature = "led_shim", feature = "async"))]
impl<I2C, I2cError> LEDShim<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = I2cError>,
{
    pub async fn pixel_rgb(&mut self, x: u8, r: u8, g: u8, b: u8) -> Result<(), Error<I2cError>> {
        self.device.pixel(Self::calc_pixel(x, 0)?, r).await?;
        self.device.pixel(Self::calc_pixel(x, 1)?, g).await?;
        self.device.pixel(Self::calc_pixel(x, 2)?, b).await?;
        Ok(())
    }
}

#[cfg(feature = "matrix")]
impl<I2C, I2cError> Matrix<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn configure(i2c: I2C) -> Self {
        Self {
            device: IS31FL3731 {
                i2c,
                address: 0x74,
                frame: 0,
            },
        }
    }

    pub fn calc_pixel(x: u8, y: u8) -> Result<u8, Error<I2cError>> {
        if x > 16 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 9 {
            return Err(Error::InvalidLocation(y));
        }
        Ok(x + y * 16)
    }
}

#[cfg(feature = "rgb_matrix_5x5")]
impl<I2C> RGBMatrix5x5<I2C> {
    pub fn configure(i2c: I2C) -> Self {
        Self {
            device: IS31FL3731 {
                i2c,
                address: 0x75,
                frame: 0,
            },
        }
    }

    pub fn calc_pixel<E>(x: u8, y: u8) -> Result<u8, Error<E>> {
        if x > 25 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 3 {
            return Err(Error::InvalidLocation(y));
        }
        let lookup = [
            [118, 69, 85],
            [117, 68, 101],
            [116, 84, 100],
            [115, 83, 99],
            [114, 82, 98],
            [132, 19, 35],
            [133, 20, 36],
            [134, 21, 37],
            [112, 80, 96],
            [113, 81, 97],
            [131, 18, 34],
            [130, 17, 50],
            [129, 33, 49],
            [128, 32, 48],
            [127, 47, 63],
            [125, 28, 44],
            [124, 27, 43],
            [123, 26, 42],
            [122, 25, 58],
            [121, 41, 57],
            [126, 29, 45],
            [15, 95, 111],
            [8, 89, 105],
            [9, 90, 106],
            [10, 91, 107],
        ];
        Ok(lookup[x as usize][y as usize])
    }
}

#[cfg(feature = "rgb_matrix_5x5")]
impl<I2C, I2cError> RGBMatrix5x5<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn pixel_rgb_blocking(
        &mut self,
        x: u8,
        y: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), Error<I2cError>> {
        let x = x + y * 5;
        self.device.pixel_blocking(Self::calc_pixel(x, 0)?, r)?;
        self.device.pixel_blocking(Self::calc_pixel(x, 1)?, g)?;
        self.device.pixel_blocking(Self::calc_pixel(x, 2)?, b)?;
        Ok(())
    }
}

#[cfg(all(feature = "rgb_matrix_5x5", feature = "async"))]
impl<I2C, I2cError> RGBMatrix5x5<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = I2cError>,
{
    pub async fn pixel_rgb(
        &mut self,
        x: u8,
        y: u8,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), Error<I2cError>> {
        let x = x + y * 5;
        self.device.pixel(Self::calc_pixel(x, 0)?, r).await?;
        self.device.pixel(Self::calc_pixel(x, 1)?, g).await?;
        self.device.pixel(Self::calc_pixel(x, 2)?, b).await?;
        Ok(())
    }
}

#[cfg(feature = "scroll_phat_hd")]
impl<I2C, I2cError> ScrollPhatHD<I2C>
where
    I2C: I2c<Error = I2cError>,
{
    pub fn configure(i2c: I2C) -> Self {
        Self {
            device: IS31FL3731 {
                i2c,
                address: 0x74,
                frame: 0,
            },
        }
    }

    pub fn calc_pixel(x: u8, y: u8) -> Result<u8, Error<I2cError>> {
        if x > 17 {
            return Err(Error::InvalidLocation(x));
        }
        if y > 7 {
            return Err(Error::InvalidLocation(y));
        }
        let mut x = x;
        let mut y = y;
        if x <= 8 {
            x = 8 - x;
            y = 6 - y;
        } else {
            x -= 8;
            y -= 8;
        }
        Ok(x * 16 + y)
    }
}
