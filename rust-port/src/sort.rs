use crate::options::SortOptions;
use crate::token::{Token, tokenize};
use std::cmp::Ordering;

fn normalize_digits(number: &str) -> &str {
    let normalized = number.trim_start_matches('0');

    if normalized.is_empty() {
        "0"
    } else {
        normalized
    }
}

fn split_sign(number: &str) -> (bool, &str) {
    if let Some(digits) = number.strip_prefix('-') {
        (true, digits)
    } else if let Some(digits) = number.strip_prefix('+') {
        (false, digits)
    } else {
        (false, number)
    }
}

fn compare_absolute_values(left: &str, right: &str) -> Ordering {
    let left_normalized = normalize_digits(left);
    let right_normalized = normalize_digits(right);

    left_normalized
        .len()
        .cmp(&right_normalized.len())
        .then_with(|| left_normalized.cmp(right_normalized))
}

fn compare_numeric_strings(left: &str, right: &str) -> Ordering {
    let (left_negative, left_digits) = split_sign(left);
    let (right_negative, right_digits) = split_sign(right);

    let left_digits = normalize_digits(left_digits);
    let right_digits = normalize_digits(right_digits);

    // -0, +0 and 0 are numerically equal.
    if left_digits == "0" && right_digits == "0" {
        return Ordering::Equal;
    }

    match (left_negative, right_negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,

        // For two negative numbers, the larger absolute value is smaller.
        (true, true) => compare_absolute_values(left_digits, right_digits).reverse(),

        // Normal positive-number comparison.
        (false, false) => compare_absolute_values(left_digits, right_digits),
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

        let left_tokens = tokenize(&left_value, options.signed);
        let right_tokens = tokenize(&right_value, options.signed);

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
}
