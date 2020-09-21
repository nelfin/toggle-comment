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

// --------------------------------
// A simplified introduction to vi/ex/ed "address patterns":
//
// N                1-indexed line number
// M,N              a range of lines, 1-indexed inclusive of end
// M,+N             a range specified by a start and a count
// /pattern/        a regular expression

enum AddressComponent {
    Line(usize),            // N
    RegexPattern(Regex),    // /pattern/
    Relative(usize),        // +N
    Step(usize),            // ~N
}

impl AddressComponent {
    fn matches(&self, line_number: usize, line: &str) -> bool {
        match &self {
            AddressComponent::Line(n) => *n == line_number,
            AddressComponent::RegexPattern(re) => re.is_match(line),
            _ => todo!(),
        }
    }
}

enum Address {
    ZeroAddress,
    OneAddress(AddressComponent),
    AddressRange(AddressComponent, AddressComponent),
}

struct AddressPattern {
    pattern: Address,
    negated: bool,
}

#[derive(Debug)]
struct MatchState {
    left_match: Option<usize>,
    right_match: Option<usize>,
}
static EMPTY_STATE: MatchState = MatchState { left_match: None, right_match: None };

impl MatchState {
    fn unchanged(&self) -> Self { MatchState { left_match: self.left_match, right_match: self.right_match } }
    fn match_left(&self, idx: usize) -> Self { MatchState { left_match: Some(idx), right_match: self.right_match } }
    fn match_right(&self, idx: usize) -> Self { MatchState { left_match: self.left_match, right_match: Some(idx) } }
    fn update(&mut self, other: MatchState) {
        self.left_match = other.left_match;
        self.right_match = other.right_match;
    }
}

use {Address::*, AddressComponent::*};
impl AddressPattern {
    fn new_single(addr: AddressComponent) -> AddressPattern {
        AddressPattern { pattern: OneAddress(addr), negated: false }
    }

    fn new_range(start: AddressComponent, end: AddressComponent) -> AddressPattern {
        AddressPattern { pattern: AddressRange(start, end), negated: false }
    }

    fn invert(self) -> AddressPattern {
        AddressPattern { pattern: self.pattern, negated: !self.negated }
    }

    fn is_range(&self) -> bool {
        match &self.pattern {
            AddressRange(_, _) => true,
            _ => false,
        }
    }

    fn matches(&self, line_number: usize, line: &str, state: &MatchState) -> (bool, MatchState) {
        let (is_match, new_state) = match &self.pattern {
            Address::ZeroAddress => (true, state.unchanged()),
            Address::OneAddress(AddressComponent::Relative(_)) => panic!("invalid usage of +N or ~N as first address"),
            Address::OneAddress(AddressComponent::Step(_)) => panic!("invalid usage of +N or ~N as first address"),
            Address::OneAddress(addr) => (addr.matches(line_number, line), state.unchanged()),
            Address::AddressRange(_, _) => self.match_range(line_number, line, state),
        };
        if self.negated { (!is_match, new_state) } else { (is_match, new_state) }
    }

    fn match_range(&self, line_number: usize, line: &str, state: &MatchState) -> (bool, MatchState) {
        assert!(match &self.pattern { Address::AddressRange { .. } => true, _ => false }, "Unexpected type");
        match &self.pattern {
            AddressRange(Line(s), Line(e)) => {
                ((*s..*e+1).contains(&line_number), state.unchanged())
            },
            AddressRange(Line(s), RegexPattern(e)) => {
                match state.right_match {
                    // NOTE: line_number > *s guard captures behaviour with 0,/regex/ addresses
                    None if e.is_match(line) && line_number > *s => (true, MatchState { left_match: None, right_match: Some(line_number) }),
                    None if line_number >= *s => (true, state.unchanged()),
                    _ => (false, state.unchanged()),
                }
            },
            AddressRange(Line(s), Relative(count)) => {
                ((*s..*s+*count+1).contains(&line_number), state.unchanged())
            },
            AddressRange(Line(s), Step(count)) => todo!(),
            AddressRange(RegexPattern(s), Line(e)) => {
                let new_state = if s.is_match(line) { state.match_left(line_number) } else { state.unchanged() };
                (s.is_match(line) || state.left_match.map_or(false, |_last| line_number <= *e), new_state)
            },
            AddressRange(RegexPattern(s), RegexPattern(e)) => {
                let new_state = if e.is_match(line) { state.match_right(line_number) } else { state.unchanged() };
                // Reset end-regex match state when start-regex matches
                let new_state = if s.is_match(line) { MatchState { left_match: Some(line_number), right_match: None } } else { new_state.unchanged() };
                (s.is_match(line) || (state.left_match.is_some() && state.right_match.is_none()), new_state)
            },
            AddressRange(RegexPattern(s), Relative(count)) => {
                match state.left_match {
                    None if s.is_match(line) => (true, MatchState { left_match: Some(line_number), right_match: None }),
                    None => (false, state.unchanged()),
                    Some(last) if line_number > last + count => (false, MatchState { left_match: None, right_match: None }),  // reset
                    Some(_) => (true, state.unchanged()),
                }
            },
            AddressRange(RegexPattern(s), Step(count)) => todo!(),
            _ => unreachable!("Shouldn't have branched into match_range"),
        }
    }
}

// --------------------------------

