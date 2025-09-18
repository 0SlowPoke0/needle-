use crate::pattern_type::{get_next_token, PatternType, Quantifier, Token};

pub fn match_pattern(pattern: &str, input_line: &str) -> bool {
    if pattern.is_empty() && !input_line.is_empty() {
        return true;
    }

    if pattern.is_empty() && input_line.is_empty() {
        return true;
    }

    if let Some((token, rest_pattern)) = get_next_token(pattern) {
        println!("token {:?}", token);
        println!("rest_pattern {:?}", rest_pattern);

        if matches!(token.kind, PatternType::EndAnchor) && input_line.is_empty() {
            return true;
        }
        if let Some(rest_text) = match_token_with_text(&token, input_line, rest_pattern) {
            println!("rest_text {:?}", rest_text);
            match_pattern(rest_pattern, rest_text)
        } else {
            let Some(first_char) = input_line.chars().next() else {
                return pattern.is_empty();
            };

            match_pattern(pattern, &input_line[first_char.len_utf8()..])
        }
    } else {
        false
    }
}

/// Try to match `token` against the start of `input`.
/// Returns `Some(remaining_input)` if it matches, or `None` if it doesn’t.
pub fn match_token_with_text<'a>(
    token: &Token,
    input: &'a str,
    rest_pattern: &str,
) -> Option<&'a str> {
    // --- Handle anchors first (they don’t consume text) ---
    match token.kind {
        PatternType::StartAnchor => return Some(input), // `^`
        PatternType::EndAnchor => return if input.is_empty() { Some(input) } else { None },
        _ => {}
    }

    let mut rest = input;

    // ---- First character must match (both One and OneOrMore) ----
    let first = rest.chars().next()?;
    if !char_matches(&token.kind, first) {
        return None;
    }
    rest = &rest[first.len_utf8()..];

    // Collect all possible “suffixes” after matching one or more chars
    let mut suffixes = Vec::new();
    suffixes.push(rest);

    if token.quant == Quantifier::OneOrMore {
        // Greedily consume as many as possible while saving suffix positions
        let mut tmp = rest;
        while let Some(c) = tmp.chars().next() {
            if char_matches(&token.kind, c) {
                tmp = &tmp[c.len_utf8()..];
                suffixes.push(tmp);
            } else {
                break;
            }
        }
    }

    // Try all suffixes from longest to shortest
    for candidate in suffixes.into_iter().rev() {
        if match_pattern(rest_pattern, candidate) {
            return Some(candidate);
        }
    }

    None
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
