#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Text(String),
    Number(String),
}

fn starts_number(chars: &[char], index: usize, signed: bool) -> bool {
    let current = chars[index];

    if current.is_ascii_digit() {
        return true;
    }

    signed
        && matches!(current, '+' | '-')
        && chars
            .get(index + 1)
            .is_some_and(|next| next.is_ascii_digit())
}

fn consume_number(chars: &[char], index: &mut usize, signed: bool, float: bool) -> String {
    let mut number = String::new();

    // Consume a leading + or - only when signed mode is enabled.
    if signed && matches!(chars[*index], '+' | '-') {
        number.push(chars[*index]);
        *index += 1;
    }

    // Consume the integer part.
    while *index < chars.len() && chars[*index].is_ascii_digit() {
        number.push(chars[*index]);
        *index += 1;
    }

    // Consume the decimal part only when the dot is followed by a digit.
    if float
        && *index < chars.len()
        && chars[*index] == '.'
        && chars
            .get(*index + 1)
            .is_some_and(|next| next.is_ascii_digit())
    {
        number.push('.');
        *index += 1;

        while *index < chars.len() && chars[*index].is_ascii_digit() {
            number.push(chars[*index]);
            *index += 1;
        }
    }

    // Consume scientific notation, such as e3, E+10, or e-4.
    if float && *index < chars.len() && matches!(chars[*index], 'e' | 'E') {
        let exponent_start = *index;
        let mut exponent_index = exponent_start + 1;

        if exponent_index < chars.len() && matches!(chars[exponent_index], '+' | '-') {
            exponent_index += 1;
        }

        let exponent_digits_start = exponent_index;

        while exponent_index < chars.len() && chars[exponent_index].is_ascii_digit() {
            exponent_index += 1;
        }

        // Consume the exponent only if at least one exponent digit exists.
        if exponent_index > exponent_digits_start {
            while *index < exponent_index {
                number.push(chars[*index]);
                *index += 1;
            }
        }
    }

    number
}

pub(crate) fn tokenize(input: &str, signed: bool, float: bool) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if starts_number(&chars, index, signed) {
            let number = consume_number(&chars, &mut index, signed, float);
            tokens.push(Token::Number(number));
        } else {
            let mut text = String::new();

            while index < chars.len() && !starts_number(&chars, index, signed) {
                text.push(chars[index]);
                index += 1;
            }

            tokens.push(Token::Text(text));
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_text_and_numbers() {
        let result = tokenize("file123test", false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("file".to_string()),
                Token::Number("123".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn preserves_very_large_numbers() {
        let result = tokenize("file999999999999999999999999", false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("file".to_string()),
                Token::Number("999999999999999999999999".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_negative_numbers_when_signed_is_enabled() {
        let result = tokenize("value-123test", true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-123".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_positive_numbers_when_signed_is_enabled() {
        let result = tokenize("value+123test", true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("+123".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn treats_sign_as_text_when_signed_is_disabled() {
        let result = tokenize("value-123", false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value-".to_string()),
                Token::Number("123".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_decimal_as_one_number_in_float_mode() {
        let result = tokenize("value1.25test", false, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1.25".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn keeps_decimal_point_as_text_when_float_mode_is_disabled() {
        let result = tokenize("value1.25", false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1".to_string()),
                Token::Text(".".to_string()),
                Token::Number("25".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_signed_decimal_number() {
        let result = tokenize("value-12.50", true, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-12.50".to_string()),
            ]
        );
    }

    #[test]
    fn does_not_consume_dot_without_fractional_digits() {
        let result = tokenize("value12.test", false, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("12".to_string()),
                Token::Text(".test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_scientific_notation() {
        let result = tokenize("value1.25e3test", false, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1.25e3".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_positive_exponent() {
        let result = tokenize("value2.5E+10", false, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("2.5E+10".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_signed_number_with_negative_exponent() {
        let result = tokenize("value-4.2E-3", true, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-4.2E-3".to_string()),
            ]
        );
    }

    #[test]
    fn leaves_invalid_exponent_as_text() {
        let result = tokenize("value1.5e-test", false, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1.5".to_string()),
                Token::Text("e-test".to_string()),
            ]
        );
    }

    #[test]
    fn leaves_exponent_without_digits_as_text() {
        let result = tokenize("value10e", false, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("10".to_string()),
                Token::Text("e".to_string()),
            ]
        );
    }
}
