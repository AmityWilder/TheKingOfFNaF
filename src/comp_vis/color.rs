//! Color analysis

use vidivici::ColorRGB;

/// Bitmap channels, not [`ColorRGB`] channels
pub const CHANNELS_PER_COLOR: usize = 4;

/// Normalized RGB color
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CNorm {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl CNorm {
    /// Normalize the color like a vector (necessary for performing dot product properly)
    pub fn normalized(&self) -> Self {
        let inv_len: f64 = 1.0 / (self.r * self.r + self.g * self.g + self.b * self.b).sqrt();
        Self {
            r: self.r * inv_len,
            g: self.g * inv_len,
            b: self.b * inv_len,
        }
    }

    /// Better for determining how close a color is to another, regardless of the scale. (brightness/darkness)
    pub const fn dot(&self, rhs: Self) -> f64 {
        self.r * rhs.r + self.g * rhs.g + self.b * rhs.b
    }

    // Convert the color components from 0..=255 to 0.0..=1.0
    pub const fn normalized(&self) -> CNorm {
        const INV_BYTE_MAX: f64 = 1.0 / 255.0;
        CNorm {
            r: self.r as f64 * INV_BYTE_MAX,
            g: self.g as f64 * INV_BYTE_MAX,
            b: self.b as f64 * INV_BYTE_MAX,
        }
    }

    pub fn similarity(&self, other: ColorRGB) -> f64 {
        self.normalized().dot(other.normalized())
    }
}

impl From<ColorRGB> for CNorm {
    /// Convert the color components from 0..=255 to 0.0..=1.0
    fn from(value: ColorRGB) -> Self {
        const INV_BYTE_MAX: f64 = 1.0 / 255.0;
        CNorm {
            r: value.r as f64 * INV_BYTE_MAX,
            g: value.g as f64 * INV_BYTE_MAX,
            b: value.b as f64 * INV_BYTE_MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ColorHSL {
    /// A degree on the color wheel [0..360]
    pub hue: f64,
    /// Percentage of color [0..100]
    pub sat: f64,
    /// Percentage of brightness [0..100]
    pub lum: f64,
}

impl From<ColorRGB> for ColorHSL {
    fn from(value: ColorRGB) -> Self {
        let col = value.normalized();

        let cmax: f64 = col.r.max(col.g.max(col.b));
        let cmin: f64 = col.r.max(col.g.min(col.b));
        let cmax_cmpnt: i32 = if col.r > col.g {
            if col.r > col.b { 0 } else { 2 }
        } else if col.g > col.b {
            1
        } else {
            2
        };

        let delta = cmax - cmin;

        // Hue
        let hue = if delta == 0.0 {
            0.0
        } else {
            match cmax_cmpnt {
                0 => 60.0 * ((col.g - col.b) / delta),         // Red
                1 => 60.0 * (((col.b - col.r) / delta) + 2.0), // Green
                2 => 60.0 * (((col.r - col.g) / delta) + 4.0), // Blue
                _ => unreachable!(),
            }
        };

        // Lum
        let lum = 0.5 * (cmax + cmin);

        // Sat
        let sat = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * lum - 1.0).abs())
        };

        // Finished
        ColorHSL { hue, sat, lum }
    }
}
