use crate::pattern_type::{next_token, token_matches};

fn match_here(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    if let Some((token, rest_pattern)) = next_token(pattern) {
        if let Some(rest_text) = token_matches(&token, text) {
            match_here(rest_pattern, rest_text)
        } else {
            false
        }
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
