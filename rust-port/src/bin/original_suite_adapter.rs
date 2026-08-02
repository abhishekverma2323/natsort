use std::env;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use num_bigint::BigInt;
use rust_port::{
    AlgorithmFlags, LocaleProfile, NaturalValue, OsSortOptions, OsSortProfile, SortOptions,
    index_natsorted_values_with_options, index_os_sorted_values_with_options,
    natsorted_with_options,
};

fn emit_flags() {
    let flags = [
        ("FLOAT", AlgorithmFlags::FLOAT.bits()),
        ("SIGNED", AlgorithmFlags::SIGNED.bits()),
        ("NOEXP", AlgorithmFlags::NOEXP.bits()),
        ("PATH", AlgorithmFlags::PATH.bits()),
        ("LOCALEALPHA", AlgorithmFlags::LOCALEALPHA.bits()),
        ("LOCALENUM", AlgorithmFlags::LOCALENUM.bits()),
        ("IGNORECASE", AlgorithmFlags::IGNORECASE.bits()),
        ("LOWERCASEFIRST", AlgorithmFlags::LOWERCASEFIRST.bits()),
        ("GROUPLETTERS", AlgorithmFlags::GROUPLETTERS.bits()),
        ("UNGROUPLETTERS", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("NANLAST", AlgorithmFlags::NANLAST.bits()),
        (
            "COMPATIBILITYNORMALIZE",
            AlgorithmFlags::COMPATIBILITYNORMALIZE.bits(),
        ),
        ("NUMAFTER", AlgorithmFlags::NUMAFTER.bits()),
        ("PRESORT", AlgorithmFlags::PRESORT.bits()),
        ("DEFAULT", AlgorithmFlags::DEFAULT.bits()),
        ("INT", AlgorithmFlags::INT.bits()),
        ("UNSIGNED", AlgorithmFlags::UNSIGNED.bits()),
        ("REAL", AlgorithmFlags::REAL.bits()),
        ("LOCALE", AlgorithmFlags::LOCALE.bits()),
        ("I", AlgorithmFlags::INT.bits()),
        ("U", AlgorithmFlags::UNSIGNED.bits()),
        ("F", AlgorithmFlags::FLOAT.bits()),
        ("S", AlgorithmFlags::SIGNED.bits()),
        ("R", AlgorithmFlags::REAL.bits()),
        ("N", AlgorithmFlags::NOEXP.bits()),
        ("P", AlgorithmFlags::PATH.bits()),
        ("LA", AlgorithmFlags::LOCALEALPHA.bits()),
        ("LN", AlgorithmFlags::LOCALENUM.bits()),
        ("L", AlgorithmFlags::LOCALE.bits()),
        ("IC", AlgorithmFlags::IGNORECASE.bits()),
        ("LF", AlgorithmFlags::LOWERCASEFIRST.bits()),
        ("G", AlgorithmFlags::GROUPLETTERS.bits()),
        ("UG", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("C", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("CAPITALFIRST", AlgorithmFlags::UNGROUPLETTERS.bits()),
        ("NL", AlgorithmFlags::NANLAST.bits()),
        ("CN", AlgorithmFlags::COMPATIBILITYNORMALIZE.bits()),
        ("NA", AlgorithmFlags::NUMAFTER.bits()),
        ("PS", AlgorithmFlags::PRESORT.bits()),
    ];

    for (name, value) in flags {
        println!("{name}\t{value}");
    }
}

fn read_u64(input: &[u8], offset: &mut usize) -> Result<u64, String> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| "binary request offset overflowed".to_string())?;

    let bytes = input
        .get(*offset..end)
        .ok_or_else(|| "binary request ended unexpectedly".to_string())?;

    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| "invalid 64-bit request field".to_string())?;

    *offset = end;

    Ok(u64::from_le_bytes(array))
}

fn decode_strings(input: &[u8]) -> Result<Vec<String>, String> {
    let mut offset = 0;
    let count = usize::try_from(read_u64(input, &mut offset)?)
        .map_err(|_| "string count does not fit in usize".to_string())?;

    let mut items = Vec::with_capacity(count);

    for _ in 0..count {
        let length = usize::try_from(read_u64(input, &mut offset)?)
            .map_err(|_| "string length does not fit in usize".to_string())?;

        let end = offset
            .checked_add(length)
            .ok_or_else(|| "string boundary overflowed".to_string())?;

        let bytes = input
            .get(offset..end)
            .ok_or_else(|| "string data ended unexpectedly".to_string())?;

        let value = String::from_utf8(bytes.to_vec())
            .map_err(|error| format!("request contained invalid UTF-8: {error}"))?;

        items.push(value);
        offset = end;
    }

    if offset != input.len() {
        return Err("binary request contained trailing bytes".to_string());
    }

    Ok(items)
}

