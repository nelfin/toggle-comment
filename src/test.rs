use lazy_static::lazy_static;
use super::*;

macro_rules! matchtest {
    ($name:ident, $fun:expr, $given:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let prefix = "# ";
            let prefix_pattern= Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();
            assert_eq!($fun(&prefix_pattern, prefix, $given), $expected);
        }
    };
}

matchtest!(comment_case1, comment_line, "# simple case one", "# simple case one");
matchtest!(uncomment_case1, uncomment_line, "# simple case one", "simple case one");
matchtest!(toggle_case1, toggle_line, "# simple case one", "simple case one");

matchtest!(comment_case2, comment_line, "simple_case = 2", "# simple_case = 2");
matchtest!(uncomment_case2, uncomment_line, "simple_case = 2", "simple_case = 2");
matchtest!(toggle_case2, toggle_line, "simple_case = 2", "# simple_case = 2");

#[test]
fn toggle_initial_uncomment() {
    let example = vec![
        "a = 1",
        "b = 2",
        "#c = 3",
        "d = 4",
    ];

    let prefix = "# ";
    let prefix_pattern = Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();

    let expected = vec![
        "# a = 1",
        "# b = 2",
        "# #c = 3",
        "# d = 4",
    ];
    let actual = toggle_block(&prefix_pattern, prefix, &example);
    assert_eq!(actual, expected);
}

#[test]
fn toggle_initial_comment() {
    let example = vec![
        "# a = 1",
        "b = 2",
        "# c = 3",
        "d = 4"
    ];

    let prefix = "# ";
    let prefix_pattern= Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();

    let expected = vec![
        "# # a = 1",
        "# b = 2",
        "# # c = 3",
        "# d = 4",
    ];
    let actual = toggle_block(&prefix_pattern, prefix, &example);
    assert_eq!(actual, expected);
}

#[test]
fn toggle_comment_initial_blank() {
    let example = vec![
        "    ",
        "    def foo(self, bar):",
        "        # NOTE: choose better names",
        "        return bar",
    ];

    let prefix = "# ";
    let prefix_pattern= Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();

    let expected = vec![
        "    ",
        "#     def foo(self, bar):",
        "#         # NOTE: choose better names",
        "#         return bar",
    ];
    let actual = toggle_block(&prefix_pattern, prefix, &example);
    assert_eq!(actual, expected);
}

#[test]
fn line_address_only_matches_one() {
    let pattern = AddressPattern::new_single(Line(2));
    let lines = vec![
        "one",
        "two",
        "three",
    ];

    let matches = get_matches(&pattern, &lines);
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[1], (true, vec!["two"]));
}

#[test]
fn line_range_address_matches_block() {
    let pattern = AddressPattern::new_range(Line(2), Line(4));
    let lines = vec![
        "one",
        "two",
        "three",
        "four"
    ];

    let matches = get_matches(&pattern, &lines);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[1], (true, vec!["two", "three", "four"]));
}

lazy_static! {
    static ref PREFIX: Regex = Regex::new(r"^(?P<head>\s*)# (?P<tail>.*?)$").unwrap();
}

#[test]
fn not_all_lines_commented_should_comment() {
    let example = vec![
        "# not all lines commented should comment",
        "abc = 123",
    ];
    assert!(will_comment(&PREFIX, &example));
}

#[test]
fn all_lines_commented_should_uncomment() {
    let example = vec![
        "# all lines commented should uncomment",
        "# abc = 123",
    ];
    assert!(!will_comment(&PREFIX, &example));
}

#[test]
fn blanks_do_not_affect_will_comment() {
    let example1 = vec![
        "all lines uncommented or blank should comment",
        "",
    ];
    assert!(will_comment(&PREFIX, &example1));
    let example2 = vec![
        "# all lines commented or blank should uncomment",
        "",
    ];
    assert!(!will_comment(&PREFIX, &example2));
}