fn try_parse_component(s: &str) -> Result<AddressComponent, &str> {
    if s.starts_with("/") {
        let x = s.trim_start_matches("/").trim_end_matches("/");
        return Ok(RegexPattern(Regex::new(x).unwrap()));
    }
    if s.starts_with("+") {
        return Ok(Relative(s.parse().map_err(|_| "unable to parse relative range")?));
    } else if let Ok(x) = s.parse() {
        return Ok(Line(x));
    }
    Err("unable to parse component")
}

fn try_parse_pattern(s: &str) -> Result<AddressPattern, &str> {
    let parts: Vec<&str> = s.split(",").take(2).collect();
    // FIXME: error on too many bits instead of ignore
    // if parts.len() > 2 {
    //     return Err("too many bits")
    // }
    if parts.len() == 1 {
        return Ok(AddressPattern::new_single(try_parse_component(parts[0])?));
    } else if parts.len() == 2 {
        let (left, right) = (try_parse_component(parts[0])?, try_parse_component(parts[1])?);
        return Ok(AddressPattern::new_range(left, right));
    }
    Err("unimplemented")
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
    if prefix_pattern.is_match(line) {
        prefix_pattern.replace(line, "$head$tail").to_string()
    } else {
        format!("{}{}", prefix, line)
    }
}

fn uncomment_line(prefix_pattern: &Regex, _prefix: &str, line: &str) -> String {
    prefix_pattern.replace(line, "$head$tail").to_string()
}

fn comment_lines(lines: Lines, pattern: &AddressPattern, prefix: &str, mode: &CommentingMode) -> Vec<String> {
    let prefix_pattern: Regex = Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();
    let operator = match mode {
        CommentingMode::Comment => comment_line,
        CommentingMode::Toggle => toggle_line,
        CommentingMode::Uncomment => uncomment_line,
    };

    let mut output = vec![];
    for (idx, line) in lines.enumerate() {
        let line_number = idx + 1;
        // XXX: shouldn't be tracking MatchState since we are not in block-commenting?
        if pattern.matches(line_number, line, &EMPTY_STATE).0 {
            output.push(format!("{}", operator(&prefix_pattern, prefix, line)));
        } else {
            output.push(format!("{}", line));
        }
    }
    return output;
}

fn get_matches<'a>(pattern: &AddressPattern, lines: &Vec<&'a str>, initial_state: MatchState) -> Vec<(bool, Vec<&'a str>)> {
    let mut i = lines.iter().enumerate()
        .scan(initial_state, |state, (idx, &l)| {
            let (is_match, new_state) = pattern.matches(idx+1, l, &state);
            state.update(new_state);
            Some((is_match, l))
        })
        .peekable();

    let mut retval = vec![];
    while let Some((last, l)) = i.next() {
        let mut v: Vec<&str> = vec![l];
        while let Some(&(matched, l)) = i.peek() {
            if matched != last {
                break;
            }
            v.push(l);
            i.next();
        }
        retval.push((last, v));
    }
    retval
}

fn will_comment<S: AsRef<str>>(prefix_pattern: &Regex, lines: &Vec<S>) -> bool {
    let blank = Regex::new(r"^\s*$").unwrap();
    // Walk once to determine if all-nonblank lines are commented or not
    for line in lines.iter() {
        let line = line.as_ref();
        if blank.is_match(line) {
            continue;
        } else if !prefix_pattern.is_match(line) {
            // Line does not match comment pattern, so we should comment out the whole block
            return true;
        }
    }
    return false;
}

fn comment_block<S: AsRef<str>>(mode: &CommentingMode, prefix_pattern: &Regex, prefix: &str, lines: &Vec<S>) -> Vec<String> {
    let blank = Regex::new(r"^\s*$").unwrap();
    let operator: fn(&Regex, &str, &str) -> String = match mode {
        CommentingMode::Comment => comment_line,
        CommentingMode::Uncomment => uncomment_line,
        CommentingMode::Toggle if will_comment(prefix_pattern, lines) => force_comment_line,
        CommentingMode::Toggle => uncomment_line,  // otherwise
    };
    let mut output = vec![];

    for line in lines.iter() {
        let line = line.as_ref();
        if blank.is_match(line) {
            output.push(line.to_string());
            continue;
        }
        output.push(operator(&prefix_pattern, prefix, line));
    }
    return output;
}

fn get_bin_name() -> OsString {
    let args: Vec<OsString> = std::env::args_os().collect();
    let p = Path::new(OsStr::new(&args[0]));
    p.file_name().unwrap_or(OsStr::new("<UNSET>")).into()
}

macro_rules! printlines {
    ($lines:expr) => {
        for line in $lines {
            println!("{}", line);
        }
    };
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
    let initial_state = EMPTY_STATE.unchanged();

    if pattern.is_range() {
        // TODO: don't collect all these lines
        for (is_match, chunk) in get_matches(&pattern, &contents.lines().collect(), initial_state) {
            if is_match {
                printlines!(comment_block(&mode, &prefix_pattern, prefix, &chunk));
            } else {
                printlines!(chunk);
            }
        }
    } else {
        // FIXME: consolidate, assumptions have changed about block/non-block
        //
        // <previous-comment>
        // NOTE: on force-comment or force-uncomment, the per-line behaviour and
        // block behaviour is the same, hence we do not branch on pattern_is_range
        // </previous-comment>
        printlines!(comment_lines(contents.lines(), &pattern, prefix, &mode));
    }
}

#[cfg(test)]
mod test;
