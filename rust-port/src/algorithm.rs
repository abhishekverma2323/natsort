use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use std::sync::OnceLock;

use crate::unicode_numeric::{unicode_digit_value, unicode_numeric_value};

/// Python-compatible `natsort.ns` algorithm flags.
///
/// The low three bits select the numeric parser. All other bits map directly
/// onto [`crate::SortOptions`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AlgorithmFlags(i64);

impl AlgorithmFlags {
    pub const DEFAULT: Self = Self(0);
    pub const INT: Self = Self::DEFAULT;
    pub const UNSIGNED: Self = Self::DEFAULT;

    pub const FLOAT: Self = Self(1);
    pub const SIGNED: Self = Self(2);
    pub const NOEXP: Self = Self(4);
    pub const PATH: Self = Self(8);
    pub const LOCALEALPHA: Self = Self(16);
    pub const LOCALENUM: Self = Self(32);
    pub const LOCALE: Self = Self(48);
    pub const IGNORECASE: Self = Self(64);
    pub const LOWERCASEFIRST: Self = Self(128);
    pub const GROUPLETTERS: Self = Self(256);
    pub const UNGROUPLETTERS: Self = Self(512);
    pub const CAPITALFIRST: Self = Self::UNGROUPLETTERS;
    pub const NANLAST: Self = Self(1024);
    pub const COMPATIBILITYNORMALIZE: Self = Self(2048);
    pub const NUMAFTER: Self = Self(4096);
    pub const PRESORT: Self = Self(8192);
    pub const REAL: Self = Self(3);

    // Rust-style aliases for the same Python-compatible values.
    pub const LOCALE_ALPHA: Self = Self::LOCALEALPHA;
    pub const LOCALE_NUMERIC: Self = Self::LOCALENUM;
    pub const IGNORE_CASE: Self = Self::IGNORECASE;
    pub const LOWERCASE_FIRST: Self = Self::LOWERCASEFIRST;
    pub const GROUP_LETTERS: Self = Self::GROUPLETTERS;
    pub const UNGROUP_LETTERS: Self = Self::UNGROUPLETTERS;
    pub const CAPITAL_FIRST: Self = Self::CAPITALFIRST;
    pub const NAN_LAST: Self = Self::NANLAST;
    pub const COMPATIBILITY_NORMALIZE: Self = Self::COMPATIBILITYNORMALIZE;
    pub const NUM_AFTER: Self = Self::NUMAFTER;

    // Python's short aliases.
    pub const F: Self = Self::FLOAT;
    pub const S: Self = Self::SIGNED;
    pub const N: Self = Self::NOEXP;
    pub const P: Self = Self::PATH;
    pub const LA: Self = Self::LOCALEALPHA;
    pub const LN: Self = Self::LOCALENUM;
    pub const L: Self = Self::LOCALE;
    pub const IC: Self = Self::IGNORECASE;
    pub const LF: Self = Self::LOWERCASEFIRST;
    pub const G: Self = Self::GROUPLETTERS;
    pub const UG: Self = Self::UNGROUPLETTERS;
    pub const NL: Self = Self::NANLAST;
    pub const CN: Self = Self::COMPATIBILITYNORMALIZE;
    pub const NA: Self = Self::NUMAFTER;
    pub const PS: Self = Self::PRESORT;
    pub const R: Self = Self::REAL;

    pub const KNOWN_MASK: i64 = Self::FLOAT.0
        | Self::SIGNED.0
        | Self::NOEXP.0
        | Self::PATH.0
        | Self::LOCALEALPHA.0
        | Self::LOCALENUM.0
        | Self::IGNORECASE.0
        | Self::LOWERCASEFIRST.0
        | Self::GROUPLETTERS.0
        | Self::UNGROUPLETTERS.0
        | Self::NANLAST.0
        | Self::COMPATIBILITYNORMALIZE.0
        | Self::NUMAFTER.0
        | Self::PRESORT.0;

