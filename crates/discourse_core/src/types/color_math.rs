pub fn brightness(color: &str) -> f64 {
    let rgb = converters::hex_to_rgb(color);
    (rgb.0 as f64 * 299.0 + rgb.1 as f64 * 587.0 + rgb.2 as f64 * 114.0) / 1000.0
}

pub fn dark_light_diff(
    adjusted_color: &str,
    comparison_color: &str,
    lightness: f64,
    darkness: f64,
) -> String {
    if brightness(adjusted_color) < brightness(comparison_color) {
        scale_color_lightness(adjusted_color, lightness)
    } else {
        scale_color_lightness(adjusted_color, darkness)
    }
}

pub fn scale_color_lightness(color: &str, adjustment: f64) -> String {
    let rgb = converters::hex_to_rgb(color);
    let (h, s, mut l) = converters::rgb_to_hsl(rgb.0, rgb.1, rgb.2);

    let l_f64 = l as f64;
    let new_l = if adjustment > 0.0 {
        l_f64 + (100.0 - l_f64) * adjustment
    } else {
        l_f64 + l_f64 * adjustment
    };
    l = new_l.round() as u32;

    let rgb = converters::hsl_to_rgb(h, s, l);
    converters::rgb_to_hex(rgb.0, rgb.1, rgb.2)
}

pub mod converters {
    pub fn hex_to_rgb(color: &str) -> (u32, u32, u32) {
        let mut hex = color.to_string();
        if hex.starts_with('#') {
            hex = hex[1..].to_string();
        }
        if hex.len() == 3 {
            hex = format!(
                "{}{}{}{}{}{}",
                &hex[0..1],
                &hex[0..1],
                &hex[1..2],
                &hex[1..2],
                &hex[2..3],
                &hex[2..3]
            );
        }
        if hex.len() != 6 {
            panic!("Hex color must be 6 characters");
        }
        let r = u32::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u32::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u32::from_str_radix(&hex[4..6], 16).unwrap();
        (r, g, b)
    }

    pub fn rgb_to_hex(r: u32, g: u32, b: u32) -> String {
        format!("{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn rgb_to_hsl(r: u32, g: u32, b: u32) -> (u32, u32, u32) {
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);

        let mut h = (max + min) / 2.0;
        let s;
        let l = h;

        if max == min {
            h = 0.0;
            s = 0.0; // achromatic
        } else {
            let d = max - min;
            s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };

            if max == r {
                h = (g - b) / d + if g < b { 6.0 } else { 0.0 };
            } else if max == g {
                h = (b - r) / d + 2.0;
            } else if max == b {
                h = (r - g) / d + 4.0;
            }
            h /= 6.0;
        }

        (
            (h * 360.0).round() as u32,
            (s * 100.0).round() as u32,
            (l * 100.0).round() as u32,
        )
    }

    pub fn hsl_to_rgb(h: u32, s: u32, l: u32) -> (u32, u32, u32) {
        let h = h as f64 / 360.0;
        let s = s as f64 / 100.0;
        let l = l as f64 / 100.0;

        let r;
        let g;
        let b;

        if s == 0.0 {
            r = l;
            g = l;
            b = l; // achromatic
        } else {
            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let p = 2.0 * l - q;
            r = hue_to_rgb(p, q, h + 1.0 / 3.0);
            g = hue_to_rgb(p, q, h);
            b = hue_to_rgb(p, q, h - 1.0 / 3.0);
        }

        (
            (r * 255.0).round() as u32,
            (g * 255.0).round() as u32,
            (b * 255.0).round() as u32,
        )
    }

    fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(converters::hex_to_rgb("000000"), (0, 0, 0));
        assert_eq!(converters::hex_to_rgb("ffffff"), (255, 255, 255));
        assert_eq!(converters::hex_to_rgb("fff"), (255, 255, 255));
        assert_eq!(converters::hex_to_rgb("ff0000"), (255, 0, 0));
    }

    #[test]
    fn test_rgb_to_hex() {
        assert_eq!(converters::rgb_to_hex(0, 0, 0), "000000");
        assert_eq!(converters::rgb_to_hex(255, 255, 255), "ffffff");
        assert_eq!(converters::rgb_to_hex(255, 0, 0), "ff0000");
    }

    #[test]
    fn test_rgb_to_hsl() {
        assert_eq!(converters::rgb_to_hsl(0, 0, 0), (0, 0, 0));
        assert_eq!(converters::rgb_to_hsl(255, 255, 255), (0, 0, 100));
        assert_eq!(converters::rgb_to_hsl(255, 0, 0), (0, 100, 50));
    }

    #[test]
    fn test_hsl_to_rgb() {
        assert_eq!(converters::hsl_to_rgb(0, 0, 0), (0, 0, 0));
        assert_eq!(converters::hsl_to_rgb(0, 0, 100), (255, 255, 255));
        assert_eq!(converters::hsl_to_rgb(0, 100, 50), (255, 0, 0));
    }

    #[test]
    fn test_brightness() {
        assert_eq!(brightness("000000"), 0.0);
        assert_eq!(brightness("ffffff"), 255.0);
        assert_eq!(brightness("ff0000"), 76.245);
    }
}
