fn next_token(pattern: &str) -> Option<(&str, &str)> {
    if pattern.is_empty() {
        return None;
    }
    if pattern.starts_with('\\') && pattern.len() >= 2 {
        // escape like \d, \w
        Some((&pattern[..2], &pattern[2..]))
    } else if pattern.starts_with("[^") {
        // negated class
        if let Some(end) = pattern.find(']') {
            Some((&pattern[..end + 1], &pattern[end + 1..]))
        } else {
            None
        }
    } else if pattern.starts_with('[') {
        // positive class
        if let Some(end) = pattern.find(']') {
            Some((&pattern[..end + 1], &pattern[end + 1..]))
        } else {
            None
        }
    } else {
        // single char
        let mut chars = pattern.chars();
        let first = chars.next().unwrap();
        Some((&pattern[..first.len_utf8()], chars.as_str()))
    }
}

fn token_matches(token: &str, ch: char) -> bool {
    match token {
        r"\d" => ch.is_ascii_digit(),
        r"\w" => ch.is_alphanumeric() || ch == '_',
        "." => true,
        t if t.starts_with("[^") => {
            let inside = &t[2..t.len() - 1]; // strip [^ and ]
            !inside.chars().any(|c| c == ch)
        }
        t if t.starts_with("[") => {
            let inside = &t[1..t.len() - 1]; // strip [ and ]
            inside.chars().any(|c| c == ch)
        }
        lit if lit.chars().count() == 1 => lit.chars().next().unwrap() == ch,
        _ => panic!("Unhandled token: {}", token),
    }
}

pub fn match_here(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    if text.is_empty() {
        return false;
    }

    let (token, rest_pattern) = next_token(pattern).unwrap();
    let first_char = text.chars().next().unwrap();
    let rest_text = &text[first_char.len_utf8()..];

    if token_matches(token, first_char) {
        match_here(rest_pattern, rest_text)
    } else {
        false
    }
}
