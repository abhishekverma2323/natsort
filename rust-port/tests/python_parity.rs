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

#[test]
fn matches_python_float_leading_decimal_point_sorting() {
    let input = vec!["value.56", "value.5", "value.125", "value1"];

    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value.125", "value.5", "value.56", "value1"]
    );
}

#[test]
fn matches_python_signed_float_leading_decimal_point_sorting() {
    let input = vec!["value-.56", "value.5", "value-.125", "value1"];

    let options = SortOptions::new().float(true).signed(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-.56", "value-.125", "value.5", "value1"]
    );
}

#[test]
fn matches_python_float_trailing_decimal_point_sorting() {
    let input = vec!["value51.", "value5.", "value10.", "value2."];

    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value2.", "value5.", "value10.", "value51."]
    );
}

#[test]
fn matches_python_signed_float_trailing_decimal_point_sorting() {
    let input = vec!["value-51.", "value5.", "value-10.", "value2."];

    let options = SortOptions::new().float(true).signed(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-51.", "value-10.", "value2.", "value5."]
    );
}

#[test]
fn matches_python_no_exp_float_sorting() {
    let input = vec!["value5.034e1", "value50", "value5.5e2", "value5.25"];

    let options = SortOptions::new().float(true).no_exp(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value5.034e1", "value5.25", "value5.5e2", "value50",]
    );
}

#[test]
fn matches_python_signed_no_exp_float_sorting() {
    let input = vec!["value-5.034e1", "value-50", "value5.5e2", "value5.25"];

    let options = SortOptions::new().float(true).signed(true).no_exp(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value-50", "value-5.034e1", "value5.25", "value5.5e2",]
    );
}

#[test]
fn matches_python_valid_and_invalid_exponent_sorting() {
    let input = vec![
        "value1e",
        "value1e+",
        "value1e-",
        "value1e2",
        "value1e+2",
        "value1e-2",
    ];

    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "value1e-2",
            "value1e",
            "value1e+",
            "value1e-",
            "value1e2",
            "value1e+2",
        ]
    );
}

#[test]
fn matches_python_presort_equivalent_value_sorting() {
    let input = vec!["a1", "a1.45", "a01", "a1.4500"];

    let options = SortOptions::new().float(true).presort(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["a01", "a1", "a1.45", "a1.4500"]
    );
}

#[test]
fn matches_python_unicode_digit_character_sorting() {
    let input = vec!["value②", "value①", "value10", "value2"];

    assert_eq!(
        natsorted(&input),
        vec!["value①", "value②", "value2", "value10"]
    );
}

#[test]
fn matches_python_unicode_numeric_character_float_sorting() {
    let input = vec!["valueⅡ", "value⅓", "value2", "value1"];
    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value⅓", "value1", "valueⅡ", "value2"]
    );
}

#[test]
fn matches_python_mixed_unicode_numeric_character_sorting() {
    let input = vec!["value١٠", "value②", "value३", "value1"];
    let options = SortOptions::new().float(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["value1", "value②", "value३", "value١٠"]
    );
}

#[test]
fn sorts_complex_filesystem_paths_like_python() {
    let input = vec![
        "/p/Folder (10)/file.tar.gz",
        "/p/Folder (1)/file (1).tar.gz",
        "/p/Folder/file.x1.9.tar.gz",
        "/p/Folder (1)/file.tar.gz",
        "/p/Folder/file.x1.10.tar.gz",
    ];

    let options = SortOptions::new().float(true).path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "/p/Folder/file.x1.10.tar.gz",
            "/p/Folder/file.x1.9.tar.gz",
            "/p/Folder (1)/file.tar.gz",
            "/p/Folder (1)/file (1).tar.gz",
            "/p/Folder (10)/file.tar.gz",
        ]
    );
}

