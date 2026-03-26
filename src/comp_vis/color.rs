//! Color analysis

use vidivici::ColorRgb;

/// Bitmap channels, not [`ColorRGB`] channels
pub const CHANNELS_PER_COLOR: usize = 4;

/// Components guaranteed to be (0,1] and have a total magnitude of 1
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UnitVector3Rgb {
    // fields are private so callers cannot break the magnitude promise
    r: f64,
    g: f64,
    b: f64,
}

impl UnitVector3Rgb {
    pub const fn mag(&self) -> f64 {
        debug_assert!(
            // dont need to sqrt because 1^2 = 1
            (1.0 - self.r * self.r + self.g * self.g + self.b * self.b).abs() <= f64::EPSILON,
            "normalized vector magnitude should be within epsilon of 1"
        );
        1.0
    }

    /// Better for determining how close a color is to another, regardless of the scale. (brightness/darkness)
    ///
    /// Result will be between -1 and +1 inclusively
    pub const fn dot(&self, rhs: &Self) -> f64 {
        self.r * rhs.r + self.g * rhs.g + self.b * rhs.b
    }
}

/// Normalized RGB color
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Vector3Rgb {
    pub const fn dot(&self, rhs: &Self) -> f64 {
        self.r * rhs.r + self.g * rhs.g + self.b * rhs.b
    }

    /// Square of magnitude - the norm
    pub const fn mag_sqr(&self) -> f64 {
        self.dot(self)
    }

    /// Magnitude
    pub fn mag(&self) -> f64 {
        self.mag_sqr().sqrt()
    }

    /// Normalize the color like a vector (necessary for performing dot product properly)
    ///
    /// Returns [`None`] if the vector cannot be normalized
    pub fn normalized(&self) -> Option<UnitVector3Rgb> {
        let mag_sqr = self.mag_sqr();
        if !(mag_sqr.is_normal() && mag_sqr > 0.0) {
            return None;
        }
        let inv_mag: f64 = 1.0 / mag_sqr.sqrt();
        if inv_mag.is_normal() {
            let r = self.r * inv_mag;
            let g = self.g * inv_mag;
            let b = self.b * inv_mag;
            debug_assert!(
                // dont need to sqrt because 1^2 = 1
                (1.0 - Self { r, g, b }.mag_sqr()).abs() <= f64::EPSILON,
                "normalized vector magnitude should be within epsilon of 1"
            );
            Some(UnitVector3Rgb { r, g, b })
        } else {
            None
        }
    }
}

impl From<ColorRgb> for Vector3Rgb {
    /// Convert the color components from 0..=255 to 0.0..=1.0
    fn from(value: ColorRgb) -> Self {
        const INV_BYTE_MAX: f64 = 1.0 / 255.0;
        Vector3Rgb {
            r: value.r as f64 * INV_BYTE_MAX,
            g: value.g as f64 * INV_BYTE_MAX,
            b: value.b as f64 * INV_BYTE_MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ColorHsl {
    /// A degree on the color wheel [0..360]
    pub hue: f64,
    /// Percentage of color [0..100]
    pub sat: f64,
    /// Percentage of brightness [0..100]
    pub lum: f64,
}

impl From<ColorRgb> for ColorHsl {
    fn from(value: ColorRgb) -> Self {
        let col = Vector3Rgb::from(value);

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
        ColorHsl { hue, sat, lum }
    }
}
