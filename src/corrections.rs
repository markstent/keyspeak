use crate::settings::Correction;

/// Apply all word corrections to transcribed text.
/// Matching is case-insensitive; replacement uses the correction's casing.
/// Only whole-word matches are replaced (no partial word substitutions).
pub fn apply(text: &str, corrections: &[Correction]) -> String {
    if corrections.is_empty() || text.is_empty() {
        return text.to_string();
    }
    let mut result = text.to_string();
    for c in corrections {
        if c.from.is_empty() {
            continue;
        }
        result = replace_whole_word(&result, &c.from, &c.to);
    }
    result
}

fn replace_whole_word(text: &str, from: &str, to: &str) -> String {
    let lower_text = text.to_lowercase();
    let lower_from = from.to_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut last = 0;
    let mut start = 0;

    while let Some(pos) = lower_text[start..].find(&lower_from) {
        let abs = start + pos;
        let end = abs + from.len();

        let before_ok = abs == 0 || !text[..abs].ends_with(char::is_alphanumeric);
        let after_ok = end >= text.len() || !text[end..].starts_with(char::is_alphanumeric);

        if before_ok && after_ok {
            result.push_str(&text[last..abs]);
            result.push_str(to);
            last = end;
            start = end;
        } else {
            start = abs + 1;
        }
    }
    result.push_str(&text[last..]);
    result
}
