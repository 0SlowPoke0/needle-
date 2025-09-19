use crate::pattern_type::{get_next_token, PatternType, Quantifier};

/// Try to match `pattern` at the current position of `input` ONLY.
/// This function does NOT try to slide the pattern across the input.
/// It returns `true` if the full `pattern` matches the start of `input`.
pub fn match_pattern_here(pattern: &str, input: &str) -> bool {
    // Base case: pattern consumed → success regardless of remaining input
    if pattern.is_empty() {
        return true;
    }

    // Parse next token
    if let Some((token, rest_pattern)) = get_next_token(pattern) {
        // Anchors that assert positions:
        if matches!(token.kind, PatternType::StartAnchor) {
            // '^' consumes nothing; just continue matching the rest at this position.
            return match_pattern_here(rest_pattern, input);
        }
        if matches!(token.kind, PatternType::EndAnchor) {
            // '$' should only match if we're at the end of input
            return input.is_empty();
        }

        // Try to match this token at the current input position
        match token.quant {
            Quantifier::One => {
                // Simple case: match exactly one character
                if let Some(ch) = input.chars().next() {
                    if char_matches(&token.kind, ch) {
                        let remaining = &input[ch.len_utf8()..];
                        return match_pattern_here(rest_pattern, remaining);
                    }
                }
                return false;
            }
            Quantifier::OneOrMore => {
                // Complex case: match one or more with backtracking
                return match_one_or_more(&token.kind, input, rest_pattern);
            }
            Quantifier::ZeroOrOne => {
                if let Some(ch) = input.chars().next() {
                    if char_matches(&token.kind, ch) {
                        let remaining = &input[ch.len_utf8()..];
                        return match_pattern_here(rest_pattern, remaining);
                    }
                }
                return true;
            }
        }
    }

    // No more tokens — success (pattern fully consumed)
    true
}

/// Handle OneOrMore quantifier with proper backtracking
fn match_one_or_more(kind: &PatternType, input: &str, rest_pattern: &str) -> bool {
    // Collect all possible positions after matching 1, 2, 3... characters
    let mut positions = Vec::new();
    let mut current_pos = input;

    // Keep consuming matching characters and save each position
    loop {
        let ch = match current_pos.chars().next() {
            Some(ch) if char_matches(kind, ch) => ch,
            _ => break,
        };
        current_pos = &current_pos[ch.len_utf8()..];
        positions.push(current_pos);

        // If we've consumed all input, stop
        if current_pos.is_empty() {
            break;
        }
    }

    // Try positions from longest match to shortest (greedy with backtracking)
    for &pos in positions.iter().rev() {
        if match_pattern_here(rest_pattern, pos) {
            return true;
        }
    }

    false
}

/// Top-level search function: if pattern begins with `^`, match only at start,
/// otherwise search (slide) pattern across entire input until a match is found.
pub fn match_pattern(pattern: &str, input: &str) -> bool {
    if pattern.starts_with('^') {
        // Remove leading '^' and require a match at the start
        return match_pattern_here(&pattern[1..], input);
    }

    // Otherwise, try every possible starting position
    let mut cur = input;
    loop {
        if match_pattern_here(pattern, cur) {
            return true;
        }
        // advance by exactly one Unicode scalar (UTF-8 safe)
        if let Some(ch) = cur.chars().next() {
            cur = &cur[ch.len_utf8()..];
        } else {
            return false; // no more positions to try
        }
    }
}

fn char_matches(kind: &PatternType, ch: char) -> bool {
    match kind {
        PatternType::Any => true,
        PatternType::Digit => ch.is_ascii_digit(),
        PatternType::Word => ch.is_alphanumeric() || ch == '_',
        PatternType::Literal(c) => *c == ch,
        PatternType::CharClass(chars) => chars.contains(ch),
        PatternType::NegClass(chars) => !chars.contains(ch),
        PatternType::StartAnchor | PatternType::EndAnchor => false,
    }
}
