# toggle-comment

toggle-comment is a utility designed around setting or toggling the line-comment
status of lines in plain text files in a do-what-I-mean fashion. It aims for
muscle-memory compatibility with GNU sed.

## Examples

```
$ cat > example.py <<'EOF'
def greet(name):
    # Give salutations
    return f'Hello, {name}!'

print(greet('world'))
EOF
$ toggle-comment '/def/,/return/' example.py
# def greet(name):
#     # Give salutations
#     return f'Hello, {name}!'

print(greet('world'))
$ toggle-comment '1,3' example.py | toggle-comment '4,5!'
def greet(name):
    # Give salutations
    return f'Hello, {name}!'

print(greet('world'))
```

## Caveats

- Regular expression syntax matches the Rust `regex` crate. Notable differences
  are in the (lack of) escapes for special characters, e.g. `/a|b/` vs `/a\|b/`
- Currently unsupported features include:
  - in-place editing of files;
  - multiple file arguments;
  - POSIX sed `M~N` "step-wise" patterns, e.g. `1~3` matching lines 1, 4, 7...;
  - GNU sed `addr,~N` "up-to-multiple", e.g. `10,~7` matching lines 10-14; and
  - non-slash regular expression delimeters, e.g. `\|http://|` (initial
    backslash followed by delimiter);
