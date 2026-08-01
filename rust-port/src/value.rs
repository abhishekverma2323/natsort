use crate::locale::locale_sort_key;
use crate::separator::numeric_prefix;
use num_bigint::BigInt;
use std::cmp::Ordering;

use crate::options::SortOptions;
use crate::sort::{compare_numeric_strings, string_key_components};
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum NaturalValue {
    Text(String),
    Bytes(Vec<u8>),
    Integer(BigInt),
    Float(f64),
    None,
    Sequence(Vec<NaturalValue>),
}

impl From<Vec<u8>> for NaturalValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<&[u8]> for NaturalValue {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(value.to_vec())
    }
}

impl NaturalValue {
    pub(crate) fn presort_string(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Bytes(value) => value.iter().map(|byte| char::from(*byte)).collect(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) if value.is_nan() => "nan".to_string(),
            Self::Float(value) if *value == f64::INFINITY => "inf".to_string(),
            Self::Float(value) if *value == f64::NEG_INFINITY => "-inf".to_string(),
            Self::Float(value) => value.to_string(),
            Self::None => "None".to_string(),
            Self::Sequence(values) => {
                let contents = values
                    .iter()
                    .map(Self::presort_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("[{contents}]")
            }
        }
    }
}

impl From<&str> for NaturalValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for NaturalValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<BigInt> for NaturalValue {
    fn from(value: BigInt) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for NaturalValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<f32> for NaturalValue {
    fn from(value: f32) -> Self {
        Self::Float(f64::from(value))
    }
}

impl From<Vec<NaturalValue>> for NaturalValue {
    fn from(value: Vec<NaturalValue>) -> Self {
        Self::Sequence(value)
    }
}

macro_rules! impl_integer_conversion {
    ($($integer:ty),+ $(,)?) => {
        $(
            impl From<$integer> for NaturalValue {
                fn from(value: $integer) -> Self {
                    Self::Integer(BigInt::from(value))
                }
            }
        )+
    };
}

impl_integer_conversion!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
);

#[derive(Debug, Clone)]
pub enum NumericKey {
    NegativeInfinity,
    Finite(String),
    PositiveInfinity,
}

impl PartialEq for NumericKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for NumericKey {}

impl PartialOrd for NumericKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NumericKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::NegativeInfinity, Self::NegativeInfinity)
            | (Self::PositiveInfinity, Self::PositiveInfinity) => Ordering::Equal,

            (Self::NegativeInfinity, _) => Ordering::Less,
            (_, Self::NegativeInfinity) => Ordering::Greater,

            (Self::PositiveInfinity, _) => Ordering::Greater,
            (_, Self::PositiveInfinity) => Ordering::Less,

            (Self::Finite(left), Self::Finite(right)) => compare_numeric_strings(left, right),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyAtom {
    Text(String),
    LocaleText(Vec<u8>),
    Number(NumericKey),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum NaturalKey {
    Atomic(Vec<KeyAtom>),
    Sequence(Vec<NaturalKey>),
}

impl PartialEq for NaturalKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for NaturalKey {}

impl PartialOrd for NaturalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaturalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Atomic(left), Self::Atomic(right)) => left.cmp(right),

            (Self::Sequence(left), Self::Sequence(right)) => left.cmp(right),

            (Self::Atomic(_), Self::Sequence(_)) => Ordering::Less,

            (Self::Sequence(_), Self::Atomic(_)) => Ordering::Greater,
        }
    }
}

fn text_atom(value: &str, options: SortOptions) -> KeyAtom {
    if options.locale_alpha {
        KeyAtom::LocaleText(locale_sort_key(value, options.locale_profile))
    } else {
        KeyAtom::Text(value.to_string())
    }
}

fn token_atoms(tokens: &[Token], options: SortOptions) -> Vec<KeyAtom> {
    let mut atoms: Vec<KeyAtom> = tokens
        .iter()
        .map(|token| match token {
            Token::Text(value) => text_atom(value, options),
            Token::Number(value) => KeyAtom::Number(NumericKey::Finite(value.clone())),
        })
        .collect();

    if !matches!(
        atoms.first(),
        Some(KeyAtom::Text(_) | KeyAtom::LocaleText(_))
    ) {
        atoms.insert(0, text_atom("", options));
    }

    atoms
}

fn prepare_bytes(input: &[u8], options: SortOptions) -> Vec<u8> {
    let mut prepared = input.to_vec();

    // Python natsort treats bytes as an opaque lexicographic value.
    // Of the text flags, only IGNORECASE transforms byte keys.
    if options.ignore_case {
        prepared.make_ascii_lowercase();
    }

    prepared
}

