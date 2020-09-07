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
use std::str::Lines;

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

    fn matches_maybe<'a >(&self, line_number: usize, line: &'a str) -> Option<&'a str> {
        if self.matches(line_number, line) {
            Some(line)
        } else {
            None
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

fn force_comment_line(_prefix_pattern: &Regex, prefix: &str, line: &str) -> String {
    format!("{}{}", prefix, line)
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

fn uncomment_line(prefix_pattern: &Regex, _prefix: &str, line: &str) -> String {
    prefix_pattern.replace(line, "$head$tail").to_string()
}

fn comment_lines(lines: Lines, pattern: AddressPattern, prefix: &str, mode: CommentingMode) -> Vec<String> {
    let prefix_pattern: Regex = Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();
    let operator = match mode {
        CommentingMode::Comment => comment_line,
        // CommentingMode::Toggle if pattern_is_range => toggle_block,
        CommentingMode::Toggle => toggle_line,
        CommentingMode::Uncomment => uncomment_line,
    };

    let mut output = vec![];
    for (idx, line) in lines.enumerate() {
        let line_number = idx + 1;
        if pattern.matches(line_number, line) {
            output.push(format!("{}", operator(&prefix_pattern, prefix, line)));
        } else {
            output.push(format!("{}", line));
        }
    }
    return output;
}

// fn toggle_block_(prefix_pattern: &Regex, prefix: &str, lines: Lines) -> impl Iterator {
//     // check to see that if the first non-whitespace line is commented
//     "TODO".to_string()
// }
fn toggle_block(prefix_pattern: Regex, prefix: &str, lines: Vec<&str>) -> Vec<String> {
    let mut operator: fn(&Regex, &str, &str) -> String = force_comment_line;
    let mut found_nonblank = false;
    let mut output = vec![];
    let blank = Regex::new(r"^\s*$").unwrap();
    // find first non-whitespace line
    for (idx, line) in lines.iter().enumerate() {
        let line_number = idx + 1;  // FIXME: address patterns and stuff
        println!("{}: {}", line_number, line);
        if blank.is_match(line) {
            println!("blank matched");
            output.push(line.to_string());
            continue;
        } else if !found_nonblank {
            println!("in first found_nonblank branch");
            found_nonblank = true;
            if !prefix_pattern.is_match(line) {
                // Line does not match comment pattern, so we should comment out the whole block
                println!("choosing comment_line");
                operator = force_comment_line;
            } else {
                // Vice versa, first nonblank line is a comment, so uncomment the whole block
                println!("choosing uncomment_line");
                operator = uncomment_line;
            }
        }
        println!("printing line");
        output.push(operator(&prefix_pattern, prefix, line));
    }
    return output;
}

fn get_bin_name() -> OsString {
    let args: Vec<OsString> = std::env::args_os().collect();
    let p = Path::new(OsStr::new(&args[0]));
    p.file_name().unwrap_or(OsStr::new("<UNSET>")).into()
}

fn get_matches<'a>(pattern: &AddressPattern, lines: Vec<&'a str>) -> Vec<&'a str> {
    lines.iter().enumerate()
        //.filter_map(|(idx, l)| pattern.matches(idx+1, l))
        .filter_map(|(idx, l)| pattern.matches_maybe(idx+1, l))
        .collect()
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
    let pattern_is_range = match pattern {
        AddressPattern::LineRange(_, _) | AddressPattern::LineRelativeRange { .. } => true,
        AddressPattern::Line(_) | AddressPattern::RegexPattern(_) | AddressPattern::Compound => false,
    };

    let contents = if let Some(file_path) = args.value_of("INPUT") {
        fs::read_to_string(file_path).expect("Unable to read file")  // TODO: edit this input file in place
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Unable to read from stdin");
        buffer
    };
    let prefix = args.value_of("comment_prefix").unwrap_or("# ");

    let prefix_pattern: Regex = Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();
    // let operator = match mode {
    //     CommentingMode::Comment => comment_line,
    //     CommentingMode::Toggle if pattern_is_range => toggle_block,
    //     CommentingMode::Toggle => toggle_line,
    //     CommentingMode::Uncomment => uncomment_line,
    // };
    if pattern_is_range {
        // FIXME: pattern is range does not imply toggle comment
        //toggle_block(contents.lines(), pattern, prefix_pattern, prefix);
        let example = vec![
            "a = 1",
            "b = 2",
            "#c = 3",
            "d = 4",
        ];
        let pattern = AddressPattern::RegexPattern(Regex::new(".").unwrap());
        let prefix = "# ";
        let prefix_pattern= Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();
        
        let expected = vec![
            "# a = 1",
            "# b = 2",
            "# #c = 3",
            "# d = 4",
        ];
        let actual = toggle_block(prefix_pattern, prefix, example);
    } else {
        comment_lines(contents.lines(), pattern, prefix, mode);
    }
}

#[cfg(test)]
mod test;
