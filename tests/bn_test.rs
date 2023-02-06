use pras::bn::{is_bn_char, is_bn_num, parse_bn_num};
use std::collections::HashMap;

#[test]
fn test_parse_bn_num() {
    let test_cases: HashMap<&str, &str> = HashMap::from([
        ("100", "100"),
        ("22", "22"),
        ("১০০", "100"),
        ("২2", "22"),
        ("৯৯", "99"),
        ("৯", "9"),
    ]);

    for (k, v) in test_cases {
        assert_eq!(parse_bn_num(k.to_string()), v.to_string());
    }
}

#[test]
fn test_is_bn_num() {
    let test_cases: HashMap<char, bool> = HashMap::from([
        ('1', false),
        ('2', false),
        ('১', true),
        ('২', true),
        ('৯', true),
        ('০', true),
    ]);

    for (k, v) in test_cases {
        assert_eq!(is_bn_num(k), v);
    }
}

#[test]
fn test_is_bn_char() {
    let test_cases: HashMap<char, bool> = HashMap::from([
        ('1', false),
        ('2', false),
        ('১', true),
        ('২', true),
        ('৯', true),
        ('০', true),
        ('a', false),
        ('r', false),
        ('আ', true),
        ('ন', true),
        ('প', true),
        ('ঁ', true),
    ]);

    for (k, v) in test_cases {
        assert_eq!(is_bn_char(k), v);
    }
}