#[test]
fn sorts_path_extension_regression_case() {
    let input = vec![
        "Try.Me.Bug - 09 - One.Two.Three.[text].mkv",
        "Try.Me.Bug - 07 - One.Two.5.[text].mkv",
        "Try.Me.Bug - 08 - One.Two.Three[text].mkv",
    ];

    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "Try.Me.Bug - 07 - One.Two.5.[text].mkv",
            "Try.Me.Bug - 08 - One.Two.Three[text].mkv",
            "Try.Me.Bug - 09 - One.Two.Three.[text].mkv",
        ]
    );
}

#[test]
fn path_mode_separates_version_from_extensions() {
    let input = vec!["file.x1.9.tar.gz", "file.x1.10.tar.gz", "file.x1.2.tar.gz"];

    let options = SortOptions::new().float(true).path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["file.x1.10.tar.gz", "file.x1.2.tar.gz", "file.x1.9.tar.gz",]
    );
}

#[test]
fn sorts_rooted_paths_naturally() {
    let input = vec![
        "/folder10/file.txt",
        "/folder2/file.txt",
        "/folder1/file.txt",
    ];

    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "/folder1/file.txt",
            "/folder2/file.txt",
            "/folder10/file.txt",
        ]
    );
}

#[test]
fn sorts_parent_paths_before_children() {
    let input = vec!["folder2/file10.txt", "folder2", "folder2/file2.txt"];

    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["folder2", "folder2/file2.txt", "folder2/file10.txt",]
    );
}

#[test]
fn combines_path_and_ignore_case_options() {
    let input = vec!["Folder/file10.txt", "folder/File2.txt", "FOLDER/file1.txt"];

    let options = SortOptions::new().path(true).ignore_case(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["FOLDER/file1.txt", "folder/File2.txt", "Folder/file10.txt",]
    );
}

macro_rules! python_text_parity_case {
    (
        $name:ident,
        $input:expr,
        $options:expr,
        $expected:expr
    ) => {
        #[test]
        fn $name() {
            let input = $input;

            assert_eq!(natsorted_with_options(&input, $options,), $expected,);
        }
    };
}

python_text_parity_case!(
    matches_python_lowercase_first_sorting,
    ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
    SortOptions::new().lowercase_first(true),
    vec!["apple", "banana", "corn", "Apple", "Banana", "Corn"]
);

python_text_parity_case!(
    matches_python_group_letters_sorting,
    ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
    SortOptions::new().group_letters(true),
    vec!["Apple", "apple", "Banana", "banana", "Corn", "corn"]
);

python_text_parity_case!(
    matches_python_group_letters_lowercase_first_sorting,
    ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
    SortOptions::new().group_letters(true).lowercase_first(true),
    vec!["apple", "Apple", "banana", "Banana", "corn", "Corn"]
);

python_text_parity_case!(
    matches_python_capital_first_sorting,
    ["apple", "Apple", "banana", "Banana", "corn", "Corn"],
    SortOptions::new().capital_first(true),
    vec!["Apple", "Banana", "Corn", "apple", "banana", "corn"]
);

python_text_parity_case!(
    matches_python_capital_and_lowercase_first_sorting,
    ["Apple", "corn", "Corn", "Banana", "apple", "banana"],
    SortOptions::new().capital_first(true).lowercase_first(true),
    vec!["apple", "banana", "corn", "Apple", "Banana", "Corn"]
);

python_text_parity_case!(
    matches_python_sharp_s_casefold_sorting,
    ["straße10", "STRASSE2", "Strasse1", "strasse3"],
    SortOptions::new().ignore_case(true),
    vec!["Strasse1", "STRASSE2", "strasse3", "straße10"]
);

python_text_parity_case!(
    matches_python_greek_sigma_casefold_sorting,
    ["Σ10", "ς2", "σ1"],
    SortOptions::new().ignore_case(true),
    vec!["σ1", "ς2", "Σ10"]
);

