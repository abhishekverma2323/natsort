use std::fmt;
use std::io::{self, Read, Write};

use crate::options::SortOptions;
use crate::sort::natsorted_with_options;
use crate::token::{Token, tokenize};

/// Numeric mode accepted by the command-line interface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CliNumberType {
    #[default]
    Integer,
    Float,
    Real,
}

impl CliNumberType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "int" | "i" => Some(Self::Integer),
            "float" | "f" => Some(Self::Float),
            "real" | "r" => Some(Self::Real),
            _ => None,
        }
    }

    const fn uses_float(self) -> bool {
        matches!(self, Self::Float | Self::Real)
    }

    const fn forces_sign(self) -> bool {
        matches!(self, Self::Real)
    }
}

/// Inclusive numeric range used by `--filter` and `--reverse-filter`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericRange {
    pub low: f64,
    pub high: f64,
}

impl NumericRange {
    pub fn new(low: f64, high: f64) -> Result<Self, CliRangeError> {
        if low >= high {
            Err(CliRangeError { low, high })
        } else {
            Ok(Self { low, high })
        }
    }

    fn contains(self, value: f64) -> bool {
        self.low <= value && value <= self.high
    }
}

/// Error returned when a CLI range has `low >= high`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CliRangeError {
    pub low: f64,
    pub high: f64,
}

impl fmt::Display for CliRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "low >= high")
    }
}

impl std::error::Error for CliRangeError {}