    pub const fn from_bits(bits: i64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> i64 {
        self.0
    }

    pub const fn known_bits(self) -> i64 {
        self.0 & Self::KNOWN_MASK
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn is_default(self) -> bool {
        self.0 == 0
    }
}

impl From<i64> for AlgorithmFlags {
    fn from(value: i64) -> Self {
        Self::from_bits(value)
    }
}

impl From<AlgorithmFlags> for i64 {
    fn from(value: AlgorithmFlags) -> Self {
        value.bits()
    }
}

impl BitOr for AlgorithmFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for AlgorithmFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for AlgorithmFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for AlgorithmFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// The six numeric regular-expression families used by Python `natsort`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumericRegexKind {
    UnsignedInteger,
    SignedInteger,
    UnsignedFloat,
    SignedFloat,
    UnsignedFloatNoExponent,
    SignedFloatNoExponent,
}

impl NumericRegexKind {
    pub const fn from_algorithm(algorithm: AlgorithmFlags) -> Self {
        let float = algorithm.intersects(AlgorithmFlags::FLOAT);
        let signed = algorithm.intersects(AlgorithmFlags::SIGNED);
        let no_exp = algorithm.intersects(AlgorithmFlags::NOEXP);

        match (float, signed, no_exp) {
            (false, false, _) => Self::UnsignedInteger,
            (false, true, _) => Self::SignedInteger,
            (true, false, false) => Self::UnsignedFloat,
            (true, true, false) => Self::SignedFloat,
            (true, false, true) => Self::UnsignedFloatNoExponent,
            (true, true, true) => Self::SignedFloatNoExponent,
        }
    }
}

static INTEGER_NUMERIC_CHARACTERS: OnceLock<String> = OnceLock::new();
static FLOAT_NUMERIC_CHARACTERS: OnceLock<String> = OnceLock::new();

static UNSIGNED_INTEGER_PATTERN: OnceLock<String> = OnceLock::new();
static SIGNED_INTEGER_PATTERN: OnceLock<String> = OnceLock::new();
static UNSIGNED_FLOAT_PATTERN: OnceLock<String> = OnceLock::new();
static SIGNED_FLOAT_PATTERN: OnceLock<String> = OnceLock::new();
static UNSIGNED_FLOAT_NO_EXP_PATTERN: OnceLock<String> = OnceLock::new();
static SIGNED_FLOAT_NO_EXP_PATTERN: OnceLock<String> = OnceLock::new();

fn unicode_character_class(float: bool) -> &'static str {
    let cache = if float {
        &FLOAT_NUMERIC_CHARACTERS
    } else {
        &INTEGER_NUMERIC_CHARACTERS
    };

    cache
        .get_or_init(|| {
            (0..=u32::from(char::MAX))
                .filter_map(char::from_u32)
                .filter(|character| {
                    if float {
                        unicode_numeric_value(*character).is_some()
                    } else {
                        unicode_digit_value(*character).is_some()
                    }
                })
                .collect()
        })
        .as_str()
}

fn build_pattern(prefix: &str, float: bool) -> String {
    let characters = unicode_character_class(float);
    let mut pattern = String::with_capacity(prefix.len() + characters.len() + 3);

    pattern.push_str(prefix);
    pattern.push_str("|[");
    pattern.push_str(characters);
    pattern.push(']');

    pattern
}

fn pattern_for_kind(kind: NumericRegexKind) -> &'static str {
    match kind {
        NumericRegexKind::UnsignedInteger => UNSIGNED_INTEGER_PATTERN
            .get_or_init(|| build_pattern(r"\d+", false))
            .as_str(),
        NumericRegexKind::SignedInteger => SIGNED_INTEGER_PATTERN
            .get_or_init(|| build_pattern(r"[-+]?\d+", false))
            .as_str(),
        NumericRegexKind::UnsignedFloat => UNSIGNED_FLOAT_PATTERN
            .get_or_init(|| build_pattern(r"(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?", true))
            .as_str(),
        NumericRegexKind::SignedFloat => SIGNED_FLOAT_PATTERN
            .get_or_init(|| build_pattern(r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?", true))
            .as_str(),
        NumericRegexKind::UnsignedFloatNoExponent => UNSIGNED_FLOAT_NO_EXP_PATTERN
            .get_or_init(|| build_pattern(r"(?:\d+\.?\d*|\.\d+)", true))
            .as_str(),
        NumericRegexKind::SignedFloatNoExponent => SIGNED_FLOAT_NO_EXP_PATTERN
            .get_or_init(|| build_pattern(r"[-+]?(?:\d+\.?\d*|\.\d+)", true))
            .as_str(),
    }
}

/// Return the Python-compatible numeric regex string for an algorithm.
pub fn numeric_regex_chooser(algorithm: AlgorithmFlags) -> &'static str {
    pattern_for_kind(NumericRegexKind::from_algorithm(algorithm))
}

