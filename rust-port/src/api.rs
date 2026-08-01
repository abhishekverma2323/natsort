use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::options::SortOptions;
use crate::sort::{compare_strings_with_options, natsorted_with_options};

fn apply_reverse(ordering: std::cmp::Ordering, reverse: bool) -> std::cmp::Ordering {
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

fn human_options(mut options: SortOptions) -> SortOptions {
    options.locale_alpha = true;
    options.locale_numeric = true;
    options
}

fn path_options(mut options: SortOptions) -> SortOptions {
    options.path = true;
    options
}

fn sort_indices_by_keys(keys: &[String], options: SortOptions) -> Vec<usize> {
    let mut indexes: Vec<usize> = (0..keys.len()).collect();

    if options.presort {
        indexes
            .sort_by(|left, right| apply_reverse(keys[*left].cmp(&keys[*right]), options.reverse));
    }

    indexes.sort_by(|left, right| {
        let ordering = compare_strings_with_options(&keys[*left], &keys[*right], options);

        apply_reverse(ordering, options.reverse)
    });

    indexes
}

/// Sort values using signed floating-point natural sorting.
pub fn realsorted<T>(items: &[T]) -> Vec<T>
where
    T: AsRef<str> + Clone,
{
    realsorted_with_options(items, SortOptions::new())
}

/// Sort values using real-number semantics plus additional options.
pub fn realsorted_with_options<T>(items: &[T], options: SortOptions) -> Vec<T>
where
    T: AsRef<str> + Clone,
{
    natsorted_with_options(items, real_options(options))
}

/// Sort values using locale-aware alphabetic and numeric rules.
pub fn humansorted<T>(items: &[T]) -> Vec<T>
where
    T: AsRef<str> + Clone,
{
    humansorted_with_options(items, SortOptions::new())
}

/// Sort values using locale-aware rules plus additional options.
pub fn humansorted_with_options<T>(items: &[T], options: SortOptions) -> Vec<T>
where
    T: AsRef<str> + Clone,
{
    natsorted_with_options(items, human_options(options))
}

/// Return the indexes that place the input in natural-sort order.
pub fn index_natsorted<T>(items: &[T]) -> Vec<usize>
where
    T: AsRef<str>,
{
    index_natsorted_with_options(items, SortOptions::new())
}

/// Return naturally sorted indexes using explicit options.
pub fn index_natsorted_with_options<T>(items: &[T], options: SortOptions) -> Vec<usize>
where
    T: AsRef<str>,
{
    let keys: Vec<String> = items.iter().map(|item| item.as_ref().to_string()).collect();

    sort_indices_by_keys(&keys, options)
}

/// Return indexes using signed floating-point sorting.
pub fn index_realsorted<T>(items: &[T]) -> Vec<usize>
where
    T: AsRef<str>,
{
    index_realsorted_with_options(items, SortOptions::new())
}

/// Return real-number sorted indexes with additional options.
pub fn index_realsorted_with_options<T>(items: &[T], options: SortOptions) -> Vec<usize>
where
    T: AsRef<str>,
{
    index_natsorted_with_options(items, real_options(options))
}

/// Return indexes using locale-aware alphabetic and numeric rules.
pub fn index_humansorted<T>(items: &[T]) -> Vec<usize>
where
    T: AsRef<str>,
{
    index_humansorted_with_options(items, SortOptions::new())
}

/// Return locale-aware indexes with additional options.
pub fn index_humansorted_with_options<T>(items: &[T], options: SortOptions) -> Vec<usize>
where
    T: AsRef<str>,
{
    index_natsorted_with_options(items, human_options(options))
}

/// Sort arbitrary values using a string-producing key function.
pub fn natsorted_by_key<'a, T, F, K>(items: &'a [T], key: F) -> Vec<T>
where
    T: Clone,
    F: FnMut(&'a T) -> K,
    K: AsRef<str>,
{
    natsorted_by_key_with_options(items, key, SortOptions::new())
}

/// Sort arbitrary values by a key function and explicit options.
pub fn natsorted_by_key_with_options<'a, T, F, K>(
    items: &'a [T],
    mut key: F,
    options: SortOptions,
) -> Vec<T>
where
    T: Clone,
    F: FnMut(&'a T) -> K,
    K: AsRef<str>,
{
    let keys: Vec<String> = items
        .iter()
        .map(|item| key(item).as_ref().to_string())
        .collect();

    sort_indices_by_keys(&keys, options)
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

/// Sort arbitrary values by a key using real-number semantics.
pub fn realsorted_by_key<'a, T, F, K>(items: &'a [T], key: F) -> Vec<T>
where
    T: Clone,
    F: FnMut(&'a T) -> K,
    K: AsRef<str>,
{
    realsorted_by_key_with_options(items, key, SortOptions::new())
}

/// Sort arbitrary values by a key using real-number semantics
/// and additional options.
pub fn realsorted_by_key_with_options<'a, T, F, K>(
    items: &'a [T],
    key: F,
    options: SortOptions,
) -> Vec<T>
where
    T: Clone,
    F: FnMut(&'a T) -> K,
    K: AsRef<str>,
{
    natsorted_by_key_with_options(items, key, real_options(options))
}

/// Sort arbitrary values by a string key using locale-aware rules.
pub fn humansorted_by_key<'a, T, F, K>(items: &'a [T], key: F) -> Vec<T>
where
    T: Clone,
    F: FnMut(&'a T) -> K,
    K: AsRef<str>,
{
    humansorted_by_key_with_options(items, key, SortOptions::new())
}

/// Sort arbitrary values by a locale-aware key plus additional options.
pub fn humansorted_by_key_with_options<'a, T, F, K>(
    items: &'a [T],
    key: F,
    options: SortOptions,
) -> Vec<T>
where
    T: Clone,
    F: FnMut(&'a T) -> K,
    K: AsRef<str>,
{
    natsorted_by_key_with_options(items, key, human_options(options))
}

/// Sort filesystem paths using natural path-component semantics.
///
/// Both `Path` and `PathBuf` inputs are accepted through `AsRef<Path>`.
/// Original values are cloned into the result. Internally, paths are converted
/// with `Path::to_string_lossy` because the sorting engine is text based.
pub fn natsorted_paths<T>(items: &[T]) -> Vec<T>
where
    T: AsRef<Path> + Clone,
{
    natsorted_paths_with_options(items, SortOptions::new())
}

/// Sort filesystem paths using path semantics plus additional options.
///
/// Path mode is always enabled even when `options.path` is false.
pub fn natsorted_paths_with_options<T>(items: &[T], options: SortOptions) -> Vec<T>
where
    T: AsRef<Path> + Clone,
{
    index_natsorted_paths_with_options(items, options)
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

/// Return indexes that place filesystem paths in natural path order.
pub fn index_natsorted_paths<T>(items: &[T]) -> Vec<usize>
where
    T: AsRef<Path>,
{
    index_natsorted_paths_with_options(items, SortOptions::new())
}

/// Return naturally sorted path indexes using additional options.
///
/// Path mode is always enabled even when `options.path` is false.
pub fn index_natsorted_paths_with_options<T>(items: &[T], options: SortOptions) -> Vec<usize>
where
    T: AsRef<Path>,
{
    let keys: Vec<String> = items
        .iter()
        .map(|item| item.as_ref().to_string_lossy().into_owned())
        .collect();

    sort_indices_by_keys(&keys, path_options(options))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderByIndexError {
    pub position: usize,
    pub index: usize,
    pub input_length: usize,
}

impl fmt::Display for OrderByIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "index {} at position {} is out of bounds for input length {}",
            self.index, self.position, self.input_length,
        )
    }
}

impl Error for OrderByIndexError {}

/// Lazily reorder values using a sequence of indexes.
///
/// Values are cloned only when the iterator advances. Duplicate indexes are
/// supported, matching Python's `order_by_index(..., iter=True)` behaviour.
///
/// # Panics
///
/// Panics when iteration reaches an index outside the input.
pub fn order_by_index_iter<'a, T>(
    items: &'a [T],
    indexes: &'a [usize],
) -> impl Iterator<Item = T> + 'a
where
    T: Clone + 'a,
{
    try_order_by_index_iter(items, indexes)
        .map(|result| result.unwrap_or_else(|error| panic!("{error}")))
}

