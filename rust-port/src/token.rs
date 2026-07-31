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

pub(crate) fn tokenize(input: &str, signed: bool) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if starts_number(&chars, index, signed) {
            let mut number = String::new();

            if signed && matches!(chars[index], '+' | '-') {
                number.push(chars[index]);
                index += 1;
            }

            while index < chars.len() && chars[index].is_ascii_digit() {
                number.push(chars[index]);
                index += 1;
            }

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
        let result = tokenize("file123test", false);

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
        let result = tokenize("file999999999999999999999999", false);

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
        let result = tokenize("value-123test", true);

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
        let result = tokenize("value+123test", true);

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
        let result = tokenize("value-123", false);

        assert_eq!(
            result,
            vec![
                Token::Text("value-".to_string()),
                Token::Number("123".to_string()),
            ]
        );
    }
}
