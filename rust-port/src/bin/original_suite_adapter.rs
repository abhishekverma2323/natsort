use std::env;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use rust_port::{AlgorithmFlags, SortOptions, natsorted_with_options};

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

        _ => Err("usage: original-suite-adapter flags | \
             sort-strings ALGORITHM_BITS REVERSE"
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