/// Compatibility entry point for raw Python-style integer flags.
///
/// Unknown bits are intentionally ignored, matching Python's bit-mask
/// behaviour. Negative integers are therefore accepted as well.
pub fn numeric_regex_chooser_from_bits(bits: i64) -> &'static str {
    numeric_regex_chooser(AlgorithmFlags::from_bits(bits))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn character_class(pattern: &str) -> &str {
        pattern
            .split_once("|[")
            .and_then(|(_, suffix)| suffix.strip_suffix(']'))
            .expect("numeric regex must end in one character class")
    }

    #[test]
    fn exposes_python_flag_values() {
        assert_eq!(AlgorithmFlags::DEFAULT.bits(), 0);
        assert_eq!(AlgorithmFlags::FLOAT.bits(), 1);
        assert_eq!(AlgorithmFlags::SIGNED.bits(), 2);
        assert_eq!(AlgorithmFlags::NOEXP.bits(), 4);
        assert_eq!(AlgorithmFlags::PATH.bits(), 8);
        assert_eq!(AlgorithmFlags::LOCALEALPHA.bits(), 16);
        assert_eq!(AlgorithmFlags::LOCALENUM.bits(), 32);
        assert_eq!(AlgorithmFlags::LOCALE.bits(), 48);
        assert_eq!(AlgorithmFlags::IGNORECASE.bits(), 64);
        assert_eq!(AlgorithmFlags::LOWERCASEFIRST.bits(), 128);
        assert_eq!(AlgorithmFlags::GROUPLETTERS.bits(), 256);
        assert_eq!(AlgorithmFlags::UNGROUPLETTERS.bits(), 512);
        assert_eq!(AlgorithmFlags::NANLAST.bits(), 1024);
        assert_eq!(AlgorithmFlags::COMPATIBILITYNORMALIZE.bits(), 2048);
        assert_eq!(AlgorithmFlags::NUMAFTER.bits(), 4096);
        assert_eq!(AlgorithmFlags::PRESORT.bits(), 8192);
        assert_eq!(AlgorithmFlags::REAL.bits(), 3);
    }

    #[test]
    fn exposes_python_aliases() {
        assert_eq!(AlgorithmFlags::DEFAULT, AlgorithmFlags::INT);
        assert_eq!(AlgorithmFlags::INT, AlgorithmFlags::UNSIGNED);
        assert_eq!(
            AlgorithmFlags::REAL,
            AlgorithmFlags::FLOAT | AlgorithmFlags::SIGNED
        );
        assert_eq!(
            AlgorithmFlags::LOCALE,
            AlgorithmFlags::LOCALEALPHA | AlgorithmFlags::LOCALENUM
        );
        assert_eq!(AlgorithmFlags::CAPITALFIRST, AlgorithmFlags::UNGROUPLETTERS);
        assert_eq!(AlgorithmFlags::R, AlgorithmFlags::REAL);
        assert_eq!(AlgorithmFlags::L, AlgorithmFlags::LOCALE);
    }

    #[test]
    fn chooses_unsigned_integer_pattern_by_default() {
        let pattern = numeric_regex_chooser(AlgorithmFlags::DEFAULT);

        assert!(pattern.starts_with(r"\d+|["));
        assert_eq!(character_class(pattern).chars().count(), 128);
        assert!(character_class(pattern).starts_with("²³¹፩፪"));
        assert!(character_class(pattern).ends_with("🄆🄇🄈🄉🄊"));
    }

    #[test]
    fn chooses_signed_integer_pattern() {
        assert!(numeric_regex_chooser(AlgorithmFlags::SIGNED).starts_with(r"[-+]?\d+|["));
    }

    #[test]
    fn chooses_float_pattern_with_unicode_numeric_class() {
        let pattern = numeric_regex_chooser(AlgorithmFlags::FLOAT);

        assert!(pattern.starts_with(r"(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?|["));
        assert_eq!(character_class(pattern).chars().count(), 1242);
        assert!(character_class(pattern).starts_with("²³¹¼½¾"));
        assert!(character_class(pattern).ends_with("𠬙𢎐𢦘𣬛𦉭廾"));
    }

    #[test]
    fn chooses_all_six_numeric_families() {
        assert_eq!(
            NumericRegexKind::from_algorithm(AlgorithmFlags::DEFAULT),
            NumericRegexKind::UnsignedInteger
        );
        assert_eq!(
            NumericRegexKind::from_algorithm(AlgorithmFlags::SIGNED),
            NumericRegexKind::SignedInteger
        );
        assert_eq!(
            NumericRegexKind::from_algorithm(AlgorithmFlags::FLOAT),
            NumericRegexKind::UnsignedFloat
        );
        assert_eq!(
            NumericRegexKind::from_algorithm(AlgorithmFlags::REAL),
            NumericRegexKind::SignedFloat
        );
        assert_eq!(
            NumericRegexKind::from_algorithm(AlgorithmFlags::FLOAT | AlgorithmFlags::NOEXP),
            NumericRegexKind::UnsignedFloatNoExponent
        );
        assert_eq!(
            NumericRegexKind::from_algorithm(AlgorithmFlags::REAL | AlgorithmFlags::NOEXP),
            NumericRegexKind::SignedFloatNoExponent
        );
    }

    #[test]
    fn ignores_non_numeric_flags_when_selecting_regex() {
        let modifiers = AlgorithmFlags::PATH
            | AlgorithmFlags::LOCALE
            | AlgorithmFlags::IGNORECASE
            | AlgorithmFlags::NUMAFTER
            | AlgorithmFlags::PRESORT;

        assert_eq!(
            numeric_regex_chooser(AlgorithmFlags::FLOAT | modifiers),
            numeric_regex_chooser(AlgorithmFlags::FLOAT)
        );
    }

    #[test]
    fn accepts_arbitrary_raw_integer_flags_like_python() {
        assert_eq!(
            numeric_regex_chooser_from_bits(-1),
            numeric_regex_chooser(
                AlgorithmFlags::FLOAT | AlgorithmFlags::SIGNED | AlgorithmFlags::NOEXP
            )
        );
        assert_eq!(
            numeric_regex_chooser_from_bits(999_999_999),
            numeric_regex_chooser_from_bits(-1)
        );
    }
}
