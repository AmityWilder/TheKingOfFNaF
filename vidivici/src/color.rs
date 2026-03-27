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

    /// Square of dot product of normalized vectors
    #[cfg(false)]
    pub const fn ndot_sqr(self, other: Self) -> f64 {
        // losslessness assertions
        const _: () = {
            const U8_MAX_SQR: Option<u16> = (u8::MAX as u16).checked_mul(u8::MAX as u16);
            assert!(U8_MAX_SQR.is_some(), "u8::MAX squared should fit in u16");
            const U8_MAX_SQR_3: Option<u32> = ((u8::MAX as u32) * (u8::MAX as u32)).checked_mul(3);
            assert!(
                U8_MAX_SQR_3.is_some(),
                "3 * u8::MAX squared should fit in u32"
            );
            const U8_MAX_SQR_3_UNCHECKED: u32 = 3 * (u8::MAX as u32) * (u8::MAX as u32);
            const U8_MAX_SQR_3_SQR: Option<u64> =
                (U8_MAX_SQR_3_UNCHECKED as u64).checked_mul(U8_MAX_SQR_3_UNCHECKED as u64);
            assert!(
                U8_MAX_SQR_3_SQR.is_some(),
                "3 * u8::MAX squared squared should fit in u64"
            );
            const U8_MAX_SQR_3_BITS: u32 = u64::BITS
                - (U8_MAX_SQR_3_UNCHECKED as u64 * U8_MAX_SQR_3_UNCHECKED as u64).leading_zeros();
            assert!(
                U8_MAX_SQR_3_BITS < f64::MANTISSA_DIGITS,
                "3 * u8::MAX squared squared should fit in f64 losslessly"
            );
        };

        let (u1, u2, u3) = (self.r as u64, self.g as u64, self.b as u64);
        let (v1, v2, v3) = (other.r as u64, other.g as u64, other.b as u64);

        let uv = u1 * v1 + u2 * v2 + u3 * v3;
        let uu = u1 * u1 + u2 * u2 + u3 * u3;
        let vv = v1 * v1 + v2 * v2 + v3 * v3;

        (uv * uv) as f64 / (uu * vv) as f64
    }
}
