pub fn is_digit(c: u8) -> bool {
    (c >= b'0') && (c <= b'9')
}

pub fn is_alnum(c: u8) -> bool {
    is_digit(c) || ((c >= b'a') && (c <= b'z')) ||
        ((c >= b'A') && (c <= b'Z'))
}

