use keyspeak::corrections::apply;
use keyspeak::settings::Correction;

fn c(from: &str, to: &str) -> Correction {
    Correction {
        from: from.into(),
        to: to.into(),
    }
}

#[test]
fn basic_replacement() {
    assert_eq!(
        apply("I use clod every day", &[c("clod", "Claude")]),
        "I use Claude every day"
    );
}

#[test]
fn case_insensitive() {
    assert_eq!(
        apply("I use CLOD every day", &[c("clod", "Claude")]),
        "I use Claude every day"
    );
}

#[test]
fn no_partial_word_match() {
    // "rusty" must NOT become "Rusty"
    assert_eq!(apply("rusty nails", &[c("rust", "Rust")]), "rusty nails");
    // But standalone "rust" should be corrected
    assert_eq!(apply("I love rust", &[c("rust", "Rust")]), "I love Rust");
}

#[test]
fn empty_input_returns_empty() {
    assert_eq!(apply("", &[c("hello", "world")]), "");
}

#[test]
fn no_corrections_returns_original() {
    assert_eq!(apply("hello world", &[]), "hello world");
}

#[test]
fn multiple_corrections_in_one_pass() {
    let corrections = vec![c("clod", "Claude"), c("key speak", "KeySpeak")];
    assert_eq!(
        apply("I use clod and key speak daily", &corrections),
        "I use Claude and KeySpeak daily"
    );
}

#[test]
fn correction_at_start_of_string() {
    assert_eq!(
        apply("clod is great", &[c("clod", "Claude")]),
        "Claude is great"
    );
}

#[test]
fn correction_at_end_of_string() {
    assert_eq!(
        apply("I love clod", &[c("clod", "Claude")]),
        "I love Claude"
    );
}
