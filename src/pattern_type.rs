#[derive(Debug)]
pub enum Token {
    Digit,
    Word,
    Any,
    Literal(char),
    CharClass(Vec<char>),
    NegClass(Vec<char>),
    StartAnchor,
    EndAnchor,
}

pub fn token_matches<'a>(token: &Token, text: &'a str) -> Option<&'a str> {
    match token {
        Token::Digit => {
            let mut chars = text.chars();
            match chars.next() {
                Some(c) if c.is_ascii_digit() => Some(chars.as_str()),
                _ => None,
            }
        }
        Token::Word => {
            let mut chars = text.chars();
            match chars.next() {
                Some(c) if c.is_alphanumeric() || c == '_' => Some(chars.as_str()),
                _ => None,
            }
        }
        Token::Any => {
            let mut chars = text.chars();
            chars.next()?; // consume one char if present
            Some(chars.as_str())
        }
        Token::Literal(lit) => {
            if text.starts_with(*lit) {
                Some(&text[1..])
            } else {
                None
            }
        }
        Token::CharClass(set) => {
            let mut chars = text.chars();
            match chars.next() {
                Some(c) if set.contains(&c) => Some(chars.as_str()),
                _ => None,
            }
        }
        Token::NegClass(set) => {
            let mut chars = text.chars();
            match chars.next() {
                Some(c) if !set.contains(&c) => Some(chars.as_str()),
                _ => None,
            }
        }
        Token::EndAnchor => {
            if text.is_empty() {
                Some(text)
            } else {
                None
            }
        }
        Token::StartAnchor => Some(&text[1..]),
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

    if pattern.starts_with('$') {
        return Some((Token::EndAnchor, ""));
    }

    if pattern.starts_with('^') {
        return Some((Token::StartAnchor, &pattern[1..]));
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
