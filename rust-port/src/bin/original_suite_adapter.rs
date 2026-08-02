use std::env;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use num_bigint::BigInt;
use rust_port::{
    AlgorithmFlags, LocaleProfile, NaturalValue, OsSortOptions, OsSortProfile, SortOptions,
    index_natsorted_values_with_options, index_os_sorted_values_with_options,
    natsorted_with_options, numeric_regex_chooser_from_bits, python_decode_bytes,
    python_fast_float, python_fast_int, python_final_transform_mode, python_group_letters,
    python_input_transform, python_input_transform_is_noop, python_number_plan, python_parse_bytes,
    python_path_components, python_separator_plan, python_unicode_tables,
};
use rust_port::{
    PythonComponent, python_normalize_string, python_parse_string_plan,
    python_transform_components, python_value_kind,
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

fn read_single_string_request(label: &str) -> Result<String, String> {
    let mut request = Vec::new();

    io::stdin()
        .read_to_end(&mut request)
        .map_err(|error| format!("failed to read {label} request: {error}"))?;

    let mut values = decode_strings(&request)?;

    if values.len() != 1 {
        return Err(format!(
            "{label} requires exactly one input string, received {}",
            values.len(),
        ));
    }

    Ok(values.remove(0))
}

fn emit_numeric_regex(algorithm_bits: i64) -> Result<(), String> {
    io::stdout()
        .write_all(numeric_regex_chooser_from_bits(algorithm_bits).as_bytes())
        .map_err(|error| format!("failed to write numeric regex: {error}"))
}

fn emit_fast_float(nan_bits: u64) -> Result<(), String> {
    let input = read_single_string_request("fast-float")?;
    let mut response = Vec::with_capacity(9);

    match python_fast_float(&input, f64::from_bits(nan_bits)) {
        Some(value) => {
            response.push(1);
            response.extend_from_slice(&value.to_le_bytes());
        }
        None => response.push(0),
    }

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write fast-float response: {error}"))
}

fn emit_fast_int() -> Result<(), String> {
    let input = read_single_string_request("fast-int")?;
    let mut response = Vec::new();

    match python_fast_int(&input) {
        Some(value) => {
            let digits = value.to_string();
            let length = u64::try_from(digits.len())
                .map_err(|_| "fast-int response length does not fit in u64".to_string())?;

            response.push(2);
            response.extend_from_slice(&length.to_le_bytes());
            response.extend_from_slice(digits.as_bytes());
        }
        None => response.push(0),
    }

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write fast-int response: {error}"))
}

fn encode_codepoints(output: &mut Vec<u8>, values: &[u32]) -> Result<(), String> {
    let count = u64::try_from(values.len())
        .map_err(|_| "Unicode table length does not fit in u64".to_string())?;

    output.extend_from_slice(&count.to_le_bytes());

    for value in values {
        output.extend_from_slice(&value.to_le_bytes());
    }

    Ok(())
}

fn emit_unicode_tables() -> Result<(), String> {
    let tables = python_unicode_tables();
    let mut response = Vec::new();

    encode_codepoints(&mut response, &tables.numeric)?;
    encode_codepoints(&mut response, &tables.digits)?;
    encode_codepoints(&mut response, &tables.decimals)?;

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write Unicode-table response: {error}"))
}

fn read_raw_request(label: &str) -> Result<Vec<u8>, String> {
    let mut request = Vec::new();

    io::stdin()
        .read_to_end(&mut request)
        .map_err(|error| format!("failed to read {label} request: {error}"))?;

    Ok(request)
}

fn write_single_string(value: String, label: &str) -> Result<(), String> {
    let response = encode_strings(&[value])?;

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write {label} response: {error}"))
}

fn emit_input_transform_plan(bits: i64) -> Result<(), String> {
    io::stdout()
        .write_all(&[u8::from(python_input_transform_is_noop(bits))])
        .map_err(|error| format!("failed to write input-transform plan: {error}"))
}

fn emit_input_transform(bits: i64, locale_identifier: &str) -> Result<(), String> {
    let input = read_single_string_request("input-transform")?;
    let profile = LocaleProfile::from_identifier(locale_identifier);
    let transformed = python_input_transform(&input, bits, profile);

    write_single_string(transformed, "input-transform")
}

fn emit_final_transform_plan(bits: i64) -> Result<(), String> {
    io::stdout()
        .write_all(&[python_final_transform_mode(bits)])
        .map_err(|error| format!("failed to write final-transform plan: {error}"))
}

fn emit_group_letters() -> Result<(), String> {
    let input = read_single_string_request("group-letters")?;
    write_single_string(python_group_letters(&input), "group-letters")
}

fn emit_separator_plan() -> Result<(), String> {
    let request = read_raw_request("separator-plan")?;
    let response = python_separator_plan(&request);

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write separator plan: {error}"))
}

fn emit_path_components(windows_host: bool, treat_base: bool) -> Result<(), String> {
    let input = read_single_string_request("path-components")?;
    let components = python_path_components(&input, windows_host, treat_base);
    let response = encode_strings(&components)?;

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write path-components response: {error}"))
}

fn emit_parse_bytes(bits: i64) -> Result<(), String> {
    let request = read_raw_request("parse-bytes")?;
    let (transformed, path) = python_parse_bytes(&request, bits);
    let mut response = Vec::with_capacity(1 + transformed.len());

    response.push(u8::from(path));
    response.extend_from_slice(&transformed);

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write parse-bytes response: {error}"))
}

fn emit_parse_number_plan(bits: i64) -> Result<(), String> {
    let request = read_raw_request("parse-number-plan")?;
    let values = decode_values(&request)?;

    if values.len() != 1 {
        return Err(format!(
            "parse-number-plan requires one value, received {}",
            values.len(),
        ));
    }

    let (wrapper, kind, positive, suffix) = python_number_plan(&values[0], bits)?;

    io::stdout()
        .write_all(&[wrapper, kind, u8::from(positive), suffix])
        .map_err(|error| format!("failed to write parse-number plan: {error}"))
}

fn emit_normalize_string(bits: i64, compose: bool) -> Result<(), String> {
    let input = read_single_string_request("normalize-string")?;
    let normalized = python_normalize_string(&input, bits, compose);
    write_single_string(normalized, "normalize-string")
}

fn emit_parse_string_plan(bits: i64) -> Result<(), String> {
    let (original_after_transform, compose) = python_parse_string_plan(bits);

    io::stdout()
        .write_all(&[u8::from(original_after_transform), u8::from(compose)])
        .map_err(|error| format!("failed to write parse-string plan: {error}"))
}

fn emit_transform_components(bits: i64) -> Result<(), String> {
    let inputs = {
        let request = read_raw_request("transform-components")?;
        decode_strings(&request)?
    };

    let (use_locale, components) = python_transform_components(&inputs, bits);
    let count = u64::try_from(components.len())
        .map_err(|_| "component count does not fit in u64".to_string())?;

    let mut response = Vec::new();
    response.push(u8::from(use_locale));
    response.extend_from_slice(&count.to_le_bytes());

    for component in components {
        match component {
            PythonComponent::Text(value) => {
                let bytes = value.as_bytes();
                let length = u64::try_from(bytes.len())
                    .map_err(|_| "component text length does not fit in u64".to_string())?;
                response.push(0);
                response.extend_from_slice(&length.to_le_bytes());
                response.extend_from_slice(bytes);
            }
            PythonComponent::Integer(value) => {
                let digits = value.to_string();
                let length = u64::try_from(digits.len())
                    .map_err(|_| "component integer length does not fit in u64".to_string())?;
                response.push(1);
                response.extend_from_slice(&length.to_le_bytes());
                response.extend_from_slice(digits.as_bytes());
            }
            PythonComponent::Float(value) => {
                response.push(2);
                response.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    io::stdout()
        .write_all(&response)
        .map_err(|error| format!("failed to write component response: {error}"))
}

fn emit_classify_value() -> Result<(), String> {
    let request = read_raw_request("classify-value")?;
    let values = decode_values(&request)?;

    if values.len() != 1 {
        return Err(format!(
            "classify-value requires one value, received {}",
            values.len(),
        ));
    }

    io::stdout()
        .write_all(&[python_value_kind(&values[0])])
        .map_err(|error| format!("failed to write value classification: {error}"))
}

fn emit_decode_bytes(encoding: &str) -> Result<(), String> {
    let request = read_raw_request("decode-bytes")?;
    let decoded = python_decode_bytes(&request, encoding)?;

    write_single_string(decoded, "decode-bytes")
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

        Some("numeric-regex") => {
            let algorithm_bits = arguments
                .next()
                .ok_or_else(|| "missing numeric-regex algorithm bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid numeric-regex algorithm bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("numeric-regex received unexpected additional arguments".to_string());
            }

            emit_numeric_regex(algorithm_bits)
        }

        Some("fast-float") => {
            let nan_bits = arguments
                .next()
                .ok_or_else(|| "missing fast-float NaN replacement bits".to_string())?
                .parse::<u64>()
                .map_err(|error| format!("invalid fast-float NaN replacement bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("fast-float received unexpected additional arguments".to_string());
            }

            emit_fast_float(nan_bits)
        }

        Some("fast-int") => {
            if arguments.next().is_some() {
                return Err("fast-int received unexpected additional arguments".to_string());
            }

            emit_fast_int()
        }

        Some("unicode-tables") => {
            if arguments.next().is_some() {
                return Err("unicode-tables received unexpected additional arguments".to_string());
            }

            emit_unicode_tables()
        }

        Some("input-transform-plan") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing input-transform-plan bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid input-transform-plan bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("input-transform-plan received unexpected arguments".to_string());
            }

            emit_input_transform_plan(bits)
        }

        Some("input-transform") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing input-transform bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid input-transform bits: {error}"))?;

            let locale_identifier = arguments.next().unwrap_or_else(|| "en-US".to_string());

            if arguments.next().is_some() {
                return Err("input-transform received unexpected arguments".to_string());
            }

            emit_input_transform(bits, &locale_identifier)
        }

        Some("final-transform-plan") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing final-transform-plan bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid final-transform-plan bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("final-transform-plan received unexpected arguments".to_string());
            }

            emit_final_transform_plan(bits)
        }

        Some("group-letters") => {
            if arguments.next().is_some() {
                return Err("group-letters received unexpected arguments".to_string());
            }

            emit_group_letters()
        }

        Some("separator-plan") => {
            if arguments.next().is_some() {
                return Err("separator-plan received unexpected arguments".to_string());
            }

            emit_separator_plan()
        }

        Some("path-components") => {
            let windows_host = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing path-components host flag".to_string())?,
            )?;

            let treat_base = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing path-components treat-base flag".to_string())?,
            )?;

            if arguments.next().is_some() {
                return Err("path-components received unexpected arguments".to_string());
            }

            emit_path_components(windows_host, treat_base)
        }

        Some("parse-bytes") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing parse-bytes bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid parse-bytes bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("parse-bytes received unexpected arguments".to_string());
            }

            emit_parse_bytes(bits)
        }

        Some("parse-number-plan") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing parse-number-plan bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid parse-number-plan bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("parse-number-plan received unexpected arguments".to_string());
            }

            emit_parse_number_plan(bits)
        }

        Some("decode-bytes") => {
            let encoding = arguments
                .next()
                .ok_or_else(|| "missing decode-bytes encoding".to_string())?;

            if arguments.next().is_some() {
                return Err("decode-bytes received unexpected arguments".to_string());
            }

            emit_decode_bytes(&encoding)
        }

        Some("validate-range") => {
            if arguments.next().is_some() {
                return Err("validate-range received unexpected arguments".to_string());
            }

            emit_validate_range()
        }

        Some("normalize-string") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing normalize-string bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid normalize-string bits: {error}"))?;

            let compose = parse_reverse(
                &arguments
                    .next()
                    .ok_or_else(|| "missing normalize-string compose flag".to_string())?,
            )?;

            if arguments.next().is_some() {
                return Err("normalize-string received unexpected arguments".to_string());
            }

            emit_normalize_string(bits, compose)
        }

        Some("parse-string-plan") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing parse-string-plan bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid parse-string-plan bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("parse-string-plan received unexpected arguments".to_string());
            }

            emit_parse_string_plan(bits)
        }

        Some("transform-components") => {
            let bits = arguments
                .next()
                .ok_or_else(|| "missing transform-components bits".to_string())?
                .parse::<i64>()
                .map_err(|error| format!("invalid transform-components bits: {error}"))?;

            if arguments.next().is_some() {
                return Err("transform-components received unexpected arguments".to_string());
            }

            emit_transform_components(bits)
        }

        Some("classify-value") => {
            if arguments.next().is_some() {
                return Err("classify-value received unexpected arguments".to_string());
            }

            emit_classify_value()
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
             numeric-regex ALGORITHM_BITS | \
             fast-float NAN_REPLACEMENT_BITS | fast-int | unicode-tables | \
             sort-strings ALGORITHM_BITS REVERSE | \
             sort-values ALGORITHM_BITS REVERSE [LOCALE] | \
             os-sort-values REVERSE PRESORT PROFILE [LOCALE]"
            .to_string()),
    }
}

