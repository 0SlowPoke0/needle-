pub enum Quantifier {
    One,       // exactly once (default)
    OneOrMore, // +
}

pub struct Token {
    pub kind: PatternType, // what it matches (char, digit, etc.)
    pub quant: Quantifier, // how many times
}

#[derive(Debug)]
pub enum PatternType {
    Digit,
    Word,
    Literal(char),
    CharClass(String),
    NegClass(String),
    StartAnchor,
    EndAnchor,
}

/// Parse the next token from a pattern string, returning:
///   - a `PatternType` describing the token
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
        // `$` is a full token; no remainder inside
        (PatternType::EndAnchor, &pattern[1..])
    } else if pattern.starts_with("[^") {
        let end = pattern.find(']')?;
        let chars: String = pattern[2..end].into();
        (PatternType::NegClass(chars), &pattern[end + 1..])
    } else if pattern.starts_with('[') {
        let end = pattern.find(']')?;
        let chars: String = pattern[1..end].into();
        (PatternType::CharClass(chars), &pattern[end + 1..])
    } else {
        let ch = pattern.chars().next()?;
        let rest = &pattern[ch.len_utf8()..];
        (PatternType::Literal(ch), rest)
    };

    // ---------- 2) Check for quantifier ----------
    let (quant, rest) = match rest_after_kind.chars().next() {
        Some('+') => (Quantifier::OneOrMore, &rest_after_kind[1..]),
        // Some('*') => (Quantifier::ZeroOrMore, &rest_after_kind[1..]),
        // Some('?') => (Quantifier::ZeroOrOne, &rest_after_kind[1..]),
        _ => (Quantifier::One, rest_after_kind),
    };

    Some((Token { kind, quant }, rest))
}
