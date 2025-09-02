use rand::Rng;

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let chroma = (1. - f32::abs(2. * l - 1.)) * s;
    let h_prime = h * 6.; // H' = H / 60deg
    let x = chroma * (1. - f32::abs(f32::rem_euclid(h_prime, 2.) - 1.)); // X = C * (1 - |H' mod 2 - 1|)
    let (r1, g1, b1) = if 0. <= h_prime && h_prime < 1. {
        (chroma, x, 0.)
    } else if h_prime < 2. {
        (x, chroma, 0.)
    } else if h_prime < 3. {
        (0., chroma, x)
    } else if h_prime < 4. {
        (0., x, chroma)
    } else if h_prime < 5. {
        (x, 0., chroma)
    } else {
        (chroma, 0., x)
    };

    let m = l - (chroma / 2.);
    let r = (r1 + m) * 255.;
    let g = (g1 + m) * 255.;
    let b = (b1 + m) * 255.;
    (r as u8, g as u8, b as u8)
}

/// Returns a [termcolor::Color] with a random hue, full saturation, and a lightness
/// between the provided `lightness_lower` and `lightness_upper` bounds (minimum 0.0, maximum 1.0)
pub fn random_color(lightness_lower: f32, lightness_upper: f32) -> termcolor::Color {
    let mut rng = rand::thread_rng();
    let (r, g, b) = hsl_to_rgb(
        rng.gen_range(0.0..1.0),
        1.0,
        rng.gen_range(lightness_lower..lightness_upper),
    );
    termcolor::Color::Rgb(r, g, b)
}
