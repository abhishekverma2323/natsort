use std::env;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use rust_port::{SortOptions, natsorted_with_options};

#[derive(Debug)]
struct Arguments {
    dataset: PathBuf,
    mode: String,
    warmups: usize,
    runs: usize,
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} --dataset PATH --mode MODE [--warmups N] [--runs N]\n\
         modes: default, float, real, path, locale"
    )
}

fn parse_usize(option: &str, value: Option<String>) -> Result<usize, String> {
    let raw = value.ok_or_else(|| format!("{option} requires a value"))?;
    raw.parse::<usize>()
        .map_err(|_| format!("{option} expects a non-negative integer"))
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut values = env::args();
    let program = values.next().unwrap_or_else(|| "benchmark".to_string());

    let mut dataset = None;
    let mut mode = None;
    let mut warmups = 2_usize;
    let mut runs = 7_usize;

    while let Some(argument) = values.next() {
        match argument.as_str() {
            "-h" | "--help" => return Err(usage(&program)),
            "--dataset" => {
                dataset = Some(PathBuf::from(
                    values
                        .next()
                        .ok_or_else(|| "--dataset requires a path".to_string())?,
                ));
            }
            "--mode" => {
                mode = Some(
                    values
                        .next()
                        .ok_or_else(|| "--mode requires a value".to_string())?,
                );
            }
            "--warmups" => {
                warmups = parse_usize("--warmups", values.next())?;
            }
            "--runs" => {
                runs = parse_usize("--runs", values.next())?;
            }
            unknown => return Err(format!("unknown argument: {unknown}\n{}", usage(&program))),
        }
    }

    let dataset = dataset.ok_or_else(|| format!("missing --dataset\n{}", usage(&program)))?;
    let mode = mode.ok_or_else(|| format!("missing --mode\n{}", usage(&program)))?;

    if runs == 0 {
        return Err("--runs must be greater than zero".to_string());
    }

    Ok(Arguments {
        dataset,
        mode,
        warmups,
        runs,
    })
}

fn options_for_mode(mode: &str) -> Result<SortOptions, String> {
    match mode {
        "default" => Ok(SortOptions::new()),
        "float" => Ok(SortOptions::new().float(true)),
        "real" => Ok(SortOptions::new().float(true).signed(true)),
        "path" => Ok(SortOptions::new().path(true)),
        "locale" => Ok(SortOptions::new().locale(true)),
        _ => Err(format!("unsupported mode: {mode}")),
    }
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    let midpoint = sorted.len() / 2;

    if sorted.len().is_multiple_of(2) {
        (sorted[midpoint - 1] + sorted[midpoint]) / 2.0
    } else {
        sorted[midpoint]
    }
}

fn json_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len() + 8);

    for character in input.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            control if control.is_control() => {
                use std::fmt::Write;
                let _ = write!(escaped, "\\u{:04x}", control as u32);
            }
            other => escaped.push(other),
        }
    }

    escaped
}

fn timings_json(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.6}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn run(arguments: Arguments) -> Result<(), String> {
    let contents = fs::read_to_string(&arguments.dataset)
        .map_err(|error| format!("failed to read {}: {error}", arguments.dataset.display()))?;
    let entries: Vec<String> = contents.lines().map(str::to_owned).collect();
    let options = options_for_mode(&arguments.mode)?;

    let mut checksum = 0_usize;

    for _ in 0..arguments.warmups {
        let result = natsorted_with_options(&entries, options);
        checksum ^= black_box(result.len());
    }

    let mut timings_ms = Vec::with_capacity(arguments.runs);

    for _ in 0..arguments.runs {
        let started = Instant::now();
        let result = natsorted_with_options(&entries, options);
        let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;

        if let Some(first) = result.first() {
            checksum ^= black_box(first.len());
        }
        if let Some(last) = result.last() {
            checksum ^= black_box(last.len() << 1);
        }
        checksum ^= black_box(result.len());
        timings_ms.push(elapsed_ms);
    }

    let median_ms = median(&timings_ms);
    let min_ms = timings_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max_ms = timings_ms.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    println!(
        concat!(
            "{{",
            "\"engine\":\"rust\",",
            "\"mode\":\"{}\",",
            "\"dataset\":\"{}\",",
            "\"entries\":{},",
            "\"warmups\":{},",
            "\"runs\":{},",
            "\"times_ms\":[{}],",
            "\"median_ms\":{:.6},",
            "\"min_ms\":{:.6},",
            "\"max_ms\":{:.6},",
            "\"checksum\":{}",
            "}}"
        ),
        json_escape(&arguments.mode),
        json_escape(&arguments.dataset.display().to_string()),
        entries.len(),
        arguments.warmups,
        arguments.runs,
        timings_json(&timings_ms),
        median_ms,
        min_ms,
        max_ms,
        checksum,
    );

    Ok(())
}

fn main() -> ExitCode {
    match parse_arguments().and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