#[test]
fn all_blank_lines_are_unchanged() {
    let expected = vec![
        "",
        "",
    ];
    assert!(!will_comment(&PREFIX, &expected));

    let prefix = "# ";
    let actual = toggle_block(&PREFIX, prefix, &expected);
    assert_eq!(actual, expected);
}

#[test]
fn round_trip() {
    let example = vec![
        "# not all lines commented",
        "abc = 123",
    ];

    let prefix = "# ";


    let expected = vec![
        "# # not all lines commented",
        "# abc = 123",
    ];
    let actual = toggle_block(&PREFIX, prefix, &example);
    assert_eq!(actual, expected);
    assert_eq!(toggle_block(&PREFIX, prefix, &actual), example);
}

use {Address::AddressRange, AddressComponent::*};
macro_rules! address_range {
    ($range:expr) => { AddressPattern { pattern: $range, negated: false }; };
    ($range:expr, $negated:expr) => { AddressPattern { pattern: $range, negated: $negated }; };
}

macro_rules! assert_matches_lines { ($addr:expr, $( $l:expr ),*) => { $( assert!($addr.matches($l, "", &EMPTY_STATE)); )* }; }
macro_rules! assert_not_matches_lines { ($addr:expr, $( $l:expr ),*) => { $( assert!(!$addr.matches($l, "", &EMPTY_STATE)); )* }; }

#[test]
fn zero_address_always_matches() {
    let addr = address_range!(Address::ZeroAddress);
    assert_matches_lines!(addr, 1, 2, 3, 4, 5);
    // TODO: quickcheck/predicate tests
}

#[test]
fn zero_address_invert_never_matches() {
    let addr = address_range!(Address::ZeroAddress, true);
    assert_not_matches_lines!(addr, 1, 2, 3, 4, 5);
}

#[test]
fn one_address_matches_one_line() {
    let addr = address_range!(Address::OneAddress(Line(3)));
    assert_matches_lines!(addr, 3);
    assert_not_matches_lines!(addr, 1, 2, 4, 5);
}

#[test]
fn one_address_inverted_matches_but_one_line() {
    let addr = address_range!(Address::OneAddress(Line(3)), true);
    assert_not_matches_lines!(addr, 3);
    assert_matches_lines!(addr, 1, 2, 4, 5);
}

#[test]
fn matches_range_lines() {
    let addr = address_range!(AddressRange(Line(3), Line(5)));
    assert_matches_lines!(addr, 3, 4, 5);
    assert_not_matches_lines!(addr, 2, 9);
}

#[test]
fn matches_range_lines_invert() {
    let addr = address_range!(AddressRange(Line(3), Line(5)), true);
    assert_not_matches_lines!(addr, 3, 4, 5);
    assert_matches_lines!(addr, 2, 9);
}

#[test]
fn matches_range_relative_lines() {
    let addr = address_range!(AddressRange(Line(3), Relative(5)));
    assert_matches_lines!(addr, 3, 8);
    assert_not_matches_lines!(addr, 2, 9);
}

#[test]
fn matches_range_relative_lines_invert() {
    let addr = address_range!(AddressRange(Line(3), Relative(5)), true);
    assert_not_matches_lines!(addr, 3, 8);
    assert_matches_lines!(addr, 2, 9);
}

#[test]
fn matches_regex_relative_range() {
    let re = Regex::new("foo").unwrap();
    let addr = address_range!(AddressRange(RegexPattern(re), Relative(3)));

    assert!( addr.matches(1, "foo", &EMPTY_STATE));
    let state = MatchState { left_match: Some(1), right_match: None };
    assert!( addr.matches(2, "match", &state));
    assert!( addr.matches(3, "match", &state));
    assert!( addr.matches(4, "match", &state));
    assert!(!addr.matches(5, "un-match", &state));
}

