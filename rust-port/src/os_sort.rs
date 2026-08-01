use std::cmp::Ordering;
use std::env;
use std::fs;
use std::sync::OnceLock;

use unicode_casefold::UnicodeCaseFold;

use crate::locale::{LocaleProfile, locale_numeric_sort_key, locale_sort_key};
use crate::path::path_components;
use crate::value::NaturalValue;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum OsSortProfile {
    #[default]
    System,
    Windows,
    Unix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsSortOptions {
    pub reverse: bool,
    pub presort: bool,
    pub profile: OsSortProfile,
    pub locale_profile: LocaleProfile,
}

impl Default for OsSortOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl OsSortOptions {
    pub const fn new() -> Self {
        Self {
            reverse: false,
            presort: false,
            profile: OsSortProfile::System,
            locale_profile: LocaleProfile::System,
        }
    }

    pub const fn reverse(mut self, enabled: bool) -> Self {
        self.reverse = enabled;
        self
    }

    pub const fn presort(mut self, enabled: bool) -> Self {
        self.presort = enabled;
        self
    }

    pub const fn profile(mut self, profile: OsSortProfile) -> Self {
        self.profile = profile;
        self
    }

    pub const fn locale_profile(mut self, profile: LocaleProfile) -> Self {
        self.locale_profile = profile;
        self
    }
}

fn running_under_wsl() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }

    if env::var_os("WSL_INTEROP").is_some() || env::var_os("WSL_DISTRO_NAME").is_some() {
        return true;
    }

    ["/proc/sys/kernel/osrelease", "/proc/version"]
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .any(|contents| contents.to_ascii_lowercase().contains("microsoft"))
}

pub fn system_os_sort_profile() -> OsSortProfile {
    static PROFILE: OnceLock<OsSortProfile> = OnceLock::new();

    *PROFILE.get_or_init(|| {
        if cfg!(target_os = "windows") || running_under_wsl() {
            OsSortProfile::Windows
        } else {
            OsSortProfile::Unix
        }
    })
}

