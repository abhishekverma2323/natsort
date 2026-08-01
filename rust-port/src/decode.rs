//! Byte-decoding adapters for natural-value sorting.
//!
//! `NaturalValue::Bytes` and `NaturalValue::Text` are distinct Rust variants.
//! Sorting without a decoder does not implicitly reinterpret bytes as text.
//! Use `Decoder` with `natsorted_values_with_decoder` when Python-style
//! `key=decoder(...)` text semantics are required for mixed bytes and strings.

use std::error::Error;
use std::fmt;

use crate::options::SortOptions;
use crate::value::{NaturalKey, NaturalValue, natsort_key_with_options};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeEncoding {
    Ascii,
    Latin1,
    Utf8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decoder {
    encoding: DecodeEncoding,
}

impl Decoder {
    pub const fn new(encoding: DecodeEncoding) -> Self {
        Self { encoding }
    }

    pub const fn ascii() -> Self {
        Self::new(DecodeEncoding::Ascii)
    }

    pub const fn latin1() -> Self {
        Self::new(DecodeEncoding::Latin1)
    }

    pub const fn utf8() -> Self {
        Self::new(DecodeEncoding::Utf8)
    }

    pub const fn encoding(self) -> DecodeEncoding {
        self.encoding
    }

    pub fn decode(self, value: &NaturalValue) -> Result<NaturalValue, DecodeError> {
        match value {
            NaturalValue::Bytes(bytes) => {
                let decoded = match self.encoding {
                    DecodeEncoding::Ascii => decode_ascii_bytes(bytes)?,

                    DecodeEncoding::Latin1 => decode_latin1_bytes(bytes),

                    DecodeEncoding::Utf8 => decode_utf8_bytes(bytes)?,
                };

                Ok(NaturalValue::Text(decoded))
            }

            _ => Ok(value.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedEncodingError {
    pub encoding: String,
}

impl fmt::Display for UnsupportedEncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unsupported decoder encoding: {}", self.encoding,)
    }
}

impl Error for UnsupportedEncodingError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidAscii {
        position: usize,
        byte: u8,
    },

    InvalidUtf8 {
        valid_up_to: usize,
        error_length: Option<usize>,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAscii { position, byte } => {
                write!(
                    formatter,
                    "invalid ASCII byte 0x{byte:02X} at position {position}",
                )
            }

            Self::InvalidUtf8 {
                valid_up_to,
                error_length,
            } => {
                write!(formatter, "invalid UTF-8 sequence at byte {valid_up_to}",)?;

                if let Some(length) = error_length {
                    write!(formatter, " with invalid length {length}",)?;
                }

                Ok(())
            }
        }
    }
}

impl Error for DecodeError {}

pub fn decoder(encoding: &str) -> Result<Decoder, UnsupportedEncodingError> {
    let normalized = encoding.trim().to_ascii_lowercase().replace(['-', '_'], "");

    match normalized.as_str() {
        "ascii" => Ok(Decoder::ascii()),

        "latin1" | "iso88591" | "l1" => Ok(Decoder::latin1()),

        "utf8" => Ok(Decoder::utf8()),

        _ => Err(UnsupportedEncodingError {
            encoding: encoding.to_string(),
        }),
    }
}

pub fn decode_ascii_bytes(bytes: &[u8]) -> Result<String, DecodeError> {
    if let Some((position, byte)) = bytes
        .iter()
        .copied()
        .enumerate()
        .find(|(_, byte)| !byte.is_ascii())
    {
        return Err(DecodeError::InvalidAscii { position, byte });
    }

    Ok(bytes.iter().map(|byte| char::from(*byte)).collect())
}

pub fn decode_latin1_bytes(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

pub fn decode_utf8_bytes(bytes: &[u8]) -> Result<String, DecodeError> {
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|error| DecodeError::InvalidUtf8 {
            valid_up_to: error.valid_up_to(),
            error_length: error.error_len(),
        })
}

pub fn as_ascii(value: &NaturalValue) -> Result<NaturalValue, DecodeError> {
    Decoder::ascii().decode(value)
}

pub fn as_utf8(value: &NaturalValue) -> Result<NaturalValue, DecodeError> {
    Decoder::utf8().decode(value)
}

pub fn natsorted_values_with_decoder(
    items: &[NaturalValue],
    decoder: Decoder,
) -> Result<Vec<NaturalValue>, DecodeError> {
    natsorted_values_with_decoder_and_options(items, decoder, SortOptions::new())
}

pub fn natsorted_values_with_decoder_and_options(
    items: &[NaturalValue],
    decoder: Decoder,
    options: SortOptions,
) -> Result<Vec<NaturalValue>, DecodeError> {
    let mut decorated: Vec<(NaturalValue, NaturalValue, NaturalKey)> = items
        .iter()
        .map(|original| {
            let decoded = decoder.decode(original)?;

            let key = natsort_key_with_options(&decoded, options);

            Ok((original.clone(), decoded, key))
        })
        .collect::<Result<_, DecodeError>>()?;

    if options.presort {
        decorated.sort_by(|left, right| {
            let ordering = left.1.presort_string().cmp(&right.1.presort_string());

            if options.reverse {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    decorated.sort_by(|left, right| {
        let ordering = left.2.cmp(&right.2);

        if options.reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });

    Ok(decorated
        .into_iter()
        .map(|(original, _, _)| original)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(value: &[u8]) -> NaturalValue {
        NaturalValue::Bytes(value.to_vec())
    }

    fn text(value: &str) -> NaturalValue {
        NaturalValue::Text(value.to_string())
    }

    #[test]
    fn creates_ascii_decoder() {
        assert_eq!(decoder("ascii"), Ok(Decoder::ascii()),);
    }

    #[test]
    fn creates_utf8_decoder() {
        assert_eq!(decoder("utf8"), Ok(Decoder::utf8()),);
    }

    #[test]
    fn accepts_utf8_dash_alias() {
        assert_eq!(decoder("UTF-8"), Ok(Decoder::utf8()),);
    }

    #[test]
    fn accepts_utf8_underscore_alias() {
        assert_eq!(decoder("utf_8"), Ok(Decoder::utf8()),);
    }

    #[test]
    fn rejects_unknown_encoding() {
        assert_eq!(
            decoder("utf16"),
            Err(UnsupportedEncodingError {
                encoding: "utf16".to_string(),
            })
        );
    }

    #[test]
    fn decodes_ascii_bytes() {
        assert_eq!(
            decode_ascii_bytes(b"natural10"),
            Ok("natural10".to_string()),
        );
    }

    #[test]
    fn rejects_non_ascii_byte() {
        assert_eq!(
            decode_ascii_bytes(&[b'a', 0xFF]),
            Err(DecodeError::InvalidAscii {
                position: 1,
                byte: 0xFF,
            })
        );
    }

    #[test]
    fn decodes_utf8_bytes() {
        assert_eq!(decode_utf8_bytes("café".as_bytes()), Ok("café".to_string()),);
    }

    #[test]
    fn rejects_invalid_utf8() {
        assert!(matches!(
            decode_utf8_bytes(&[0xFF]),
            Err(DecodeError::InvalidUtf8 { valid_up_to: 0, .. })
        ));
    }

    #[test]
    fn as_ascii_decodes_bytes() {
        assert_eq!(as_ascii(&bytes(b"natural10")), Ok(text("natural10")),);
    }

    #[test]
    fn as_ascii_preserves_text() {
        assert_eq!(as_ascii(&text("natural10")), Ok(text("natural10")),);
    }

    #[test]
    fn as_utf8_decodes_bytes() {
        assert_eq!(as_utf8(&bytes("café".as_bytes())), Ok(text("café")),);
    }

    #[test]
    fn as_utf8_preserves_integer() {
        let value = NaturalValue::from(123);

        assert_eq!(as_utf8(&value), Ok(value),);
    }

    #[test]
    fn decoder_preserves_none() {
        assert_eq!(
            Decoder::utf8().decode(&NaturalValue::None),
            Ok(NaturalValue::None),
        );
    }

    #[test]
    fn decoder_preserves_float() {
        let value = NaturalValue::from(12.5);

        assert_eq!(Decoder::utf8().decode(&value), Ok(value),);
    }

    #[test]
    fn sorts_mixed_bytes_and_strings() {
        let input = vec![bytes(b"a10"), text("a2"), bytes(b"a1")];

        assert_eq!(
            natsorted_values_with_decoder(&input, Decoder::utf8(),),
            Ok(vec![bytes(b"a1"), text("a2"), bytes(b"a10"),])
        );
    }

    #[test]
    fn decoder_sort_preserves_original_values() {
        let input = vec![bytes(b"a10"), text("a2")];

        let sorted = natsorted_values_with_decoder(&input, Decoder::utf8()).expect("valid UTF-8");

        assert!(matches!(sorted[1], NaturalValue::Bytes(_)));
    }

    #[test]
    fn decoder_sort_supports_reverse() {
        let input = vec![bytes(b"a10"), text("a2"), bytes(b"a1")];

        let options = SortOptions::new().reverse(true);

        assert_eq!(
            natsorted_values_with_decoder_and_options(&input, Decoder::utf8(), options,),
            Ok(vec![bytes(b"a10"), text("a2"), bytes(b"a1"),])
        );
    }

    #[test]
    fn decoder_sort_supports_ignore_case() {
        let input = vec![bytes(b"A10"), text("a2"), bytes(b"a1")];

        let options = SortOptions::new().ignore_case(true);

        assert_eq!(
            natsorted_values_with_decoder_and_options(&input, Decoder::utf8(), options,),
            Ok(vec![bytes(b"a1"), text("a2"), bytes(b"A10"),])
        );
    }

    #[test]
    fn decoder_sort_supports_paths() {
        let input = vec![
            bytes(b"folder10/file"),
            bytes(b"folder2/file"),
            bytes(b"folder1/file"),
        ];

        let options = SortOptions::new().path(true);

        assert_eq!(
            natsorted_values_with_decoder_and_options(&input, Decoder::utf8(), options,),
            Ok(vec![
                bytes(b"folder1/file"),
                bytes(b"folder2/file"),
                bytes(b"folder10/file"),
            ])
        );
    }

    #[test]
    fn decoder_sort_propagates_utf8_error() {
        let input = vec![bytes(b"a1"), bytes(&[0xFF])];

        assert!(matches!(
            natsorted_values_with_decoder(&input, Decoder::utf8(),),
            Err(DecodeError::InvalidUtf8 { .. })
        ));
    }

    #[test]
    fn decoder_sort_is_stable() {
        let input = vec![bytes(b"file01"), text("file1"), bytes(b"file001")];

        assert_eq!(
            natsorted_values_with_decoder(&input, Decoder::utf8(),),
            Ok(input),
        );
    }

    #[test]
    fn empty_decoder_sort_is_supported() {
        let input = Vec::new();

        assert_eq!(
            natsorted_values_with_decoder(&input, Decoder::utf8(),),
            Ok(Vec::new()),
        );
    }

    #[test]
    fn creates_latin1_decoder_and_common_aliases() {
        for alias in [
            "latin1",
            "latin-1",
            "latin_1",
            "ISO-8859-1",
            "iso_8859_1",
            "l1",
        ] {
            assert_eq!(decoder(alias), Ok(Decoder::latin1()), "alias={alias}");
        }

        assert_eq!(Decoder::latin1().encoding(), DecodeEncoding::Latin1);
    }

    #[test]
    fn decodes_latin1_bytes_without_loss() {
        assert_eq!(
            decode_latin1_bytes(&[b'c', b'a', b'f', 0xE9]),
            "café".to_string(),
        );
        assert_eq!(decode_latin1_bytes(&[0xFF]), "ÿ".to_string());
    }

    #[test]
    fn latin1_decoder_preserves_non_byte_values() {
        let values = [
            NaturalValue::from(123),
            NaturalValue::from(12.5),
            NaturalValue::None,
            text("café"),
        ];

        for value in values {
            assert_eq!(Decoder::latin1().decode(&value), Ok(value));
        }
    }

    #[test]
    fn latin1_decoder_sorts_mixed_bytes_and_text() {
        let input = vec![bytes(b"caf\xe910"), text("café2"), bytes(b"caf\xe91")];

        assert_eq!(
            natsorted_values_with_decoder(&input, Decoder::latin1()),
            Ok(vec![bytes(b"caf\xe91"), text("café2"), bytes(b"caf\xe910"),]),
        );
    }
}