fn encode_strings(items: &[String]) -> Result<Vec<u8>, String> {
    let count = u64::try_from(items.len())
        .map_err(|_| "response item count does not fit in u64".to_string())?;

    let mut output = Vec::new();
    output.extend_from_slice(&count.to_le_bytes());

    for item in items {
        let bytes = item.as_bytes();
        let length = u64::try_from(bytes.len())
            .map_err(|_| "response string length does not fit in u64".to_string())?;

        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(bytes);
    }

    Ok(output)
}

fn sort_strings(algorithm_bits: i64, reverse: bool) -> Result<(), String> {
    let mut request = Vec::new();

    io::stdin()
        .read_to_end(&mut request)
        .map_err(|error| format!("failed to read request: {error}"))?;

    let items = decode_strings(&request)?;

    let options =
        SortOptions::from_algorithm(AlgorithmFlags::from_bits(algorithm_bits)).reverse(reverse);

    let sorted = natsorted_with_options(&items, options);
    let response = encode_strings(&sorted)?;

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write response: {error}"))?;

    Ok(())
}

fn read_bytes<'a>(input: &'a [u8], offset: &mut usize, length: usize) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| "typed request boundary overflowed".to_string())?;

    let bytes = input
        .get(*offset..end)
        .ok_or_else(|| "typed request ended unexpectedly".to_string())?;

    *offset = end;
    Ok(bytes)
}

fn decode_natural_value(input: &[u8], offset: &mut usize) -> Result<NaturalValue, String> {
    let tag = *input
        .get(*offset)
        .ok_or_else(|| "typed value tag was missing".to_string())?;

    *offset = offset
        .checked_add(1)
        .ok_or_else(|| "typed value offset overflowed".to_string())?;

    match tag {
        0 => {
            let length = usize::try_from(read_u64(input, offset)?)
                .map_err(|_| "text length does not fit in usize".to_string())?;

            let bytes = read_bytes(input, offset, length)?;
            let value = String::from_utf8(bytes.to_vec())
                .map_err(|error| format!("text value is not UTF-8: {error}"))?;

            Ok(NaturalValue::Text(value))
        }

        1 => {
            let length = usize::try_from(read_u64(input, offset)?)
                .map_err(|_| "byte length does not fit in usize".to_string())?;

            Ok(NaturalValue::Bytes(
                read_bytes(input, offset, length)?.to_vec(),
            ))
        }

        2 => {
            let length = usize::try_from(read_u64(input, offset)?)
                .map_err(|_| "integer length does not fit in usize".to_string())?;

            let bytes = read_bytes(input, offset, length)?;
            let value = std::str::from_utf8(bytes)
                .map_err(|error| format!("integer text is not UTF-8: {error}"))?
                .parse::<BigInt>()
                .map_err(|error| format!("invalid integer value: {error}"))?;

            Ok(NaturalValue::Integer(value))
        }

        3 => {
            let bytes = read_bytes(input, offset, 8)?;
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| "invalid floating-point field".to_string())?;

            Ok(NaturalValue::Float(f64::from_le_bytes(array)))
        }

        4 => Ok(NaturalValue::None),

        5 => {
            let count = usize::try_from(read_u64(input, offset)?)
                .map_err(|_| "sequence count does not fit in usize".to_string())?;

            let mut values = Vec::with_capacity(count);

            for _ in 0..count {
                values.push(decode_natural_value(input, offset)?);
            }

            Ok(NaturalValue::Sequence(values))
        }

        _ => Err(format!("unknown typed value tag: {tag}")),
    }
}

fn decode_values(input: &[u8]) -> Result<Vec<NaturalValue>, String> {
    let mut offset = 0;

    let count = usize::try_from(read_u64(input, &mut offset)?)
        .map_err(|_| "value count does not fit in usize".to_string())?;

    let mut values = Vec::with_capacity(count);

    for _ in 0..count {
        values.push(decode_natural_value(input, &mut offset)?);
    }

    if offset != input.len() {
        return Err("typed request contained trailing bytes".to_string());
    }

    Ok(values)
}

