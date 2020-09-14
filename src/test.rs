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
    let pattern = AddressPattern::Line(2);
    let lines = vec![
        "one",
        "two",
        "three",
    ];

    assert_eq!(get_matches(&pattern, lines), vec!["two"]);
}

#[test]
fn line_range_address_matches_block() {
    let pattern = AddressPattern::LineRange(2, 4);
    let lines = vec![
        "one",
        "two",
        "three",
        "four"
    ];

    assert_eq!(get_matches(&pattern, lines), vec!["two", "three", "four"]);
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
