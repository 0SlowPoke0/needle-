use std::env;
use std::io;
use std::process;
mod pattern_matching;
mod pattern_type;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let Some(pattern) = env::args().nth(2) else {
        eprintln!("No pattern to check was provided");
        process::exit(1)
    };
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();
    input_line = input_line.trim_end().to_string(); // remove trailing newline(s)

    // Uncomment this block to pass the first stage
    if pattern_matching::match_pattern(&pattern, &input_line) {
        process::exit(0)
    } else {
        println!("Error Occurred");
        process::exit(1)
    }
}
