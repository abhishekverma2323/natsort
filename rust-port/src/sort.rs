use crate::locale::locale_sort_key;
use crate::options::SortOptions;
use crate::path::path_components;
use crate::separator::NUM_AFTER_SEPARATOR;
use crate::text::{prepare_input, transform_text_component};
use crate::token::{Token, tokenize};

use std::cmp::Ordering;

#[derive(Debug)]
struct NumericParts {
    negative: bool,
    digits: String,
    decimal_position: i64,
}

#[derive(Debug)]
enum CachedToken {
    Text {
        value: String,
        locale_key: Option<Vec<u8>>,
    },
    Number(NumericParts),
}

#[derive(Debug)]
struct CachedStringKey {
    components: Vec<Vec<CachedToken>>,
}

fn split_sign(number: &str) -> (bool, &str) {
    if let Some(value) = number.strip_prefix('-') {
        (true, value)
    } else if let Some(value) = number.strip_prefix('+') {
        (false, value)
    } else {
        (false, number)
    }
}

fn split_exponent(number: &str) -> (&str, i64) {
    if let Some(index) = number.find(['e', 'E']) {
        let mantissa = &number[..index];
        let exponent = number[index + 1..].parse::<i64>().unwrap_or(0);

        (mantissa, exponent)
    } else {
        (number, 0)
    }
}

fn split_decimal(number: &str) -> (&str, &str) {
    match number.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (number, ""),
    }
}

fn parse_numeric_parts(number: &str) -> NumericParts {
    let (negative, unsigned) = split_sign(number);
    let (mantissa, exponent) = split_exponent(unsigned);
    let (integer, fraction) = split_decimal(mantissa);

    let combined = format!("{integer}{fraction}");
    let leading_zeros = combined.bytes().take_while(|digit| *digit == b'0').count();

    let mut digits = combined[leading_zeros..].to_string();

    if digits.is_empty() {
        digits.push('0');
    } else {
        while digits.ends_with('0') {
            digits.pop();
        }
    }

    let decimal_position = integer.len() as i64 - leading_zeros as i64 + exponent;

    NumericParts {
        negative: negative && digits != "0",
        digits,
        decimal_position,
    }
}

fn compare_digit_sequences(left: &NumericParts, right: &NumericParts) -> Ordering {
    if left.digits == "0" && right.digits == "0" {
        return Ordering::Equal;
    }

    let position_ordering = left.decimal_position.cmp(&right.decimal_position);

    if position_ordering != Ordering::Equal {
        return position_ordering;
    }

    let max_length = left.digits.len().max(right.digits.len());
    let left_bytes = left.digits.as_bytes();
    let right_bytes = right.digits.as_bytes();

    for index in 0..max_length {
        let left_digit = left_bytes.get(index).copied().unwrap_or(b'0');
        let right_digit = right_bytes.get(index).copied().unwrap_or(b'0');

        match left_digit.cmp(&right_digit) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }

    Ordering::Equal
}

fn compare_numeric_parts(left: &NumericParts, right: &NumericParts) -> Ordering {
    match (left.negative, right.negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,

        (true, true) => compare_digit_sequences(left, right).reverse(),

        (false, false) => compare_digit_sequences(left, right),
    }
}

pub(crate) fn compare_numeric_strings(left: &str, right: &str) -> Ordering {
    let left_parts = parse_numeric_parts(left);
    let right_parts = parse_numeric_parts(right);

    compare_numeric_parts(&left_parts, &right_parts)
}