fn encode_indexes(indexes: &[usize]) -> Result<Vec<u8>, String> {
    let count =
        u64::try_from(indexes.len()).map_err(|_| "index count does not fit in u64".to_string())?;

    let mut output = Vec::with_capacity(8 + indexes.len() * 8);
    output.extend_from_slice(&count.to_le_bytes());

    for index in indexes {
        let value =
            u64::try_from(*index).map_err(|_| "sorted index does not fit in u64".to_string())?;

        output.extend_from_slice(&value.to_le_bytes());
    }

    Ok(output)
}

fn sort_values(algorithm_bits: i64, reverse: bool, locale_identifier: &str) -> Result<(), String> {
    let mut request = Vec::new();

    io::stdin()
        .read_to_end(&mut request)
        .map_err(|error| format!("failed to read typed request: {error}"))?;

    let values = decode_values(&request)?;
    // In Python natsort, bit 512 (CAPITALFIRST/UNGROUPLETTERS)
    // only affects ordering when LOCALEALPHA is enabled.
    let mut effective_bits = algorithm_bits;

    if effective_bits & AlgorithmFlags::LOCALEALPHA.bits() == 0 {
        effective_bits &= !AlgorithmFlags::UNGROUPLETTERS.bits();
    }

    let options = SortOptions::from_algorithm(AlgorithmFlags::from_bits(effective_bits))
        .reverse(reverse)
        .locale_profile(LocaleProfile::from_identifier(locale_identifier));

    let indexes = index_natsorted_values_with_options(&values, options);
    let response = encode_indexes(&indexes)?;

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write index response: {error}"))?;

    Ok(())
}

fn sort_os_values(
    reverse: bool,
    presort: bool,
    profile: OsSortProfile,
    locale_identifier: &str,
) -> Result<(), String> {
    let mut request = Vec::new();

    io::stdin()
        .read_to_end(&mut request)
        .map_err(|error| format!("failed to read OS-sort request: {error}"))?;

    let values = decode_values(&request)?;
    let options = OsSortOptions::new()
        .reverse(reverse)
        .presort(presort)
        .profile(profile)
        .locale_profile(LocaleProfile::from_identifier(locale_identifier));

    let indexes = index_os_sorted_values_with_options(&values, options);
    let response = encode_indexes(&indexes)?;

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write OS-sort response: {error}"))?;

    Ok(())
}

fn parse_reverse(value: &str) -> Result<bool, String> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("reverse must be 0 or 1, received {value:?}")),
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);

    match arguments.next().as_deref() {
        Some("flags") => {
            if arguments.next().is_some() {
                return Err("flags does not accept additional arguments".to_string());
            }

            emit_flags();
            Ok(())
        }

        Some("sort-strings") => {
            let algorithm_bits = arguments
                .next()
                .ok_or_else(|| "missing algorithm bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid algorithm bits: {error}"))?;

            let reverse = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing reverse value".to_string())?,
            )?;

            if arguments.next().is_some() {
                return Err("sort-strings received unexpected additional arguments".to_string());
            }

            sort_strings(algorithm_bits, reverse)
        }

        Some("sort-values") => {
            let algorithm_bits = arguments
                .next()
                .ok_or_else(|| "missing algorithm bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid algorithm bits: {error}"))?;

            let reverse = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing reverse value".to_string())?,
            )?;

            let locale_identifier = arguments.next().unwrap_or_else(|| "en-US".to_string());

            if arguments.next().is_some() {
                return Err("sort-values received unexpected additional arguments".to_string());
            }

            sort_values(algorithm_bits, reverse, &locale_identifier)
        }

        Some("os-sort-values") => {
            let reverse = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing OS-sort reverse value".to_string())?,
            )?;

            let presort = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing OS-sort presort value".to_string())?,
            )?;

            let profile = match arguments
                .next()
                .ok_or_else(|| "missing OS-sort profile".to_string())?
                .as_str()
            {
                "windows" => OsSortProfile::Windows,
                "unix" => OsSortProfile::Unix,
                value => {
                    return Err(format!("invalid OS-sort profile: {value}"));
                }
            };

            let locale_identifier = arguments.next().unwrap_or_else(|| "en-US".to_string());

            if arguments.next().is_some() {
                return Err("os-sort-values received unexpected additional arguments".to_string());
            }

            sort_os_values(reverse, presort, profile, &locale_identifier)
        }

        _ => Err("usage: original-suite-adapter flags | \
             sort-strings ALGORITHM_BITS REVERSE | \
             sort-values ALGORITHM_BITS REVERSE [LOCALE] | \
             os-sort-values REVERSE PRESORT PROFILE [LOCALE]"
            .to_string()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("original-suite-adapter: {error}");
            ExitCode::from(2)
        }
    }
}
