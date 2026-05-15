const WIDE_RANGES: &[(u32, u32)] = &[
    (0x1100, 0x115f),
    (0x2329, 0x232a),
    (0x2e80, 0xa4cf),
    (0xac00, 0xd7a3),
    (0xf000, 0xf8ff),
    (0xfe10, 0xfe19),
    (0xfe30, 0xfe6f),
    (0xff00, 0xff60),
    (0xffe0, 0xffe6),
    (0x1f300, 0x1faff),
];

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for n in chars.by_ref() {
                if n == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn is_wide(code: u32) -> bool {
    WIDE_RANGES
        .iter()
        .any(|(lo, hi)| code >= *lo && code <= *hi)
}

fn is_non_display(code: u32) -> bool {
    code == 0 || code < 32 || (0x7f..0xa0).contains(&code)
}

pub fn char_width(c: char) -> usize {
    let code = c as u32;
    if is_non_display(code) {
        0
    } else if is_wide(code) {
        2
    } else {
        1
    }
}

pub fn display_width(s: &str) -> usize {
    strip_ansi(s).chars().map(char_width).sum()
}

pub fn truncate(s: &str, width: usize) -> String {
    let current = display_width(s);
    if current <= width {
        return format!("{}{}", s, " ".repeat(width - current));
    }
    let plain = strip_ansi(s);
    let mut result = String::new();
    let mut used = 0;
    for c in plain.chars() {
        let next = used + char_width(c);
        if next >= width {
            break;
        }
        result.push(c);
        used = next;
    }
    format!("{}…{}", result, " ".repeat(width.saturating_sub(used + 1)))
}

pub fn auto_alias(title: &str) -> Option<String> {
    let words: Vec<&str> = title
        .split_whitespace()
        .filter(|word| word.chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
        .collect();
    if words.len() < 2 {
        return None;
    }
    Some(
        words
            .iter()
            .filter_map(|word| word.chars().next())
            .collect::<String>()
            .to_lowercase(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_ascii_and_wide_glyphs() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('😀'), 2);
        assert_eq!(display_width("a😀b"), 4);
    }

    #[test]
    fn ignores_ansi_color_escapes() {
        assert_eq!(display_width("\x1b[31mred\x1b[0m"), 3);
    }

    #[test]
    fn pads_short_strings_to_requested_display_width() {
        assert_eq!(truncate("tmux", 6), "tmux  ");
    }

    #[test]
    fn truncates_long_strings_with_ellipsis() {
        assert_eq!(truncate("tmux-palette", 6), "tmux-…");
    }

    #[test]
    fn does_not_split_wide_glyph_across_boundary() {
        assert_eq!(truncate("ab😀cd", 5), "ab😀…");
    }

    #[test]
    fn auto_alias_builds_initials_from_multi_word_titles() {
        assert_eq!(auto_alias("Split Horizontal"), Some("sh".to_string()));
    }

    #[test]
    fn auto_alias_ignores_single_word_titles() {
        assert_eq!(auto_alias("Detach"), None);
    }
}
