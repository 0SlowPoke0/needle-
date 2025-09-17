use crate::pattern_type::{get_next_token, PatternType, Quantifier, Token};

pub fn match_pattern(pattern: &str, input_line: &str) -> bool {
    if pattern.is_empty() && !input_line.is_empty() {
        return true;
    }

    if pattern.is_empty() && input_line.is_empty() {
        return true;
    }

    if let Some((token, rest_pattern)) = get_next_token(pattern) {
        if matches!(token.kind, PatternType::EndAnchor) && input_line.is_empty() {
            return true;
        }

        if let Some(rest_text) = match_token_with_text(&token, input_line) {
            match_pattern(rest_pattern, rest_text)
        } else {
            match_pattern(pattern, &input_line[1..])
        }
    } else {
        false
    }
}

pub fn match_token_with_text<'a>(token: &Token, input: &'a str) -> Option<&'a str> {
    // --- Handle anchors first (they don't consume text or allow quantifiers) ---
    match token.kind {
        PatternType::StartAnchor => return Some(input), // `^` matches start, consumes nothing
        PatternType::EndAnchor => return if input.is_empty() { Some("") } else { None },
        _ => {}
    }

    let mut rest = input;
    let mut chars = rest.chars();

    // ---- First character must match (for both `One` and `OneOrMore`) ----
    let first = chars.next()?;
    if !char_matches(&token.kind, first) {
        return None;
    }
    rest = &rest[first.len_utf8()..];

    // ---- Handle quantifier ----
    match token.quant {
        Quantifier::One => Some(rest),
        Quantifier::OneOrMore => {
            while let Some(c) = rest.chars().next() {
                if char_matches(&token.kind, c) {
                    rest = &rest[c.len_utf8()..];
                } else {
                    break;
                }
            }
            Some(rest)
        }
    }
}

fn char_matches(kind: &PatternType, ch: char) -> bool {
    match kind {
        PatternType::Digit => ch.is_ascii_digit(),
        PatternType::Word => ch.is_alphanumeric() || ch == '_',
        PatternType::Literal(c) => *c == ch,
        PatternType::CharClass(chars) => chars.contains(ch),
        PatternType::NegClass(chars) => !chars.contains(ch),
        PatternType::StartAnchor | PatternType::EndAnchor => false,
    }
}
