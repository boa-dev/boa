/// Helper function for converting HSV to RGB color format.
#[allow(clippy::many_single_char_names)]
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> u32 {
    let h_i = (h * 6.0) as i64;
    let f = h * 6.0 - h_i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match h_i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => unreachable!(),
    };

    let r = (r * 256.0) as u32;
    let g = (g * 256.0) as u32;
    let b = (b * 256.0) as u32;

    let mut result = 0;
    result |= r << 16;
    result |= g << 8;
    result |= b;

    result
}

/// Represents the color of a node or edge.
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Represents the default color.
    None,
    /// Represents the color red.
    Red,
    /// Represents the color green.
    Green,
    /// Represents the color blue.
    Blue,
    /// Represents the color yellow.
    Yellow,
    /// Represents the color purple.
    Purple,
    /// Represents a RGB color.
    Rgb(u32),
}

impl Color {
    /// This funcition takes a random value and converts it to
    /// a pleasant to look at RGB color.
    pub fn from_random_number(mut random: f64) -> Color {
        const GOLDEN_RATIO_CONJUGATE: f64 = 0.618033988749895;
        random += GOLDEN_RATIO_CONJUGATE;
        random %= 1.0;
        Self::Rgb(hsv_to_rgb(random, 0.7, 0.95))
    }
}
