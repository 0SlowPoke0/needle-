use crate::pattern_type::{next_token, token_matches};

pub fn match_here(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    if text.is_empty() {
        return false;
    }

    let (token, rest_pattern) = next_token(pattern).unwrap();
    let first_char = text.chars().next().unwrap();
    let rest_text = &text[first_char.len_utf8()..];

    if token_matches(&token, first_char) {
        match_here(rest_pattern, rest_text)
    } else {
        false
    }
}

pub fn match_pattern(pattern: &str, text: &str) -> bool {
    // If pattern starts with ^, only check at the beginning
    if let Some(stripped) = pattern.strip_prefix('^') {
        return match_here(stripped, text);
    }

    // Otherwise, try matching at every position
    let mut slice = text;
    loop {
        if match_here(pattern, slice) {
            return true;
        }
        if slice.is_empty() {
            break;
        }
        let ch = slice.chars().next().unwrap();
        slice = &slice[ch.len_utf8()..];
    }
    false
}
