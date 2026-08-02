use grift_unicode::digit_value;
use num_bigint::BigInt;
use unicode_normalization::UnicodeNormalization;

use crate::decode::{decode_ascii_bytes, decode_latin1_bytes, decode_utf8_bytes};
use crate::locale::{LocaleProfile, normalize_localized_numbers};
use crate::text::{case_fold, group_letters, swap_case};
use crate::unicode_numeric::{unicode_digit_value, unicode_numeric_value};
use crate::value::NaturalValue;

/// Unicode collections generated from the same Rust-side tables used by the
/// tokenizer and numeric regular-expression implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonUnicodeTables {
    pub numeric: Vec<u32>,
    pub digits: Vec<u32>,
    pub decimals: Vec<u32>,
}

fn normalize_decimal_digits(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    for character in input.chars() {
        if let Some(value) = digit_value(character)
            && unicode_numeric_value(character).is_none()
        {
            let ascii = char::from_digit(value, 10)
                .expect("Unicode decimal digit value must be between 0 and 9");
            output.push(ascii);
        } else {
            output.push(character);
        }
    }

    output
}

/// Reproduce the conversion decision made by Python natsort's compatibility
/// `fast_float`. Failure is represented by `None`; the Python bridge then
/// invokes the caller-provided `key`/`on_fail` callback.
pub fn python_fast_float(input: &str, nan_value: f64) -> Option<f64> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return None;
    }

    let normalized = normalize_decimal_digits(trimmed);
    let lowercase = normalized.to_ascii_lowercase();

    let parsed = match lowercase.as_str() {
        "nan" | "+nan" | "-nan" => Some(f64::NAN),
        "inf" | "+inf" | "infinity" | "+infinity" => Some(f64::INFINITY),
        "-inf" | "-infinity" => Some(f64::NEG_INFINITY),
        _ => normalized.parse::<f64>().ok(),
    };

    if let Some(value) = parsed {
        return Some(if value.is_nan() { nan_value } else { value });
    }

    if input.chars().count() == 1 {
        return input.chars().next().and_then(unicode_numeric_value);
    }

    None
}

/// Reproduce the conversion decision made by Python natsort's compatibility
/// `fast_int`. Failure is represented by `None`; the Python bridge then
/// invokes the caller-provided `key`/`on_fail` callback.
pub fn python_fast_int(input: &str) -> Option<BigInt> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return None;
    }

    let normalized = normalize_decimal_digits(trimmed);

    if let Ok(value) = normalized.parse::<BigInt>() {
        return Some(value);
    }

    if input.chars().count() == 1 {
        return input
            .chars()
            .next()
            .and_then(unicode_digit_value)
            .map(BigInt::from);
    }

    None
}

/// Build the Python-visible Unicode collections from Rust-side numeric data.
/// ASCII digits are intentionally excluded to match `unicode_numeric_hex.py`.
pub fn python_unicode_tables() -> PythonUnicodeTables {
    let mut numeric = Vec::new();
    let mut digits = Vec::new();
    let mut decimals = Vec::new();

    for code_point in 0..=0x10FFFF {
        if (u32::from(b'0')..=u32::from(b'9')).contains(&code_point) {
            continue;
        }

        let Some(character) = char::from_u32(code_point) else {
            continue;
        };

        let numeric_value = unicode_numeric_value(character);
        let decimal = digit_value(character).is_some() && numeric_value.is_none();
        let digit = decimal || unicode_digit_value(character).is_some();
        let number = decimal || numeric_value.is_some();

        if number {
            numeric.push(code_point);
        }
        if digit {
            digits.push(code_point);
        }
        if decimal {
            decimals.push(code_point);
        }
    }

    PythonUnicodeTables {
        numeric,
        digits,
        decimals,
    }
}

const PYTHON_NS_DUMB: i64 = 1_i64 << 31;
const FLAG_FLOAT: i64 = 1;
const FLAG_PATH: i64 = 1 << 3;
const FLAG_LOCALE_ALPHA: i64 = 1 << 4;
const FLAG_LOCALE_NUMERIC: i64 = 1 << 5;
const FLAG_IGNORE_CASE: i64 = 1 << 6;
const FLAG_LOWERCASE_FIRST: i64 = 1 << 7;
const FLAG_UNGROUP_LETTERS: i64 = 1 << 9;
const FLAG_NAN_LAST: i64 = 1 << 10;