impl OsSortProfile {
    fn resolve(self) -> Self {
        match self {
            Self::System => system_os_sort_profile(),
            profile => profile,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsSortKey {
    components: Vec<String>,
    profile: OsSortProfile,
    locale_profile: LocaleProfile,
}

impl OsSortKey {
    pub fn new(input: &str, options: OsSortOptions) -> Self {
        let profile = options.profile.resolve();
        let components = match profile {
            OsSortProfile::Windows => windows_path_components(input),
            OsSortProfile::Unix => path_components(input),
            OsSortProfile::System => unreachable!("system OS profile must be resolved"),
        };

        Self {
            components,
            profile,
            locale_profile: options.locale_profile,
        }
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }

    pub const fn profile(&self) -> OsSortProfile {
        self.profile
    }

    pub const fn locale_profile(&self) -> LocaleProfile {
        self.locale_profile
    }
}

impl PartialEq for OsSortKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OsSortKey {}

impl PartialOrd for OsSortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OsSortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.profile.cmp(&other.profile) {
            Ordering::Equal => {}
            ordering => return ordering,
        }

        match self.locale_profile.cmp(&other.locale_profile) {
            Ordering::Equal => {}
            ordering => return ordering,
        }

        for (left, right) in self.components.iter().zip(other.components.iter()) {
            let ordering = match self.profile {
                OsSortProfile::Windows => {
                    compare_windows_component(left, right, self.locale_profile)
                }
                OsSortProfile::Unix => compare_unix_component(left, right, self.locale_profile),
                OsSortProfile::System => unreachable!("system OS profile must be resolved"),
            };

            if ordering != Ordering::Equal {
                return ordering;
            }
        }

        self.components.len().cmp(&other.components.len())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OsSortKeyGenerator {
    options: OsSortOptions,
}

impl OsSortKeyGenerator {
    pub const fn new(options: OsSortOptions) -> Self {
        Self { options }
    }

    pub fn key<T>(&self, value: &T) -> OsSortKey
    where
        T: ToString + ?Sized,
    {
        OsSortKey::new(&value.to_string(), self.options)
    }

    pub const fn options(&self) -> OsSortOptions {
        self.options
    }
}

pub fn os_sort_key<T>(value: &T) -> OsSortKey
where
    T: ToString + ?Sized,
{
    os_sort_key_with_options(value, OsSortOptions::new())
}

pub fn os_sort_key_with_options<T>(value: &T, options: OsSortOptions) -> OsSortKey
where
    T: ToString + ?Sized,
{
    OsSortKey::new(&value.to_string(), options)
}

pub fn os_sort_key_value(value: &NaturalValue) -> OsSortKey {
    os_sort_key_value_with_options(value, OsSortOptions::new())
}

pub fn os_sort_key_value_with_options(value: &NaturalValue, options: OsSortOptions) -> OsSortKey {
    OsSortKey::new(&value.presort_string(), options)
}

pub fn os_sort_keygen() -> OsSortKeyGenerator {
    OsSortKeyGenerator::default()
}

pub const fn os_sort_keygen_with_options(options: OsSortOptions) -> OsSortKeyGenerator {
    OsSortKeyGenerator::new(options)
}

fn apply_reverse(ordering: Ordering, reverse: bool) -> Ordering {
    if reverse {
        ordering.reverse()
    } else {
        ordering
    }
}

fn sort_os_indexes(
    keys: &[String],
    presort_values: &[String],
    options: OsSortOptions,
) -> Vec<usize> {
    debug_assert_eq!(keys.len(), presort_values.len());

    let os_keys: Vec<OsSortKey> = keys
        .iter()
        .map(|key| OsSortKey::new(key, options))
        .collect();

    let mut indexes: Vec<usize> = (0..keys.len()).collect();

    if options.presort {
        indexes.sort_by(|left, right| {
            apply_reverse(
                presort_values[*left].cmp(&presort_values[*right]),
                options.reverse,
            )
        });
    }

    indexes.sort_by(|left, right| {
        apply_reverse(os_keys[*left].cmp(&os_keys[*right]), options.reverse)
    });

    indexes
}

pub fn index_os_sorted<T>(items: &[T]) -> Vec<usize>
where
    T: ToString,
{
    index_os_sorted_with_options(items, OsSortOptions::new())
}

pub fn index_os_sorted_with_options<T>(items: &[T], options: OsSortOptions) -> Vec<usize>
where
    T: ToString,
{
    let strings: Vec<String> = items.iter().map(ToString::to_string).collect();
    sort_os_indexes(&strings, &strings, options)
}

pub fn os_sorted<T>(items: &[T]) -> Vec<T>
where
    T: Clone + ToString,
{
    os_sorted_with_options(items, OsSortOptions::new())
}

pub fn os_sorted_with_options<T>(items: &[T], options: OsSortOptions) -> Vec<T>
where
    T: Clone + ToString,
{
    index_os_sorted_with_options(items, options)
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

pub fn index_os_sorted_by_key<T, F, K>(items: &[T], key: F) -> Vec<usize>
where
    T: ToString,
    F: FnMut(&T) -> K,
    K: ToString,
{
    index_os_sorted_by_key_with_options(items, key, OsSortOptions::new())
}

pub fn index_os_sorted_by_key_with_options<T, F, K>(
    items: &[T],
    mut key: F,
    options: OsSortOptions,
) -> Vec<usize>
where
    T: ToString,
    F: FnMut(&T) -> K,
    K: ToString,
{
    let keys: Vec<String> = items.iter().map(|item| key(item).to_string()).collect();
    let presort_values: Vec<String> = items.iter().map(ToString::to_string).collect();

    sort_os_indexes(&keys, &presort_values, options)
}

pub fn os_sorted_by_key<T, F, K>(items: &[T], key: F) -> Vec<T>
where
    T: Clone + ToString,
    F: FnMut(&T) -> K,
    K: ToString,
{
    os_sorted_by_key_with_options(items, key, OsSortOptions::new())
}

pub fn os_sorted_by_key_with_options<T, F, K>(items: &[T], key: F, options: OsSortOptions) -> Vec<T>
where
    T: Clone + ToString,
    F: FnMut(&T) -> K,
    K: ToString,
{
    index_os_sorted_by_key_with_options(items, key, options)
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

pub fn index_os_sorted_values(items: &[NaturalValue]) -> Vec<usize> {
    index_os_sorted_values_with_options(items, OsSortOptions::new())
}

pub fn index_os_sorted_values_with_options(
    items: &[NaturalValue],
    options: OsSortOptions,
) -> Vec<usize> {
    let strings: Vec<String> = items.iter().map(NaturalValue::presort_string).collect();
    sort_os_indexes(&strings, &strings, options)
}

pub fn os_sorted_values(items: &[NaturalValue]) -> Vec<NaturalValue> {
    os_sorted_values_with_options(items, OsSortOptions::new())
}

pub fn os_sorted_values_with_options(
    items: &[NaturalValue],
    options: OsSortOptions,
) -> Vec<NaturalValue> {
    index_os_sorted_values_with_options(items, options)
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

fn windows_path_components(input: &str) -> Vec<String> {
    let mut components = Vec::new();

    if let Some(root) = input
        .chars()
        .next()
        .filter(|character| matches!(character, '/' | '\\'))
    {
        components.push(root.to_string());
    }

    components.extend(
        input
            .split(['/', '\\'])
            .filter(|component| !component.is_empty())
            .map(str::to_string),
    );

    if components.is_empty() {
        components.push(".".to_string());
    }

    components
}

fn compare_unix_component(left: &str, right: &str, locale_profile: LocaleProfile) -> Ordering {
    locale_numeric_sort_key(left, locale_profile)
        .cmp(&locale_numeric_sort_key(right, locale_profile))
}

#[cfg(target_os = "windows")]
#[link(name = "Shlwapi")]
unsafe extern "system" {
    #[link_name = "StrCmpLogicalW"]
    fn str_cmp_logical_w(left: *const u16, right: *const u16) -> i32;
}

#[cfg(target_os = "windows")]
fn compare_windows_native(left: &str, right: &str) -> Ordering {
    let left_wide: Vec<u16> = left.encode_utf16().chain(std::iter::once(0)).collect();
    let right_wide: Vec<u16> = right.encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: Both buffers are valid, null-terminated UTF-16 strings and
    // remain alive for the duration of the call.
    let result = unsafe { str_cmp_logical_w(left_wide.as_ptr(), right_wide.as_ptr()) };

    result.cmp(&0)
}

fn compare_windows_component(left: &str, right: &str, locale_profile: LocaleProfile) -> Ordering {
    #[cfg(target_os = "windows")]
    if locale_profile == LocaleProfile::System {
        return compare_windows_native(left, right);
    }

    compare_windows_emulated(left, right, locale_profile)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WindowsToken {
    Punctuation(char),
    Number {
        digits: String,
        original_length: usize,
    },
    Text(String),
}

fn is_windows_special(character: char) -> bool {
    matches!(
        character,
        ' ' | '\t'
            | '.'
            | '\''
            | '-'
            | '!'
            | '#'
            | '$'
            | '%'
            | '&'
            | '('
            | ')'
            | ','
            | ';'
            | '@'
            | '['
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '}'
            | '~'
            | '´'
            | '€'
            | '+'
            | '='
            | '§'
            | '°'
            | 'µ'
    )
}

fn windows_punctuation_rank(character: char) -> (u16, u32) {
    let rank = match character {
        ' ' | '\t' => 0,
        '.' => 1,
        '\'' => 2,
        '-' => 3,
        '!' => 4,
        '#' => 5,
        '$' => 6,
        '%' => 7,
        '&' => 8,
        '(' => 9,
        ')' => 10,
        ',' => 11,
        ';' => 12,
        '@' => 13,
        '[' => 14,
        ']' => 15,
        '^' => 16,
        '_' => 17,
        '`' => 18,
        '{' => 19,
        '}' => 20,
        '~' => 21,
        '´' => 22,
        '€' => 23,
        '+' => 24,
        '=' => 25,
        '§' => 26,
        '°' => 27,
        'µ' => 28,
        _ => 100,
    };

    (rank, u32::from(character))
}

fn windows_tokens(input: &str) -> Vec<WindowsToken> {
    let characters: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        if is_windows_special(character) {
            tokens.push(WindowsToken::Punctuation(character));
            index += 1;
            continue;
        }

        if character.is_ascii_digit() {
            let start = index;
            let mut digits = String::new();

            while let Some(digit) = characters.get(index).and_then(|value| value.to_digit(10)) {
                digits.push(char::from_digit(digit, 10).expect("base-ten digit must be valid"));
                index += 1;
            }

            tokens.push(WindowsToken::Number {
                digits,
                original_length: index - start,
            });
            continue;
        }

        if character.is_alphabetic() {
            let start = index;
            index += 1;

            while characters.get(index).is_some_and(|value| {
                value.is_alphabetic() && value.to_digit(10).is_none() && !is_windows_special(*value)
            }) {
                index += 1;
            }

            let text: String = characters[start..index].iter().collect();
            let folded: String = text.as_str().case_fold().collect();
            tokens.push(WindowsToken::Text(folded));
            continue;
        }

        tokens.push(WindowsToken::Punctuation(character));
        index += 1;
    }

    tokens
}

fn token_rank(token: &WindowsToken) -> u8 {
    match token {
        WindowsToken::Punctuation(_) => 0,
        WindowsToken::Number { .. } => 1,
        WindowsToken::Text(_) => 2,
    }
}

fn significant_digits(digits: &str) -> &str {
    let significant = digits.trim_start_matches('0');

    if significant.is_empty() {
        "0"
    } else {
        significant
    }
}

fn compare_windows_numbers(left: &str, right: &str) -> Ordering {
    let left_significant = significant_digits(left);
    let right_significant = significant_digits(right);

    match left_significant.len().cmp(&right_significant.len()) {
        Ordering::Equal => left_significant.cmp(right_significant),
        ordering => ordering,
    }
}

fn compare_windows_emulated(left: &str, right: &str, locale_profile: LocaleProfile) -> Ordering {
    let left_tokens = windows_tokens(left);
    let right_tokens = windows_tokens(right);
    let mut leading_zero_tiebreak = Ordering::Equal;

    for (left_token, right_token) in left_tokens.iter().zip(right_tokens.iter()) {
        let rank_ordering = token_rank(left_token).cmp(&token_rank(right_token));

        if rank_ordering != Ordering::Equal {
            return rank_ordering;
        }

        let ordering = match (left_token, right_token) {
            (WindowsToken::Punctuation(left), WindowsToken::Punctuation(right)) => {
                windows_punctuation_rank(*left).cmp(&windows_punctuation_rank(*right))
            }
            (
                WindowsToken::Number {
                    digits: left,
                    original_length: left_length,
                },
                WindowsToken::Number {
                    digits: right,
                    original_length: right_length,
                },
            ) => {
                let ordering = compare_windows_numbers(left, right);

                if ordering == Ordering::Equal
                    && leading_zero_tiebreak == Ordering::Equal
                    && left_length != right_length
                {
                    leading_zero_tiebreak = right_length.cmp(left_length);
                }

                ordering
            }
            (WindowsToken::Text(left), WindowsToken::Text(right)) => {
                locale_sort_key(left, locale_profile).cmp(&locale_sort_key(right, locale_profile))
            }
            _ => unreachable!("token ranks ensure matching variants"),
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    match left_tokens.len().cmp(&right_tokens.len()) {
        Ordering::Equal => leading_zero_tiebreak,
        ordering => ordering,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn windows_options() -> OsSortOptions {
        OsSortOptions::new()
            .profile(OsSortProfile::Windows)
            .locale_profile(LocaleProfile::EnglishIndia)
    }

    #[test]
    fn windows_sorts_basic_names_like_explorer() {
        let input = ["file10", "file2", "File3", "file1", "file_0"];

        assert_eq!(
            os_sorted_with_options(&input, windows_options()),
            vec!["file_0", "file1", "file2", "File3", "file10"]
        );
    }

    #[test]
    fn windows_reverse_sort_matches_python() {
        let input = ["file10", "file2", "File3", "file1", "file_0"];

        assert_eq!(
            os_sorted_with_options(&input, windows_options().reverse(true)),
            vec!["file10", "File3", "file2", "file1", "file_0"]
        );
    }

    #[test]
    fn windows_uses_more_leading_zeroes_as_tiebreaker() {
        let input = ["a1", "a01", "a001"];

        assert_eq!(
            os_sorted_with_options(&input, windows_options()),
            vec!["a001", "a01", "a1"]
        );
    }

    #[test]
    fn windows_presort_keeps_python_order() {
        let input = ["a1", "a01"];

        assert_eq!(
            os_sorted_with_options(&input, windows_options().presort(true)),
            vec!["a01", "a1"]
        );
    }

    #[test]
    fn windows_supports_key_functions() {
        let input = ["foo0", "foo2", "goo1"];

        assert_eq!(
            os_sorted_by_key_with_options(
                &input,
                |value| value.replace('g', "f"),
                windows_options(),
            ),
            vec!["foo0", "goo1", "foo2"]
        );
    }

    #[test]
    fn windows_sorts_path_components() {
        let input = [
            "Folder10/file2.txt",
            "Folder2/file10.txt",
            "Folder2/file2.txt",
            "folder1/file20.txt",
        ];

        assert_eq!(
            os_sorted_with_options(&input, windows_options()),
            vec![
                "folder1/file20.txt",
                "Folder2/file2.txt",
                "Folder2/file10.txt",
                "Folder10/file2.txt",
            ]
        );
    }

    #[test]
    fn windows_sorts_unicode_names() {
        let input = ["Äpfel10", "apple2", "Apple10", "äpfel2", "Öl5", "Oase4"];

        assert_eq!(
            os_sorted_with_options(&input, windows_options()),
            vec!["äpfel2", "Äpfel10", "apple2", "Apple10", "Oase4", "Öl5"]
        );
    }

    #[test]
    fn windows_coerces_mixed_values_to_strings() {
        let input = vec![
            NaturalValue::from("10"),
            NaturalValue::from(2),
            NaturalValue::from("1"),
            NaturalValue::from(11),
            NaturalValue::from(5.5),
            NaturalValue::None,
            NaturalValue::from(f64::NAN),
        ];

        assert_eq!(
            index_os_sorted_values_with_options(&input, windows_options()),
            vec![2, 1, 4, 0, 3, 6, 5]
        );
    }

    #[test]
    fn windows_matches_punctuation_corpus() {
        let input = [
            "11111", "aaaaa", "foo0", "foo_0", "foo1", "foo2", "foo4", "foo10", "Foo3", "!", "#",
            "$", "%", "&", "'", "(", ")", "+", "+11111", "+aaaaa", ",", "-", ";", "=", "@", "[",
            "]", "^", "_", "`", "{", "}", "~", "§", "°", "´", "µ", "€",
        ];

        assert_eq!(
            os_sorted_with_options(&input, windows_options()),
            vec![
                "'", "-", "!", "#", "$", "%", "&", "(", ")", ",", ";", "@", "[", "]", "^", "_",
                "`", "{", "}", "~", "´", "€", "+", "+11111", "+aaaaa", "=", "§", "°", "µ", "11111",
                "aaaaa", "foo_0", "foo0", "foo1", "foo2", "Foo3", "foo4", "foo10",
            ]
        );
    }

    #[test]
    fn windows_matches_compound_path_order() {
        let input = [
            "/p/Folder (10)/file.tar.gz",
            "/p/Folder (1)/file (1).tar.gz",
            "/p/Folder/file.x1.9.tar.gz",
            "/p/Folder (2)/file.tar.gz",
            "/p/Folder (1)/file.tar.gz",
            "/p/Folder/file.x1.10.tar.gz",
        ];

        assert_eq!(
            os_sorted_with_options(&input, windows_options()),
            vec![
                "/p/Folder/file.x1.9.tar.gz",
                "/p/Folder/file.x1.10.tar.gz",
                "/p/Folder (1)/file (1).tar.gz",
                "/p/Folder (1)/file.tar.gz",
                "/p/Folder (2)/file.tar.gz",
                "/p/Folder (10)/file.tar.gz",
            ]
        );
    }

    #[test]
    fn key_generator_uses_configured_profile() {
        let generator = os_sort_keygen_with_options(windows_options());

        assert!(generator.key("file2") < generator.key("file10"));
    }

    #[test]
    fn windows_comparison_is_case_insensitive() {
        let left = os_sort_key_with_options("file2", windows_options());
        let right = os_sort_key_with_options("File2", windows_options());

        assert_eq!(left, right);
    }

    #[test]
    fn unix_profile_uses_locale_aware_natural_sorting() {
        let options = OsSortOptions::new()
            .profile(OsSortProfile::Unix)
            .locale_profile(LocaleProfile::EnglishUnitedStates);
        let input = ["file10", "File2", "file1"];

        assert_eq!(
            os_sorted_with_options(&input, options),
            vec!["file1", "File2", "file10"]
        );
    }
}
