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

    if signed && matches!(chars[*index], '+' | '-') {
        number.push(chars[*index]);
        *index += 1;
    }

    while *index < chars.len() && chars[*index].is_ascii_digit() {
        number.push(chars[*index]);
        *index += 1;
    }

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
}