fn has_flag(bits: i64, flag: i64) -> bool {
    bits & flag != 0
}

/// True when Python's input-string factory would return its identity function.
pub fn python_input_transform_is_noop(bits: i64) -> bool {
    let dumb = has_flag(bits, PYTHON_NS_DUMB);
    let lowercase_first = has_flag(bits, FLAG_LOWERCASE_FIRST);
    let swap = dumb ^ lowercase_first;

    !swap && !has_flag(bits, FLAG_IGNORE_CASE) && !has_flag(bits, FLAG_LOCALE_NUMERIC)
}

/// Apply the exact branch decisions of Python's input-string transform factory.
///
/// Unicode normalization is intentionally not performed here; upstream applies
/// normalization later in `parse_string_factory`, not in this factory.
pub fn python_input_transform(input: &str, bits: i64, locale_profile: LocaleProfile) -> String {
    let dumb = has_flag(bits, PYTHON_NS_DUMB);
    let lowercase_first = has_flag(bits, FLAG_LOWERCASE_FIRST);
    let mut output = if dumb ^ lowercase_first {
        swap_case(input)
    } else {
        input.to_string()
    };

    if has_flag(bits, FLAG_IGNORE_CASE) {
        output = case_fold(&output);
    }

    if has_flag(bits, FLAG_LOCALE_NUMERIC) {
        output = normalize_localized_numbers(&output, locale_profile, has_flag(bits, FLAG_FLOAT));
    }

    output
}

/// Return the final-data factory mode.
///
/// 0: return the split tuple unchanged
/// 1: CAPITALFIRST wrapper, no swap-case
/// 2: CAPITALFIRST wrapper, swap-case the gross-sort prefix
pub fn python_final_transform_mode(bits: i64) -> u8 {
    let capital_first = has_flag(bits, FLAG_UNGROUP_LETTERS) && has_flag(bits, FLAG_LOCALE_ALPHA);

    if !capital_first {
        return 0;
    }

    let swap = has_flag(bits, PYTHON_NS_DUMB) && has_flag(bits, FLAG_LOWERCASE_FIRST);

    if swap { 2 } else { 1 }
}

pub fn python_group_letters(input: &str) -> String {
    group_letters(input)
}

/// Return a marker for each original item indicating whether a separator must
/// be emitted immediately before it. A tag value of 1 means exact Python
/// `int`/`float`; every other value is non-numeric.
pub fn python_separator_plan(type_tags: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(type_tags.len());
    let mut previous_numeric = false;

    for (index, tag) in type_tags.iter().copied().enumerate() {
        let numeric = tag == 1;
        let insert = numeric && (index == 0 || previous_numeric);
        output.push(u8::from(insert));
        previous_numeric = numeric;
    }

    output
}

fn is_decimal_digit(character: char) -> bool {
    digit_value(character).is_some() && unicode_numeric_value(character).is_none()
}

fn suffix_candidates(base: &str) -> Vec<String> {
    let dot_positions: Vec<usize> = base
        .char_indices()
        .filter_map(|(index, character)| {
            let valid = character == '.' && index != 0 && index + character.len_utf8() < base.len();
            valid.then_some(index)
        })
        .collect();

    dot_positions
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = dot_positions.get(index + 1).copied().unwrap_or(base.len());
            base[*start..end].to_string()
        })
        .collect()
}

fn split_python_base_extensions(base: &str) -> (String, Vec<String>) {
    let candidates = suffix_candidates(base);
    let mut selected_reversed = Vec::new();

    for (index, suffix) in candidates.iter().rev().enumerate() {
        let starts_with_number = suffix.chars().nth(1).is_some_and(is_decimal_digit);

        if starts_with_number || index > 1 || suffix.chars().count() > 5 {
            break;
        }

        selected_reversed.push(suffix.clone());
    }

    selected_reversed.reverse();
    let suffix_length: usize = selected_reversed.iter().map(String::len).sum();
    let stem = base[..base.len() - suffix_length].to_string();

    (stem, selected_reversed)
}

