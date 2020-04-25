# show-my-errors

![License](https://img.shields.io/crates/l/show-my-errors?style=flat-square)
![Version](https://img.shields.io/crates/v/show-my-errors?style=flat-square)
![Build status](https://img.shields.io/github/workflow/status/GoldsteinE/show-my-errors/Build%20%26%20test?style=flat-square)

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


#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
