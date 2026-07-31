use crate::options::SortOptions;
use crate::token::{Token, tokenize};
use std::cmp::Ordering;

fn split_sign(number: &str) -> (bool, &str) {
    if let Some(value) = number.strip_prefix('-') {
        (true, value)
    } else if let Some(value) = number.strip_prefix('+') {
        (false, value)
    } else {
        (false, number)
    }
}

fn split_decimal(number: &str) -> (&str, &str) {
    match number.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (number, ""),
    }
}

fn normalize_integer(integer: &str) -> &str {
    let normalized = integer.trim_start_matches('0');

    if normalized.is_empty() {
        "0"
    } else {
        normalized
    }
}

fn normalize_fraction(fraction: &str) -> &str {
    fraction.trim_end_matches('0')
}

fn is_zero(number: &str) -> bool {
    let (_, unsigned) = split_sign(number);
    let (integer, fraction) = split_decimal(unsigned);

    normalize_integer(integer) == "0" && normalize_fraction(fraction).is_empty()
}

fn compare_fractional_parts(left: &str, right: &str) -> Ordering {
    let max_length = left.len().max(right.len());
    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();

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

fn compare_absolute_values(left: &str, right: &str) -> Ordering {
    let (left_integer, left_fraction) = split_decimal(left);
    let (right_integer, right_fraction) = split_decimal(right);

    let left_integer = normalize_integer(left_integer);
    let right_integer = normalize_integer(right_integer);

    let integer_ordering = left_integer
        .len()
        .cmp(&right_integer.len())
        .then_with(|| left_integer.cmp(right_integer));

    if integer_ordering != Ordering::Equal {
        return integer_ordering;
    }

    compare_fractional_parts(left_fraction, right_fraction)
}

fn compare_numeric_strings(left: &str, right: &str) -> Ordering {
    if is_zero(left) && is_zero(right) {
        return Ordering::Equal;
    }

    let (left_negative, left_unsigned) = split_sign(left);
    let (right_negative, right_unsigned) = split_sign(right);

    match (left_negative, right_negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,

        // Negative values have reversed absolute ordering:
        // -10.5 is less than -2.5.
        (true, true) => compare_absolute_values(left_unsigned, right_unsigned).reverse(),

        (false, false) => compare_absolute_values(left_unsigned, right_unsigned),
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

        let left_tokens = tokenize(&left_value, options.signed, options.float);
        let right_tokens = tokenize(&right_value, options.signed, options.float);

        let ordering = compare_tokens(&left_tokens, &right_tokens);

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
            vec!["value-10.5", "value-2.25", "value0.25", "value1.5",]
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
}
