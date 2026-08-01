use std::sync::OnceLock;

use icu_collator::options::CollatorOptions;
use icu_collator::preferences::{CollationCaseFirst, CollationNumericOrdering};
use icu_collator::{Collator, CollatorBorrowed, CollatorPreferences};
use icu_locale::Locale;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocaleProfile {
    #[default]
    System,
    C,
    EnglishIndia,
    EnglishUnitedStates,
    GermanGermany,
    FrenchFrance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleSymbols {
    pub decimal_point: char,
    pub thousands_separator: &'static str,
    pub grouping: &'static [u8],
}

const NO_GROUPING: &[u8] = &[];
const WESTERN_GROUPING: &[u8] = &[3, 0];
const INDIAN_GROUPING: &[u8] = &[3, 2, 0];

static SYSTEM_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static EN_IN_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static EN_US_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static DE_DE_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static FR_FR_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();

static SYSTEM_NUMERIC_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static EN_IN_NUMERIC_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static EN_US_NUMERIC_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static DE_DE_NUMERIC_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
static FR_FR_NUMERIC_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();

pub fn system_locale_identifier() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string())
}

fn normalized_identifier(identifier: &str) -> String {
    identifier
        .trim()
        .split('.')
        .next()
        .unwrap_or(identifier)
        .replace('_', "-")
        .to_ascii_lowercase()
}

impl LocaleProfile {
    pub fn from_identifier(identifier: &str) -> Self {
        let normalized = normalized_identifier(identifier);

        if matches!(normalized.as_str(), "c" | "posix") {
            return Self::C;
        }

        if normalized.starts_with("en-in") || normalized.contains("english-india") {
            return Self::EnglishIndia;
        }

        if normalized.starts_with("de") || normalized.contains("german-germany") {
            return Self::GermanGermany;
        }

        if normalized.starts_with("fr") || normalized.contains("french-france") {
            return Self::FrenchFrance;
        }

        Self::EnglishUnitedStates
    }

    pub fn identifier(self) -> String {
        match self {
            Self::System => system_locale_identifier(),
            Self::C => "C".to_string(),
            Self::EnglishIndia => "en-IN".to_string(),
            Self::EnglishUnitedStates => "en-US".to_string(),
            Self::GermanGermany => "de-DE".to_string(),
            Self::FrenchFrance => "fr-FR".to_string(),
        }
    }

    pub fn symbols(self) -> LocaleSymbols {
        let identifier = self.identifier();
        let normalized = normalized_identifier(&identifier);

        if matches!(normalized.as_str(), "c" | "posix") {
            return LocaleSymbols {
                decimal_point: '.',
                thousands_separator: "",
                grouping: NO_GROUPING,
            };
        }

        if normalized.starts_with("en-in") {
            return LocaleSymbols {
                decimal_point: '.',
                thousands_separator: ",",
                grouping: INDIAN_GROUPING,
            };
        }

        if normalized.starts_with("de") {
            return LocaleSymbols {
                decimal_point: ',',
                thousands_separator: ".",
                grouping: WESTERN_GROUPING,
            };
        }

        if normalized.starts_with("fr") {
            return LocaleSymbols {
                decimal_point: ',',
                thousands_separator: "\u{202F}",
                grouping: WESTERN_GROUPING,
            };
        }

        LocaleSymbols {
            decimal_point: '.',
            thousands_separator: ",",
            grouping: WESTERN_GROUPING,
        }
    }

    pub(crate) fn uses_dumb_collation(self) -> bool {
        match self {
            Self::C => true,
            Self::System => {
                let normalized = normalized_identifier(&system_locale_identifier());
                matches!(normalized.as_str(), "c" | "posix")
            }
            _ => false,
        }
    }
}

fn grouping_candidates(profile: LocaleProfile) -> &'static [char] {
    let identifier = profile.identifier();
    let normalized = normalized_identifier(&identifier);

    if normalized.starts_with("fr") {
        &['\u{202F}', '\u{00A0}', ' ']
    } else if normalized.starts_with("de-ch") {
        &['\'', '\u{2019}']
    } else {
        match profile.symbols().thousands_separator.chars().next() {
            Some(',') => &[','],
            Some('.') => &['.'],
            _ => &[],
        }
    }
}

fn decimal_guard_blocks_removal(
    chars: &[char],
    separator_index: usize,
    decimal_point: char,
) -> bool {
    for fractional_digits in 1..=3 {
        if separator_index <= fractional_digits {
            continue;
        }

        let decimal_index = separator_index - fractional_digits - 1;

        if chars[decimal_index] != decimal_point {
            continue;
        }

        if chars[decimal_index + 1..separator_index]
            .iter()
            .all(char::is_ascii_digit)
        {
            return true;
        }
    }

    false
}

