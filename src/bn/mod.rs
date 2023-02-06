const BN_NUM_ZERO: char = '\u{09e6}';
const BN_NUM_NINE: char = '\u{09ef}';
//const BN_ALPHA_START : char = '\u{0985}';
const BN_RANGE_START: char = '\u{0980}';
const BN_RANGE_END: char = '\u{09fe}';

pub const fn is_bn_num(c: char) -> bool {
    c >= BN_NUM_ZERO && c <= BN_NUM_NINE
}

pub const fn is_bn_char(c: char) -> bool {
    c >= BN_RANGE_START && c <= BN_RANGE_END
}

pub fn parse_bn_num(c: String) -> String {
    let mut result: Vec<char> = vec![];

    for item in c.chars() {
        match item {
            '\u{09e6}' => result.push('0'),
            '\u{09e7}' => result.push('1'),
            '\u{09e8}' => result.push('2'),
            '\u{09e9}' => result.push('3'),
            '\u{09ea}' => result.push('4'),
            '\u{09eb}' => result.push('5'),
            '\u{09ec}' => result.push('6'),
            '\u{09ed}' => result.push('7'),
            '\u{09ee}' => result.push('8'),
            '\u{09ef}' => result.push('9'),
            _ => result.push(item),
        }
    }

    return String::from_iter(result.iter());
}

#[cfg(test)]
mod tests {
    use crate::bn::{is_bn_char, is_bn_num};
    use std::collections::HashMap;

    use super::parse_bn_num;

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
}
