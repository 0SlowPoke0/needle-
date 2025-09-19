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

        // Handle groups with alternation
        if let PatternType::Group(alternatives) = &token.kind {
            return match_group(&alternatives, input, rest_pattern, &token.quant);
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
                // Try matching with the token first
                if let Some(ch) = input.chars().next() {
                    if char_matches(&token.kind, ch) {
                        let remaining = &input[ch.len_utf8()..];
                        if match_pattern_here(rest_pattern, remaining) {
                            return true;
                        }
                    }
                }
                // If that fails, try without consuming any character (zero matches)
                return match_pattern_here(rest_pattern, input);
            }
        }
    }

    // No more tokens — success (pattern fully consumed)
    true
}

/// Handle group matching with alternation
fn match_group(
    alternatives: &[String],
    input: &str,
    rest_pattern: &str,
    quant: &Quantifier,
) -> bool {
    match quant {
        Quantifier::One => {
            // Try each alternative in the group
            for alternative in alternatives {
                if let Some(remaining) = try_match_alternative(alternative, input) {
                    if match_pattern_here(rest_pattern, remaining) {
                        return true;
                    }
                }
            }
            false
        }
        Quantifier::OneOrMore => {
            // Groups with + quantifier: match the group one or more times
            return match_group_one_or_more(alternatives, input, rest_pattern);
        }
        Quantifier::ZeroOrOne => {
            // Try matching the group once
            for alternative in alternatives {
                if let Some(remaining) = try_match_alternative(alternative, input) {
                    if match_pattern_here(rest_pattern, remaining) {
                        return true;
                    }
                }
            }
            // If no alternative matches, try without matching the group (zero matches)
            match_pattern_here(rest_pattern, input)
        }
    }
}

/// Try to match a single alternative at the current position
/// Returns Some(remaining_input) if successful, None otherwise
fn try_match_alternative<'a>(alternative: &str, input: &'a str) -> Option<&'a str> {
    let mut current_input = input;
    let mut current_pattern = alternative;

    while !current_pattern.is_empty() {
        if let Some((token, rest_pattern)) = get_next_token(current_pattern) {
            // Handle groups recursively
            if let PatternType::Group(alternatives) = &token.kind {
                let mut found_match = false;
                for alt in alternatives {
                    if let Some(remaining) = try_match_alternative(alt, current_input) {
                        current_input = remaining;
                        found_match = true;
                        break;
                    }
                }
                if !found_match {
                    return None;
                }
                current_pattern = rest_pattern;
                continue;
            }

            match token.quant {
                Quantifier::One => {
                    if let Some(ch) = current_input.chars().next() {
                        if char_matches(&token.kind, ch) {
                            current_input = &current_input[ch.len_utf8()..];
                            current_pattern = rest_pattern;
                            continue;
                        }
                    }
                    return None; // Failed to match
                }
                Quantifier::OneOrMore => {
                    // For simplicity in alternatives, do greedy matching
                    if !match_one_or_more_greedy(&token.kind, &mut current_input) {
                        return None;
                    }
                    current_pattern = rest_pattern;
                }
                Quantifier::ZeroOrOne => {
                    // Try matching once first
                    if let Some(ch) = current_input.chars().next() {
                        if char_matches(&token.kind, ch) {
                            current_input = &current_input[ch.len_utf8()..];
                        }
                    }
                    // If no match, that's also fine (zero matches)
                    current_pattern = rest_pattern;
                }
            }
        } else {
            break;
        }
    }

    Some(current_input)
}

/// Simple greedy matching for + quantifier (without backtracking)
fn match_one_or_more_greedy(kind: &PatternType, input: &mut &str) -> bool {
    // Must match at least one character
    let first = match input.chars().next() {
        Some(ch) if char_matches(kind, ch) => ch,
        _ => return false,
    };

    *input = &input[first.len_utf8()..];

    // Consume as many more as possible
    while let Some(ch) = input.chars().next() {
        if char_matches(kind, ch) {
            *input = &input[ch.len_utf8()..];
        } else {
            break;
        }
    }

    true
}

/// Handle groups with OneOrMore quantifier
fn match_group_one_or_more(alternatives: &[String], input: &str, rest_pattern: &str) -> bool {
    // Collect all possible positions after matching the group 1, 2, 3... times
    let mut positions = Vec::new();
    let mut current_pos = input;

    // Keep trying to match one of the alternatives
    loop {
        let mut found_match = false;

        for alternative in alternatives {
            if let Some(remaining) = try_match_alternative(alternative, current_pos) {
                current_pos = remaining;
                positions.push(current_pos);
                found_match = true;
                break;
            }
        }

        if !found_match || current_pos.is_empty() {
            break;
        }
    }

    // Must have matched at least once
    if positions.is_empty() {
        return false;
    }

    // Try positions from longest match to shortest (greedy with backtracking)
    for &pos in positions.iter().rev() {
        if match_pattern_here(rest_pattern, pos) {
            return true;
        }
    }

    false
}

/// Handle OneOrMore quantifier with proper backtracking (for non-group tokens)
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
        PatternType::Group(_) => false, // Groups don't match individual characters
    }
}
