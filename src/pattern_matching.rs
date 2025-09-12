use std::collections::HashSet;

pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        p if p.starts_with(r"\d") => input_line.chars().any(|c| c.is_ascii_digit()),
        p if p.starts_with(r"\w") => input_line.chars().any(|c| c.is_alphanumeric() || c == '_'),
        p if p.starts_with("[^") => check_character_groups(input_line, pattern, false),
        p if p.starts_with("[") => check_character_groups(input_line, pattern, true),
        p if p.chars().count() == 1 => input_line.contains(pattern),
        _ => panic!("Unhandled pattern: {}", pattern),
    }
}

fn check_character_groups(input_line: &str, pattern: &str, positive: bool) -> bool {
    let prefix = if positive { "[" } else { "[^" };
    let Some(target) = pattern
        .strip_prefix(prefix)
        .map(|c| c.strip_suffix(']'))
        .flatten()
    else {
        return false;
    };

    let check_word_contains_target = |c: &str, positive: bool| -> bool {
        let target_chars = target.chars().collect::<HashSet<_>>();

        if positive {
            c.chars()
                .any(|word_element| target_chars.contains(&word_element))
        } else {
            c.chars()
                .any(|word_element| !target_chars.contains(&word_element))
        }
    };

    input_line
        .split_whitespace()
        .any(|c| check_word_contains_target(c, positive))
}
