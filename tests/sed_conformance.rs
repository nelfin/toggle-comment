use std::process::Command;

#[test]
fn sed_is_present() {
    Command::new("sed").output().expect("Failed to spawn sed");
}

#[test]
fn smoke_test() {
    let mut e = std::env::current_exe().unwrap();
    e.pop(); // bin name
    e.pop(); // deps/
    e.push("toggle-comment");
    Command::new(e).output().expect("Failed to spawn toggle-comment");
}

macro_rules! pattern_test_force_comment {
    ($name:ident, $pattern:expr) => {
        #[test]
        fn $name() {
            let mut e = std::env::current_exe().unwrap();
            e.pop(); // bin name
            e.pop(); // deps/
            e.push("toggle-comment");
            let child = Command::new(e)
                .arg("--mode").arg("comment")  // force commenting to mark lines for checking
                .arg($pattern)
                .arg("examples/poem.txt")
                .output()
                .expect("Failed to start toggle-comment");
            let sed = Command::new("sed")
                .arg(format!(r"{}s/^/# /", $pattern))
                .arg("examples/poem.txt")
                .output()
                .expect("Failed to start sed");

            let actual = String::from_utf8(child.stdout).unwrap();
            let expected = String::from_utf8(sed.stdout).unwrap();
            println!("--- [toggle-comment]\n{}", actual);
            println!("--- [sed]\n{}", expected);
            assert!(actual == expected);  // diff output above is more important
        }
    };
}

pattern_test_force_comment!(single_line, "2");
pattern_test_force_comment!(single_line_range, "3,3");
pattern_test_force_comment!(simple_range, "3,7");
pattern_test_force_comment!(simple_regex, "/you/");
pattern_test_force_comment!(relative_range, "5,+2");
pattern_test_force_comment!(regex_range, "/nobody/,/somebody/");
pattern_test_force_comment!(regex_relative_range, "/banish/,+3");
pattern_test_force_comment!(double_regex_relative_range, "/re/,+2");
pattern_test_force_comment!(regex_absolute_range, "/public/,1");  // only matches line with regex
pattern_test_force_comment!(regex_relative_from_first_match, "/The/,+4");  // should only match 5 lines, second "The" doesn't reset counter
pattern_test_force_comment!(nonmatched_first_address, "1,/nobody/");
pattern_test_force_comment!(matched_first_address, "0,/nobody/"); // GNU extension

pattern_test_force_comment!(negated_single_line, "2!");
pattern_test_force_comment!(negated_single_line_range, "3,3!");
pattern_test_force_comment!(negated_simple_range, "3,7!");
pattern_test_force_comment!(negated_simple_regex, "/you/!");
pattern_test_force_comment!(negated_relative_range, "5,+2!");
pattern_test_force_comment!(negated_regex_range, "/nobody/,/somebody/!");
pattern_test_force_comment!(negated_regex_relative_range, "/banish/,+3!");
pattern_test_force_comment!(negated_double_regex_relative_range, "/re/,+2!");
pattern_test_force_comment!(negated_regex_absolute_range, "/public/,1!");
pattern_test_force_comment!(negated_regex_relative_from_first_match, "/The/,+4!");
pattern_test_force_comment!(negated_nonmatched_first_address, "1,/nobody/!");
pattern_test_force_comment!(negated_matched_first_address, "0,/nobody/!");

pattern_test_force_comment!(empty_pattern, "");
pattern_test_force_comment!(negated_empty_pattern, "!"); // lol