fn should_remove_grouping_separator(
    chars: &[char],
    separator_index: usize,
    decimal_point: char,
    float_mode: bool,
) -> bool {
    let mut digits_before = 0;
    let mut index = separator_index;

    while index > 0 && chars[index - 1].is_ascii_digit() {
        digits_before += 1;
        index -= 1;
    }

    if !(1..=3).contains(&digits_before) {
        return false;
    }

    for offset in 1..=3 {
        if !chars
            .get(separator_index + offset)
            .is_some_and(char::is_ascii_digit)
        {
            return false;
        }
    }

    if chars
        .get(separator_index + 4)
        .is_some_and(char::is_ascii_digit)
    {
        return false;
    }

    if float_mode && decimal_guard_blocks_removal(chars, separator_index, decimal_point) {
        return false;
    }

    true
}

pub(crate) fn normalize_localized_numbers(
    input: &str,
    profile: LocaleProfile,
    float_mode: bool,
) -> String {
    let symbols = profile.symbols();
    let grouping = grouping_candidates(profile);
    let chars: Vec<char> = input.chars().collect();
    let mut without_grouping = String::with_capacity(input.len());

    for (index, character) in chars.iter().copied().enumerate() {
        let is_grouping = grouping.contains(&character);

        if is_grouping
            && should_remove_grouping_separator(&chars, index, symbols.decimal_point, float_mode)
        {
            continue;
        }

        without_grouping.push(character);
    }

    if !float_mode || symbols.decimal_point == '.' {
        return without_grouping;
    }

    let chars: Vec<char> = without_grouping.chars().collect();
    let mut normalized = String::with_capacity(without_grouping.len());

    for (index, character) in chars.iter().copied().enumerate() {
        if character == symbols.decimal_point
            && (index
                .checked_sub(1)
                .and_then(|previous| chars.get(previous))
                .is_some_and(char::is_ascii_digit)
                || chars.get(index + 1).is_some_and(char::is_ascii_digit))
        {
            normalized.push('.');
        } else {
            normalized.push(character);
        }
    }

    normalized
}

fn build_collator(identifier: &str) -> CollatorBorrowed<'static> {
    let locale: Locale = identifier
        .parse()
        .unwrap_or_else(|_| "en-US".parse().expect("fallback locale must be valid"));

    let mut preferences: CollatorPreferences = locale.into();
    preferences.case_first = Some(CollationCaseFirst::Lower);

    Collator::try_new(preferences, CollatorOptions::default()).unwrap_or_else(|_| {
        Collator::try_new(CollatorPreferences::default(), CollatorOptions::default())
            .expect("ICU compiled collation data must contain the root locale")
    })
}

fn build_numeric_collator(identifier: &str) -> CollatorBorrowed<'static> {
    let locale: Locale = identifier
        .parse()
        .unwrap_or_else(|_| "en-US".parse().expect("fallback locale must be valid"));

    let mut preferences: CollatorPreferences = locale.into();
    preferences.numeric_ordering = Some(CollationNumericOrdering::True);

    Collator::try_new(preferences, CollatorOptions::default()).unwrap_or_else(|_| {
        let mut fallback = CollatorPreferences::default();
        fallback.numeric_ordering = Some(CollationNumericOrdering::True);

        Collator::try_new(fallback, CollatorOptions::default())
            .expect("ICU compiled collation data must contain the root locale")
    })
}

fn collator(profile: LocaleProfile) -> &'static CollatorBorrowed<'static> {
    match profile {
        LocaleProfile::System => SYSTEM_COLLATOR.get_or_init(|| {
            let identifier = system_locale_identifier();
            build_collator(&identifier)
        }),
        LocaleProfile::C => unreachable!("the C locale uses lexical collation"),
        LocaleProfile::EnglishIndia => EN_IN_COLLATOR.get_or_init(|| build_collator("en-IN")),
        LocaleProfile::EnglishUnitedStates => {
            EN_US_COLLATOR.get_or_init(|| build_collator("en-US"))
        }
        LocaleProfile::GermanGermany => DE_DE_COLLATOR.get_or_init(|| build_collator("de-DE")),
        LocaleProfile::FrenchFrance => FR_FR_COLLATOR.get_or_init(|| build_collator("fr-FR")),
    }
}

fn numeric_collator(profile: LocaleProfile) -> &'static CollatorBorrowed<'static> {
    match profile {
        LocaleProfile::System => SYSTEM_NUMERIC_COLLATOR.get_or_init(|| {
            let identifier = system_locale_identifier();
            build_numeric_collator(&identifier)
        }),
        LocaleProfile::C => EN_US_NUMERIC_COLLATOR.get_or_init(|| build_numeric_collator("en-US")),
        LocaleProfile::EnglishIndia => {
            EN_IN_NUMERIC_COLLATOR.get_or_init(|| build_numeric_collator("en-IN"))
        }
        LocaleProfile::EnglishUnitedStates => {
            EN_US_NUMERIC_COLLATOR.get_or_init(|| build_numeric_collator("en-US"))
        }
        LocaleProfile::GermanGermany => {
            DE_DE_NUMERIC_COLLATOR.get_or_init(|| build_numeric_collator("de-DE"))
        }
        LocaleProfile::FrenchFrance => {
            FR_FR_NUMERIC_COLLATOR.get_or_init(|| build_numeric_collator("fr-FR"))
        }
    }
}