fn compare_tokens(left: &[Token], right: &[Token], options: SortOptions) -> Ordering {
    for (left_token, right_token) in left.iter().zip(right.iter()) {
        let ordering = match (left_token, right_token) {
            (Token::Number(a), Token::Number(b)) => compare_numeric_strings(a, b),
            (Token::Text(a), Token::Text(b)) => {
                if options.locale_alpha {
                    locale_sort_key(a, options.locale_profile)
                        .cmp(&locale_sort_key(b, options.locale_profile))
                } else {
                    a.cmp(b)
                }
            }
            (Token::Text(_), Token::Number(_)) => Ordering::Less,
            (Token::Number(_), Token::Text(_)) => Ordering::Greater,
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left.len().cmp(&right.len())
}

pub(crate) fn transform_text_tokens(mut tokens: Vec<Token>, options: SortOptions) -> Vec<Token> {
    for token in &mut tokens {
        if let Token::Text(text) = token {
            *text = transform_text_component(text, options);
        }
    }

    tokens
}

pub(crate) fn string_key_tokens(input: &str, options: SortOptions) -> Vec<Token> {
    let prepared = prepare_input(input, options);

    let tokens = tokenize(&prepared, options.signed, options.float, options.no_exp);

    let mut tokens = transform_text_tokens(tokens, options);

    if matches!(tokens.first(), Some(Token::Number(_))) {
        let prefix = if options.num_after {
            NUM_AFTER_SEPARATOR
        } else {
            ""
        };

        tokens.insert(0, Token::Text(prefix.to_string()));
    }

    tokens
}

pub(crate) fn string_key_components(input: &str, options: SortOptions) -> Vec<Vec<Token>> {
    if options.path {
        path_components(input)
            .iter()
            .map(|component| string_key_tokens(component, options))
            .collect()
    } else {
        vec![string_key_tokens(input, options)]
    }
}

fn cache_token(token: Token, options: SortOptions) -> CachedToken {
    match token {
        Token::Text(value) => {
            let locale_key = options
                .locale_alpha
                .then(|| locale_sort_key(&value, options.locale_profile));

            CachedToken::Text { value, locale_key }
        }
        Token::Number(number) => CachedToken::Number(parse_numeric_parts(&number)),
    }
}

fn cached_string_key(input: &str, options: SortOptions) -> CachedStringKey {
    let components: Vec<Vec<CachedToken>> = string_key_components(input, options)
        .into_iter()
        .map(|tokens| {
            tokens
                .into_iter()
                .map(|token| cache_token(token, options))
                .collect()
        })
        .collect();

    CachedStringKey { components }
}

fn compare_cached_tokens(left: &[CachedToken], right: &[CachedToken]) -> Ordering {
    for (left_token, right_token) in left.iter().zip(right.iter()) {
        let ordering = match (left_token, right_token) {
            (
                CachedToken::Text {
                    value: left_value,
                    locale_key: left_locale_key,
                },
                CachedToken::Text {
                    value: right_value,
                    locale_key: right_locale_key,
                },
            ) => match (left_locale_key, right_locale_key) {
                (Some(left_key), Some(right_key)) => left_key.cmp(right_key),
                _ => left_value.cmp(right_value),
            },

            (CachedToken::Number(left_number), CachedToken::Number(right_number)) => {
                compare_numeric_parts(left_number, right_number)
            }

            (CachedToken::Text { .. }, CachedToken::Number(_)) => Ordering::Less,
            (CachedToken::Number(_), CachedToken::Text { .. }) => Ordering::Greater,
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left.len().cmp(&right.len())
}

fn compare_cached_string_keys(left: &CachedStringKey, right: &CachedStringKey) -> Ordering {
    for (left_component, right_component) in left.components.iter().zip(right.components.iter()) {
        let ordering = compare_cached_tokens(left_component, right_component);

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left.components.len().cmp(&right.components.len())
}

fn compare_natural_strings(left: &str, right: &str, options: SortOptions) -> Ordering {
    let left_tokens = string_key_tokens(left, options);
    let right_tokens = string_key_tokens(right, options);

    compare_tokens(&left_tokens, &right_tokens, options)
}
fn compare_path_strings(left: &str, right: &str, options: SortOptions) -> Ordering {
    let left_components = path_components(left);
    let right_components = path_components(right);

    for (left_component, right_component) in left_components.iter().zip(right_components.iter()) {
        let ordering = compare_natural_strings(left_component, right_component, options);

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left_components.len().cmp(&right_components.len())
}

pub(crate) fn compare_strings_with_options(
    left: &str,
    right: &str,
    options: SortOptions,
) -> Ordering {
    if options.path {
        compare_path_strings(left, right, options)
    } else {
        compare_natural_strings(left, right, options)
    }
}

pub fn natsorted<T>(items: &[T]) -> Vec<T>
where
    T: AsRef<str> + Clone,
{
    natsorted_with_options(items, SortOptions::default())
}

pub fn natsorted_with_options<T>(items: &[T], options: SortOptions) -> Vec<T>
where
    T: AsRef<str> + Clone,
{
    let mut result = items.to_vec();

    if options.presort {
        result.sort_by(|left, right| {
            let ordering = left.as_ref().cmp(right.as_ref());

            if options.reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    let mut keyed_items: Vec<(CachedStringKey, T)> = result
        .into_iter()
        .map(|item| {
            let key = cached_string_key(item.as_ref(), options);
            (key, item)
        })
        .collect();

    keyed_items.sort_by(|(left_key, _), (right_key, _)| {
        let ordering = compare_cached_string_keys(left_key, right_key);

        if options.reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });

    keyed_items.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_file_names_naturally() {
        let input = vec!["file10", "file2", "file1"];

        let result = natsorted(&input);

        assert_eq!(result, vec!["file1", "file2", "file10"]);
    }

    #[test]
    fn sorts_multiple_numeric_components() {
        let input = vec!["version1.10", "version1.2", "version1.9"];

        let result = natsorted(&input);

        assert_eq!(result, vec!["version1.2", "version1.9", "version1.10"]);
    }

    #[test]
    fn sorts_plain_text() {
        let input = vec!["banana", "apple", "cherry"];

        let result = natsorted(&input);

        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn sorts_numbers_larger_than_u64() {
        let input = vec![
            "file999999999999999999999999",
            "file20",
            "file18446744073709551616",
        ];

        let result = natsorted(&input);

        assert_eq!(
            result,
            vec![
                "file20",
                "file18446744073709551616",
                "file999999999999999999999999",
            ]
        );
    }

    #[test]
    fn compares_numbers_with_leading_zeros_by_numeric_value() {
        let input = vec!["file10", "file002", "file2"];

        let result = natsorted(&input);

        assert_eq!(result, vec!["file002", "file2", "file10"]);
    }

    #[test]
    fn sorts_case_insensitively() {
        let input = vec!["File10", "file2", "FILE1"];

        let options = SortOptions::new().ignore_case(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(result, vec!["FILE1", "file2", "File10"]);
    }

    #[test]
    fn sorts_in_reverse_order() {
        let input = vec!["file1", "file10", "file2"];

        let options = SortOptions::new().reverse(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(result, vec!["file10", "file2", "file1"]);
    }

    #[test]
    fn sorts_signed_integers() {
        let input = vec!["value5", "value-2", "value1", "value-10"];

        let options = SortOptions::new().signed(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(result, vec!["value-10", "value-2", "value1", "value5"]);
    }

    #[test]
    fn sorts_explicit_positive_numbers() {
        let input = vec!["value+10", "value+2", "value-1"];

        let options = SortOptions::new().signed(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(result, vec!["value-1", "value+2", "value+10"]);
    }

    #[test]
    fn sorts_very_large_negative_integers() {
        let input = vec![
            "value-20",
            "value-999999999999999999999999",
            "value-18446744073709551616",
        ];

        let options = SortOptions::new().signed(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec![
                "value-999999999999999999999999",
                "value-18446744073709551616",
                "value-20",
            ]
        );
    }

    #[test]
    fn keeps_unsigned_behavior_by_default() {
        let input = vec!["value-10", "value-2"];

        let result = natsorted(&input);

        assert_eq!(result, vec!["value-2", "value-10"]);
    }

    #[test]
    fn sorts_decimal_numbers_in_float_mode() {
        let input = vec!["value1.5", "value1.25", "value10.01", "value2.0"];

        let options = SortOptions::new().float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec!["value1.25", "value1.5", "value2.0", "value10.01"]
        );
    }

    #[test]
    fn treats_equivalent_decimal_values_as_equal() {
        assert_eq!(compare_numeric_strings("1.5", "1.50"), Ordering::Equal);
        assert_eq!(compare_numeric_strings("001.5000", "1.5"), Ordering::Equal);
    }

    #[test]
    fn sorts_signed_decimal_numbers() {
        let input = vec!["value1.5", "value-2.25", "value-10.5", "value0.25"];

        let options = SortOptions::new().signed(true).float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec!["value-10.5", "value-2.25", "value0.25", "value1.5"]
        );
    }

    #[test]
    fn sorts_high_precision_decimal_numbers() {
        let input = vec![
            "value1.000000000000000000000002",
            "value1.000000000000000000000001",
            "value1.000000000000000000000010",
        ];

        let options = SortOptions::new().float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec![
                "value1.000000000000000000000001",
                "value1.000000000000000000000002",
                "value1.000000000000000000000010",
            ]
        );
    }

    #[test]
    fn compares_signed_zero_values_as_equal() {
        assert_eq!(compare_numeric_strings("-0.0", "0"), Ordering::Equal);
        assert_eq!(compare_numeric_strings("+0.000", "-0"), Ordering::Equal);
    }

    #[test]
    fn sorts_scientific_notation() {
        let input = vec!["value1e3", "value2.5e2", "value4.2e-3", "value1", "value10"];

        let options = SortOptions::new().float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec!["value4.2e-3", "value1", "value10", "value2.5e2", "value1e3",]
        );
    }

    #[test]
    fn compares_equivalent_scientific_values() {
        assert_eq!(compare_numeric_strings("1e3", "1000"), Ordering::Equal);
        assert_eq!(compare_numeric_strings("1.5e2", "150"), Ordering::Equal);
        assert_eq!(compare_numeric_strings("0.001e3", "1"), Ordering::Equal);
    }

    #[test]
    fn sorts_negative_scientific_notation() {
        let input = vec!["value-1e2", "value-2.5e3", "value-4e-2", "value1"];

        let options = SortOptions::new().signed(true).float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec!["value-2.5e3", "value-1e2", "value-4e-2", "value1",]
        );
    }

    #[test]
    fn sorts_scientific_values_with_positive_exponents() {
        let input = vec!["value1E+2", "value5e1", "value2E+3"];

        let options = SortOptions::new().float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(result, vec!["value5e1", "value1E+2", "value2E+3"]);
    }

    #[test]
    fn handles_high_precision_scientific_values() {
        let input = vec![
            "value1.000000000000000000000002e10",
            "value1.000000000000000000000001e10",
            "value9.9e9",
        ];

        let options = SortOptions::new().float(true);
        let result = natsorted_with_options(&input, options);

        assert_eq!(
            result,
            vec![
                "value9.9e9",
                "value1.000000000000000000000001e10",
                "value1.000000000000000000000002e10",
            ]
        );
    }

    #[test]
    fn sorts_arabic_indic_digits() {
        let input = vec!["file١٠", "file٢", "file١"];

        assert_eq!(natsorted(&input), vec!["file١", "file٢", "file١٠"]);
    }

    #[test]
    fn sorts_devanagari_digits() {
        let input = vec!["file१०", "file२", "file१"];

        assert_eq!(natsorted(&input), vec!["file१", "file२", "file१०"]);
    }

    #[test]
    fn sorts_fullwidth_digits() {
        let input = vec!["file１０", "file２", "file１"];

        assert_eq!(natsorted(&input), vec!["file１", "file２", "file１０"]);
    }

    #[test]
    fn sorts_mixed_unicode_digit_scripts() {
        let input = vec!["file10", "file٢", "file३", "file１"];

        assert_eq!(
            natsorted(&input),
            vec!["file１", "file٢", "file३", "file10"]
        );
    }

    #[test]
    fn sorts_signed_unicode_integers() {
        let input = vec!["value-१०", "value२", "value-१"];
        let options = SortOptions::new().signed(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value-१०", "value-१", "value२"]
        );
    }

    #[test]
    fn sorts_unicode_decimal_values() {
        let input = vec!["value١.٥", "value١.٢٥", "value٢.٠"];
        let options = SortOptions::new().float(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value١.٢٥", "value١.٥", "value٢.٠"]
        );
    }

    #[test]
    fn sorts_basic_file_paths() {
        let input = vec!["folder/file10.txt", "folder/file2.txt", "folder/file1.txt"];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["folder/file1.txt", "folder/file2.txt", "folder/file10.txt",]
        );
    }

    #[test]
    fn sorts_nested_directory_paths() {
        let input = vec![
            "folder10/file1.txt",
            "folder2/file10.txt",
            "folder2/file2.txt",
        ];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "folder2/file2.txt",
                "folder2/file10.txt",
                "folder10/file1.txt",
            ]
        );
    }

    #[test]
    fn sorts_paths_with_file_extensions() {
        let input = vec!["file10.tar.gz", "file2.txt", "file1.tar.gz", "file10.txt"];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["file1.tar.gz", "file2.txt", "file10.tar.gz", "file10.txt",]
        );
    }

    #[test]
    fn sorts_relative_paths() {
        let input = vec!["./folder10/file1", "./folder2/file10", "./folder2/file2"];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["./folder2/file2", "./folder2/file10", "./folder10/file1",]
        );
    }

    #[test]
    fn sorts_hidden_file_paths() {
        let input = vec![".file10", ".file2", ".file1", "file1"];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![".file1", ".file2", ".file10", "file1"]
        );
    }

    #[test]
    fn sorts_parent_directory_before_child_path() {
        let input = vec!["folder10/", "folder2/file1", "folder2/", "folder1/"];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["folder1/", "folder2/", "folder2/file1", "folder10/",]
        );
    }

    #[test]
    fn sorts_windows_style_paths() {
        let input = vec![
            r"folder10\file1.txt",
            r"folder2\file10.txt",
            r"folder2\file2.txt",
        ];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                r"folder2\file2.txt",
                r"folder2\file10.txt",
                r"folder10\file1.txt",
            ]
        );
    }

    #[test]
    fn sorts_paths_with_multiple_numeric_components() {
        let input = vec![
            "release1/version10/file2.txt",
            "release1/version2/file10.txt",
            "release1/version2/file2.txt",
        ];
        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "release1/version2/file2.txt",
                "release1/version2/file10.txt",
                "release1/version10/file2.txt",
            ]
        );
    }

    #[test]
    fn sorts_floats_with_leading_decimal_points() {
        let input = vec!["value.56", "value.5", "value.125", "value1"];

        let options = SortOptions::new().float(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value.125", "value.5", "value.56", "value1"]
        );
    }

    #[test]
    fn sorts_signed_floats_with_leading_decimal_points() {
        let input = vec!["value-.56", "value.5", "value-.125", "value1"];

        let options = SortOptions::new().float(true).signed(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value-.56", "value-.125", "value.5", "value1"]
        );
    }

    #[test]
    fn sorts_floats_with_trailing_decimal_points() {
        let input = vec!["value51.", "value5.", "value10.", "value2."];

        let options = SortOptions::new().float(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value2.", "value5.", "value10.", "value51."]
        );
    }

    #[test]
    fn sorts_signed_floats_with_trailing_decimal_points() {
        let input = vec!["value-51.", "value5.", "value-10.", "value2."];

        let options = SortOptions::new().float(true).signed(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value-51.", "value-10.", "value2.", "value5."]
        );
    }

    #[test]
    fn ignores_exponents_when_no_exp_is_enabled() {
        let input = vec!["value5.034e1", "value50", "value5.5e2", "value5.25"];

        let options = SortOptions::new().float(true).no_exp(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value5.034e1", "value5.25", "value5.5e2", "value50",]
        );
    }

    #[test]
    fn ignores_exponents_for_signed_floats() {
        let input = vec!["value-5.034e1", "value-50", "value5.5e2", "value5.25"];

        let options = SortOptions::new().float(true).signed(true).no_exp(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value-50", "value-5.034e1", "value5.25", "value5.5e2",]
        );
    }

    #[test]
    fn sorts_valid_and_invalid_exponents_like_python() {
        let input = vec![
            "value1e",
            "value1e+",
            "value1e-",
            "value1e2",
            "value1e+2",
            "value1e-2",
        ];

        let options = SortOptions::new().float(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "value1e-2",
                "value1e",
                "value1e+",
                "value1e-",
                "value1e2",
                "value1e+2",
            ]
        );
    }

    #[test]
    fn presorts_equivalent_numeric_values() {
        let input = vec!["a1", "a1.45", "a01", "a1.4500"];

        let options = SortOptions::new().float(true).presort(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["a01", "a1", "a1.45", "a1.4500"]
        );
    }

    #[test]
    fn sorts_non_decimal_unicode_digits() {
        let input = vec!["value②", "value①", "value10", "value2"];

        assert_eq!(
            natsorted(&input),
            vec!["value①", "value②", "value2", "value10"]
        );
    }

    #[test]
    fn sorts_broader_unicode_numeric_characters_in_float_mode() {
        let input = vec!["valueⅡ", "value⅓", "value2", "value1"];
        let options = SortOptions::new().float(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value⅓", "value1", "valueⅡ", "value2"]
        );
    }

    #[test]
    fn sorts_mixed_unicode_numeric_characters() {
        let input = vec!["value١٠", "value②", "value३", "value1"];
        let options = SortOptions::new().float(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["value1", "value②", "value३", "value١٠"]
        );
    }

    #[test]
    fn sorts_complex_filesystem_paths_like_python() {
        let input = vec![
            "/p/Folder (10)/file.tar.gz",
            "/p/Folder (1)/file (1).tar.gz",
            "/p/Folder/file.x1.9.tar.gz",
            "/p/Folder (1)/file.tar.gz",
            "/p/Folder/file.x1.10.tar.gz",
        ];

        let options = SortOptions::new().float(true).path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "/p/Folder/file.x1.10.tar.gz",
                "/p/Folder/file.x1.9.tar.gz",
                "/p/Folder (1)/file.tar.gz",
                "/p/Folder (1)/file (1).tar.gz",
                "/p/Folder (10)/file.tar.gz",
            ]
        );
    }

    #[test]
    fn sorts_path_extension_regression_case() {
        let input = vec![
            "Try.Me.Bug - 09 - One.Two.Three.[text].mkv",
            "Try.Me.Bug - 07 - One.Two.5.[text].mkv",
            "Try.Me.Bug - 08 - One.Two.Three[text].mkv",
        ];

        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "Try.Me.Bug - 07 - One.Two.5.[text].mkv",
                "Try.Me.Bug - 08 - One.Two.Three[text].mkv",
                "Try.Me.Bug - 09 - One.Two.Three.[text].mkv",
            ]
        );
    }

    #[test]
    fn path_mode_separates_version_from_extensions() {
        let input = vec!["file.x1.9.tar.gz", "file.x1.10.tar.gz", "file.x1.2.tar.gz"];

        let options = SortOptions::new().float(true).path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["file.x1.10.tar.gz", "file.x1.2.tar.gz", "file.x1.9.tar.gz",]
        );
    }

    #[test]
    fn sorts_rooted_paths_naturally() {
        let input = vec![
            "/folder10/file.txt",
            "/folder2/file.txt",
            "/folder1/file.txt",
        ];

        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "/folder1/file.txt",
                "/folder2/file.txt",
                "/folder10/file.txt",
            ]
        );
    }

    #[test]
    fn sorts_parent_paths_before_children() {
        let input = vec!["folder2/file10.txt", "folder2", "folder2/file2.txt"];

        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["folder2", "folder2/file2.txt", "folder2/file10.txt",]
        );
    }

    #[test]
    fn combines_path_and_ignore_case_options() {
        let input = vec!["Folder/file10.txt", "folder/File2.txt", "FOLDER/file1.txt"];

        let options = SortOptions::new().path(true).ignore_case(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["FOLDER/file1.txt", "folder/File2.txt", "Folder/file10.txt",]
        );
    }

    macro_rules! text_sort_case {
        (
        $name:ident,
        $input:expr,
        $options:expr,
        $expected:expr
    ) => {
            #[test]
            fn $name() {
                let input = $input;

                assert_eq!(natsorted_with_options(&input, $options,), $expected,);
            }
        };
    }

    text_sort_case!(
        sorts_lowercase_first,
        ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
        SortOptions::new().lowercase_first(true),
        vec!["apple", "banana", "corn", "Apple", "Banana", "Corn"]
    );

    text_sort_case!(
        sorts_grouped_letters,
        ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
        SortOptions::new().group_letters(true),
        vec!["Apple", "apple", "Banana", "banana", "Corn", "corn"]
    );

    text_sort_case!(
        sorts_grouped_letters_lowercase_first,
        ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
        SortOptions::new().group_letters(true).lowercase_first(true),
        vec!["apple", "Apple", "banana", "Banana", "corn", "Corn"]
    );

    text_sort_case!(
        sorts_capitals_first,
        ["apple", "Apple", "banana", "Banana", "corn", "Corn"],
        SortOptions::new().capital_first(true),
        vec!["Apple", "Banana", "Corn", "apple", "banana", "corn"]
    );

    text_sort_case!(
        combines_capital_and_lowercase_first,
        ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
        SortOptions::new().capital_first(true).lowercase_first(true),
        vec!["apple", "banana", "corn", "Apple", "Banana", "Corn"]
    );

    text_sort_case!(
        case_folds_sharp_s,
        ["straße10", "STRASSE2", "Strasse1", "strasse3"],
        SortOptions::new().ignore_case(true),
        vec!["Strasse1", "STRASSE2", "strasse3", "straße10"]
    );

    text_sort_case!(
        case_folds_greek_sigma,
        ["Σ10", "ς2", "σ1"],
        SortOptions::new().ignore_case(true),
        vec!["σ1", "ς2", "Σ10"]
    );

    text_sort_case!(
        case_folds_kelvin_sign,
        ["K10", "k2", "K1"],
        SortOptions::new().ignore_case(true),
        vec!["K1", "k2", "K10"]
    );

    text_sort_case!(
        combines_ignore_case_and_lowercase_first,
        ["Apple10", "apple2", "APPLE1", "aPpLe3"],
        SortOptions::new().ignore_case(true).lowercase_first(true),
        vec!["APPLE1", "apple2", "aPpLe3", "Apple10"]
    );

    text_sort_case!(
        combines_group_letters_and_ignore_case,
        ["Apple10", "apple2", "APPLE1", "aPpLe3"],
        SortOptions::new().group_letters(true).ignore_case(true),
        vec!["APPLE1", "apple2", "aPpLe3", "Apple10"]
    );

    text_sort_case!(
        canonically_normalizes_equivalent_text,
        ["café10", "cafe\u{301}2", "café1"],
        SortOptions::new(),
        vec!["café1", "cafe\u{301}2", "café10"]
    );

    text_sort_case!(
        canonically_normalizes_ring_characters,
        ["Å10", "A\u{30A}2", "Å1", "A2"],
        SortOptions::new(),
        vec!["A2", "Å1", "A\u{30A}2", "Å10"]
    );

    text_sort_case!(
        compatibility_normalizes_ligatures,
        ["ﬀile10", "ffile2", "ﬀile1"],
        SortOptions::new().compatibility_normalize(true),
        vec!["ﬀile1", "ffile2", "ﬀile10"]
    );

    text_sort_case!(
        compatibility_normalizes_fullwidth_letters,
        ["Ａ10", "A2", "Ａ1"],
        SortOptions::new().compatibility_normalize(true),
        vec!["Ａ1", "A2", "Ａ10"]
    );

    text_sort_case!(
        compatibility_normalizes_circled_letters,
        ["Ⓐ10", "A2", "Ⓐ1"],
        SortOptions::new().compatibility_normalize(true),
        vec!["Ⓐ1", "A2", "Ⓐ10"]
    );

    text_sort_case!(
        compatibility_normalizes_numbers,
        ["item²", "item2", "item①", "item1"],
        SortOptions::new().compatibility_normalize(true),
        vec!["item①", "item1", "item²", "item2"]
    );

    text_sort_case!(
        sorts_lowercase_first_with_numbers,
        ["A10", "a2", "A1", "a1"],
        SortOptions::new().lowercase_first(true),
        vec!["a1", "a2", "A1", "A10"]
    );

    text_sort_case!(
        sorts_group_letters_with_numbers,
        ["A10", "a2", "A1", "a1"],
        SortOptions::new().group_letters(true),
        vec!["A1", "A10", "a1", "a2"]
    );

    text_sort_case!(
        sorts_paths_lowercase_first,
        [
            "Folder10/File2",
            "folder2/file10",
            "Folder2/file1",
            "folder2/File2",
        ],
        SortOptions::new().path(true).lowercase_first(true),
        vec![
            "folder2/file10",
            "folder2/File2",
            "Folder2/file1",
            "Folder10/File2",
        ]
    );

    text_sort_case!(
        sorts_paths_with_grouped_letters,
        [
            "Folder10/File2",
            "folder2/file10",
            "Folder2/file1",
            "folder2/File2",
        ],
        SortOptions::new().path(true).group_letters(true),
        vec![
            "Folder2/file1",
            "Folder10/File2",
            "folder2/File2",
            "folder2/file10",
        ]
    );

    text_sort_case!(
        sorts_unicode_paths_ignoring_case,
        ["Straße10/File2", "STRASSE2/file10", "strasse2/File1",],
        SortOptions::new().path(true).ignore_case(true),
        vec!["strasse2/File1", "STRASSE2/file10", "Straße10/File2",]
    );

    #[test]
    fn reverse_presort_matches_python_ordering() {
        let input = vec!["a1", "a1.45", "a01", "a1.4500"];

        let options = SortOptions::new().float(true).presort(true).reverse(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["a1.4500", "a1.45", "a1", "a01"]
        );
    }

    #[test]
    fn num_after_places_pure_numbers_after_text() {
        let input = vec!["73", "5039", "Banana", "apple", "corn", "~~~~~~"];

        let options = SortOptions::new().num_after(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["Banana", "apple", "corn", "~~~~~~", "73", "5039",]
        );
    }

    #[test]
    fn num_after_preserves_embedded_numbers() {
        let input = vec!["file10", "10", "file2", "2", "apple"];

        let options = SortOptions::new().num_after(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["apple", "file2", "file10", "2", "10",]
        );
    }

    #[test]
    fn num_after_combines_with_ignore_case() {
        let input = vec!["10", "Apple", "apple", "2", "Banana", "banana"];

        let options = SortOptions::new().num_after(true).ignore_case(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["Apple", "apple", "Banana", "banana", "2", "10",]
        );
    }

    #[test]
    fn num_after_combines_with_group_letters() {
        let input = vec!["10", "Apple", "apple", "2", "Banana", "banana"];

        let options = SortOptions::new().num_after(true).group_letters(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["Apple", "apple", "Banana", "banana", "2", "10",]
        );
    }

    #[test]
    fn num_after_combines_with_path_mode() {
        let input = vec!["10", "folder10/file", "folder2/file", "2", "apple"];

        let options = SortOptions::new().num_after(true).path(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["apple", "folder2/file", "folder10/file", "2", "10",]
        );
    }

    #[test]
    fn reverse_num_after_matches_python() {
        let input = vec!["73", "5039", "Banana", "apple", "corn", "~~~~~~"];

        let options = SortOptions::new().num_after(true).reverse(true);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["5039", "73", "~~~~~~", "corn", "apple", "Banana",]
        );
    }

    #[test]
    fn sorts_english_locale_alpha_like_python() {
        use crate::locale::LocaleProfile;

        let input = vec![
            "Apple", "apple", "Äpfel", "äpfel", "Banana", "banana", "Öl", "Oase", "Zebra",
        ];

        let options = SortOptions::new()
            .locale_alpha(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "äpfel", "Äpfel", "apple", "Apple", "banana", "Banana", "Oase", "Öl", "Zebra",
            ]
        );
    }

    #[test]
    fn sorts_c_locale_alpha_like_python() {
        use crate::locale::LocaleProfile;

        let input = vec![
            "Apple", "apple", "Äpfel", "äpfel", "Banana", "banana", "Öl", "Oase", "Zebra",
        ];

        let options = SortOptions::new()
            .locale_alpha(true)
            .locale_profile(LocaleProfile::C);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "apple", "Apple", "banana", "Banana", "Oase", "Zebra", "äpfel", "Äpfel", "Öl",
            ]
        );
    }

    #[test]
    fn sorts_localized_english_numbers() {
        use crate::locale::LocaleProfile;

        let input = vec!["1,234.50", "12.50", "2.75", "1,000.25", "10.25"];

        let options = SortOptions::new()
            .float(true)
            .locale_numeric(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["2.75", "10.25", "12.50", "1,000.25", "1,234.50"]
        );
    }

    #[test]
    fn sorts_localized_german_numbers() {
        use crate::locale::LocaleProfile;

        let input = vec!["1.234,50", "12,50", "2,75", "1.000,25", "10,25"];

        let options = SortOptions::new()
            .float(true)
            .locale_numeric(true)
            .locale_profile(LocaleProfile::GermanGermany);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["2,75", "10,25", "12,50", "1.000,25", "1.234,50"]
        );
    }

    #[test]
    fn sorts_localized_french_numbers() {
        use crate::locale::LocaleProfile;

        let input = vec![
            "1\u{202F}234,50",
            "12,50",
            "2,75",
            "1\u{202F}000,25",
            "10,25",
        ];

        let options = SortOptions::new()
            .float(true)
            .locale_numeric(true)
            .locale_profile(LocaleProfile::FrenchFrance);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec![
                "2,75",
                "10,25",
                "12,50",
                "1\u{202F}000,25",
                "1\u{202F}234,50",
            ]
        );
    }

    #[test]
    fn combines_locale_alpha_and_numeric_sorting() {
        use crate::locale::LocaleProfile;

        let input = vec!["1,000.50", "Apple2", "10.25", "apple10", "2.50", "Äpfel1"];

        let options = SortOptions::new()
            .float(true)
            .locale(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            natsorted_with_options(&input, options),
            vec!["2.50", "10.25", "1,000.50", "Äpfel1", "apple10", "Apple2"]
        );
    }

    #[test]
    fn default_places_pure_numeric_strings_before_text() {
        let input = vec!["apple", "10", "2", "banana"];

        assert_eq!(natsorted(&input), vec!["2", "10", "apple", "banana"]);
    }
}