python_text_parity_case!(
    matches_python_kelvin_sign_casefold_sorting,
    ["K10", "k2", "K1"],
    SortOptions::new().ignore_case(true),
    vec!["K1", "k2", "K10"]
);

python_text_parity_case!(
    matches_python_ignore_case_lowercase_first_sorting,
    ["Apple10", "apple2", "APPLE1", "aPpLe3"],
    SortOptions::new().ignore_case(true).lowercase_first(true),
    vec!["APPLE1", "apple2", "aPpLe3", "Apple10"]
);

python_text_parity_case!(
    matches_python_group_letters_ignore_case_sorting,
    ["Apple10", "apple2", "APPLE1", "aPpLe3"],
    SortOptions::new().group_letters(true).ignore_case(true),
    vec!["APPLE1", "apple2", "aPpLe3", "Apple10"]
);

python_text_parity_case!(
    matches_python_canonical_normalization_sorting,
    ["café10", "cafe\u{301}2", "café1"],
    SortOptions::new(),
    vec!["café1", "cafe\u{301}2", "café10"]
);

python_text_parity_case!(
    matches_python_canonical_ring_normalization_sorting,
    ["Å10", "A\u{30A}2", "Å1", "A2"],
    SortOptions::new(),
    vec!["A2", "Å1", "A\u{30A}2", "Å10"]
);

python_text_parity_case!(
    matches_python_ligature_compatibility_normalization,
    ["ﬀile10", "ffile2", "ﬀile1"],
    SortOptions::new().compatibility_normalize(true),
    vec!["ﬀile1", "ffile2", "ﬀile10"]
);

python_text_parity_case!(
    matches_python_fullwidth_compatibility_normalization,
    ["Ａ10", "A2", "Ａ1"],
    SortOptions::new().compatibility_normalize(true),
    vec!["Ａ1", "A2", "Ａ10"]
);

python_text_parity_case!(
    matches_python_circled_letter_compatibility_normalization,
    ["Ⓐ10", "A2", "Ⓐ1"],
    SortOptions::new().compatibility_normalize(true),
    vec!["Ⓐ1", "A2", "Ⓐ10"]
);

python_text_parity_case!(
    matches_python_number_compatibility_normalization,
    ["item²", "item2", "item①", "item1"],
    SortOptions::new().compatibility_normalize(true),
    vec!["item①", "item1", "item²", "item2"]
);

python_text_parity_case!(
    matches_python_lowercase_first_numeric_sorting,
    ["A10", "a2", "A1", "a1"],
    SortOptions::new().lowercase_first(true),
    vec!["a1", "a2", "A1", "A10"]
);

python_text_parity_case!(
    matches_python_group_letters_numeric_sorting,
    ["A10", "a2", "A1", "a1"],
    SortOptions::new().group_letters(true),
    vec!["A1", "A10", "a1", "a2"]
);

python_text_parity_case!(
    matches_python_path_lowercase_first_sorting,
    [
        "Folder10/File2",
        "folder2/file10",
        "Folder2/file1",
        "folder2/File2",
    ],
    SortOptions::new().path(true).lowercase_first(true),
    vec![
        "folder2/file10",
        "folder2/File2",
        "Folder2/file1",
        "Folder10/File2",
    ]
);

python_text_parity_case!(
    matches_python_path_group_letters_sorting,
    [
        "Folder10/File2",
        "folder2/file10",
        "Folder2/file1",
        "folder2/File2",
    ],
    SortOptions::new().path(true).group_letters(true),
    vec![
        "Folder2/file1",
        "Folder10/File2",
        "folder2/File2",
        "folder2/file10",
    ]
);

python_text_parity_case!(
    matches_python_unicode_path_ignore_case_sorting,
    ["Straße10/File2", "STRASSE2/file10", "strasse2/File1",],
    SortOptions::new().path(true).ignore_case(true),
    vec!["strasse2/File1", "STRASSE2/file10", "Straße10/File2",]
);