fn compare_bigint_to_f64(integer: &BigInt, float: f64) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;

    if float.is_nan() {
        return None;
    }

    if float == f64::INFINITY {
        return Some(Ordering::Less);
    }

    if float == f64::NEG_INFINITY {
        return Some(Ordering::Greater);
    }

    let bits = float.to_bits();
    let negative = bits >> 63 != 0;
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);

    let (significand, exponent) = if exponent_bits == 0 {
        (BigInt::from(fraction), -1074)
    } else {
        (
            BigInt::from((1_u64 << 52) | fraction),
            exponent_bits - 1023 - 52,
        )
    };

    let signed_significand = if negative { -significand } else { significand };

    if exponent >= 0 {
        let exact_float = signed_significand << exponent as usize;
        Some(integer.cmp(&exact_float))
    } else {
        let scaled_integer = integer << (-exponent) as usize;
        Some(scaled_integer.cmp(&signed_significand))
    }
}

fn compare_python_numbers(
    left: &NaturalValue,
    right: &NaturalValue,
) -> Result<std::cmp::Ordering, String> {
    use std::cmp::Ordering;

    match (left, right) {
        (NaturalValue::Integer(left), NaturalValue::Integer(right)) => Ok(left.cmp(right)),
        (NaturalValue::Float(left), NaturalValue::Float(right)) => left
            .partial_cmp(right)
            .ok_or_else(|| "validate-range does not accept NaN".to_string()),
        (NaturalValue::Integer(left), NaturalValue::Float(right)) => {
            compare_bigint_to_f64(left, *right)
                .ok_or_else(|| "validate-range does not accept NaN".to_string())
        }
        (NaturalValue::Float(left), NaturalValue::Integer(right)) => {
            compare_bigint_to_f64(right, *left)
                .map(Ordering::reverse)
                .ok_or_else(|| "validate-range does not accept NaN".to_string())
        }
        _ => Err("validate-range requires exactly two integer or float values".to_string()),
    }
}

fn emit_validate_range() -> Result<(), String> {
    let request = read_raw_request("validate-range")?;
    let values = decode_values(&request)?;

    if values.len() != 2 {
        return Err(format!(
            "validate-range requires two values, received {}",
            values.len(),
        ));
    }

    let valid = compare_python_numbers(&values[0], &values[1])? == std::cmp::Ordering::Less;

    io::stdout()
        .write_all(&[u8::from(valid)])
        .map_err(|error| format!("failed to write validate-range response: {error}"))
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
