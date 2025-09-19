#[derive(Debug, PartialEq)]
pub enum Quantifier {
    One,       // exactly once (default)
    OneOrMore, // +
    ZeroOrOne, // ?
}

#[derive(Debug)]
pub struct Token {
    pub kind: PatternType, // what it matches (char, digit, etc.)
    pub quant: Quantifier, // how many times
}

#[derive(Debug)]
pub enum PatternType {
    Any,
    Digit,
    Word,
    Literal(char),
    CharClass(String),
    NegClass(String),
    StartAnchor,
    EndAnchor,
    Group(Vec<String>), // New: represents (pattern1|pattern2|...)
}

/// Parse the next token from a pattern string, returning:
///   - a `Token` describing the token
///   - the remainder of the pattern after that token.
///
/// Returns `None` if the pattern is empty.
pub fn get_next_token(pattern: &str) -> Option<(Token, &str)> {
    if pattern.is_empty() {
        return None;
    }

    // ---------- 1) Detect the base kind ----------
    let (kind, rest_after_kind) = if pattern.starts_with(r"\d") {
        (PatternType::Digit, &pattern[2..])
    } else if pattern.starts_with(r"\w") {
        (PatternType::Word, &pattern[2..])
    } else if pattern.starts_with('^') {
        (PatternType::StartAnchor, &pattern[1..])
    } else if pattern.starts_with('$') {
        (PatternType::EndAnchor, &pattern[1..])
    } else if pattern.starts_with('(') {
        // Parse group with alternation
        let group_content = parse_group(&pattern[1..])?;
        let alternatives = group_content.0;
        let rest = group_content.1;
        (PatternType::Group(alternatives), rest)
    } else if pattern.starts_with("[^") {
        let end = pattern.find(']')?;
        let chars: String = pattern[2..end].into();
        (PatternType::NegClass(chars), &pattern[end + 1..])
    } else if pattern.starts_with('[') {
        let end = pattern.find(']')?;
        let chars: String = pattern[1..end].into();
        (PatternType::CharClass(chars), &pattern[end + 1..])
    } else if pattern.starts_with('.') {
        (PatternType::Any, &pattern[1..])
    } else {
        let ch = pattern.chars().next()?;
        let rest = &pattern[ch.len_utf8()..];
        (PatternType::Literal(ch), rest)
    };

    // ---------- 2) Check for quantifier ----------
    let (quant, rest) = match rest_after_kind.chars().next() {
        Some('+') => (Quantifier::OneOrMore, &rest_after_kind[1..]),
        Some('?') => (Quantifier::ZeroOrOne, &rest_after_kind[1..]),
        _ => (Quantifier::One, rest_after_kind),
    };

    Some((Token { kind, quant }, rest))
}

/// Parse a group like "(cat|dog|bird)" and return the alternatives and remaining pattern
fn parse_group(pattern: &str) -> Option<(Vec<String>, &str)> {
    // Find the matching closing parenthesis
    let mut paren_count = 1;
    let mut pos = 0;

    for (i, ch) in pattern.char_indices() {
        match ch {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count == 0 {
                    pos = i;
                    break;
                }
            }
            _ => {}
        }
    }

    if paren_count != 0 {
        return None; // Unmatched parentheses
    }

    // Extract content between parentheses
    let group_content = &pattern[..pos];
    let rest = &pattern[pos + 1..];

    // Split by '|' to get alternatives
    let alternatives: Vec<String> = group_content.split('|').map(|s| s.to_string()).collect();

    Some((alternatives, rest))
}
