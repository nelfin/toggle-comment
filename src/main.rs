// toggle-comment   Andrew Haigh <hello@nelf.in>    2020 CE
//
// toggle-comment is a utility designed around setting or toggling the line-comment status
// of lines in plain text files in a do-what-i-mean fashion. It should, where possible,
// run without configuration, guess the current language/line-comment character, match
// lines based on ex/vi-like patterns, and edit files in place if given or manipulate text
// streams if not.
//
// $ cat > example.py
// def greeting(num_greeted):
//     print("Hello, world!")
//     return num_greeted+1
// $ toggle-comment 2 < example.py
// def greeting(num_greeted):
// #    print("Hello, world!")
//     return num_greeted+1
// $ cat > example.rs
// fn main() {
//     println!("Hello, world!");
// }
// $ toggle-comment 2 < example.rs
// fn main() {
// //    println!("Hello, world!");
// }

use std::{fs, env};

// A simplified introduction to vi/ex/ed "address patterns":
//
// N                1-indexed line number
// M,N              a range of lines, 1-indexed inclusive of end
// M,+N             a range specified by a start and a count
// /pattern/        a regular expression
enum AddressPattern {
    Line(usize),
    LineRange(usize, usize),
    LineRelativeRange { start: usize, count: usize },
    SubstringPattern(String),
    RegexPattern(String),
    Compound
}

fn try_parse_pattern(pattern_str: &str) -> Result<AddressPattern, &str> {
    if let Ok(x) = pattern_str.parse() {
        return Ok(AddressPattern::Line(x));
    } else if pattern_str.matches(|x: char| x.is_alphabetic()).count() > 0 {
        return Ok(AddressPattern::SubstringPattern(pattern_str.to_owned()));
    }
    let lines: Vec<usize> = pattern_str.split(",")
        .map(|x| { x.parse().expect("Unable to parse number") })
        .collect();
    Ok(AddressPattern::LineRange(lines[0], lines[1]))
}

struct Predicate {
    pattern: AddressPattern
}

impl Predicate {
    fn matches(&self, line_number: usize, line: &str) -> bool {
        match &self.pattern {
            AddressPattern::Line(n) => *n == line_number,
            AddressPattern::LineRange(start, end) => (*start..*end).contains(&line_number),
            AddressPattern::SubstringPattern(s) => line.contains(s),
            _ => false
        }
    }
}

fn build_predicate(pattern: AddressPattern) -> Predicate {
    Predicate { pattern }
}

fn main() {
    // Check options, do we have a pattern? A filename? A target state?
    // Open streams
    // Guess language if not specified
    // Match lines and set/toggle comment status
    let args: Vec<String> = env::args().collect();

    let pattern = try_parse_pattern(&args[1]).expect("Unable to parse pattern");
    let contents = fs::read_to_string(&args[2]).expect("Unable to read file");
    let predicate = build_predicate(pattern);

    for (idx, line) in contents.lines().enumerate() {
        let line_number = idx+1;
        if predicate.matches(line_number, line) {
            println!("# {}", line);
        } else {
            println!("{}", line);
        }
    }
}
