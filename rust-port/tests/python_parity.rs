use rust_port::{SortOptions, natsorted, natsorted_with_options};

#[test]
fn matches_python_basic_natural_sort() {
    let input = vec!["file10", "file2", "file1"];

    assert_eq!(natsorted(&input), vec!["file1", "file2", "file10"]);
}

#[test]
fn matches_python_leading_zero_behavior() {
    let input = vec!["file10", "file002", "file2"];

    assert_eq!(natsorted(&input), vec!["file002", "file2", "file10"]);
}

#[test]
fn matches_python_large_integer_sorting() {
    let input = vec![
        "file999999999999999999999999",
        "file20",
        "file18446744073709551616",
    ];

    assert_eq!(
        natsorted(&input),
        vec![
            "file20",
            "file18446744073709551616",
            "file999999999999999999999999",
        ]
    );
}

#[test]
fn matches_python_signed_integer_sorting() {
    let input = vec!["value5", "value-2", "value1", "value-10"];
    let options = SortOptions::new().signed(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-10", "value-2", "value1", "value5"]
    );
}

#[test]
fn matches_python_float_sorting() {
    let input = vec!["value1.5", "value1.25", "value10.01", "value2.0"];
    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value1.25", "value1.5", "value2.0", "value10.01"]
    );
}

#[test]
fn matches_python_scientific_notation_sorting() {
    let input = vec!["value1e3", "value2.5e2", "value4.2e-3", "value1", "value10"];

    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value4.2e-3", "value1", "value10", "value2.5e2", "value1e3",]
    );
}

#[test]
fn matches_python_signed_float_sorting() {
    let input = vec!["value1.5", "value-2.25", "value-10.5", "value0.25"];

    let options = SortOptions::new().signed(true).float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-10.5", "value-2.25", "value0.25", "value1.5",]
    );
}

#[test]
fn matches_python_ignore_case_sorting() {
    let input = vec!["File10", "file2", "FILE1"];
    let options = SortOptions::new().ignore_case(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["FILE1", "file2", "File10"]
    );
}

#[test]
fn matches_python_reverse_sorting() {
    let input = vec!["file1", "file10", "file2"];
    let options = SortOptions::new().reverse(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["file10", "file2", "file1"]
    );
}
