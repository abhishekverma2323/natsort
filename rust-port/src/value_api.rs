use std::cmp::Ordering;

use crate::decode::{DecodeError, Decoder};
use crate::options::SortOptions;
use crate::value::{NaturalKey, NaturalValue, natsort_key_with_options};

fn apply_reverse(ordering: Ordering, reverse: bool) -> Ordering {
    if reverse {
        ordering.reverse()
    } else {
        ordering
    }
}

fn real_options(mut options: SortOptions) -> SortOptions {
    options.float = true;
    options.signed = true;
    options
}

fn sort_value_indexes(items: &[NaturalValue], options: SortOptions) -> Vec<usize> {
    let keys: Vec<NaturalKey> = items
        .iter()
        .map(|value| natsort_key_with_options(value, options))
        .collect();

    let mut indexes: Vec<usize> = (0..items.len()).collect();

    if options.presort {
        let presort_values: Vec<String> = items.iter().map(NaturalValue::presort_string).collect();

        indexes.sort_by(|left, right| {
            apply_reverse(
                presort_values[*left].cmp(&presort_values[*right]),
                options.reverse,
            )
        });
    }

    indexes.sort_by(|left, right| apply_reverse(keys[*left].cmp(&keys[*right]), options.reverse));

    indexes
}

pub fn index_natsorted_values(items: &[NaturalValue]) -> Vec<usize> {
    index_natsorted_values_with_options(items, SortOptions::new())
}

pub fn index_natsorted_values_with_options(
    items: &[NaturalValue],
    options: SortOptions,
) -> Vec<usize> {
    sort_value_indexes(items, options)
}

pub fn index_realsorted_values(items: &[NaturalValue]) -> Vec<usize> {
    index_realsorted_values_with_options(items, SortOptions::new())
}

pub fn index_realsorted_values_with_options(
    items: &[NaturalValue],
    options: SortOptions,
) -> Vec<usize> {
    sort_value_indexes(items, real_options(options))
}

fn values_from_key<T, F>(items: &[T], mut key: F) -> Vec<NaturalValue>
where
    F: FnMut(&T) -> NaturalValue,
{
    items.iter().map(&mut key).collect()
}

pub fn natsorted_by_value_key<T, F>(items: &[T], key: F) -> Vec<T>
where
    T: Clone,
    F: FnMut(&T) -> NaturalValue,
{
    natsorted_by_value_key_with_options(items, key, SortOptions::new())
}

