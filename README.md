# show-my-errors

This is a library to display simple errors in colorful, rustc-like way.
It can't show multi-line errors or draw arrows between parts of code, but its interface
is simple and easy to use. If you want something more complex, you probably should use
[annotate-snippets](https://docs.rs/annotate-snippets), which is used by rustc itself.

![example output](/example.png)

### Basic usage
Entry point of this library is `AnnotationList`. You should create it, add some errors
and then use `.show_stderr()` or
`.show_stdout()`
with some `Stylesheet` to display the message.
```rust
let mut list = AnnotationList::new("hello.txt", "Hello world!");
list
    .warning(4..7, "punctuation problem", "you probably forgot a comma")?
    .info(0..0, "consider adding some translations", None)?;
assert_eq!(list.to_string()?, r#"
warning: punctuation problem
  --> hello.txt:1:5
   |
 1 | Hello world!
   |     ^^^ you probably forgot a comma

info: consider adding some translations
  --> hello.txt:1:1
   |
 1 | Hello world!
   |
"#);
```
