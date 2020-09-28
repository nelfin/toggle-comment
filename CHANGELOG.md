# toggle-comment

## [0.5.0] - 2020-09-28
- Add support for empty patterns (matches the whole file)
- Add support for negated patterns (e.g. /regex/!)

## [0.4.0] - 2020-09-21
- Add support for relative (addr,+N) ranges
- Add support for regex ranges (e.g. /start/,/end/ or /start/,+100)

## [0.3.0] - 2020-09-15
- Add support for uncommenting and toggling comment blocks
- Change default commenting behaviour based on binary name (support for symlink aliases)
- Fix line range behaviour to be inclusive intervals to match sed definition

## [0.2.0] - 2020-09-01
- Add support for custom comment prefixes
- Add support for simple regular expression patterns
- Add support for piped input on stdin

## [0.1.0] - 2020-08-31
- Initial release, basic support for adding '#' comments based on line ranges
