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
    let actual = toggle_block(prefix_pattern, prefix, example);
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
        "a = 1",
        "b = 2",
        "c = 3",
        "d = 4",
    ];
    let actual = toggle_block(prefix_pattern, prefix, example);
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
    let actual = toggle_block(prefix_pattern, prefix, example);
    assert_eq!(actual, expected);
}

#[test]
fn toggle_comment_unintended_maybe() {
    let example = vec![
        "# TODO: lol this might uncomment everything",
        "def whoops(hello):",
        "    # lots of comments",
        "    # lol",
        "    pass",
    ];

    let prefix = "# ";
    let prefix_pattern= Regex::new(&format!(r"^(?P<head>\s*){}(?P<tail>.*?)$", prefix)).unwrap();

    let expected = vec![
        "TODO: lol this might uncomment everything",
        "def whoops(hello):",
        "    lots of comments",
        "    lol",
        "    pass",
    ];
    let actual = toggle_block(prefix_pattern, prefix, example);
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