#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Text(String),
    Number(String),
}

pub(crate) fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            let mut number = String::new();

            while let Some(&digit) = chars.peek() {
                if digit.is_ascii_digit() {
                    number.push(digit);
                    chars.next();
                } else {
                    break;
                }
            }

            tokens.push(Token::Number(number));
        } else {
            let mut text = String::new();

            while let Some(&current) = chars.peek() {
                if !current.is_ascii_digit() {
                    text.push(current);
                    chars.next();
                } else {
                    break;
                }
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
        let result = tokenize("file123test");

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
        let result = tokenize("file999999999999999999999999");

        assert_eq!(
            result,
            vec![
                Token::Text("file".to_string()),
                Token::Number("999999999999999999999999".to_string()),
            ]
        );
    }
}
