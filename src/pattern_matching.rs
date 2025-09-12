pub fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        p if p.starts_with(r"\d") => input_line.chars().any(|c| c.is_ascii_digit()),
        p if p.starts_with(r"\w") => input_line.chars().any(|c| c.is_alphanumeric() || c == '_'),
        p if p.chars().count() == 1 => input_line.contains(pattern),
        _ => panic!("Unhandled pattern: {}", pattern),
    }
}