fn bytes_key(input: &[u8], options: SortOptions) -> NaturalKey {
    let key = NaturalKey::Atomic(vec![KeyAtom::Bytes(prepare_bytes(input, options))]);

    // PATH does not split bytes into path components; it only wraps
    // the byte key in the same outer path-key shape as Python natsort.
    if options.path {
        NaturalKey::Sequence(vec![key])
    } else {
        key
    }
}

fn text_key(input: &str, options: SortOptions) -> NaturalKey {
    let components = string_key_components(input, options);

    if options.path {
        NaturalKey::Sequence(
            components
                .iter()
                .map(|component| NaturalKey::Atomic(token_atoms(component, options)))
                .collect(),
        )
    } else {
        let tokens = components
            .first()
            .expect("non-path string key has one component");

        NaturalKey::Atomic(token_atoms(tokens, options))
    }
}

fn numeric_atomic_key(
    number: NumericKey,
    marker: Option<&str>,
    options: SortOptions,
) -> NaturalKey {
    let mut atoms = vec![
        text_atom(numeric_prefix(options), options),
        KeyAtom::Number(number),
    ];

    if let Some(marker) = marker {
        atoms.push(text_atom(marker, options));
    }

    NaturalKey::Atomic(atoms)
}

fn wrap_for_path(key: NaturalKey, options: SortOptions) -> NaturalKey {
    if options.path {
        NaturalKey::Sequence(vec![key])
    } else {
        key
    }
}

fn integer_key(value: &BigInt, options: SortOptions) -> NaturalKey {
    wrap_for_path(
        numeric_atomic_key(NumericKey::Finite(value.to_string()), None, options),
        options,
    )
}

fn float_key(value: f64, options: SortOptions) -> NaturalKey {
    let key = if options.nan_last {
        if value.is_nan() {
            numeric_atomic_key(NumericKey::PositiveInfinity, Some("3"), options)
        } else if value == f64::INFINITY {
            numeric_atomic_key(NumericKey::PositiveInfinity, Some("1"), options)
        } else if value == f64::NEG_INFINITY {
            numeric_atomic_key(NumericKey::NegativeInfinity, None, options)
        } else {
            numeric_atomic_key(NumericKey::Finite(value.to_string()), None, options)
        }
    } else if value.is_nan() {
        numeric_atomic_key(NumericKey::NegativeInfinity, Some("1"), options)
    } else if value == f64::NEG_INFINITY {
        numeric_atomic_key(NumericKey::NegativeInfinity, Some("3"), options)
    } else if value == f64::INFINITY {
        numeric_atomic_key(NumericKey::PositiveInfinity, None, options)
    } else {
        numeric_atomic_key(NumericKey::Finite(value.to_string()), None, options)
    };

    wrap_for_path(key, options)
}

fn none_key(options: SortOptions) -> NaturalKey {
    let infinity = if options.nan_last {
        NumericKey::PositiveInfinity
    } else {
        NumericKey::NegativeInfinity
    };

    wrap_for_path(numeric_atomic_key(infinity, Some("2"), options), options)
}

pub fn natsort_key_with_options(value: &NaturalValue, options: SortOptions) -> NaturalKey {
    match value {
        NaturalValue::Text(value) => text_key(value, options),

        NaturalValue::Bytes(value) => bytes_key(value, options),

        NaturalValue::Integer(value) => integer_key(value, options),

        NaturalValue::Float(value) => float_key(*value, options),

        NaturalValue::None => none_key(options),

        NaturalValue::Sequence(values) => NaturalKey::Sequence(
            values
                .iter()
                .map(|value| natsort_key_with_options(value, options))
                .collect(),
        ),
    }
}

