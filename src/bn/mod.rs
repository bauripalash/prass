const BN_NUM_ZERO: char = '\u{09e6}';
const BN_NUM_NINE: char = '\u{09ef}';

pub const fn is_bn_num(c: char) -> bool {
    c >= BN_NUM_ZERO && c <= BN_NUM_NINE
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