/// Parsed command-line options.
#[derive(Debug, Clone, PartialEq)]
pub struct CliOptions {
    pub paths: bool,
    pub filters: Vec<NumericRange>,
    pub reverse_filters: Vec<NumericRange>,
    pub exclude: Vec<f64>,
    pub reverse: bool,
    pub number_type: CliNumberType,
    pub signed: bool,
    pub exponent: bool,
    pub locale: bool,
    pub zero_terminated: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl CliOptions {
    pub const fn new() -> Self {
        Self {
            paths: false,
            filters: Vec::new(),
            reverse_filters: Vec::new(),
            exclude: Vec::new(),
            reverse: false,
            number_type: CliNumberType::Integer,
            signed: false,
            exponent: true,
            locale: false,
            zero_terminated: false,
        }
    }

    pub const fn sort_options(&self) -> SortOptions {
        SortOptions::new()
            .path(self.paths)
            .reverse(self.reverse)
            .float(self.number_type.uses_float())
            .signed(self.signed || self.number_type.forces_sign())
            .no_exp(!self.exponent)
            .locale(self.locale)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedCli {
    options: CliOptions,
    entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseOutcome {
    Run(ParsedCli),
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliParseError {
    message: String,
}

impl CliParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

fn parse_float(option: &str, value: &str) -> Result<f64, CliParseError> {
    value.parse::<f64>().map_err(|_| {
        CliParseError::new(format!("argument {option}: invalid float value: '{value}'"))
    })
}

fn number_type_error(value: &str) -> CliParseError {
    CliParseError::new(format!(
        "argument -t/--number-type/--number_type: invalid choice: '{value}' \
         (choose from int, float, real, f, i, r)"
    ))
}

fn required_argument(
    arguments: &[String],
    index: usize,
    option: &str,
) -> Result<String, CliParseError> {
    arguments
        .get(index)
        .cloned()
        .ok_or_else(|| CliParseError::new(format!("argument {option}: expected one argument")))
}

fn required_range_values(
    arguments: &[String],
    index: usize,
    option: &str,
) -> Result<(String, String), CliParseError> {
    match (arguments.get(index), arguments.get(index + 1)) {
        (Some(low), Some(high)) => Ok((low.clone(), high.clone())),
        _ => Err(CliParseError::new(format!(
            "argument {option}: expected 2 arguments"
        ))),
    }
}

fn parse_range(
    arguments: &[String],
    index: usize,
    option: &str,
) -> Result<NumericRange, CliParseError> {
    let (low, high) = required_range_values(arguments, index, option)?;

    Ok(NumericRange {
        low: parse_float(option, &low)?,
        high: parse_float(option, &high)?,
    })
}

fn is_negative_numeric_entry(value: &str) -> bool {
    value.starts_with('-') && value.parse::<f64>().is_ok()
}

fn parse_arguments(arguments: Vec<String>) -> Result<ParseOutcome, CliParseError> {
    let mut options = CliOptions::new();
    let mut entries = Vec::new();
    let mut positional_only = false;
    let mut index = 0;

    while index < arguments.len() {
        let argument = &arguments[index];

        if positional_only {
            entries.push(argument.clone());
            index += 1;
            continue;
        }

        match argument.as_str() {
            "--" => {
                positional_only = true;
                index += 1;
            }
            "-h" | "--help" => return Ok(ParseOutcome::Help),
            "--version" => return Ok(ParseOutcome::Version),
            "-p" | "--paths" => {
                options.paths = true;
                index += 1;
            }
            "-r" | "--reverse" => {
                options.reverse = true;
                index += 1;
            }
            "-s" | "--sign" => {
                options.signed = true;
                index += 1;
            }
            "--nosign" => {
                options.signed = false;
                index += 1;
            }
            "--noexp" => {
                options.exponent = false;
                index += 1;
            }
            "-l" | "--locale" => {
                options.locale = true;
                index += 1;
            }
            "-z" | "--zero-terminated" => {
                options.zero_terminated = true;
                index += 1;
            }
            "-t" | "--number-type" | "--number_type" => {
                let value = required_argument(&arguments, index + 1, argument)?;
                options.number_type =
                    CliNumberType::parse(&value).ok_or_else(|| number_type_error(&value))?;
                index += 2;
            }
            "-f" | "--filter" => {
                options
                    .filters
                    .push(parse_range(&arguments, index + 1, "-f/--filter")?);
                index += 3;
            }
            "-F" | "--reverse-filter" => {
                options.reverse_filters.push(parse_range(
                    &arguments,
                    index + 1,
                    "-F/--reverse-filter",
                )?);
                index += 3;
            }
            "-e" | "--exclude" => {
                let value = required_argument(&arguments, index + 1, argument)?;
                options.exclude.push(parse_float("-e/--exclude", &value)?);
                index += 2;
            }
            _ => {
                if let Some(value) = argument
                    .strip_prefix("--number-type=")
                    .or_else(|| argument.strip_prefix("--number_type="))
                {
                    options.number_type =
                        CliNumberType::parse(value).ok_or_else(|| number_type_error(value))?;
                    index += 1;
                } else if argument.starts_with('-') && !is_negative_numeric_entry(argument) {
                    return Err(CliParseError::new(format!(
                        "unrecognized arguments: {argument}"
                    )));
                } else {
                    entries.push(argument.clone());
                    index += 1;
                }
            }
        }
    }

    Ok(ParseOutcome::Run(ParsedCli { options, entries }))
}

fn check_ranges(ranges: &[NumericRange], option: &str) -> Result<(), String> {
    if ranges.iter().any(|range| range.low >= range.high) {
        Err(format!("Error in {option}: low >= high"))
    } else {
        Ok(())
    }
}

fn numeric_values(entry: &str, options: &CliOptions) -> Vec<f64> {
    let sort_options = options.sort_options();

    tokenize(
        entry,
        sort_options.signed,
        sort_options.float,
        sort_options.no_exp,
    )
    .into_iter()
    .filter_map(|token| match token {
        Token::Number(value) => value.parse::<f64>().ok(),
        Token::Text(_) => None,
    })
    .collect()
}

/// Return `true` when at least one number in `entry` falls inside any range.
pub fn keep_entry_range(entry: &str, ranges: &[NumericRange], options: &CliOptions) -> bool {
    numeric_values(entry, options)
        .into_iter()
        .any(|value| ranges.iter().any(|range| range.contains(value)))
}

/// Return `true` when `entry` contains none of the excluded numeric values.
pub fn keep_entry_value(entry: &str, values: &[f64], options: &CliOptions) -> bool {
    numeric_values(entry, options)
        .into_iter()
        .all(|number| !values.contains(&number))
}

/// Filter and naturally sort CLI entries using Python-compatible semantics.
pub fn sort_cli_entries(entries: &[String], options: &CliOptions) -> Vec<String> {
    let filtered: Vec<String> = entries
        .iter()
        .filter(|entry| {
            (options.filters.is_empty() || keep_entry_range(entry, &options.filters, options))
                && (options.reverse_filters.is_empty()
                    || !keep_entry_range(entry, &options.reverse_filters, options))
                && keep_entry_value(entry, &options.exclude, options)
        })
        .cloned()
        .collect();

    natsorted_with_options(&filtered, options.sort_options())
}

fn stdin_entries(input: &str, zero_terminated: bool) -> Vec<String> {
    if input.is_empty() {
        return vec![String::new()];
    }

    if zero_terminated {
        input.split_terminator('\0').map(str::to_string).collect()
    } else {
        input.lines().map(str::to_string).collect()
    }
}

/// Line ending used by the native CLI platform.
pub const fn native_line_ending() -> &'static str {
    if cfg!(windows) { "\r\n" } else { "\n" }
}

fn platform_newlines(input: &str) -> String {
    if cfg!(windows) {
        input.replace('\n', "\r\n")
    } else {
        input.to_string()
    }
}

fn write_entries<W: Write>(writer: &mut W, entries: &[String]) -> io::Result<()> {
    for entry in entries {
        writer.write_all(entry.as_bytes())?;
        writer.write_all(native_line_ending().as_bytes())?;
    }

    Ok(())
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} [-h] [--version] [-p] [-f LOW HIGH] [-F LOW HIGH]\n\
         \x20              [-e EXCLUDE] [-r] [-t {{int,float,real,f,i,r}}]\n\
         \x20              [--nosign] [-s] [--noexp] [-l] [-z]\n\
         \x20              [entries ...]\n"
    )
}

fn help(program: &str) -> String {
    format!(
        "{}\nPerform a natural sort on entries given on the command-line.\n\n\
         Arguments are read from sys.argv.\n\n\
         positional arguments:\n\
         \x20 entries               The entries to sort. Taken from stdin if nothing is\n\
         \x20                       given on the command line.\n\n\
         options:\n\
         \x20 -h, --help            show this help message and exit\n\
         \x20 --version             show program's version number and exit\n\
         \x20 -p, --paths           Interpret the input as file paths.\n\
         \x20 -f, --filter LOW HIGH\n\
         \x20                       Keep entries containing a number in this range.\n\
         \x20 -F, --reverse-filter LOW HIGH\n\
         \x20                       Exclude entries containing a number in this range.\n\
         \x20 -e, --exclude EXCLUDE Exclude entries containing this number.\n\
         \x20 -r, --reverse         Return entries in reversed order.\n\
         \x20 -t, --number-type, --number_type {{int,float,real,f,i,r}}\n\
         \x20                       Choose integer, float, or real-number parsing.\n\
         \x20 --nosign              Do not consider signs as part of a number.\n\
         \x20 -s, --sign            Consider signs as part of a number.\n\
         \x20 --noexp               Do not consider exponents as part of a float.\n\
         \x20 -l, --locale          Use locale-aware sorting.\n\
         \x20 -z, --zero-terminated Split stdin entries on nulls instead of newlines.\n",
        usage(program).trim_end()
    )
}

fn write_error<E: Write>(writer: &mut E, message: &str) -> io::Result<()> {
    writer.write_all(message.as_bytes())?;
    writer.write_all(native_line_ending().as_bytes())
}

/// Execute the CLI with injectable input/output streams.
///
/// `arguments` excludes the executable name. The return value is the process
/// exit code: 0 for success, 1 for runtime/range errors, and 2 for argument
/// parsing errors.
pub fn run_cli_with_program<I, S, R, W, E>(
    program: &str,
    arguments: I,
    input: &mut R,
    output: &mut W,
    error: &mut E,
) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    R: Read,
    W: Write,
    E: Write,
{
    let arguments: Vec<String> = arguments.into_iter().map(Into::into).collect();

    let parsed = match parse_arguments(arguments) {
        Ok(result) => result,
        Err(parse_error) => {
            let _ = error.write_all(platform_newlines(&usage(program)).as_bytes());
            let _ = write_error(error, &format!("{program}: error: {}", parse_error.message));
            return 2;
        }
    };

    let ParsedCli {
        options,
        mut entries,
    } = match parsed {
        ParseOutcome::Help => {
            let _ = output.write_all(platform_newlines(&help(program)).as_bytes());
            return 0;
        }
        ParseOutcome::Version => {
            let _ = write_error(output, &format!("{program} {}", env!("CARGO_PKG_VERSION")));
            return 0;
        }
        ParseOutcome::Run(parsed) => parsed,
    };

    if let Err(message) = check_ranges(&options.filters, "--filter") {
        let _ = write_error(error, &message);
        return 1;
    }

    if let Err(message) = check_ranges(&options.reverse_filters, "--reverse-filter") {
        let _ = write_error(error, &message);
        return 1;
    }

    if entries.is_empty() {
        let mut buffer = String::new();

        if let Err(read_error) = input.read_to_string(&mut buffer) {
            let _ = write_error(error, &format!("error reading stdin: {read_error}"));
            return 1;
        }

        entries = stdin_entries(&buffer, options.zero_terminated);
    }

    let sorted = sort_cli_entries(&entries, &options);

    if let Err(write_error_value) = write_entries(output, &sorted) {
        let _ = write_error(error, &format!("error writing stdout: {write_error_value}"));
        return 1;
    }

    0
}

/// Execute the CLI using the default `natsort` program name.
pub fn run_cli<I, S, R, W, E>(arguments: I, input: &mut R, output: &mut W, error: &mut E) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    R: Read,
    W: Write,
    E: Write,
{
    run_cli_with_program("natsort", arguments, input, output, error)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn output_lines(values: &[&str]) -> Vec<u8> {
        let mut result = Vec::new();

        for value in values {
            result.extend_from_slice(value.as_bytes());
            result.extend_from_slice(native_line_ending().as_bytes());
        }

        result
    }

    fn run(arguments: &[&str], stdin: &str) -> (i32, Vec<u8>, Vec<u8>) {
        let mut input = Cursor::new(stdin.as_bytes());
        let mut output = Vec::new();
        let mut error = Vec::new();
        let code = run_cli(
            arguments.iter().copied(),
            &mut input,
            &mut output,
            &mut error,
        );

        (code, output, error)
    }

    #[test]
    fn parses_default_options() {
        let ParseOutcome::Run(parsed) =
            parse_arguments(vec!["num-2".into(), "num-1".into()]).expect("valid arguments")
        else {
            panic!("expected run outcome");
        };

        assert_eq!(parsed.options, CliOptions::new());
        assert_eq!(parsed.entries, vec!["num-2", "num-1"]);
    }

    #[test]
    fn parses_all_repeatable_options() {
        let arguments = [
            "--paths",
            "--reverse",
            "--locale",
            "--filter",
            "4",
            "10",
            "--reverse-filter",
            "100",
            "110",
            "--number-type",
            "float",
            "--noexp",
            "--sign",
            "--exclude",
            "34",
            "--exclude",
            "35",
            "num-2",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();

        let ParseOutcome::Run(parsed) = parse_arguments(arguments).expect("valid arguments") else {
            panic!("expected run outcome");
        };

        assert!(parsed.options.paths);
        assert!(parsed.options.reverse);
        assert!(parsed.options.locale);
        assert_eq!(
            parsed.options.filters,
            vec![NumericRange {
                low: 4.0,
                high: 10.0
            }]
        );
        assert_eq!(
            parsed.options.reverse_filters,
            vec![NumericRange {
                low: 100.0,
                high: 110.0
            }]
        );
        assert_eq!(parsed.options.exclude, vec![34.0, 35.0]);
        assert_eq!(parsed.options.number_type, CliNumberType::Float);
        assert!(parsed.options.signed);
        assert!(!parsed.options.exponent);
    }

    #[test]
    fn real_mode_forces_signed_float_even_after_nosign() {
        let options = CliOptions {
            number_type: CliNumberType::Real,
            signed: false,
            ..CliOptions::new()
        };
        let sort_options = options.sort_options();

        assert!(sort_options.float);
        assert!(sort_options.signed);
    }

    #[test]
    fn validates_ranges() {
        assert_eq!(
            NumericRange::new(1.0, 2.0),
            Ok(NumericRange {
                low: 1.0,
                high: 2.0
            })
        );
        assert!(NumericRange::new(2.0, 2.0).is_err());
        assert!(NumericRange::new(3.0, 2.0).is_err());
    }

    #[test]
    fn keeps_entries_matching_any_range() {
        let options = CliOptions::new();
        let ranges = [
            NumericRange {
                low: 1.0,
                high: 20.0,
            },
            NumericRange {
                low: 88.0,
                high: 90.0,
            },
        ];

        assert!(keep_entry_range("a56b23c89", &ranges, &options));
        assert!(!keep_entry_range(
            "a56b23c89",
            &[NumericRange {
                low: 1.0,
                high: 20.0
            }],
            &options,
        ));
    }

    #[test]
    fn excludes_entries_by_numeric_equality() {
        let options = CliOptions::new();

        assert!(!keep_entry_value("value05", &[5.0], &options));
        assert!(!keep_entry_value("value5.0", &[5.0], &options));
        assert!(keep_entry_value("value15", &[5.0], &options));
    }

    #[test]
    fn runs_basic_positional_sort() {
        let (code, output, error) = run(&["file10", "file2", "file1"], "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["file1", "file2", "file10"]));
        assert!(error.is_empty());
    }

    #[test]
    fn runs_reverse_sort() {
        let (code, output, _) = run(&["--reverse", "file10", "file2", "file1"], "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["file10", "file2", "file1"]));
    }

    #[test]
    fn reads_newline_separated_stdin() {
        let (code, output, _) = run(&[], "file10\nfile2\nfile1\n");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["file1", "file2", "file10"]));
    }

    #[test]
    fn reads_zero_terminated_stdin() {
        let (code, output, _) = run(&["-z"], "file10\0file2\0file1\0");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["file1", "file2", "file10"]));
    }

    #[test]
    fn empty_stdin_outputs_one_blank_entry() {
        let (code, output, _) = run(&[], "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&[""]));
    }

    #[test]
    fn applies_inclusive_filter() {
        let arguments = [
            "--filter", "2", "10", "value1", "value2", "value5", "value10", "value11", "plain",
        ];
        let (code, output, _) = run(&arguments, "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["value2", "value5", "value10"]));
    }

    #[test]
    fn applies_reverse_filter_and_keeps_plain_text() {
        let arguments = [
            "--reverse-filter",
            "2",
            "10",
            "value1",
            "value2",
            "value5",
            "value10",
            "value11",
            "plain",
        ];
        let (code, output, _) = run(&arguments, "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["plain", "value1", "value11"]));
    }

    #[test]
    fn supports_float_filtering() {
        let arguments = [
            "-t",
            "float",
            "--filter",
            "1.5",
            "3.0",
            "value1.25",
            "value1.5",
            "value2.75",
            "value3.0",
            "value3.5",
        ];
        let (code, output, _) = run(&arguments, "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["value1.5", "value2.75", "value3.0"]));
    }

    #[test]
    fn supports_signed_real_sorting() {
        let arguments = ["-t", "real", "--", "value-2.5", "value1", "value-10"];
        let (code, output, _) = run(&arguments, "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["value-10", "value-2.5", "value1"]));
    }

    #[test]
    fn noexp_changes_float_order() {
        let arguments = ["-t", "float", "--noexp", "value1e3", "value20", "value3e1"];
        let (code, output, _) = run(&arguments, "");

        assert_eq!(code, 0);
        assert_eq!(output, output_lines(&["value1e3", "value3e1", "value20"]));
    }

    #[test]
    fn reversed_filter_bounds_return_runtime_error() {
        let (code, output, error) = run(&["--filter", "10", "2", "value1"], "");

        assert_eq!(code, 1);
        assert!(output.is_empty());
        assert_eq!(error, output_lines(&["Error in --filter: low >= high"]));
    }

    #[test]
    fn invalid_number_type_returns_parse_error() {
        let (code, output, error) = run(&["--number-type", "decimal", "value1"], "");

        assert_eq!(code, 2);
        assert!(output.is_empty());
        assert!(
            String::from_utf8(error)
                .expect("UTF-8 error output")
                .contains("invalid choice: 'decimal'")
        );
    }

    #[test]
    fn unknown_option_returns_parse_error() {
        let (code, output, error) = run(&["--unknown-option", "value1"], "");

        assert_eq!(code, 2);
        assert!(output.is_empty());
        assert!(
            String::from_utf8(error)
                .expect("UTF-8 error output")
                .contains("unrecognized arguments: --unknown-option")
        );
    }
}
