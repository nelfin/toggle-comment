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

use std::{fs, io};
use std::io::Read;
use regex::Regex;
use clap::{Arg, App, crate_version};

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
    RegexPattern(Regex),
    Compound
}

fn try_parse_pattern(pattern_str: &str) -> Result<AddressPattern, &str> {
    if pattern_str.starts_with("/") {
        let x = pattern_str.trim_start_matches("/").trim_end_matches("/");
        return Ok(AddressPattern::RegexPattern(Regex::new(x).unwrap()));
    }
    if let Ok(x) = pattern_str.parse() {
        return Ok(AddressPattern::Line(x));
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
            AddressPattern::RegexPattern(re) => re.is_match(line),
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
    let args = App::new("toggle-comment")
        .version(crate_version!())
        .arg(Arg::with_name("comment_prefix")
            .value_name("PREFIX")
            .short("c")
            .long("comment-prefix")
            .takes_value(true)
            .help("Line comment prefix string [default: \"# \"]"))
        .arg(Arg::with_name("PATTERN")
            .help("ed-like address pattern for selecting lines.")
            .required(true))
        .arg(Arg::with_name("INPUT")
            .help("Sets the input file."))
        .get_matches();

    let pattern_str = args.value_of("PATTERN").unwrap_or("");
    let pattern = try_parse_pattern(pattern_str).expect("Unable to parse pattern");
    let contents = if let Some(file_path) = args.value_of("INPUT") {
        fs::read_to_string(file_path).expect("Unable to read file")
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Unable to read from stdin");
        buffer
    };
    let predicate = build_predicate(pattern);
    let prefix = args.value_of("comment_prefix").unwrap_or("# ");

    for (idx, line) in contents.lines().enumerate() {
        let line_number = idx+1;
        if predicate.matches(line_number, line) {
            println!("{}{}", prefix, line);
        } else {
            println!("{}", line);
        }
    }
}
