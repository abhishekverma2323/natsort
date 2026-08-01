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

#[test]
fn matches_python_empty_input() {
    let input: Vec<&str> = vec![];

    assert_eq!(natsorted(&input), Vec::<&str>::new());
}

#[test]
fn matches_python_single_item_input() {
    let input = vec!["file10"];

    assert_eq!(natsorted(&input), vec!["file10"]);
}

#[test]
fn matches_python_plain_text_sorting() {
    let input = vec!["banana", "apple", "cherry"];

    assert_eq!(natsorted(&input), vec!["apple", "banana", "cherry"]);
}

#[test]
fn matches_python_multiple_numeric_components() {
    let input = vec!["version1.10.2", "version1.2.10", "version1.2.2"];

    assert_eq!(
        natsorted(&input),
        vec!["version1.2.2", "version1.2.10", "version1.10.2"]
    );
}

#[test]
fn matches_python_equivalent_leading_zero_values() {
    let input = vec!["file1", "file01", "file001"];

    assert_eq!(natsorted(&input), vec!["file1", "file01", "file001"]);
}

#[test]
fn matches_python_signed_zero_sorting() {
    let input = vec!["value-0", "value0", "value+0"];
    let options = SortOptions::new().signed(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-0", "value0", "value+0"]
    );
}

#[test]
fn matches_python_precise_decimal_sorting() {
    let input = vec!["value1.000000000002", "value1.000000000001", "value1.1"];
    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value1.000000000001", "value1.000000000002", "value1.1",]
    );
}

#[test]
fn matches_python_mixed_scientific_notation_sorting() {
    let input = vec!["value1E3", "value2e2", "value5E-1", "value10"];
    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value5E-1", "value10", "value2e2", "value1E3"]
    );
}

#[test]
fn matches_python_punctuation_and_separator_sorting() {
    let input = vec!["file-10", "file_2", "file.1", "file-2"];

    assert_eq!(
        natsorted(&input),
        vec!["file-2", "file-10", "file.1", "file_2"]
    );
}

#[test]
fn matches_python_arabic_indic_digit_sorting() {
    let input = vec!["file١٠", "file٢", "file١"];

    assert_eq!(natsorted(&input), vec!["file١", "file٢", "file١٠"]);
}

#[test]
fn matches_python_devanagari_digit_sorting() {
    let input = vec!["file१०", "file२", "file१"];

    assert_eq!(natsorted(&input), vec!["file१", "file२", "file१०"]);
}

#[test]
fn matches_python_fullwidth_digit_sorting() {
    let input = vec!["file１０", "file２", "file１"];

    assert_eq!(natsorted(&input), vec!["file１", "file２", "file１０"]);
}

#[test]
fn matches_python_mixed_unicode_digit_sorting() {
    let input = vec!["file10", "file٢", "file३", "file１"];

    assert_eq!(
        natsorted(&input),
        vec!["file１", "file٢", "file३", "file10"]
    );
}

#[test]
fn matches_python_signed_unicode_integer_sorting() {
    let input = vec!["value-१०", "value२", "value-१"];
    let options = SortOptions::new().signed(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-१०", "value-१", "value२"]
    );
}

#[test]
fn matches_python_unicode_decimal_sorting() {
    let input = vec!["value١.٥", "value١.٢٥", "value٢.٠"];
    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value١.٢٥", "value١.٥", "value٢.٠"]
    );
}

#[test]
fn matches_python_basic_path_sorting() {
    let input = vec!["folder/file10.txt", "folder/file2.txt", "folder/file1.txt"];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["folder/file1.txt", "folder/file2.txt", "folder/file10.txt",]
    );
}

#[test]
fn matches_python_nested_directory_path_sorting() {
    let input = vec![
        "folder10/file1.txt",
        "folder2/file10.txt",
        "folder2/file2.txt",
    ];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "folder2/file2.txt",
            "folder2/file10.txt",
            "folder10/file1.txt",
        ]
    );
}

#[test]
fn matches_python_path_file_extension_sorting() {
    let input = vec!["file10.tar.gz", "file2.txt", "file1.tar.gz", "file10.txt"];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["file1.tar.gz", "file2.txt", "file10.tar.gz", "file10.txt",]
    );
}

#[test]
fn matches_python_relative_path_sorting() {
    let input = vec!["./folder10/file1", "./folder2/file10", "./folder2/file2"];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["./folder2/file2", "./folder2/file10", "./folder10/file1",]
    );
}

#[test]
fn matches_python_hidden_file_path_sorting() {
    let input = vec![".file10", ".file2", ".file1", "file1"];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![".file1", ".file2", ".file10", "file1"]
    );
}

#[test]
fn matches_python_trailing_path_separator_sorting() {
    let input = vec!["folder10/", "folder2/file1", "folder2/", "folder1/"];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["folder1/", "folder2/", "folder2/file1", "folder10/",]
    );
}

#[test]
fn matches_python_windows_path_sorting() {
    let input = vec![
        r"folder10\file1.txt",
        r"folder2\file10.txt",
        r"folder2\file2.txt",
    ];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            r"folder2\file2.txt",
            r"folder2\file10.txt",
            r"folder10\file1.txt",
        ]
    );
}

#[test]
fn matches_python_path_numeric_component_sorting() {
    let input = vec![
        "release1/version10/file2.txt",
        "release1/version2/file10.txt",
        "release1/version2/file2.txt",
    ];
    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "release1/version2/file2.txt",
            "release1/version2/file10.txt",
            "release1/version10/file2.txt",
        ]
    );
}