/// Python `PurePath`-compatible path components for the adapter host.
///
/// `windows_host` controls separator rules. The current original-suite runner
/// passes the host value explicitly, so the Rust answer mirrors the Python
/// interpreter executing the unchanged tests.
pub fn python_path_components(input: &str, windows_host: bool, treat_base: bool) -> Vec<String> {
    let normalized_empty = input.is_empty() || input == ".";
    if normalized_empty {
        return vec![".".to_string()];
    }

    let is_separator = |character: char| character == '/' || (windows_host && character == '\\');

    let mut components = Vec::new();

    if input.chars().next().is_some_and(is_separator) {
        components.push(if windows_host {
            input.chars().next().unwrap_or('\\').to_string()
        } else {
            "/".to_string()
        });
    }

    let mut raw: Vec<&str> = input
        .split(is_separator)
        .filter(|component| !component.is_empty() && *component != ".")
        .collect();

    if raw.is_empty() {
        return if components.is_empty() {
            vec![".".to_string()]
        } else {
            components
        };
    }

    let base = raw.pop().expect("checked non-empty path component list");
    components.extend(raw.into_iter().map(str::to_string));

    if treat_base {
        let (stem, suffixes) = split_python_base_extensions(base);

        if !stem.is_empty() {
            components.push(stem);
        }

        components.extend(suffixes);
    } else {
        components.push(base.to_string());
    }

    components
}

/// Transform bytes and report whether Python's PATH nesting is required.
pub fn python_parse_bytes(input: &[u8], bits: i64) -> (Vec<u8>, bool) {
    let transformed = if has_flag(bits, FLAG_IGNORE_CASE) {
        input.iter().map(u8::to_ascii_lowercase).collect()
    } else {
        input.to_vec()
    };

    (transformed, has_flag(bits, FLAG_PATH))
}

/// Return `(wrapper, kind, positive_infinity, suffix)`.
///
/// wrapper: 0 plain, 1 PATH, 2 CAPITALFIRST, 3 PATH+CAPITALFIRST
/// kind: 0 preserve original value, 1 use infinity/suffix
pub fn python_number_plan(value: &NaturalValue, bits: i64) -> Result<(u8, u8, bool, u8), String> {
    let path = has_flag(bits, FLAG_PATH);
    let capital = has_flag(bits, FLAG_UNGROUP_LETTERS) && has_flag(bits, FLAG_LOCALE_ALPHA);

    let wrapper = match (path, capital) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    };

    let nan_last = has_flag(bits, FLAG_NAN_LAST);
    let positive = nan_last;

    let special_suffix = match value {
        NaturalValue::None => Some(2),

        NaturalValue::Float(number) if number.is_nan() => Some(if nan_last { 3 } else { 1 }),

        NaturalValue::Float(number)
            if (!nan_last && *number == f64::NEG_INFINITY)
                || (nan_last && *number == f64::INFINITY) =>
        {
            Some(if nan_last { 1 } else { 3 })
        }

        NaturalValue::Integer(_) | NaturalValue::Float(_) => None,

        _ => {
            return Err("parse-number-plan requires an integer, float, or None value".to_string());
        }
    };

    Ok(match special_suffix {
        Some(suffix) => (wrapper, 1, positive, suffix),
        None => (wrapper, 0, positive, 0),
    })
}

/// A Python-visible string component after Rust performs numeric conversion
/// and any groupletters transformation. Locale collation is intentionally
/// applied by the Python bridge because the unchanged suite inspects the host
/// Python locale library's exact return representation.
const FLAG_COMPATIBILITY_NORMALIZE: i64 = 1 << 11;

#[derive(Debug, Clone, PartialEq)]
pub enum PythonComponent {
    Text(String),
    Integer(BigInt),
    Float(f64),
}

