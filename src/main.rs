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
use std::{path::Path, io::Read, ffi::{OsString, OsStr}};
use regex::Regex;
use clap::{Arg, App, crate_version, arg_enum, value_t};

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

impl AddressPattern {
    fn matches(&self, line_number: usize, line: &str) -> bool {
        match &self {
            AddressPattern::Line(n) => *n == line_number,
            AddressPattern::LineRange(start, end) => (*start..*end).contains(&line_number),
            AddressPattern::RegexPattern(re) => re.is_match(line),
            _ => todo!(),
        }
    }
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

arg_enum! {
    enum CommentingMode {
        Toggle,
        Comment,
        Uncomment,
    }
}

fn comment_line(prefix_pattern: &Regex, prefix: &str, line: &str) -> String {
    if !prefix_pattern.is_match(line) {
        format!("{}{}", prefix, line)
    } else {
        line.to_string()
    }
}

fn toggle_line(prefix_pattern: &Regex, prefix: &str, line: &str) -> String {
    // TODO: have a smart "block-based" commenting status instead of per-line
    // Is it possible to define this in such a way that it's properly it's own inverse?
    // c.f. "  # leading-whitespace" -> "  leading-whitespace" -> "#   leading-whitespace"
    if prefix_pattern.is_match(line) {
        prefix_pattern.replace(line, "$head$tail").to_string()
    } else {
        format!("{}{}", prefix, line)
    }
}

fn uncomment_line(prefix_pattern: &Regex, prefix: &str, line: &str) -> String {
    prefix_pattern.replace(line, "$head$tail").to_string()
}

fn get_bin_name() -> OsString {
    let args: Vec<OsString> = std::env::args_os().collect();
    let p = Path::new(OsStr::new(&args[0]));
    p.file_name().unwrap_or(OsStr::new("<UNSET>")).into()
}

fn main() {
    // Check options, do we have a pattern? A filename? A target state?
    // Open streams
    // Guess language if not specified
    // Match lines and set/toggle comment status
    let default_mode = match get_bin_name().to_str() {
        Some("comment") => "comment",
        Some("uncomment") => "uncomment",
        _ => "toggle",
    };

    let args = App::new("toggle-comment")
        .version(crate_version!())
        .about("A utility for setting or toggling the line-comment status of lines in text files")
        .arg(Arg::with_name("comment_mode")
            .long("mode")
            .value_name("comment|toggle|uncomment")
            .help(&format!("Commenting behaviour [default: {}]", default_mode))
            .default_value(default_mode)
            .hide_default_value(true)
            .possible_values(&["comment", "toggle", "uncomment"])
            .case_insensitive(true)
            .hide_possible_values(true))
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

    let mode = value_t!(args.value_of("comment_mode"), CommentingMode).unwrap();
    let pattern_str = args.value_of("PATTERN").unwrap_or("");
    let pattern = try_parse_pattern(pattern_str).expect("Unable to parse pattern");
    let contents = if let Some(file_path) = args.value_of("INPUT") {
        fs::read_to_string(file_path).expect("Unable to read file")  // TODO: edit this input file in place
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Unable to read from stdin");
        buffer
    };
    let prefix = args.value_of("comment_prefix").unwrap_or("# ");

    let prefix_pattern: Regex = Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();
    let operator = match mode {
        CommentingMode::Comment => comment_line,
        CommentingMode::Toggle => toggle_line,
        CommentingMode::Uncomment => uncomment_line,
    };
    for (idx, line) in contents.lines().enumerate() {
        let line_number = idx+1;
        if pattern.matches(line_number, line) {
            println!("{}", operator(&prefix_pattern, prefix, line));
        } else {
            println!("{}", line);
        }
    }
}

#[cfg(test)]
mod test;
