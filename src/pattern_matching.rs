pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        p if p.starts_with(r"\d") => input_line.chars().any(|c| c.is_ascii_digit()),
        p if p.starts_with(r"\w") => input_line.chars().any(|c| c.is_alphanumeric() || c == '_'),
        p if p.starts_with("[") => check_positive_character_groups(input_line, pattern),
        p if p.chars().count() == 1 => input_line.contains(pattern),
        _ => panic!("Unhandled pattern: {}", pattern),
    }
}

fn check_positive_character_groups(input_line: &str, pattern: &str) -> bool {
    let Some(target) = pattern
        .strip_prefix('[')
        .map(|c| c.strip_suffix(']'))
        .flatten()
    else {
        return false;
    };

    let check_word_contains_target = |c: &str| -> bool {
        let target_chars = target.chars().collect::<Vec<_>>();

        c.chars()
            .any(|word_element| target_chars.contains(&word_element))
    };
    input_line
        .split_whitespace()
        .any(|c| check_word_contains_target(c))
}