/// Normalize text exactly at the boundary used by Python's
/// `parse_string_factory`.
pub fn python_normalize_string(input: &str, bits: i64, compose: bool) -> String {
    let compatibility = has_flag(bits, FLAG_COMPATIBILITY_NORMALIZE);

    match (compatibility, compose) {
        (true, true) => input.nfkc().collect(),
        (true, false) => input.nfkd().collect(),
        (false, true) => input.nfc().collect(),
        (false, false) => input.nfd().collect(),
    }
}

/// Return `(original_after_transform, compose_for_locale)` for the Python
/// parse-string orchestration layer.
pub fn python_parse_string_plan(bits: i64) -> (bool, bool) {
    let locale = has_flag(bits, FLAG_LOCALE_ALPHA);
    let dumb = has_flag(bits, PYTHON_NS_DUMB);
    (!(dumb && locale), locale)
}

/// Convert string components through Rust. Numeric parsing and groupletters
/// decisions happen here; Python only applies its platform-specific strxfrm
/// representation to returned text when `use_locale` is true.
pub fn python_transform_components(inputs: &[String], bits: i64) -> (bool, Vec<PythonComponent>) {
    let use_locale = has_flag(bits, FLAG_LOCALE_ALPHA);
    let dumb = has_flag(bits, PYTHON_NS_DUMB);
    let group = has_flag(bits, 1 << 8) || (use_locale && dumb);
    let use_float = has_flag(bits, FLAG_FLOAT);
    let nan_value = if has_flag(bits, FLAG_NAN_LAST) {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    };

    let values = inputs
        .iter()
        .map(|input| {
            if use_float {
                if let Some(value) = python_fast_float(input, nan_value) {
                    return PythonComponent::Float(value);
                }
            } else if let Some(value) = python_fast_int(input) {
                return PythonComponent::Integer(value);
            }

            let text = if group {
                python_group_letters(input)
            } else {
                input.clone()
            };

            PythonComponent::Text(text)
        })
        .collect();

    (use_locale, values)
}

/// Classify a typed bridge value using Rust's value model.
///
/// 0 text, 1 bytes, 2 sequence, 3 scalar number/None.
pub fn python_value_kind(value: &NaturalValue) -> u8 {
    match value {
        NaturalValue::Text(_) => 0,
        NaturalValue::Bytes(_) => 1,
        NaturalValue::Sequence(_) => 2,
        NaturalValue::Integer(_) | NaturalValue::Float(_) | NaturalValue::None => 3,
    }
}

pub fn python_decode_bytes(input: &[u8], encoding: &str) -> Result<String, String> {
    let normalized = encoding.trim().to_ascii_lowercase().replace(['-', '_'], "");

    match normalized.as_str() {
        "ascii" => decode_ascii_bytes(input).map_err(|error| error.to_string()),
        "utf8" => decode_utf8_bytes(input).map_err(|error| error.to_string()),
        "latin1" | "iso88591" | "l1" => Ok(decode_latin1_bytes(input)),
        _ => Err(format!("unsupported decoder encoding: {encoding}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_python_style_float_inputs() {
        assert_eq!(python_fast_float("45.8e-2", f64::INFINITY), Some(0.458));
        assert_eq!(python_fast_float("۱۲.۱۲", f64::INFINITY), Some(12.12));
        assert_eq!(python_fast_float("⅓", f64::INFINITY), Some(1.0 / 3.0));
        assert_eq!(python_fast_float("invalid", f64::INFINITY), None);
    }

    #[test]
    fn replaces_nan_with_requested_sentinel() {
        assert_eq!(python_fast_float("-NaN", 7.0), Some(7.0));
    }

    #[test]
    fn parses_python_style_integer_inputs() {
        assert_eq!(python_fast_int("-۱۲"), Some(BigInt::from(-12)));
        assert_eq!(python_fast_int("²"), Some(BigInt::from(2)));
        assert_eq!(python_fast_int("45.8"), None);
    }

    #[test]
    fn exposes_non_ascii_unicode_collections() {
        let tables = python_unicode_tables();

        assert!(tables.numeric.contains(&u32::from('⅓')));
        assert!(tables.digits.contains(&u32::from('①')));
        assert!(tables.decimals.contains(&u32::from('۱')));
        assert!(!tables.numeric.contains(&u32::from('1')));
    }
}
