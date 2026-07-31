use std::cmp::Ordering;

use crate::token::{Token, tokenize};

fn compare_numeric_strings(left: &str, right: &str) -> Ordering {
    let left_normalized = left.trim_start_matches('0');
    let right_normalized = right.trim_start_matches('0');

    let left_normalized = if left_normalized.is_empty() {
        "0"
    } else {
        left_normalized
    };

    let right_normalized = if right_normalized.is_empty() {
        "0"
    } else {
        right_normalized
    };

    left_normalized
        .len()
        .cmp(&right_normalized.len())
        .then_with(|| left_normalized.cmp(right_normalized))
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
    let mut result = items.to_vec();

    result.sort_by(|left, right| {
        let left_tokens = tokenize(left.as_ref());
        let right_tokens = tokenize(right.as_ref());

        compare_tokens(&left_tokens, &right_tokens)
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
}
