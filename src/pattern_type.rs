#[derive(Debug)]
pub enum Token {
    Digit,
    Word,
    Any,
    Literal(char),
    CharClass(Vec<char>),
    NegClass(Vec<char>),
}

pub fn token_matches(token: &Token, ch: char) -> bool {
    match token {
        Token::Digit => ch.is_ascii_digit(),
        Token::Word => ch.is_alphanumeric() || ch == '_',
        Token::Any => true,
        Token::Literal(c) => *c == ch,
        Token::CharClass(set) => set.contains(&ch),
        Token::NegClass(set) => !set.contains(&ch),
    }
}

pub fn next_token(pattern: &str) -> Option<(Token, &str)> {
    if pattern.is_empty() {
        return None;
    }

    // escape sequences
    if pattern.starts_with("\\d") {
        return Some((Token::Digit, &pattern[2..]));
    }
    if pattern.starts_with("\\w") {
        return Some((Token::Word, &pattern[2..]));
    }

    // wildcard
    if pattern.starts_with('.') {
        return Some((Token::Any, &pattern[1..]));
    }

    // character classes
    if pattern.starts_with("[^") {
        if let Some(end) = pattern.find(']') {
            let inside = &pattern[2..end]; // content between [^ and ]
            let chars = inside.chars().collect::<Vec<_>>();
            return Some((Token::NegClass(chars), &pattern[end + 1..]));
        }
    } else if pattern.starts_with('[') {
        if let Some(end) = pattern.find(']') {
            let inside = &pattern[1..end]; // content between [ and ]
            let chars = inside.chars().collect::<Vec<_>>();
            return Some((Token::CharClass(chars), &pattern[end + 1..]));
        }
    }

    // single literal character
    let mut chars = pattern.chars();
    let first = chars.next().unwrap();
    let rest = chars.as_str();
    Some((Token::Literal(first), rest))
}
