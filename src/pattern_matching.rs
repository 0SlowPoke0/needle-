use crate::pattern_type::{get_next_token, PatternType, Quantifier};

/// Result of trying to match a pattern at a position
#[derive(Debug)]
struct MatchResult<'a> {
    success: bool,
    remaining_input: &'a str,
}

impl<'a> MatchResult<'a> {
    fn success(remaining: &'a str) -> Self {
        Self {
            success: true,
            remaining_input: remaining,
        }
    }

    fn failure() -> Self {
        Self {
            success: false,
            remaining_input: "",
        }
    }
}

/// Core matching function that returns remaining input on success
/// This eliminates the duplication between match_pattern_here and try_match_alternative
fn match_pattern_core<'a>(pattern: &str, input: &'a str) -> MatchResult<'a> {
    // Base case: pattern consumed → success regardless of remaining input
    if pattern.is_empty() {
        return MatchResult::success(input);
    }

    // Parse next token
    let Some((token, rest_pattern)) = get_next_token(pattern) else {
        return MatchResult::success(input);
    };

    // Handle anchors
    match token.kind {
        PatternType::StartAnchor => {
            return match_pattern_core(rest_pattern, input);
        }
        PatternType::EndAnchor => {
            return if input.is_empty() {
                MatchResult::success(input)
            } else {
                MatchResult::failure()
            };
        }
        PatternType::Group(ref alternatives) => {
            return match_group_core(alternatives, input, rest_pattern, &token.quant);
        }
        _ => {} // Handle regular tokens below
    }

    // Handle quantifiers
    match token.quant {
        Quantifier::One => match_token_once(&token.kind, input, rest_pattern),
        Quantifier::OneOrMore => match_token_one_or_more(&token.kind, input, rest_pattern),
        Quantifier::ZeroOrOne => match_token_zero_or_one(&token.kind, input, rest_pattern),
    }
}

/// Match a token exactly once
fn match_token_once<'a>(kind: &PatternType, input: &'a str, rest_pattern: &str) -> MatchResult<'a> {
    let Some(ch) = input.chars().next() else {
        return MatchResult::failure();
    };

    if char_matches(kind, ch) {
        let remaining = &input[ch.len_utf8()..];
        match_pattern_core(rest_pattern, remaining)
    } else {
        MatchResult::failure()
    }
}

/// Match a token zero or one times (? quantifier)
fn match_token_zero_or_one<'a>(
    kind: &PatternType,
    input: &'a str,
    rest_pattern: &str,
) -> MatchResult<'a> {
    // Try matching once first
    if let Some(ch) = input.chars().next() {
        if char_matches(kind, ch) {
            let remaining = &input[ch.len_utf8()..];
            let result = match_pattern_core(rest_pattern, remaining);
            if result.success {
                return result;
            }
        }
    }

    // If matching once failed or wasn't possible, try zero matches
    match_pattern_core(rest_pattern, input)
}

/// Match a token one or more times (+ quantifier)
fn match_token_one_or_more<'a>(
    kind: &PatternType,
    input: &'a str,
    rest_pattern: &str,
) -> MatchResult<'a> {
    let positions = collect_quantifier_positions(kind, input);

    if positions.is_empty() {
        return MatchResult::failure(); // Must match at least once
    }

    // Try positions from greediest to least greedy
    for &pos in positions.iter().rev() {
        let result = match_pattern_core(rest_pattern, pos);
        if result.success {
            return result;
        }
    }

    MatchResult::failure()
}

/// Collect all possible positions after matching a quantified pattern
fn collect_quantifier_positions<'a>(kind: &PatternType, input: &'a str) -> Vec<&'a str> {
    let mut positions = Vec::new();
    let mut current_pos = input;

    // Keep consuming matching characters and save each position
    while let Some(ch) = current_pos.chars().next() {
        if char_matches(kind, ch) {
            current_pos = &current_pos[ch.len_utf8()..];
            positions.push(current_pos);
        } else {
            break;
        }
    }

    positions
}

/// Handle group matching with different quantifiers
fn match_group_core<'a>(
    alternatives: &[String],
    input: &'a str,
    rest_pattern: &str,
    quant: &Quantifier,
) -> MatchResult<'a> {
    match quant {
        Quantifier::One => match_group_once(alternatives, input, rest_pattern),
        Quantifier::OneOrMore => match_group_one_or_more(alternatives, input, rest_pattern),
        Quantifier::ZeroOrOne => match_group_zero_or_one(alternatives, input, rest_pattern),
    }
}

/// Match a group exactly once
fn match_group_once<'a>(
    alternatives: &[String],
    input: &'a str,
    rest_pattern: &str,
) -> MatchResult<'a> {
    for alternative in alternatives {
        let result = match_pattern_core(alternative, input);
        if result.success {
            let final_result = match_pattern_core(rest_pattern, result.remaining_input);
            if final_result.success {
                return final_result;
            }
        }
    }
    MatchResult::failure()
}

/// Match a group zero or one times
fn match_group_zero_or_one<'a>(
    alternatives: &[String],
    input: &'a str,
    rest_pattern: &str,
) -> MatchResult<'a> {
    // Try matching once first
    let once_result = match_group_once(alternatives, input, rest_pattern);
    if once_result.success {
        return once_result;
    }

    // If that fails, try zero matches
    match_pattern_core(rest_pattern, input)
}

/// Match a group one or more times
fn match_group_one_or_more<'a>(
    alternatives: &[String],
    input: &'a str,
    rest_pattern: &str,
) -> MatchResult<'a> {
    let positions = collect_group_positions(alternatives, input);

    if positions.is_empty() {
        return MatchResult::failure(); // Must match at least once
    }

    // Try positions from greediest to least greedy
    for &pos in positions.iter().rev() {
        let result = match_pattern_core(rest_pattern, pos);
        if result.success {
            return result;
        }
    }

    MatchResult::failure()
}

/// Collect all possible positions after matching a group multiple times
fn collect_group_positions<'a>(alternatives: &[String], input: &'a str) -> Vec<&'a str> {
    let mut positions = Vec::new();
    let mut current_pos = input;

    // Keep trying to match one of the alternatives
    loop {
        let mut found_match = false;

        for alternative in alternatives {
            let result = match_pattern_core(alternative, current_pos);
            if result.success {
                current_pos = result.remaining_input;
                positions.push(current_pos);
                found_match = true;
                break;
            }
        }

        if !found_match || current_pos.is_empty() {
            break;
        }
    }

    positions
}

// Public API functions - these wrap the core functionality

/// Try to match `pattern` at the current position of `input` ONLY.
pub fn match_pattern_here(pattern: &str, input: &str) -> bool {
    match_pattern_core(pattern, input).success
}

/// Top-level search function: if pattern begins with `^`, match only at start,
/// otherwise search (slide) pattern across entire input until a match is found.
pub fn match_pattern(pattern: &str, input: &str) -> bool {
    if pattern.starts_with('^') {
        return match_pattern_here(&pattern[1..], input);
    }

    let mut cur = input;
    loop {
        if match_pattern_here(pattern, cur) {
            return true;
        }

        if let Some(ch) = cur.chars().next() {
            cur = &cur[ch.len_utf8()..];
        } else {
            return false;
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
        PatternType::Group(_) => false,
    }
}