pub fn natsort_key(value: &NaturalValue) -> NaturalKey {
    natsort_key_with_options(value, SortOptions::new())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NaturalKeyGenerator {
    options: SortOptions,
}

impl NaturalKeyGenerator {
    pub const fn new(options: SortOptions) -> Self {
        Self { options }
    }

    pub fn key(&self, value: &NaturalValue) -> NaturalKey {
        natsort_key_with_options(value, self.options)
    }

    pub const fn options(&self) -> SortOptions {
        self.options
    }
}

pub fn natsort_keygen() -> NaturalKeyGenerator {
    NaturalKeyGenerator::default()
}

pub const fn natsort_keygen_with_options(options: SortOptions) -> NaturalKeyGenerator {
    NaturalKeyGenerator::new(options)
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

pub fn natsorted_values(items: &[NaturalValue]) -> Vec<NaturalValue> {
    natsorted_values_with_options(items, SortOptions::new())
}

pub fn natsorted_values_with_options(
    items: &[NaturalValue],
    options: SortOptions,
) -> Vec<NaturalValue> {
    let mut values: Vec<(NaturalValue, NaturalKey)> = items
        .iter()
        .cloned()
        .map(|value| {
            let key = natsort_key_with_options(&value, options);

            (value, key)
        })
        .collect();

    if options.presort {
        values.sort_by(|left, right| {
            let ordering = left.0.presort_string().cmp(&right.0.presort_string());

            if options.reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    values.sort_by(|left, right| {
        let ordering = left.1.cmp(&right.1);

        if options.reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });

    values.into_iter().map(|(value, _)| value).collect()
}

pub fn realsorted_values(items: &[NaturalValue]) -> Vec<NaturalValue> {
    realsorted_values_with_options(items, SortOptions::new())
}

pub fn realsorted_values_with_options(
    items: &[NaturalValue],
    options: SortOptions,
) -> Vec<NaturalValue> {
    natsorted_values_with_options(items, real_options(options))
}

pub fn humansorted_values(items: &[NaturalValue]) -> Vec<NaturalValue> {
    humansorted_values_with_options(items, SortOptions::new())
}

pub fn humansorted_values_with_options(
    items: &[NaturalValue],
    options: SortOptions,
) -> Vec<NaturalValue> {
    natsorted_values_with_options(items, human_options(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> NaturalValue {
        NaturalValue::from(value)
    }

    fn sequence(values: Vec<NaturalValue>) -> NaturalValue {
        NaturalValue::Sequence(values)
    }

    #[test]
    fn sorts_mixed_text_and_integers() {
        let input = vec![text("a2"), 3.into(), text("a1"), 2.into()];

        assert_eq!(
            natsorted_values(&input),
            vec![2.into(), 3.into(), text("a1"), text("a2"),]
        );
    }

    #[test]
    fn sorts_numeric_strings_and_integers() {
        let input = vec![text("10"), 2.into(), text("1"), 11.into()];

        assert_eq!(
            natsorted_values(&input),
            vec![text("1"), 2.into(), text("10"), 11.into(),]
        );
    }

    #[test]
    fn real_sorts_signed_mixed_values() {
        let input = vec![
            text("value5"),
            (-3).into(),
            text("value-2"),
            1.into(),
            text("value1"),
        ];

        assert_eq!(
            realsorted_values(&input),
            vec![
                (-3).into(),
                1.into(),
                text("value-2"),
                text("value1"),
                text("value5"),
            ]
        );
    }

    #[test]
    fn sorts_direct_numeric_values() {
        let input = vec![5.1.into(), (-3.0).into(), 5.3.into(), 2.into()];

        assert_eq!(
            natsorted_values(&input),
            vec![(-3.0).into(), 2.into(), 5.1.into(), 5.3.into(),]
        );
    }

    #[test]
    fn preserves_equivalent_number_stability() {
        let input = vec![1.into(), 1.0.into(), 1.0.into(), 2.into()];

        assert_eq!(natsorted_values(&input), input);
    }

    #[test]
    fn sorts_positive_and_negative_infinity() {
        let input = vec![
            f64::INFINITY.into(),
            5.into(),
            f64::NEG_INFINITY.into(),
            0.into(),
        ];

        assert_eq!(
            natsorted_values(&input),
            vec![
                f64::NEG_INFINITY.into(),
                0.into(),
                5.into(),
                f64::INFINITY.into(),
            ]
        );
    }

    #[test]
    fn default_places_nan_then_none_then_negative_infinity() {
        let input = vec![
            3.into(),
            NaturalValue::None,
            f64::NAN.into(),
            f64::NEG_INFINITY.into(),
            2.into(),
        ];

        let sorted = natsorted_values(&input);

        assert!(matches!(
            sorted[0],
            NaturalValue::Float(value) if value.is_nan()
        ));
        assert_eq!(sorted[1], NaturalValue::None);
        assert_eq!(sorted[2], NaturalValue::Float(f64::NEG_INFINITY),);
        assert_eq!(sorted[3], 2.into());
        assert_eq!(sorted[4], 3.into());
    }

    #[test]
    fn nan_last_places_infinity_then_none_then_nan() {
        let input = vec![
            3.into(),
            NaturalValue::None,
            f64::NAN.into(),
            f64::INFINITY.into(),
            2.into(),
        ];

        let options = SortOptions::new().nan_last(true);

        let sorted = natsorted_values_with_options(&input, options);

        assert_eq!(sorted[0], 2.into());
        assert_eq!(sorted[1], 3.into());
        assert_eq!(sorted[2], NaturalValue::Float(f64::INFINITY),);
        assert_eq!(sorted[3], NaturalValue::None);
        assert!(matches!(
            sorted[4],
            NaturalValue::Float(value) if value.is_nan()
        ));
    }

    #[test]
    fn sorts_nested_string_sequences() {
        let input = vec![
            sequence(vec![text("a10"), text("b2")]),
            sequence(vec![text("a2"), text("b10")]),
            sequence(vec![text("a2"), text("b2")]),
        ];

        assert_eq!(
            natsorted_values(&input),
            vec![
                sequence(vec![text("a2"), text("b2")]),
                sequence(vec![text("a2"), text("b10")]),
                sequence(vec![text("a10"), text("b2")]),
            ]
        );
    }

    #[test]
    fn sorts_nested_real_sequences() {
        let input = vec![
            sequence(vec![text("x5.10"), text("y-2")]),
            sequence(vec![text("x5.3"), text("y1")]),
            sequence(vec![text("x2"), text("y10")]),
        ];

        assert_eq!(
            realsorted_values(&input),
            vec![
                sequence(vec![text("x2"), text("y10")]),
                sequence(vec![text("x5.10"), text("y-2"),]),
                sequence(vec![text("x5.3"), text("y1")]),
            ]
        );
    }

    #[test]
    fn sorts_nested_mixed_sequences() {
        let input = vec![
            sequence(vec![text("a2"), 10.into()]),
            sequence(vec![text("a2"), 2.into()]),
            sequence(vec![text("a1"), 20.into()]),
        ];

        assert_eq!(
            natsorted_values(&input),
            vec![
                sequence(vec![text("a1"), 20.into()]),
                sequence(vec![text("a2"), 2.into()]),
                sequence(vec![text("a2"), 10.into()]),
            ]
        );
    }

    #[test]
    fn sorts_deeply_nested_sequences() {
        let input = vec![
            sequence(vec![
                sequence(vec![text("a2")]),
                sequence(vec![text("b10")]),
            ]),
            sequence(vec![sequence(vec![text("a2")]), sequence(vec![text("b2")])]),
            sequence(vec![
                sequence(vec![text("a1")]),
                sequence(vec![text("b20")]),
            ]),
        ];

        assert_eq!(
            natsorted_values(&input),
            vec![
                sequence(vec![
                    sequence(vec![text("a1")]),
                    sequence(vec![text("b20")]),
                ]),
                sequence(vec![sequence(vec![text("a2")]), sequence(vec![text("b2")]),]),
                sequence(vec![
                    sequence(vec![text("a2")]),
                    sequence(vec![text("b10")]),
                ]),
            ]
        );
    }

    #[test]
    fn mixed_sort_supports_reverse() {
        let input = vec![text("10"), 2.into(), text("1"), 11.into()];

        let options = SortOptions::new().reverse(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![11.into(), text("10"), 2.into(), text("1"),]
        );
    }

    #[test]
    fn keygen_uses_default_options() {
        let generator = natsort_keygen();

        assert_eq!(generator.options(), SortOptions::new(),);
    }

    #[test]
    fn real_key_equates_scientific_value() {
        let generator = natsort_keygen_with_options(SortOptions::new().float(true).signed(true));

        assert_eq!(
            generator.key(&text("a-5.034e2")),
            natsort_key_with_options(
                &text("a-503.4"),
                SortOptions::new().float(true).signed(true),
            ),
        );
    }

    #[test]
    fn default_direct_number_key_differs_from_decimal_string_key() {
        assert_ne!(
            natsort_key(&NaturalValue::from(56.7)),
            natsort_key(&text("56.7")),
        );
    }

    #[test]
    fn float_mode_direct_number_key_matches_decimal_string_key() {
        let options = SortOptions::new().float(true);

        assert_eq!(
            natsort_key_with_options(&NaturalValue::from(56.7), options,),
            natsort_key_with_options(&text("56.7"), options,),
        );
    }

    #[test]
    fn nested_key_is_recursive() {
        let value = sequence(vec![text("x5.10"), text("y-2"), 3.into()]);

        let options = SortOptions::new().float(true).signed(true);

        assert!(matches!(
            natsort_key_with_options(&value, options),
            NaturalKey::Sequence(values)
                if values.len() == 3
        ));
    }

    #[test]
    fn none_and_nan_keys_have_different_markers() {
        let none = natsort_key(&NaturalValue::None);
        let nan = natsort_key(&NaturalValue::from(f64::NAN));

        assert_ne!(none, nan);
        assert!(nan < none);
    }

    #[test]
    fn nan_last_reverses_special_value_location() {
        let options = SortOptions::new().nan_last(true);

        let infinity = natsort_key_with_options(&NaturalValue::from(f64::INFINITY), options);

        let none = natsort_key_with_options(&NaturalValue::None, options);

        let nan = natsort_key_with_options(&NaturalValue::from(f64::NAN), options);

        assert!(infinity < none);
        assert!(none < nan);
    }

    #[test]
    fn supports_arbitrarily_large_direct_integers() {
        let huge =
            BigInt::parse_bytes(b"999999999999999999999999999999999", 10).expect("valid integer");

        let input = vec![NaturalValue::Integer(huge.clone()), 2.into(), 10.into()];

        assert_eq!(
            natsorted_values(&input),
            vec![2.into(), 10.into(), NaturalValue::Integer(huge),]
        );
    }

    #[test]
    fn sorts_bytes_lexicographically() {
        let input = vec![
            NaturalValue::from(b"a10".as_slice()),
            NaturalValue::from(b"a2".as_slice()),
            NaturalValue::from(b"A1".as_slice()),
        ];

        assert_eq!(
            natsorted_values(&input),
            vec![
                NaturalValue::from(b"A1".as_slice()),
                NaturalValue::from(b"a10".as_slice()),
                NaturalValue::from(b"a2".as_slice()),
            ]
        );
    }

    #[test]
    fn bytes_ignore_case_matches_python() {
        let input = vec![
            NaturalValue::from(b"a10".as_slice()),
            NaturalValue::from(b"A2".as_slice()),
            NaturalValue::from(b"a1".as_slice()),
        ];

        let options = SortOptions::new().ignore_case(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![
                NaturalValue::from(b"a1".as_slice()),
                NaturalValue::from(b"a10".as_slice()),
                NaturalValue::from(b"A2".as_slice()),
            ]
        );
    }

    #[test]
    fn bytes_path_mode_remains_lexicographic() {
        let input = vec![
            NaturalValue::from(b"folder10/file".as_slice()),
            NaturalValue::from(b"folder2/file".as_slice()),
            NaturalValue::from(b"folder1/file".as_slice()),
        ];

        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![
                NaturalValue::from(b"folder1/file".as_slice()),
                NaturalValue::from(b"folder10/file".as_slice()),
                NaturalValue::from(b"folder2/file".as_slice()),
            ]
        );
    }

    #[test]
    fn bytes_support_reverse_order() {
        let input = vec![
            NaturalValue::from(b"a10".as_slice()),
            NaturalValue::from(b"a2".as_slice()),
            NaturalValue::from(b"A1".as_slice()),
        ];

        let options = SortOptions::new().reverse(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![
                NaturalValue::from(b"a2".as_slice()),
                NaturalValue::from(b"a10".as_slice()),
                NaturalValue::from(b"A1".as_slice()),
            ]
        );
    }

    #[test]
    fn byte_key_contains_one_bytes_atom() {
        assert!(matches!(
            natsort_key(&NaturalValue::from(b"a10".as_slice())),
            NaturalKey::Atomic(atoms)
                if atoms == vec![KeyAtom::Bytes(b"a10".to_vec())]
        ));
    }

    #[test]
    fn byte_path_key_is_wrapped_without_splitting() {
        let options = SortOptions::new().path(true);

        assert!(matches!(
            natsort_key_with_options(
                &NaturalValue::from(b"folder10/file".as_slice()),
                options,
            ),
            NaturalKey::Sequence(values)
                if values
                    == vec![NaturalKey::Atomic(vec![
                        KeyAtom::Bytes(b"folder10/file".to_vec())
                    ])]
        ));
    }

    #[test]
    fn byte_lowercase_first_does_not_modify_key() {
        let options = SortOptions::new().lowercase_first(true);

        assert_eq!(
            natsort_key_with_options(&NaturalValue::from(b"A10".as_slice()), options,),
            natsort_key(&NaturalValue::from(b"A10".as_slice())),
        );
    }

    #[test]
    fn byte_group_letters_does_not_modify_key() {
        let options = SortOptions::new().group_letters(true);

        assert_eq!(
            natsort_key_with_options(&NaturalValue::from(b"A10".as_slice()), options,),
            natsort_key(&NaturalValue::from(b"A10".as_slice())),
        );
    }

    #[test]
    fn num_after_places_direct_numbers_after_text() {
        let input = vec![
            NaturalValue::from("0"),
            NaturalValue::from(1.5),
            NaturalValue::from("2"),
            NaturalValue::from(3),
            NaturalValue::from("ä"),
            NaturalValue::from("Ä"),
            NaturalValue::from("b"),
            NaturalValue::from("Z"),
        ];

        let options = SortOptions::new().num_after(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![
                NaturalValue::from("Ä"),
                NaturalValue::from("Z"),
                NaturalValue::from("ä"),
                NaturalValue::from("b"),
                NaturalValue::from("0"),
                NaturalValue::from(1.5),
                NaturalValue::from("2"),
                NaturalValue::from(3),
            ]
        );
    }

    #[test]
    fn num_after_combines_with_signed_values() {
        let input = vec![
            NaturalValue::from(-10),
            NaturalValue::from("value-2"),
            NaturalValue::from(2),
            NaturalValue::from("apple"),
            NaturalValue::from("value1"),
        ];

        let options = SortOptions::new().num_after(true).signed(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![
                NaturalValue::from("apple"),
                NaturalValue::from("value-2"),
                NaturalValue::from("value1"),
                NaturalValue::from(-10),
                NaturalValue::from(2),
            ]
        );
    }

    #[test]
    fn num_after_combines_with_float_values() {
        let input = vec![
            NaturalValue::from(1.5),
            NaturalValue::from("value1.25"),
            NaturalValue::from(2),
            NaturalValue::from("apple"),
            NaturalValue::from("value1.5"),
        ];

        let options = SortOptions::new().num_after(true).float(true);

        assert_eq!(
            natsorted_values_with_options(&input, options),
            vec![
                NaturalValue::from("apple"),
                NaturalValue::from("value1.25"),
                NaturalValue::from("value1.5"),
                NaturalValue::from(1.5),
                NaturalValue::from(2),
            ]
        );
    }

    #[test]
    fn num_after_numeric_string_and_integer_keys_are_equal() {
        let options = SortOptions::new().num_after(true);

        assert_eq!(
            natsort_key_with_options(&NaturalValue::from("73"), options,),
            natsort_key_with_options(&NaturalValue::from(73), options,),
        );
    }

    #[test]
    fn num_after_places_text_key_before_direct_number_key() {
        let options = SortOptions::new().num_after(true);

        let text_key = natsort_key_with_options(&NaturalValue::from("apple"), options);

        let number_key = natsort_key_with_options(&NaturalValue::from(73), options);

        assert!(text_key < number_key);
    }

    #[test]
    fn num_after_does_not_change_embedded_number_key() {
        let default_key = natsort_key(&NaturalValue::from("file2"));

        let num_after_key = natsort_key_with_options(
            &NaturalValue::from("file2"),
            SortOptions::new().num_after(true),
        );

        assert_eq!(default_key, num_after_key);
    }

    #[test]
    fn human_sort_matches_locale_options() {
        use crate::locale::LocaleProfile;

        let input = vec![text("Apple"), text("apple"), text("Äpfel"), text("banana")];

        let options = SortOptions::new().locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            humansorted_values_with_options(&input, options),
            natsorted_values_with_options(&input, options.locale(true)),
        );
    }

    #[test]
    fn locale_key_equates_localized_string_and_direct_number() {
        use crate::locale::LocaleProfile;

        let options = SortOptions::new()
            .float(true)
            .locale(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            natsort_key_with_options(&text("1,234.50"), options),
            natsort_key_with_options(&NaturalValue::from(1234.5), options),
        );
    }

    #[test]
    fn human_sort_handles_mixed_localized_values() {
        use crate::locale::LocaleProfile;

        let input = vec![
            text("1,000.50"),
            text("Apple2"),
            text("10.25"),
            text("apple10"),
            text("2.50"),
            text("Äpfel1"),
        ];

        let options = SortOptions::new()
            .float(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(
            humansorted_values_with_options(&input, options),
            vec![
                text("2.50"),
                text("10.25"),
                text("1,000.50"),
                text("Äpfel1"),
                text("apple10"),
                text("Apple2"),
            ]
        );
    }
}
