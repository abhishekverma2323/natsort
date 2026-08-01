use crate::options::SortOptions;
use crate::token::{Token, tokenize};
use std::cmp::Ordering;

#[derive(Debug)]
struct NumericParts {
    negative: bool,
    digits: String,
    decimal_position: i64,
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

fn compare_numeric_strings(left: &str, right: &str) -> Ordering {
    let left_parts = parse_numeric_parts(left);
    let right_parts = parse_numeric_parts(right);

    match (left_parts.negative, right_parts.negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,

        (true, true) => compare_digit_sequences(&left_parts, &right_parts).reverse(),

        (false, false) => compare_digit_sequences(&left_parts, &right_parts),
    }
}

fn compare_tokens(left: &[Token], right: &[Token]) -> Ordering {
    for (left_token, right_token) in left.iter().zip(right.iter()) {
        let ordering = match (left_token, right_token) {
            (Token::Number(a), Token::Number(b)) => compare_numeric_strings(a, b),
            (Token::Text(a), Token::Text(b)) => a.cmp(b),
            (Token::Text(_), Token::Number(_)) => Ordering::Less,
            (Token::Number(_), Token::Text(_)) => Ordering::Greater,
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left.len().cmp(&right.len())
}

fn is_path_separator(character: char) -> bool {
    matches!(character, '/' | '\\')
}

fn compare_natural_strings(left: &str, right: &str, options: SortOptions) -> Ordering {
    let left_tokens = tokenize(left, options.signed, options.float, options.no_exp);

    let right_tokens = tokenize(right, options.signed, options.float, options.no_exp);
    compare_tokens(&left_tokens, &right_tokens)
}

fn compare_path_strings(left: &str, right: &str, options: SortOptions) -> Ordering {
    let left_components: Vec<&str> = left.split(is_path_separator).collect();
    let right_components: Vec<&str> = right.split(is_path_separator).collect();

    for (left_component, right_component) in left_components.iter().zip(right_components.iter()) {
        let ordering = compare_natural_strings(left_component, right_component, options);

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left_components.len().cmp(&right_components.len())
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
        result.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
    }

    result.sort_by(|left, right| {
        let left_value = if options.ignore_case {
            left.as_ref().to_lowercase()
        } else {
            left.as_ref().to_string()
        };

        let right_value = if options.ignore_case {
            right.as_ref().to_lowercase()
        } else {
            right.as_ref().to_string()
        };

        let ordering = if options.path {
            compare_path_strings(&left_value, &right_value, options)
        } else {
            compare_natural_strings(&left_value, &right_value, options)
        };

        if options.reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });

    result
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
}
