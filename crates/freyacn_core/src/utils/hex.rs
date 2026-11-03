//! Hex color parsing. Single source of truth for the `#rgb` / `#rgba` /
//! `#rrggbb` / `#rrggbbaa` grammar.

/// Parse a hex color string into `(r, g, b, a)` bytes.
///
/// Accepts `#`-prefixed and bare input, 3/4/6/8 digits, case-insensitive.
/// Returns `None` for wrong lengths or non-hex characters.
///
/// # Examples
///
/// ```
/// use freya_cn::utils::parse_hex_rgba;
///
/// assert_eq!(parse_hex_rgba("#fff"),    Some((255, 255, 255, 255)));
/// assert_eq!(parse_hex_rgba("#ffff"),   Some((255, 255, 255, 255)));
/// assert_eq!(parse_hex_rgba("#ff0000"), Some((255, 0, 0, 255)));
/// assert_eq!(parse_hex_rgba("#ff000080"), Some((255, 0, 0, 128)));
/// assert_eq!(parse_hex_rgba("abc"),     Some((170, 187, 204, 255)));
/// assert_eq!(parse_hex_rgba("#xyz"),    None);
/// assert_eq!(parse_hex_rgba(""),        None);
/// ```
pub fn parse_hex_rgba(input: &str) -> Option<(u8, u8, u8, u8)> {
    let s = input.trim().trim_start_matches('#');
    let expand = |h: &str| -> String { h.chars().flat_map(|c| [c, c]).collect() };
    let (rgb, alpha) = match s.len() {
        3 => (expand(s), 255u8),
        4 => {
            let a = u8::from_str_radix(&expand(&s[3..]), 16).ok()?;
            (expand(&s[..3]), a)
        }
        6 => (s.to_string(), 255u8),
        8 => {
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            (s[..6].to_string(), a)
        }
        _ => return None,
    };
    if rgb.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&rgb[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb[4..6], 16).ok()?;
    Some((r, g, b, alpha))
}
