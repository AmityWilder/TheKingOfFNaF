//! Screen color

/// 24-bit RGB color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ColorRgb {
    /// Red
    pub r: u8,
    /// Green
    pub g: u8,
    /// Blue
    pub b: u8,
}

impl ColorRgb {
    /// Average magnitude of components
    pub const fn gray(&self) -> u8 {
        ((self.r as u16 + self.g as u16 + self.b as u16) / 3) as u8
    }

    /// Red deviation
    pub const fn red_dev(&self) -> i32 {
        let dist_from_mean = self.r as i32 - self.gray() as i32;
        ((dist_from_mean * dist_from_mean) / 3).isqrt()
    }

    /// Green deviation
    pub const fn green_dev(&self) -> i32 {
        let dist_from_mean = self.g as i32 - self.gray() as i32;
        ((dist_from_mean * dist_from_mean) / 3).isqrt()
    }

    /// Blue deviation
    pub const fn blue_dev(&self) -> i32 {
        let dist_from_mean = self.b as i32 - self.gray() as i32;
        ((dist_from_mean * dist_from_mean) / 3).isqrt()
    }
}