pub fn natsorted_by_value_key_with_options<T, F>(
    items: &[T],
    key: F,
    options: SortOptions,
) -> Vec<T>
where
    T: Clone,
    F: FnMut(&T) -> NaturalValue,
{
    let values = values_from_key(items, key);

    sort_value_indexes(&values, options)
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

pub fn realsorted_by_value_key<T, F>(items: &[T], key: F) -> Vec<T>
where
    T: Clone,
    F: FnMut(&T) -> NaturalValue,
{
    realsorted_by_value_key_with_options(items, key, SortOptions::new())
}

pub fn realsorted_by_value_key_with_options<T, F>(
    items: &[T],
    key: F,
    options: SortOptions,
) -> Vec<T>
where
    T: Clone,
    F: FnMut(&T) -> NaturalValue,
{
    natsorted_by_value_key_with_options(items, key, real_options(options))
}

pub fn index_natsorted_by_value_key<T, F>(items: &[T], key: F) -> Vec<usize>
where
    F: FnMut(&T) -> NaturalValue,
{
    index_natsorted_by_value_key_with_options(items, key, SortOptions::new())
}

pub fn index_natsorted_by_value_key_with_options<T, F>(
    items: &[T],
    key: F,
    options: SortOptions,
) -> Vec<usize>
where
    F: FnMut(&T) -> NaturalValue,
{
    let values = values_from_key(items, key);
    sort_value_indexes(&values, options)
}

pub fn index_realsorted_by_value_key<T, F>(items: &[T], key: F) -> Vec<usize>
where
    F: FnMut(&T) -> NaturalValue,
{
    index_realsorted_by_value_key_with_options(items, key, SortOptions::new())
}

pub fn index_realsorted_by_value_key_with_options<T, F>(
    items: &[T],
    key: F,
    options: SortOptions,
) -> Vec<usize>
where
    F: FnMut(&T) -> NaturalValue,
{
    index_natsorted_by_value_key_with_options(items, key, real_options(options))
}

pub fn index_natsorted_values_with_decoder(
    items: &[NaturalValue],
    decoder: Decoder,
) -> Result<Vec<usize>, DecodeError> {
    index_natsorted_values_with_decoder_and_options(items, decoder, SortOptions::new())
}

pub fn index_natsorted_values_with_decoder_and_options(
    items: &[NaturalValue],
    decoder: Decoder,
    options: SortOptions,
) -> Result<Vec<usize>, DecodeError> {
    let decoded: Vec<NaturalValue> = items
        .iter()
        .map(|value| decoder.decode(value))
        .collect::<Result<_, _>>()?;

    Ok(sort_value_indexes(&decoded, options))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    fn text(value: &str) -> NaturalValue {
        NaturalValue::from(value)
    }

    fn bytes(value: &[u8]) -> NaturalValue {
        NaturalValue::from(value)
    }

    fn sequence(values: Vec<NaturalValue>) -> NaturalValue {
        NaturalValue::Sequence(values)
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Record {
        id: &'static str,
        key: NaturalValue,
    }

    fn ids(records: &[Record]) -> Vec<&str> {
        records.iter().map(|record| record.id).collect()
    }

    #[test]
    fn returns_mixed_value_indexes() {
        let input = vec![text("a2"), 3.into(), text("a1"), 2.into()];

        assert_eq!(index_natsorted_values(&input), vec![3, 1, 2, 0]);
    }

    #[test]
    fn returns_numeric_string_and_integer_indexes() {
        let input = vec![text("10"), 2.into(), text("1"), 11.into()];

        assert_eq!(index_natsorted_values(&input), vec![2, 1, 0, 3]);
    }

    #[test]
    fn returns_real_mixed_value_indexes() {
        let input = vec![
            text("value5"),
            (-3).into(),
            text("value-2"),
            1.into(),
            text("value1"),
        ];

        assert_eq!(index_realsorted_values(&input), vec![1, 3, 2, 4, 0]);
    }

    #[test]
    fn returns_direct_numeric_indexes() {
        let input = vec![5.1.into(), (-3.0).into(), 5.3.into(), 2.into()];

        assert_eq!(index_natsorted_values(&input), vec![1, 3, 0, 2]);
    }

    #[test]
    fn keeps_equivalent_value_indexes_stable() {
        let input = vec![1.into(), 1.0.into(), 1.0.into(), 2.into()];

        assert_eq!(index_natsorted_values(&input), vec![0, 1, 2, 3]);
    }

    #[test]
    fn returns_infinity_indexes() {
        let input = vec![
            f64::INFINITY.into(),
            5.into(),
            f64::NEG_INFINITY.into(),
            0.into(),
        ];

        assert_eq!(index_natsorted_values(&input), vec![2, 3, 1, 0]);
    }

    #[test]
    fn returns_default_none_and_nan_indexes() {
        let input = vec![
            3.into(),
            NaturalValue::None,
            f64::NAN.into(),
            f64::NEG_INFINITY.into(),
            2.into(),
        ];

        assert_eq!(index_natsorted_values(&input), vec![2, 1, 3, 4, 0]);
    }

    #[test]
    fn returns_nan_last_indexes() {
        let input = vec![
            3.into(),
            NaturalValue::None,
            f64::NAN.into(),
            f64::INFINITY.into(),
            2.into(),
        ];

        let options = SortOptions::new().nan_last(true);

        assert_eq!(
            index_natsorted_values_with_options(&input, options),
            vec![4, 0, 3, 1, 2]
        );
    }

    #[test]
    fn returns_reverse_indexes() {
        let input = vec![text("10"), 2.into(), text("1"), 11.into()];
        let options = SortOptions::new().reverse(true);

        assert_eq!(
            index_natsorted_values_with_options(&input, options),
            vec![3, 0, 1, 2]
        );
    }

    #[test]
    fn value_indexes_support_presort() {
        let input = vec![text("a1"), text("a1.45"), text("a01"), text("a1.4500")];
        let options = SortOptions::new().float(true).presort(true);

        assert_eq!(
            index_natsorted_values_with_options(&input, options),
            vec![2, 0, 1, 3]
        );
    }

    #[test]
    fn value_indexes_support_reverse_presort() {
        let input = vec![text("a1"), text("a1.45"), text("a01"), text("a1.4500")];
        let options = SortOptions::new().float(true).presort(true).reverse(true);

        assert_eq!(
            index_natsorted_values_with_options(&input, options),
            vec![3, 1, 0, 2]
        );
    }

    #[test]
    fn value_indexes_support_empty_input() {
        assert!(index_natsorted_values(&[]).is_empty());
    }

    #[test]
    fn value_indexes_support_single_input() {
        assert_eq!(index_natsorted_values(&[text("file1")]), vec![0]);
    }

    #[test]
    fn returns_nested_sequence_indexes() {
        let input = vec![
            sequence(vec![text("a10"), text("b2")]),
            sequence(vec![text("a2"), text("b10")]),
            sequence(vec![text("a2"), text("b2")]),
        ];

        assert_eq!(index_natsorted_values(&input), vec![2, 1, 0]);
    }

    #[test]
    fn returns_nested_mixed_sequence_indexes() {
        let input = vec![
            sequence(vec![text("a2"), 10.into()]),
            sequence(vec![text("a2"), 2.into()]),
            sequence(vec![text("a1"), 20.into()]),
        ];

        assert_eq!(index_natsorted_values(&input), vec![2, 1, 0]);
    }

    #[test]
    fn sequence_prefixes_sort_shorter_first() {
        let input = vec![
            sequence(vec![text("a2"), text("b1")]),
            sequence(vec![text("a2")]),
            sequence(vec![text("a2"), text("b1"), text("c1")]),
        ];

        assert_eq!(index_natsorted_values(&input), vec![1, 0, 2]);
    }

    #[test]
    fn returns_byte_indexes() {
        let input = vec![bytes(b"a10"), bytes(b"a2"), bytes(b"A1")];

        assert_eq!(index_natsorted_values(&input), vec![2, 0, 1]);
    }

    #[test]
    fn byte_indexes_support_ignore_case() {
        let input = vec![bytes(b"a10"), bytes(b"A2"), bytes(b"a1")];
        let options = SortOptions::new().ignore_case(true);

        assert_eq!(
            index_natsorted_values_with_options(&input, options),
            vec![2, 0, 1]
        );
    }

    #[test]
    fn sorts_records_by_natural_value_key() {
        let input = vec![
            Record {
                id: "ten",
                key: text("file10"),
            },
            Record {
                id: "two",
                key: text("file2"),
            },
            Record {
                id: "one",
                key: text("file1"),
            },
        ];

        let sorted = natsorted_by_value_key(&input, |record| record.key.clone());

        assert_eq!(ids(&sorted), vec!["one", "two", "ten"]);
    }

    #[test]
    fn sorts_records_by_value_key_in_reverse() {
        let input = vec![
            Record {
                id: "ten",
                key: text("file10"),
            },
            Record {
                id: "two",
                key: text("file2"),
            },
            Record {
                id: "one",
                key: text("file1"),
            },
        ];
        let options = SortOptions::new().reverse(true);

        let sorted =
            natsorted_by_value_key_with_options(&input, |record| record.key.clone(), options);

        assert_eq!(ids(&sorted), vec!["ten", "two", "one"]);
    }

    #[test]
    fn real_sorts_records_by_value_key() {
        let input = vec![
            Record {
                id: "five-one",
                key: text("num5.10"),
            },
            Record {
                id: "negative",
                key: text("num-3"),
            },
            Record {
                id: "five-three",
                key: text("num5.3"),
            },
            Record {
                id: "two",
                key: text("num2"),
            },
        ];

        let sorted = realsorted_by_value_key(&input, |record| record.key.clone());

        assert_eq!(
            ids(&sorted),
            vec!["negative", "two", "five-one", "five-three"]
        );
    }

    #[test]
    fn value_key_sort_is_stable() {
        let input = vec![
            Record {
                id: "first",
                key: text("file01"),
            },
            Record {
                id: "second",
                key: text("file1"),
            },
            Record {
                id: "third",
                key: text("file001"),
            },
        ];

        let sorted = natsorted_by_value_key(&input, |record| record.key.clone());

        assert_eq!(ids(&sorted), vec!["first", "second", "third"]);
    }

    #[test]
    fn value_key_sort_supports_nested_keys() {
        let input = vec![
            Record {
                id: "ten",
                key: sequence(vec![text("a2"), 10.into()]),
            },
            Record {
                id: "two",
                key: sequence(vec![text("a2"), 2.into()]),
            },
            Record {
                id: "one",
                key: sequence(vec![text("a1"), 20.into()]),
            },
        ];

        let sorted = natsorted_by_value_key(&input, |record| record.key.clone());

        assert_eq!(ids(&sorted), vec!["one", "two", "ten"]);
    }

    #[test]
    fn value_key_sort_supports_paths() {
        let input = vec![
            Record {
                id: "ten",
                key: text("folder10/file"),
            },
            Record {
                id: "two",
                key: text("folder2/file"),
            },
            Record {
                id: "one",
                key: text("folder1/file"),
            },
        ];
        let options = SortOptions::new().path(true);

        let sorted =
            natsorted_by_value_key_with_options(&input, |record| record.key.clone(), options);

        assert_eq!(ids(&sorted), vec!["one", "two", "ten"]);
    }

    #[test]
    fn value_key_sort_supports_presort() {
        let input = vec![
            Record {
                id: "zero",
                key: text("a1"),
            },
            Record {
                id: "one",
                key: text("a1.45"),
            },
            Record {
                id: "two",
                key: text("a01"),
            },
            Record {
                id: "three",
                key: text("a1.4500"),
            },
        ];
        let options = SortOptions::new().float(true).presort(true);

        let sorted =
            natsorted_by_value_key_with_options(&input, |record| record.key.clone(), options);

        assert_eq!(ids(&sorted), vec!["two", "zero", "one", "three"]);
    }

    #[test]
    fn returns_indexes_by_value_key() {
        let input = vec!["file10", "file2", "file1"];

        assert_eq!(
            index_natsorted_by_value_key(&input, |value| text(value)),
            vec![2, 1, 0]
        );
    }

    #[test]
    fn returns_reverse_indexes_by_value_key() {
        let input = vec!["file10", "file2", "file1"];
        let options = SortOptions::new().reverse(true);

        assert_eq!(
            index_natsorted_by_value_key_with_options(&input, |value| text(value), options,),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn returns_real_indexes_by_value_key() {
        let input = vec!["num5.10", "num-3", "num5.3", "num2"];

        assert_eq!(
            index_realsorted_by_value_key(&input, |value| text(value)),
            vec![1, 3, 0, 2]
        );
    }

    #[test]
    fn key_function_runs_once_per_item() {
        let input = vec!["file10", "file2", "file1"];
        let calls = Cell::new(0);

        let sorted = natsorted_by_value_key(&input, |value| {
            calls.set(calls.get() + 1);
            text(value)
        });

        assert_eq!(sorted, vec!["file1", "file2", "file10"]);
        assert_eq!(calls.get(), input.len());
    }

    #[test]
    fn decoder_indexes_mixed_bytes_and_text() {
        let input = vec![bytes(b"a10"), text("a2"), bytes(b"a1")];

        assert_eq!(
            index_natsorted_values_with_decoder(&input, Decoder::utf8()),
            Ok(vec![2, 1, 0])
        );
    }

    #[test]
    fn decoder_indexes_support_reverse() {
        let input = vec![bytes(b"a10"), text("a2"), bytes(b"a1")];
        let options = SortOptions::new().reverse(true);

        assert_eq!(
            index_natsorted_values_with_decoder_and_options(&input, Decoder::utf8(), options,),
            Ok(vec![0, 1, 2])
        );
    }

    #[test]
    fn decoder_indexes_support_ignore_case() {
        let input = vec![bytes(b"A10"), text("a2"), bytes(b"a1")];
        let options = SortOptions::new().ignore_case(true);

        assert_eq!(
            index_natsorted_values_with_decoder_and_options(&input, Decoder::utf8(), options,),
            Ok(vec![2, 1, 0])
        );
    }

    #[test]
    fn decoder_indexes_propagate_errors() {
        let input = vec![bytes(b"a1"), bytes(&[0xFF])];

        assert!(matches!(
            index_natsorted_values_with_decoder(&input, Decoder::utf8()),
            Err(DecodeError::InvalidUtf8 { .. })
        ));
    }

    #[test]
    fn value_indexes_support_num_after() {
        let input = vec![
            text("73"),
            text("5039"),
            text("Banana"),
            text("apple"),
            text("corn"),
            text("~~~~~~"),
        ];

        let options = SortOptions::new().num_after(true);

        assert_eq!(
            index_natsorted_values_with_options(&input, options,),
            vec![2, 3, 4, 5, 0, 1]
        );
    }
}
