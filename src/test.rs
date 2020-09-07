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