/// Lazily and safely reorder values using a sequence of indexes.
///
/// An invalid index is reported only when iteration reaches that position.
pub fn try_order_by_index_iter<'a, T>(
    items: &'a [T],
    indexes: &'a [usize],
) -> impl Iterator<Item = Result<T, OrderByIndexError>> + 'a
where
    T: Clone + 'a,
{
    let input_length = items.len();

    indexes
        .iter()
        .copied()
        .enumerate()
        .map(move |(position, index)| {
            items.get(index).cloned().ok_or(OrderByIndexError {
                position,
                index,
                input_length,
            })
        })
}

/// Reorder values using a sequence of indexes.
///
/// Duplicate indexes are supported, matching Python's
/// `order_by_index` behaviour.
///
/// # Panics
///
/// Panics if any index is outside the input.
pub fn order_by_index<T>(items: &[T], indexes: &[usize]) -> Vec<T>
where
    T: Clone,
{
    order_by_index_iter(items, indexes).collect()
}

/// Safely reorder values using a sequence of indexes.
pub fn try_order_by_index<T>(items: &[T], indexes: &[usize]) -> Result<Vec<T>, OrderByIndexError>
where
    T: Clone,
{
    try_order_by_index_iter(items, indexes).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Record {
        name: &'static str,
        id: &'static str,
    }

    fn records() -> Vec<Record> {
        vec![
            Record {
                name: "file10",
                id: "10",
            },
            Record {
                name: "file2",
                id: "2",
            },
            Record {
                name: "file1",
                id: "1",
            },
        ]
    }

    fn names(records: &[Record]) -> Vec<&str> {
        records.iter().map(|record| record.name).collect()
    }

    #[test]
    fn real_sorts_signed_floats() {
        let input = vec!["num5.10", "num-3", "num5.3", "num2"];

        assert_eq!(
            realsorted(&input),
            vec!["num-3", "num2", "num5.10", "num5.3"]
        );
    }

    #[test]
    fn real_sort_supports_reverse() {
        let input = vec!["num5.10", "num-3", "num5.3", "num2"];

        let options = SortOptions::new().reverse(true);

        assert_eq!(
            realsorted_with_options(&input, options),
            vec!["num5.3", "num5.10", "num2", "num-3"]
        );
    }

    #[test]
    fn real_sort_preserves_ignore_case_option() {
        let input = vec!["A2", "a-3", "a1"];

        let options = SortOptions::new().ignore_case(true);

        assert_eq!(
            realsorted_with_options(&input, options),
            vec!["a-3", "a1", "A2"]
        );
    }

    #[test]
    fn real_sort_preserves_path_option() {
        let input = vec!["folder/file-2.txt", "folder/file10.txt", "folder/file1.txt"];

        let options = SortOptions::new().path(true);

        assert_eq!(
            realsorted_with_options(&input, options),
            vec!["folder/file-2.txt", "folder/file1.txt", "folder/file10.txt",]
        );
    }

    #[test]
    fn returns_natural_sort_indexes() {
        let input = vec!["num3", "num5", "num2"];

        assert_eq!(index_natsorted(&input), vec![2, 0, 1]);
    }

    #[test]
    fn returns_reverse_natural_sort_indexes() {
        let input = vec!["num3", "num5", "num2"];

        let options = SortOptions::new().reverse(true);

        assert_eq!(index_natsorted_with_options(&input, options), vec![1, 0, 2]);
    }

    #[test]
    fn returns_real_sort_indexes() {
        let input = vec!["num5.10", "num-3", "num5.3", "num2"];

        assert_eq!(index_realsorted(&input), vec![1, 3, 0, 2]);
    }

    #[test]
    fn index_sort_is_stable_for_equivalent_values() {
        let input = vec!["file01", "file1", "file001"];

        assert_eq!(index_natsorted(&input), vec![0, 1, 2]);
    }

    #[test]
    fn index_sort_supports_presort() {
        let input = vec!["a1", "a1.45", "a01", "a1.4500"];

        let options = SortOptions::new().float(true).presort(true);

        assert_eq!(
            index_natsorted_with_options(&input, options),
            vec![2, 0, 1, 3]
        );
    }

    #[test]
    fn index_sort_supports_reverse_presort() {
        let input = vec!["a1", "a1.45", "a01", "a1.4500"];

        let options = SortOptions::new().float(true).presort(true).reverse(true);

        assert_eq!(
            index_natsorted_with_options(&input, options),
            vec![3, 1, 0, 2]
        );
    }

    #[test]
    fn index_sort_handles_empty_input() {
        let input: Vec<&str> = Vec::new();

        assert!(index_natsorted(&input).is_empty());
    }

    #[test]
    fn index_sort_handles_single_input() {
        assert_eq!(index_natsorted(&["file1"]), vec![0]);
    }

    #[test]
    fn index_sort_supports_paths() {
        let input = vec!["folder10/file", "folder2/file", "folder1/file"];

        let options = SortOptions::new().path(true);

        assert_eq!(index_natsorted_with_options(&input, options), vec![2, 1, 0]);
    }

    #[test]
    fn index_sort_supports_ignore_case() {
        let input = vec!["FILE10", "file2", "File1"];

        let options = SortOptions::new().ignore_case(true);

        assert_eq!(index_natsorted_with_options(&input, options), vec![2, 1, 0]);
    }

    #[test]
    fn orders_primary_values_by_indexes() {
        let values = vec!["num3", "num5", "num2"];

        assert_eq!(
            order_by_index(&values, &[2, 0, 1]),
            vec!["num2", "num3", "num5"]
        );
    }

    #[test]
    fn orders_secondary_values_by_indexes() {
        let values = vec!["foo", "bar", "baz"];

        assert_eq!(
            order_by_index(&values, &[2, 0, 1]),
            vec!["baz", "foo", "bar"]
        );
    }

    #[test]
    fn order_by_index_supports_duplicates() {
        let values = vec!["a", "b", "c"];

        assert_eq!(order_by_index(&values, &[1, 1, 2]), vec!["b", "b", "c"]);
    }

    #[test]
    fn order_by_index_supports_empty_indexes() {
        let values = vec!["a", "b"];

        assert!(order_by_index(&values, &[]).is_empty());
    }

    #[test]
    fn safe_order_reports_invalid_index() {
        let values = vec!["a", "b"];

        assert_eq!(
            try_order_by_index(&values, &[0, 3]),
            Err(OrderByIndexError {
                position: 1,
                index: 3,
                input_length: 2,
            })
        );
    }

    #[test]
    fn safe_order_returns_values() {
        let values = vec!["a", "b", "c"];

        assert_eq!(try_order_by_index(&values, &[2, 0]), Ok(vec!["c", "a"]));
    }

    #[test]
    #[should_panic(expected = "index 4 at position 1 is out of bounds")]
    fn order_by_index_panics_for_invalid_index() {
        let values = vec!["a", "b"];

        let _ = order_by_index(&values, &[0, 4]);
    }

    #[test]
    fn sorts_records_by_key() {
        let sorted = natsorted_by_key(&records(), |record| record.name);

        assert_eq!(names(&sorted), vec!["file1", "file2", "file10"]);
    }

    #[test]
    fn sorts_records_by_key_in_reverse() {
        let input = records();

        let options = SortOptions::new().reverse(true);

        let sorted = natsorted_by_key_with_options(&input, |record| record.name, options);

        assert_eq!(names(&sorted), vec!["file10", "file2", "file1"]);
    }

    #[test]
    fn key_sort_is_stable() {
        let input = vec![
            Record {
                name: "file01",
                id: "first",
            },
            Record {
                name: "file1",
                id: "second",
            },
            Record {
                name: "file001",
                id: "third",
            },
        ];

        let sorted = natsorted_by_key(&input, |record| record.name);

        let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

        assert_eq!(ids, vec!["first", "second", "third"]);
    }

    #[test]
    fn key_sort_supports_ignore_case() {
        let input = vec![
            Record {
                name: "FILE10",
                id: "10",
            },
            Record {
                name: "file2",
                id: "2",
            },
            Record {
                name: "File1",
                id: "1",
            },
        ];

        let options = SortOptions::new().ignore_case(true);

        let sorted = natsorted_by_key_with_options(&input, |record| record.name, options);

        assert_eq!(names(&sorted), vec!["File1", "file2", "FILE10"]);
    }

    #[test]
    fn key_sort_supports_paths() {
        let input = vec![
            Record {
                name: "folder10/file",
                id: "10",
            },
            Record {
                name: "folder2/file",
                id: "2",
            },
            Record {
                name: "folder1/file",
                id: "1",
            },
        ];

        let options = SortOptions::new().path(true);

        let sorted = natsorted_by_key_with_options(&input, |record| record.name, options);

        assert_eq!(
            names(&sorted),
            vec!["folder1/file", "folder2/file", "folder10/file",]
        );
    }

    #[test]
    fn key_sort_supports_presort() {
        let input = vec![
            Record {
                name: "a1",
                id: "0",
            },
            Record {
                name: "a1.45",
                id: "1",
            },
            Record {
                name: "a01",
                id: "2",
            },
            Record {
                name: "a1.4500",
                id: "3",
            },
        ];

        let options = SortOptions::new().float(true).presort(true);

        let sorted = natsorted_by_key_with_options(&input, |record| record.name, options);

        let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

        assert_eq!(ids, vec!["2", "0", "1", "3"]);
    }

    #[test]
    fn key_sort_supports_reverse_presort() {
        let input = vec![
            Record {
                name: "a1",
                id: "0",
            },
            Record {
                name: "a1.45",
                id: "1",
            },
            Record {
                name: "a01",
                id: "2",
            },
            Record {
                name: "a1.4500",
                id: "3",
            },
        ];

        let options = SortOptions::new().float(true).presort(true).reverse(true);

        let sorted = natsorted_by_key_with_options(&input, |record| record.name, options);

        let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

        assert_eq!(ids, vec!["3", "1", "0", "2"]);
    }

    #[test]
    fn real_sorts_records_by_key() {
        let input = vec![
            Record {
                name: "num5.10",
                id: "0",
            },
            Record {
                name: "num-3",
                id: "1",
            },
            Record {
                name: "num5.3",
                id: "2",
            },
            Record {
                name: "num2",
                id: "3",
            },
        ];

        let sorted = realsorted_by_key(&input, |record| record.name);

        let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

        assert_eq!(ids, vec!["1", "3", "0", "2"]);
    }

    #[test]
    fn real_key_sort_supports_reverse() {
        let input = records();

        let options = SortOptions::new().reverse(true);

        let sorted = realsorted_by_key_with_options(&input, |record| record.name, options);

        assert_eq!(names(&sorted), vec!["file10", "file2", "file1"]);
    }

    #[test]
    fn real_key_sort_is_stable() {
        let input = vec![
            Record {
                name: "num1",
                id: "first",
            },
            Record {
                name: "num1.0",
                id: "second",
            },
            Record {
                name: "num1e0",
                id: "third",
            },
        ];

        let sorted = realsorted_by_key(&input, |record| record.name);

        let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

        assert_eq!(ids, vec!["first", "second", "third"]);
    }

    #[test]
    fn key_sort_handles_empty_input() {
        let input: Vec<Record> = Vec::new();

        let sorted = natsorted_by_key(&input, |record| record.name);

        assert!(sorted.is_empty());
    }

    #[test]
    fn key_sort_handles_single_item() {
        let input = vec![Record {
            name: "file1",
            id: "1",
        }];

        assert_eq!(natsorted_by_key(&input, |record| record.name,), input);
    }

    #[test]
    fn key_function_can_borrow_owned_string() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct OwnedRecord {
            name: String,
        }

        let input = vec![
            OwnedRecord {
                name: "file10".to_string(),
            },
            OwnedRecord {
                name: "file2".to_string(),
            },
        ];

        let sorted = natsorted_by_key(&input, |record| record.name.as_str());

        assert_eq!(sorted[0].name, "file2");
        assert_eq!(sorted[1].name, "file10");
    }

    #[test]
    fn human_sort_matches_locale_sort_options() {
        use crate::locale::LocaleProfile;

        let input = vec!["Apple", "apple", "Äpfel", "banana"];
        let options = SortOptions::new().locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            humansorted_with_options(&input, options),
            natsorted_with_options(&input, options.locale(true)),
        );
    }

    #[test]
    fn human_indexes_match_locale_indexes() {
        use crate::locale::LocaleProfile;

        let input = vec![
            "file10", "File2", "file1", "Äpfel20", "Äpfel3", "apple11", "apple2", "Öl5", "Oase4",
        ];
        let options = SortOptions::new().locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            index_humansorted_with_options(&input, options),
            vec![4, 3, 6, 5, 2, 0, 1, 8, 7],
        );
    }

    #[test]
    fn human_sort_supports_localized_numbers() {
        use crate::locale::LocaleProfile;

        let input = vec!["1,234.50", "12.50", "2.75", "1,000.25", "10.25"];
        let options = SortOptions::new()
            .float(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            humansorted_with_options(&input, options),
            vec!["2.75", "10.25", "12.50", "1,000.25", "1,234.50"],
        );
    }

    #[test]
    fn human_sort_supports_key_functions() {
        use crate::locale::LocaleProfile;

        let records = vec![
            Record {
                name: "Äpfel20",
                id: "3",
            },
            Record {
                name: "apple2",
                id: "1",
            },
            Record {
                name: "Apple10",
                id: "2",
            },
        ];
        let options = SortOptions::new().locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            names(&humansorted_by_key_with_options(
                &records,
                |record| record.name,
                options,
            )),
            vec!["Äpfel20", "apple2", "Apple10"],
        );
    }

    #[test]
    fn sorts_path_buf_values_directly() {
        use std::path::PathBuf;

        let input = vec![
            PathBuf::from("folder/file10.txt"),
            PathBuf::from("folder/file2.txt"),
            PathBuf::from("folder/file1.txt"),
        ];

        assert_eq!(
            natsorted_paths(&input),
            vec![
                PathBuf::from("folder/file1.txt"),
                PathBuf::from("folder/file2.txt"),
                PathBuf::from("folder/file10.txt"),
            ],
        );
    }

    #[test]
    fn sorts_borrowed_path_values_directly() {
        use std::path::Path;

        let input = [
            Path::new("folder10/file.txt"),
            Path::new("folder2/file.txt"),
            Path::new("folder1/file.txt"),
        ];

        assert_eq!(
            natsorted_paths(&input),
            vec![
                Path::new("folder1/file.txt"),
                Path::new("folder2/file.txt"),
                Path::new("folder10/file.txt"),
            ],
        );
    }

    #[test]
    fn direct_path_sort_forces_path_mode_and_supports_reverse() {
        use std::path::PathBuf;

        let input = vec![
            PathBuf::from("folder/file10.txt"),
            PathBuf::from("folder/file2.txt"),
            PathBuf::from("folder/file1.txt"),
        ];
        let options = SortOptions::new().reverse(true);

        assert_eq!(
            natsorted_paths_with_options(&input, options),
            vec![
                PathBuf::from("folder/file10.txt"),
                PathBuf::from("folder/file2.txt"),
                PathBuf::from("folder/file1.txt"),
            ],
        );
    }

    #[test]
    fn returns_direct_path_sort_indexes() {
        use std::path::PathBuf;

        let input = vec![
            PathBuf::from("folder10/file.txt"),
            PathBuf::from("folder2/file.txt"),
            PathBuf::from("folder1/file.txt"),
        ];

        assert_eq!(index_natsorted_paths(&input), vec![2, 1, 0]);
    }

    #[test]
    fn order_by_index_iterator_yields_values_lazily() {
        let values = vec!["a", "b", "c"];
        let indexes = vec![2, 0, 1];
        let mut ordered = order_by_index_iter(&values, &indexes);

        assert_eq!(ordered.next(), Some("c"));
        assert_eq!(ordered.next(), Some("a"));
        assert_eq!(ordered.next(), Some("b"));
        assert_eq!(ordered.next(), None);
    }

    #[test]
    fn safe_order_by_index_iterator_defers_invalid_index_error() {
        let values = vec!["a", "b"];
        let indexes = vec![1, 4];
        let mut ordered = try_order_by_index_iter(&values, &indexes);

        assert_eq!(ordered.next(), Some(Ok("b")));
        assert_eq!(
            ordered.next(),
            Some(Err(OrderByIndexError {
                position: 1,
                index: 4,
                input_length: 2,
            })),
        );
        assert_eq!(ordered.next(), None);
    }
}