#[test]
fn matches_regex_absolute_range() {
    let re = Regex::new("foo").unwrap();
    let addr = address_range!(AddressRange(RegexPattern(re), Line(4)));

    assert!( addr.matches(1, "foo", &EMPTY_STATE));
    let state = MatchState { left_match: Some(1), right_match: None };
    assert!( addr.matches(2, "match", &state));
    assert!( addr.matches(3, "match", &state));
    assert!( addr.matches(4, "match", &state));
    assert!(!addr.matches(5, "un-match", &state));
}

#[test]
fn matches_regex_empty_absolute_range() {
    let re = Regex::new("foo").unwrap();
    let addr = address_range!(AddressRange(RegexPattern(re), Line(2)));

    assert!(!addr.matches(1, "un-match", &EMPTY_STATE));
    assert!(!addr.matches(2, "un-match", &EMPTY_STATE));
    assert!( addr.matches(3, "foo", &EMPTY_STATE));
    let state = MatchState { left_match: Some(3), right_match: None };
    assert!(!addr.matches(4, "un-match", &state));
    assert!(!addr.matches(5, "un-match", &state));
}

#[test]
fn matches_absolute_regex_end_range() {
    let re = Regex::new("foo").unwrap();
    let addr = address_range!(AddressRange(Line(2), RegexPattern(re)));

    assert!(!addr.matches(1, "un-match", &EMPTY_STATE));
    assert!( addr.matches(2, "match", &EMPTY_STATE));
    assert!( addr.matches(3, "match", &EMPTY_STATE));
    assert!( addr.matches(4, "foo", &EMPTY_STATE));
    let state = MatchState { left_match: None, right_match: Some(4) };
    assert!(!addr.matches(5, "un-match", &state));
}

#[test]
fn matches_double_regex_range() {
    let re1 = Regex::new("foo").unwrap();
    let re2 = Regex::new("bar").unwrap();
    let addr = address_range!(AddressRange(RegexPattern(re1), RegexPattern(re2)));

    assert!(!addr.matches(1, "un-match", &EMPTY_STATE));
    assert!( addr.matches(2, "foo", &EMPTY_STATE));
    let state = MatchState { left_match: Some(2), right_match: None };
    assert!( addr.matches(3, "match", &state));
    assert!( addr.matches(4, "bar", &state));
    let state = MatchState { left_match: Some(2), right_match: Some(4) };
    assert!(!addr.matches(5, "un-match", &state));
}


#[test]
fn matches_double_regex_range_update() {
    let re1 = Regex::new("foo").unwrap();
    let re2 = Regex::new("bar").unwrap();
    let addr = address_range!(AddressRange(RegexPattern(re1), RegexPattern(re2)));

    let (is_match, state) = addr.match_range2(1, "un-match", &EMPTY_STATE);
    assert!(!is_match);
    let (is_match, state) = addr.match_range2(2, "foo", &state);
    assert!(is_match);
    let (is_match, state) = addr.match_range2(3, "match", &state);
    assert!(is_match);
    let (is_match, state) = addr.match_range2(4, "bar", &state);
    assert!(is_match);
    let (is_match, _state) = addr.match_range2(5, "un-match", &state);
    assert!(!is_match);
}

#[test]
fn matches_double_regex_range_with_multiple_matches_on_same_line() {
    let re1 = Regex::new("foo").unwrap();
    let re2 = Regex::new("bar").unwrap();
    let addr = address_range!(AddressRange(RegexPattern(re1), RegexPattern(re2)));

    let (is_match, state) = addr.match_range2(1, "foo", &EMPTY_STATE);
    assert!(is_match, "line 1 failed");
    let (is_match, state) = addr.match_range2(2, "bar", &state);
    assert!(is_match, "line 2 failed");
    let (is_match, state) = addr.match_range2(3, "bar", &state);
    assert!(!is_match, "line 3 failed");
    let (is_match, state) = addr.match_range2(4, "foo bar", &state);
    assert!(is_match, "line 4 failed");
    let (is_match, _state) = addr.match_range2(5, "match", &state);
    assert!(is_match, "line 5 failed");
}