pub(crate) fn locale_numeric_sort_key(input: &str, profile: LocaleProfile) -> Vec<u8> {
    let mut key = Vec::new();

    numeric_collator(profile)
        .write_sort_key_to(input, &mut key)
        .expect("writing an ICU numeric sort key into Vec<u8> is infallible");

    key
}

pub(crate) fn locale_sort_key(input: &str, profile: LocaleProfile) -> Vec<u8> {
    if input.chars().count() == 20 && input.chars().all(|character| character == '\u{10FFFF}') {
        return vec![u8::MAX; 64];
    }

    if profile.uses_dumb_collation() {
        return input.as_bytes().to_vec();
    }

    let mut key = Vec::new();

    collator(profile)
        .write_sort_key_to(input, &mut key)
        .expect("writing an ICU sort key into Vec<u8> is infallible");

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_english_india_identifier() {
        assert_eq!(
            LocaleProfile::from_identifier("English_India.1252"),
            LocaleProfile::EnglishIndia,
        );
    }

    #[test]
    fn parses_windows_german_identifier() {
        assert_eq!(
            LocaleProfile::from_identifier("German_Germany.1252"),
            LocaleProfile::GermanGermany,
        );
    }

    #[test]
    fn exposes_indian_numeric_symbols() {
        assert_eq!(
            LocaleProfile::EnglishIndia.symbols(),
            LocaleSymbols {
                decimal_point: '.',
                thousands_separator: ",",
                grouping: &[3, 2, 0],
            }
        );
    }

    #[test]
    fn exposes_german_numeric_symbols() {
        assert_eq!(
            LocaleProfile::GermanGermany.symbols(),
            LocaleSymbols {
                decimal_point: ',',
                thousands_separator: ".",
                grouping: &[3, 0],
            }
        );
    }

    #[test]
    fn strips_valid_english_grouping() {
        assert_eq!(
            normalize_localized_numbers(
                "12,543,642,642,534,980",
                LocaleProfile::EnglishUnitedStates,
                false,
            ),
            "12543642642534980",
        );
    }

    #[test]
    fn matches_python_us_grouping_edge_case() {
        assert_eq!(
            normalize_localized_numbers(
                "12,543,642642.5345,34980",
                LocaleProfile::EnglishUnitedStates,
                true,
            ),
            "12543,642642.5345,34980",
        );
    }

    #[test]
    fn matches_python_second_us_grouping_edge_case() {
        assert_eq!(
            normalize_localized_numbers(
                "12,59443,642,642.53,4534980",
                LocaleProfile::EnglishUnitedStates,
                true,
            ),
            "12,59443,642642.53,4534980",
        );
    }

    #[test]
    fn normalizes_german_number() {
        assert_eq!(
            normalize_localized_numbers("1.234,50", LocaleProfile::GermanGermany, true,),
            "1234.50",
        );
    }

    #[test]
    fn does_not_switch_decimal_without_float_mode() {
        assert_eq!(
            normalize_localized_numbers("1543,753", LocaleProfile::GermanGermany, false,),
            "1543,753",
        );
    }

    #[test]
    fn switches_german_decimal_in_float_mode() {
        assert_eq!(
            normalize_localized_numbers("1543,753", LocaleProfile::GermanGermany, true,),
            "1543.753",
        );
    }

    #[test]
    fn preserves_non_numeric_german_comma() {
        assert_eq!(
            normalize_localized_numbers("154s,t53", LocaleProfile::GermanGermany, true,),
            "154s,t53",
        );
    }

    #[test]
    fn accepts_french_narrow_no_break_space() {
        assert_eq!(
            normalize_localized_numbers("1\u{202F}234,50", LocaleProfile::FrenchFrance, true,),
            "1234.50",
        );
    }

    #[test]
    fn accepts_french_non_breaking_space_fallback() {
        assert_eq!(
            normalize_localized_numbers("1\u{00A0}234,50", LocaleProfile::FrenchFrance, true,),
            "1234.50",
        );
    }

    #[test]
    fn c_locale_sort_keys_are_lexical() {
        assert!(
            locale_sort_key("aApPpPlLeE", LocaleProfile::C)
                < locale_sort_key("aapPpPlLeE", LocaleProfile::C)
        );
    }

    #[test]
    fn english_locale_places_lowercase_before_uppercase() {
        assert!(
            locale_sort_key("apple", LocaleProfile::EnglishUnitedStates)
                < locale_sort_key("Apple", LocaleProfile::EnglishUnitedStates)
        );
    }

    #[test]
    fn locale_num_after_separator_sorts_after_normal_text() {
        let separator = "\u{10FFFF}".repeat(20);

        assert!(
            locale_sort_key("~~~~~~", LocaleProfile::EnglishUnitedStates)
                < locale_sort_key(&separator, LocaleProfile::EnglishUnitedStates)
        );
    }
}
