use grift_unicode::digit_value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Text(String),
    Number(String),
}

fn is_decimal_digit(character: char) -> bool {
    digit_value(character).is_some()
}

fn push_normalized_digit(output: &mut String, character: char) {
    let value = digit_value(character).expect("character must be a Unicode decimal digit");

    let ascii_digit =
        char::from_digit(value, 10).expect("Unicode decimal digit value must be between 0 and 9");

    output.push(ascii_digit);
}

fn starts_unsigned_number(chars: &[char], index: usize, float: bool) -> bool {
    let current = chars[index];

    if is_decimal_digit(current) {
        return true;
    }

    float
        && current == '.'
        && chars
            .get(index + 1)
            .is_some_and(|next| is_decimal_digit(*next))
}

fn starts_number(chars: &[char], index: usize, signed: bool, float: bool) -> bool {
    if starts_unsigned_number(chars, index, float) {
        return true;
    }

    if !signed || !matches!(chars[index], '+' | '-') {
        return false;
    }

    chars
        .get(index + 1)
        .is_some_and(|_| starts_unsigned_number(chars, index + 1, float))
}

fn consume_exponent(chars: &[char], index: &mut usize, number: &mut String) {
    let exponent_marker_index = *index;
    let mut exponent_index = exponent_marker_index + 1;

    if exponent_index < chars.len() && matches!(chars[exponent_index], '+' | '-') {
        exponent_index += 1;
    }

    let exponent_digits_start = exponent_index;

    while exponent_index < chars.len() && is_decimal_digit(chars[exponent_index]) {
        exponent_index += 1;
    }

    // An exponent marker must be followed by at least one digit.
    // Otherwise, leave the exponent marker and sign as text.
    if exponent_index == exponent_digits_start {
        return;
    }

    number.push(chars[*index]);
    *index += 1;

    if *index < chars.len() && matches!(chars[*index], '+' | '-') {
        number.push(chars[*index]);
        *index += 1;
    }

    while *index < exponent_index {
        push_normalized_digit(number, chars[*index]);
        *index += 1;
    }
}

fn consume_number(
    chars: &[char],
    index: &mut usize,
    signed: bool,
    float: bool,
    no_exp: bool,
) -> String {
    let mut number = String::new();

    if signed && matches!(chars[*index], '+' | '-') {
        number.push(chars[*index]);
        *index += 1;
    }

    let mut integer_digits = 0;

    while *index < chars.len() && is_decimal_digit(chars[*index]) {
        push_normalized_digit(&mut number, chars[*index]);
        *index += 1;
        integer_digits += 1;
    }

    if float && *index < chars.len() && chars[*index] == '.' {
        let has_fractional_digit = chars
            .get(*index + 1)
            .is_some_and(|next| is_decimal_digit(*next));

        // Supported float forms:
        // 1.25
        // .25
        // 25.
        if integer_digits > 0 || has_fractional_digit {
            number.push('.');
            *index += 1;

            while *index < chars.len() && is_decimal_digit(chars[*index]) {
                push_normalized_digit(&mut number, chars[*index]);
                *index += 1;
            }
        }
    }

    if float && !no_exp && *index < chars.len() && matches!(chars[*index], 'e' | 'E') {
        consume_exponent(chars, index, &mut number);
    }

    number
}

pub(crate) fn tokenize(input: &str, signed: bool, float: bool, no_exp: bool) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        if starts_number(&chars, index, signed, float) {
            let number = consume_number(&chars, &mut index, signed, float, no_exp);

            tokens.push(Token::Number(number));
        } else {
            let mut text = String::new();

            while index < chars.len() && !starts_number(&chars, index, signed, float) {
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
        let result = tokenize("file123test", false, false, false);

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
        let result = tokenize("file999999999999999999999999", false, false, false);

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
        let result = tokenize("value-123test", true, false, false);

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
        let result = tokenize("value+123test", true, false, false);

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
        let result = tokenize("value-123", false, false, false);

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
        let result = tokenize("value1.25test", false, true, false);

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
        let result = tokenize("value1.25", false, false, false);

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
        let result = tokenize("value-12.50", true, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-12.50".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_float_with_leading_decimal_point() {
        let result = tokenize("value.56test", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number(".56".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_signed_float_with_leading_decimal_point() {
        let result = tokenize("value-.56test", true, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-.56".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_float_with_trailing_decimal_point() {
        let result = tokenize("value12.test", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("12.".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_signed_float_with_trailing_decimal_point() {
        let result = tokenize("value-12.test", true, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-12.".to_string()),
                Token::Text("test".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_scientific_notation() {
        let result = tokenize("value1.25e3test", false, true, false);

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
        let result = tokenize("value2.5E+10", false, true, false);

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
        let result = tokenize("value-4.2E-3", true, true, false);

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
        let result = tokenize("value1.5e-test", false, true, false);

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
        let result = tokenize("value10e", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("10".to_string()),
                Token::Text("e".to_string()),
            ]
        );
    }

    #[test]
    fn leaves_invalid_positive_exponent_as_text() {
        let result = tokenize("value1e+", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1".to_string()),
                Token::Text("e+".to_string()),
            ]
        );
    }

    #[test]
    fn leaves_invalid_negative_exponent_as_text() {
        let result = tokenize("value1e-", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1".to_string()),
                Token::Text("e-".to_string()),
            ]
        );
    }

    #[test]
    fn leaves_exponent_as_separate_tokens_when_no_exp_is_enabled() {
        let result = tokenize("value5.034e1", false, true, true);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("5.034".to_string()),
                Token::Text("e".to_string()),
                Token::Number("1".to_string()),
            ]
        );
    }

    #[test]
    fn normalizes_unicode_integer_digits() {
        let result = tokenize("file١٢٣", false, false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("file".to_string()),
                Token::Number("123".to_string()),
            ]
        );
    }

    #[test]
    fn normalizes_devanagari_digits() {
        let result = tokenize("file१२३", false, false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("file".to_string()),
                Token::Number("123".to_string()),
            ]
        );
    }

    #[test]
    fn normalizes_fullwidth_digits() {
        let result = tokenize("file１２３", false, false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("file".to_string()),
                Token::Number("123".to_string()),
            ]
        );
    }

    #[test]
    fn normalizes_unicode_decimal_digits() {
        let result = tokenize("value١.٥", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1.5".to_string()),
            ]
        );
    }

    #[test]
    fn normalizes_unicode_exponent_digits() {
        let result = tokenize("value١.٥e٢", false, true, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("1.5e2".to_string()),
            ]
        );
    }

    #[test]
    fn normalizes_signed_unicode_numbers() {
        let result = tokenize("value-१२", true, false, false);

        assert_eq!(
            result,
            vec![
                Token::Text("value".to_string()),
                Token::Number("-12".to_string()),
            ]
        );
    }
}
