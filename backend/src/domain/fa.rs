//! Persian presentation helpers for text the backend composes (missions, advice).

const FA_DIGITS: [char; 10] = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];

/// Replaces ASCII digits with Persian digits and `.` between digits with `٫`.
pub fn digits(input: impl AsRef<str>) -> String {
    let chars: Vec<char> = input.as_ref().chars().collect();
    let mut out = String::with_capacity(chars.len() * 2);
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '0'..='9' => out.push(FA_DIGITS[c as usize - '0' as usize]),
            '.' if i > 0
                && chars[i - 1].is_ascii_digit()
                && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit()) =>
            {
                out.push('٫')
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_digits_and_decimal_separator() {
        assert_eq!(digits("سال 3"), "سال ۳");
        assert_eq!(digits("17.85"), "۱۷٫۸۵");
        assert_eq!(digits("end."), "end.");
    }
}
