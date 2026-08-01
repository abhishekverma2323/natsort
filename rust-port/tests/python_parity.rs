use rust_port::{
    AlgorithmFlags, Decoder, LocaleProfile, NaturalValue, NumericRegexKind, OsSortOptions,
    OsSortProfile, SortOptions, as_ascii, as_utf8, humansorted_with_options,
    index_humansorted_with_options, index_natsorted, index_natsorted_values,
    index_natsorted_values_with_decoder, index_natsorted_values_with_options,
    index_natsorted_with_options, index_os_sorted_values_with_options, index_realsorted,
    index_realsorted_values, natsort_key_with_options, natsorted, natsorted_by_key,
    natsorted_by_key_with_options, natsorted_values, natsorted_values_with_decoder,
    natsorted_values_with_options, natsorted_with_options, order_by_index,
    os_sorted_by_key_with_options, os_sorted_with_options, realsorted, realsorted_values,
    realsorted_with_options,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApiRecord {
    name: &'static str,
    id: &'static str,
}

fn natural_text(value: &str) -> NaturalValue {
    NaturalValue::from(value)
}

fn natural_sequence(values: Vec<NaturalValue>) -> NaturalValue {
    NaturalValue::Sequence(values)
}

#[test]
fn matches_python_realsorted_basic() {
    let input = vec!["num5.10", "num-3", "num5.3", "num2"];

    assert_eq!(
        realsorted(&input),
        vec!["num-3", "num2", "num5.10", "num5.3"]
    );
}

#[test]
fn matches_python_realsorted_reverse() {
    let input = vec!["num5.10", "num-3", "num5.3", "num2"];

    assert_eq!(
        realsorted_with_options(&input, SortOptions::new().reverse(true),),
        vec!["num5.3", "num5.10", "num2", "num-3"]
    );
}

#[test]
fn matches_python_index_natsorted_basic() {
    assert_eq!(index_natsorted(&["num3", "num5", "num2"]), vec![2, 0, 1]);
}

#[test]
fn matches_python_index_natsorted_reverse() {
    let input = ["num3", "num5", "num2"];

    assert_eq!(
        index_natsorted_with_options(&input, SortOptions::new().reverse(true),),
        vec![1, 0, 2]
    );
}

#[test]
fn matches_python_index_realsorted_basic() {
    let input = ["num5.10", "num-3", "num5.3", "num2"];

    assert_eq!(index_realsorted(&input), vec![1, 3, 0, 2]);
}

#[test]
fn matches_python_order_by_index_primary() {
    let values = ["num3", "num5", "num2"];

    assert_eq!(
        order_by_index(&values, &[2, 0, 1]),
        vec!["num2", "num3", "num5"]
    );
}

#[test]
fn matches_python_order_by_index_secondary() {
    let values = ["foo", "bar", "baz"];

    assert_eq!(
        order_by_index(&values, &[2, 0, 1]),
        vec!["baz", "foo", "bar"]
    );
}

#[test]
fn matches_python_natsorted_with_key() {
    let input = vec![
        ApiRecord {
            name: "file10",
            id: "10",
        },
        ApiRecord {
            name: "file2",
            id: "2",
        },
        ApiRecord {
            name: "file1",
            id: "1",
        },
    ];

    let sorted = natsorted_by_key(&input, |record| record.name);

    let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

    assert_eq!(ids, vec!["1", "2", "10"]);
}

#[test]
fn matches_python_natsorted_with_key_reverse() {
    let input = vec![
        ApiRecord {
            name: "file10",
            id: "10",
        },
        ApiRecord {
            name: "file2",
            id: "2",
        },
        ApiRecord {
            name: "file1",
            id: "1",
        },
    ];

    let sorted = natsorted_by_key_with_options(
        &input,
        |record| record.name,
        SortOptions::new().reverse(true),
    );

    let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

    assert_eq!(ids, vec!["10", "2", "1"]);
}

#[test]
fn matches_python_key_sort_stability() {
    let input = vec![
        ApiRecord {
            name: "file01",
            id: "first",
        },
        ApiRecord {
            name: "file1",
            id: "second",
        },
        ApiRecord {
            name: "file001",
            id: "third",
        },
    ];

    let sorted = natsorted_by_key(&input, |record| record.name);

    let ids: Vec<&str> = sorted.iter().map(|record| record.id).collect();

    assert_eq!(ids, vec!["first", "second", "third"]);
}

#[test]
fn matches_python_index_equivalent_value_stability() {
    let input = ["file01", "file1", "file001"];

    assert_eq!(index_natsorted(&input), vec![0, 1, 2]);
}

#[test]
fn matches_python_index_presort() {
    let input = ["a1", "a1.45", "a01", "a1.4500"];

    let options = SortOptions::new().float(true).presort(true);

    assert_eq!(
        index_natsorted_with_options(&input, options),
        vec![2, 0, 1, 3]
    );
}

#[test]
fn matches_python_index_presort_reverse() {
    let input = ["a1", "a1.45", "a01", "a1.4500"];

    let options = SortOptions::new().float(true).presort(true).reverse(true);

    assert_eq!(
        index_natsorted_with_options(&input, options),
        vec![3, 1, 0, 2]
    );
}

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

#[test]
fn matches_python_mixed_text_and_integer_sorting() {
    let input = vec![natural_text("a2"), 3.into(), natural_text("a1"), 2.into()];

    assert_eq!(
        natsorted_values(&input),
        vec![2.into(), 3.into(), natural_text("a1"), natural_text("a2"),]
    );
}

#[test]
fn matches_python_numeric_string_and_integer_sorting() {
    let input = vec![natural_text("10"), 2.into(), natural_text("1"), 11.into()];

    assert_eq!(
        natsorted_values(&input),
        vec![natural_text("1"), 2.into(), natural_text("10"), 11.into(),]
    );
}

#[test]
fn matches_python_signed_mixed_value_sorting() {
    let input = vec![
        natural_text("value5"),
        (-3).into(),
        natural_text("value-2"),
        1.into(),
        natural_text("value1"),
    ];

    assert_eq!(
        realsorted_values(&input),
        vec![
            (-3).into(),
            1.into(),
            natural_text("value-2"),
            natural_text("value1"),
            natural_text("value5"),
        ]
    );
}

#[test]
fn matches_python_direct_numeric_sorting() {
    let input = vec![5.1.into(), (-3.0).into(), 5.3.into(), 2.into()];

    assert_eq!(
        natsorted_values(&input),
        vec![(-3.0).into(), 2.into(), 5.1.into(), 5.3.into(),]
    );
}

#[test]
fn matches_python_infinity_sorting() {
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
fn matches_python_none_and_nan_default_sorting() {
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
}

#[test]
fn matches_python_none_and_nan_last_sorting() {
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
fn matches_python_nested_string_sequence_sorting() {
    let input = vec![
        natural_sequence(vec![natural_text("a10"), natural_text("b2")]),
        natural_sequence(vec![natural_text("a2"), natural_text("b10")]),
        natural_sequence(vec![natural_text("a2"), natural_text("b2")]),
    ];

    assert_eq!(
        natsorted_values(&input),
        vec![
            natural_sequence(vec![natural_text("a2"), natural_text("b2"),]),
            natural_sequence(vec![natural_text("a2"), natural_text("b10"),]),
            natural_sequence(vec![natural_text("a10"), natural_text("b2"),]),
        ]
    );
}

#[test]
fn matches_python_nested_mixed_sequence_sorting() {
    let input = vec![
        natural_sequence(vec![natural_text("a2"), 10.into()]),
        natural_sequence(vec![natural_text("a2"), 2.into()]),
        natural_sequence(vec![natural_text("a1"), 20.into()]),
    ];

    assert_eq!(
        natsorted_values(&input),
        vec![
            natural_sequence(vec![natural_text("a1"), 20.into(),]),
            natural_sequence(vec![natural_text("a2"), 2.into(),]),
            natural_sequence(vec![natural_text("a2"), 10.into(),]),
        ]
    );
}

#[test]
fn matches_python_bytes_default_sorting() {
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
fn matches_python_bytes_ignore_case_sorting() {
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
fn matches_python_bytes_path_mode() {
    let input = vec![
        NaturalValue::from(b"folder10/file".as_slice()),
        NaturalValue::from(b"folder2/file".as_slice()),
        NaturalValue::from(b"folder1/file".as_slice()),
    ];

    let options = SortOptions::new().path(true);

    assert_eq!(
        natsorted_values_with_options(&input, options),
        vec![
            NaturalValue::from(b"folder1/file".as_slice(),),
            NaturalValue::from(b"folder10/file".as_slice(),),
            NaturalValue::from(b"folder2/file".as_slice(),),
        ]
    );
}

#[test]
fn matches_python_mixed_bytes_string_decoder_sorting() {
    let input = vec![
        NaturalValue::from(b"a10".as_slice()),
        NaturalValue::from("a2"),
        NaturalValue::from(b"a1".as_slice()),
    ];

    assert_eq!(
        natsorted_values_with_decoder(&input, Decoder::utf8(),),
        Ok(vec![
            NaturalValue::from(b"a1".as_slice()),
            NaturalValue::from("a2"),
            NaturalValue::from(b"a10".as_slice()),
        ])
    );
}

#[test]
fn matches_python_ascii_decoder() {
    assert_eq!(
        as_ascii(&NaturalValue::from(b"natural10".as_slice())),
        Ok(NaturalValue::from("natural10")),
    );
}

#[test]
fn matches_python_utf8_decoder() {
    assert_eq!(
        as_utf8(&NaturalValue::from("café".as_bytes())),
        Ok(NaturalValue::from("café")),
    );
}

#[test]
fn matches_python_decoder_non_bytes_passthrough() {
    let value = NaturalValue::from(123);

    assert_eq!(as_utf8(&value), Ok(value));
}

#[test]
fn matches_python_invalid_utf8_error() {
    assert!(as_utf8(&NaturalValue::from([0xFF].as_slice())).is_err());
}

#[test]
fn matches_python_mixed_value_indexes() {
    let input = vec![
        NaturalValue::from("a2"),
        3.into(),
        NaturalValue::from("a1"),
        2.into(),
    ];

    assert_eq!(index_natsorted_values(&input), vec![3, 1, 2, 0]);
}

#[test]
fn matches_python_numeric_string_and_integer_indexes() {
    let input = vec![
        NaturalValue::from("10"),
        2.into(),
        NaturalValue::from("1"),
        11.into(),
    ];

    assert_eq!(index_natsorted_values(&input), vec![2, 1, 0, 3]);
}

#[test]
fn matches_python_signed_mixed_value_indexes() {
    let input = vec![
        NaturalValue::from("value5"),
        (-3).into(),
        NaturalValue::from("value-2"),
        1.into(),
        NaturalValue::from("value1"),
    ];

    assert_eq!(index_realsorted_values(&input), vec![1, 3, 2, 4, 0]);
}

#[test]
fn matches_python_direct_numeric_value_indexes() {
    let input = vec![5.1.into(), (-3.0).into(), 5.3.into(), 2.into()];

    assert_eq!(index_natsorted_values(&input), vec![1, 3, 0, 2]);
}

#[test]
fn matches_python_infinity_value_indexes() {
    let input = vec![
        f64::INFINITY.into(),
        5.into(),
        f64::NEG_INFINITY.into(),
        0.into(),
    ];

    assert_eq!(index_natsorted_values(&input), vec![2, 3, 1, 0]);
}

#[test]
fn matches_python_none_and_nan_default_indexes() {
    let input = vec![
        3.into(),
        NaturalValue::None,
        f64::NAN.into(),
        f64::NEG_INFINITY.into(),
        2.into(),
    ];

    assert_eq!(index_natsorted_values(&input), vec![2, 1, 3, 4, 0]);
}

#[test]
fn matches_python_none_and_nan_last_indexes() {
    let input = vec![
        3.into(),
        NaturalValue::None,
        f64::NAN.into(),
        f64::INFINITY.into(),
        2.into(),
    ];
    let options = SortOptions::new().nan_last(true);

    assert_eq!(
        index_natsorted_values_with_options(&input, options),
        vec![4, 0, 3, 1, 2]
    );
}

#[test]
fn matches_python_nested_sequence_indexes() {
    let input = vec![
        NaturalValue::Sequence(vec![NaturalValue::from("a10"), NaturalValue::from("b2")]),
        NaturalValue::Sequence(vec![NaturalValue::from("a2"), NaturalValue::from("b10")]),
        NaturalValue::Sequence(vec![NaturalValue::from("a2"), NaturalValue::from("b2")]),
    ];

    assert_eq!(index_natsorted_values(&input), vec![2, 1, 0]);
}

#[test]
fn matches_python_nested_mixed_sequence_indexes() {
    let input = vec![
        NaturalValue::Sequence(vec![NaturalValue::from("a2"), 10.into()]),
        NaturalValue::Sequence(vec![NaturalValue::from("a2"), 2.into()]),
        NaturalValue::Sequence(vec![NaturalValue::from("a1"), 20.into()]),
    ];

    assert_eq!(index_natsorted_values(&input), vec![2, 1, 0]);
}

#[test]
fn matches_python_byte_value_indexes() {
    let input = vec![
        NaturalValue::from(b"a10".as_slice()),
        NaturalValue::from(b"a2".as_slice()),
        NaturalValue::from(b"A1".as_slice()),
    ];

    assert_eq!(index_natsorted_values(&input), vec![2, 0, 1]);
}

#[test]
fn matches_python_decoder_mixed_value_indexes() {
    let input = vec![
        NaturalValue::from(b"a10".as_slice()),
        NaturalValue::from("a2"),
        NaturalValue::from(b"a1".as_slice()),
    ];

    assert_eq!(
        index_natsorted_values_with_decoder(&input, Decoder::utf8()),
        Ok(vec![2, 1, 0])
    );
}

#[test]
fn matches_python_num_after_basic_sorting() {
    let input = ["73", "5039", "Banana", "apple", "corn", "~~~~~~"];

    let options = SortOptions::new().num_after(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["Banana", "apple", "corn", "~~~~~~", "73", "5039",]
    );
}

#[test]
fn matches_python_num_after_mixed_direct_values() {
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
fn matches_python_num_after_ignore_case() {
    let input = ["10", "Apple", "apple", "2", "Banana", "banana"];

    let options = SortOptions::new().num_after(true).ignore_case(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["Apple", "apple", "Banana", "banana", "2", "10",]
    );
}

#[test]
fn matches_python_num_after_group_letters() {
    let input = ["10", "Apple", "apple", "2", "Banana", "banana"];

    let options = SortOptions::new().num_after(true).group_letters(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["Apple", "apple", "Banana", "banana", "2", "10",]
    );
}

#[test]
fn matches_python_num_after_path_sorting() {
    let input = ["10", "folder10/file", "folder2/file", "2", "apple"];

    let options = SortOptions::new().num_after(true).path(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["apple", "folder2/file", "folder10/file", "2", "10",]
    );
}

#[test]
fn matches_python_num_after_signed_values() {
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
fn matches_python_num_after_float_values() {
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
fn matches_python_num_after_reverse() {
    let input = ["73", "5039", "Banana", "apple", "corn", "~~~~~~"];

    let options = SortOptions::new().num_after(true).reverse(true);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["5039", "73", "~~~~~~", "corn", "apple", "Banana",]
    );
}

#[test]
fn matches_python_num_after_indexes() {
    let input = vec![
        NaturalValue::from("73"),
        NaturalValue::from("5039"),
        NaturalValue::from("Banana"),
        NaturalValue::from("apple"),
        NaturalValue::from("corn"),
        NaturalValue::from("~~~~~~"),
    ];

    let options = SortOptions::new().num_after(true);

    assert_eq!(
        index_natsorted_values_with_options(&input, options,),
        vec![2, 3, 4, 5, 0, 1]
    );
}

#[test]
fn matches_python_english_locale_alpha() {
    let input = [
        "Apple", "apple", "Äpfel", "äpfel", "Banana", "banana", "Öl", "Oase", "Zebra",
    ];
    let options = SortOptions::new()
        .locale_alpha(true)
        .locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "äpfel", "Äpfel", "apple", "Apple", "banana", "Banana", "Oase", "Öl", "Zebra",
        ]
    );
}

#[test]
fn matches_python_c_locale_alpha() {
    let input = [
        "Apple", "apple", "Äpfel", "äpfel", "Banana", "banana", "Öl", "Oase", "Zebra",
    ];
    let options = SortOptions::new()
        .locale_alpha(true)
        .locale_profile(LocaleProfile::C);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "apple", "Apple", "banana", "Banana", "Oase", "Zebra", "äpfel", "Äpfel", "Öl",
        ]
    );
}

#[test]
fn matches_python_english_human_natural_sort() {
    let input = [
        "file10", "File2", "file1", "Äpfel20", "Äpfel3", "apple11", "apple2", "Öl5", "Oase4",
    ];
    let options = SortOptions::new().locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        humansorted_with_options(&input, options),
        vec![
            "Äpfel3", "Äpfel20", "apple2", "apple11", "file1", "file10", "File2", "Oase4", "Öl5",
        ]
    );
}

#[test]
fn matches_python_english_human_indexes() {
    let input = [
        "file10", "File2", "file1", "Äpfel20", "Äpfel3", "apple11", "apple2", "Öl5", "Oase4",
    ];
    let options = SortOptions::new().locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        index_humansorted_with_options(&input, options),
        vec![4, 3, 6, 5, 2, 0, 1, 8, 7],
    );
}

#[test]
fn matches_python_english_localized_numbers() {
    let input = ["1,234.50", "12.50", "2.75", "1,000.25", "10.25"];
    let options = SortOptions::new()
        .float(true)
        .locale_numeric(true)
        .locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["2.75", "10.25", "12.50", "1,000.25", "1,234.50"]
    );
}

#[test]
fn matches_python_german_localized_numbers() {
    let input = ["1.234,50", "12,50", "2,75", "1.000,25", "10,25"];
    let options = SortOptions::new()
        .float(true)
        .locale_numeric(true)
        .locale_profile(LocaleProfile::GermanGermany);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["2,75", "10,25", "12,50", "1.000,25", "1.234,50"]
    );
}

#[test]
fn matches_python_french_localized_numbers() {
    let input = [
        "1\u{202F}234,50",
        "12,50",
        "2,75",
        "1\u{202F}000,25",
        "10,25",
    ];
    let options = SortOptions::new()
        .float(true)
        .locale_numeric(true)
        .locale_profile(LocaleProfile::FrenchFrance);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec![
            "2,75",
            "10,25",
            "12,50",
            "1\u{202F}000,25",
            "1\u{202F}234,50",
        ]
    );
}

#[test]
fn matches_python_mixed_english_locale_sort() {
    let input = ["1,000.50", "Apple2", "10.25", "apple10", "2.50", "Äpfel1"];
    let options = SortOptions::new()
        .float(true)
        .locale(true)
        .locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["2.50", "10.25", "1,000.50", "Äpfel1", "apple10", "Apple2"]
    );
}

#[test]
fn matches_python_mixed_english_locale_indexes() {
    let input = ["1,000.50", "Apple2", "10.25", "apple10", "2.50", "Äpfel1"];
    let options = SortOptions::new()
        .float(true)
        .locale(true)
        .locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        index_natsorted_with_options(&input, options),
        vec![4, 2, 0, 5, 3, 1]
    );
}

#[test]
fn locale_key_equates_english_localized_number_and_direct_number() {
    let options = SortOptions::new()
        .float(true)
        .locale(true)
        .locale_profile(LocaleProfile::EnglishUnitedStates);

    assert_eq!(
        natsort_key_with_options(&NaturalValue::from("1,234.50"), options),
        natsort_key_with_options(&NaturalValue::from(1234.5), options),
    );
}

#[test]
fn locale_key_equates_german_localized_number_and_direct_number() {
    let options = SortOptions::new()
        .float(true)
        .locale(true)
        .locale_profile(LocaleProfile::GermanGermany);

    assert_eq!(
        natsort_key_with_options(&NaturalValue::from("1.234,50"), options),
        natsort_key_with_options(&NaturalValue::from(1234.5), options),
    );
}

fn windows_os_options() -> OsSortOptions {
    OsSortOptions::new()
        .profile(OsSortProfile::Windows)
        .locale_profile(LocaleProfile::EnglishIndia)
}

#[test]
fn matches_python_windows_os_basic_sort() {
    let input = ["file10", "file2", "File3", "file1", "file_0"];

    assert_eq!(
        os_sorted_with_options(&input, windows_os_options()),
        vec!["file_0", "file1", "file2", "File3", "file10"]
    );
}

#[test]
fn matches_python_windows_os_reverse_sort() {
    let input = ["file10", "file2", "File3", "file1", "file_0"];

    assert_eq!(
        os_sorted_with_options(&input, windows_os_options().reverse(true)),
        vec!["file10", "File3", "file2", "file1", "file_0"]
    );
}

#[test]
fn matches_python_windows_os_leading_zero_order() {
    let input = ["a1", "a01", "a001"];

    assert_eq!(
        os_sorted_with_options(&input, windows_os_options()),
        vec!["a001", "a01", "a1"]
    );
}

#[test]
fn matches_python_windows_os_key_sort() {
    let input = ["foo0", "foo2", "goo1"];

    assert_eq!(
        os_sorted_by_key_with_options(
            &input,
            |value| value.replace('g', "f"),
            windows_os_options(),
        ),
        vec!["foo0", "goo1", "foo2"]
    );
}

#[test]
fn matches_python_windows_os_path_sort() {
    let input = [
        "Folder10/file2.txt",
        "Folder2/file10.txt",
        "Folder2/file2.txt",
        "folder1/file20.txt",
    ];

    assert_eq!(
        os_sorted_with_options(&input, windows_os_options()),
        vec![
            "folder1/file20.txt",
            "Folder2/file2.txt",
            "Folder2/file10.txt",
            "Folder10/file2.txt",
        ]
    );
}

#[test]
fn matches_python_windows_os_unicode_sort() {
    let input = ["Äpfel10", "apple2", "Apple10", "äpfel2", "Öl5", "Oase4"];

    assert_eq!(
        os_sorted_with_options(&input, windows_os_options()),
        vec!["äpfel2", "Äpfel10", "apple2", "Apple10", "Oase4", "Öl5"]
    );
}

#[test]
fn matches_python_windows_os_mixed_indexes() {
    let input = vec![
        NaturalValue::from("10"),
        NaturalValue::from(2),
        NaturalValue::from("1"),
        NaturalValue::from(11),
        NaturalValue::from(5.5),
        NaturalValue::None,
        NaturalValue::from(f64::NAN),
    ];

    assert_eq!(
        index_os_sorted_values_with_options(&input, windows_os_options()),
        vec![2, 1, 4, 0, 3, 6, 5]
    );
}

#[test]
fn matches_python_windows_os_punctuation_sort() {
    let input = [
        "11111", "aaaaa", "foo0", "foo_0", "foo1", "foo2", "foo4", "foo10", "Foo3", "!", "#", "$",
        "%", "&", "'", "(", ")", "+", "+11111", "+aaaaa", ",", "-", ";", "=", "@", "[", "]", "^",
        "_", "`", "{", "}", "~", "§", "°", "´", "µ", "€",
    ];

    assert_eq!(
        os_sorted_with_options(&input, windows_os_options()),
        vec![
            "'", "-", "!", "#", "$", "%", "&", "(", ")", ",", ";", "@", "[", "]", "^", "_", "`",
            "{", "}", "~", "´", "€", "+", "+11111", "+aaaaa", "=", "§", "°", "µ", "11111", "aaaaa",
            "foo_0", "foo0", "foo1", "foo2", "Foo3", "foo4", "foo10",
        ]
    );
}

#[test]
fn matches_python_algorithm_flag_values() {
    assert_eq!(AlgorithmFlags::DEFAULT.bits(), 0);
    assert_eq!(AlgorithmFlags::INT.bits(), 0);
    assert_eq!(AlgorithmFlags::UNSIGNED.bits(), 0);
    assert_eq!(AlgorithmFlags::FLOAT.bits(), 1);
    assert_eq!(AlgorithmFlags::SIGNED.bits(), 2);
    assert_eq!(AlgorithmFlags::NOEXP.bits(), 4);
    assert_eq!(AlgorithmFlags::PATH.bits(), 8);
    assert_eq!(AlgorithmFlags::LOCALEALPHA.bits(), 16);
    assert_eq!(AlgorithmFlags::LOCALENUM.bits(), 32);
    assert_eq!(AlgorithmFlags::LOCALE.bits(), 48);
    assert_eq!(AlgorithmFlags::IGNORECASE.bits(), 64);
    assert_eq!(AlgorithmFlags::LOWERCASEFIRST.bits(), 128);
    assert_eq!(AlgorithmFlags::GROUPLETTERS.bits(), 256);
    assert_eq!(AlgorithmFlags::UNGROUPLETTERS.bits(), 512);
    assert_eq!(AlgorithmFlags::NANLAST.bits(), 1024);
    assert_eq!(AlgorithmFlags::COMPATIBILITYNORMALIZE.bits(), 2048);
    assert_eq!(AlgorithmFlags::NUMAFTER.bits(), 4096);
    assert_eq!(AlgorithmFlags::PRESORT.bits(), 8192);
    assert_eq!(AlgorithmFlags::REAL.bits(), 3);
}

#[test]
fn matches_python_algorithm_aliases() {
    assert_eq!(AlgorithmFlags::DEFAULT, AlgorithmFlags::INT);
    assert_eq!(AlgorithmFlags::INT, AlgorithmFlags::UNSIGNED);
    assert_eq!(
        AlgorithmFlags::REAL,
        AlgorithmFlags::SIGNED | AlgorithmFlags::FLOAT
    );
    assert_eq!(
        AlgorithmFlags::LOCALE,
        AlgorithmFlags::LOCALEALPHA | AlgorithmFlags::LOCALENUM
    );
}

#[test]
fn matches_python_default_numeric_regex() {
    let pattern = rust_port::numeric_regex_chooser(AlgorithmFlags::DEFAULT);

    assert!(pattern.starts_with(r"\d+|[²³¹፩፪፫"));
    assert!(pattern.ends_with("🄆🄇🄈🄉🄊]"));
}

#[test]
fn matches_python_signed_integer_numeric_regex() {
    let pattern = rust_port::numeric_regex_chooser(AlgorithmFlags::SIGNED);

    assert!(pattern.starts_with(r"[-+]?\d+|[²³¹"));
}

#[test]
fn matches_python_float_numeric_regex() {
    let pattern = rust_port::numeric_regex_chooser(AlgorithmFlags::FLOAT);

    assert!(pattern.starts_with(r"(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?|[²³¹¼½¾"));
    assert!(pattern.ends_with("𠫽𠬙𢎐𢦘𣬛𦉭廾]"));
}

#[test]
fn matches_python_signed_float_noexp_regex() {
    let flags = AlgorithmFlags::FLOAT | AlgorithmFlags::SIGNED | AlgorithmFlags::NOEXP;

    assert!(rust_port::numeric_regex_chooser(flags).starts_with(r"[-+]?(?:\d+\.?\d*|\.\d+)|["));
    assert_eq!(
        NumericRegexKind::from_algorithm(flags),
        NumericRegexKind::SignedFloatNoExponent
    );
}

#[test]
fn matches_python_regex_selection_with_non_numeric_flags() {
    let base = AlgorithmFlags::FLOAT;
    let decorated = base
        | AlgorithmFlags::PATH
        | AlgorithmFlags::LOCALE
        | AlgorithmFlags::IGNORECASE
        | AlgorithmFlags::PRESORT;

    assert_eq!(
        rust_port::numeric_regex_chooser(base),
        rust_port::numeric_regex_chooser(decorated)
    );
}

#[test]
fn matches_python_regex_selection_for_arbitrary_integers() {
    assert_eq!(
        rust_port::numeric_regex_chooser_from_bits(-1),
        rust_port::numeric_regex_chooser_from_bits(999_999_999)
    );
}

#[test]
fn algorithm_flags_drive_existing_sort_options() {
    let options = SortOptions::from_algorithm(AlgorithmFlags::REAL | AlgorithmFlags::PRESORT);
    let input = ["a1", "a1.45", "a01", "a1.4500"];

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["a01", "a1", "a1.45", "a1.4500"]
    );
}

#[test]
fn algorithm_flags_drive_locale_sorting() {
    let options = SortOptions::from_algorithm(AlgorithmFlags::FLOAT | AlgorithmFlags::LOCALE)
        .locale_profile(LocaleProfile::EnglishUnitedStates);

    let input = ["1,000.25", "12.50", "2.75", "10.25"];

    assert_eq!(
        natsorted_with_options(&input, options),
        vec!["2.75", "10.25", "12.50", "1,000.25"]
    );
}

#[test]
fn sort_options_round_trip_python_algorithm_bits() {
    let flags = AlgorithmFlags::REAL
        | AlgorithmFlags::PATH
        | AlgorithmFlags::LOCALE
        | AlgorithmFlags::IGNORECASE
        | AlgorithmFlags::LOWERCASEFIRST
        | AlgorithmFlags::GROUPLETTERS
        | AlgorithmFlags::UNGROUPLETTERS
        | AlgorithmFlags::NANLAST
        | AlgorithmFlags::COMPATIBILITYNORMALIZE
        | AlgorithmFlags::NUMAFTER
        | AlgorithmFlags::PRESORT;

    assert_eq!(SortOptions::from_algorithm(flags).algorithm(), flags);
}

fn cli_output_lines(values: &[&str]) -> Vec<u8> {
    let mut result = Vec::new();

    for value in values {
        result.extend_from_slice(value.as_bytes());
        result.extend_from_slice(rust_port::native_line_ending().as_bytes());
    }

    result
}

fn run_cli_case(arguments: &[&str], stdin: &str) -> (i32, Vec<u8>, Vec<u8>) {
    use std::io::Cursor;

    let mut input = Cursor::new(stdin.as_bytes());
    let mut output = Vec::new();
    let mut error = Vec::new();
    let code = rust_port::run_cli(
        arguments.iter().copied(),
        &mut input,
        &mut output,
        &mut error,
    );

    (code, output, error)
}

#[test]
fn matches_python_cli_positional_basic() {
    let (code, output, error) = run_cli_case(&["file10", "file2", "file1"], "");

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["file1", "file2", "file10"]));
    assert!(error.is_empty());
}

#[test]
fn matches_python_cli_path_sort() {
    let (code, output, _) =
        run_cli_case(&["--paths", "Folder (10)/", "Folder (1)/", "Folder/"], "");

    assert_eq!(code, 0);
    assert_eq!(
        output,
        cli_output_lines(&["Folder/", "Folder (1)/", "Folder (10)/"])
    );
}

#[test]
fn matches_python_cli_real_sort() {
    let (code, output, _) =
        run_cli_case(&["-t", "real", "--", "value-2.5", "value1", "value-10"], "");

    assert_eq!(code, 0);
    assert_eq!(
        output,
        cli_output_lines(&["value-10", "value-2.5", "value1"])
    );
}

#[test]
fn matches_python_cli_real_with_nosign() {
    let (code, output, _) = run_cli_case(
        &[
            "-t",
            "real",
            "--nosign",
            "--",
            "value-2.5",
            "value1",
            "value-10",
        ],
        "",
    );

    assert_eq!(code, 0);
    assert_eq!(
        output,
        cli_output_lines(&["value-10", "value-2.5", "value1"])
    );
}

#[test]
fn matches_python_cli_float_exponent_sort() {
    let (code, output, _) = run_cli_case(&["-t", "float", "value1e3", "value20", "value3e1"], "");

    assert_eq!(code, 0);
    assert_eq!(
        output,
        cli_output_lines(&["value20", "value3e1", "value1e3"])
    );
}

#[test]
fn matches_python_cli_float_noexp_sort() {
    let (code, output, _) = run_cli_case(
        &["-t", "float", "--noexp", "value1e3", "value20", "value3e1"],
        "",
    );

    assert_eq!(code, 0);
    assert_eq!(
        output,
        cli_output_lines(&["value1e3", "value3e1", "value20"])
    );
}

#[test]
fn matches_python_cli_newline_stdin() {
    let (code, output, _) = run_cli_case(&[], "file10\nfile2\nfile1\n");

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["file1", "file2", "file10"]));
}

#[test]
fn matches_python_cli_whitespace_from_newline_stdin() {
    for stdin in [" num-2\n num-6 \n num-1", " num-2\n num-6 \n num-1\n"] {
        let (code, output, error) = run_cli_case(&[], stdin);

        assert_eq!(code, 0);
        assert!(error.is_empty());
        assert_eq!(output, cli_output_lines(&[" num-1", " num-2", " num-6 "]));
    }
}

#[test]
fn matches_python_cli_whitespace_from_zero_terminated_stdin() {
    for stdin in [" num-2\0 num-6 \0 num-1", " num-2\0 num-6 \0 num-1\0"] {
        let (code, output, error) = run_cli_case(&["--zero-terminated"], stdin);

        assert_eq!(code, 0);
        assert!(error.is_empty());
        assert_eq!(output, cli_output_lines(&[" num-1", " num-2", " num-6 "]));
    }
}

#[test]
fn matches_python_cli_whitespace_from_command_line_entries() {
    let (code, output, error) = run_cli_case(&[" num-2", " num-6 ", " num-1"], "");

    assert_eq!(code, 0);
    assert!(error.is_empty());
    assert_eq!(output, cli_output_lines(&[" num-1", " num-2", " num-6 "]));
}

#[test]
fn matches_python_cli_zero_terminated_stdin() {
    let (code, output, _) = run_cli_case(&["--zero-terminated"], "file10\0file2\0file1\0");

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["file1", "file2", "file10"]));
}

#[test]
fn matches_python_cli_empty_stdin() {
    let (code, output, _) = run_cli_case(&[], "");

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&[""]));
}

#[test]
fn matches_python_cli_inclusive_filter() {
    let (code, output, _) = run_cli_case(
        &[
            "--filter", "2", "10", "value1", "value2", "value5", "value10", "value11", "plain",
        ],
        "",
    );

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["value2", "value5", "value10"]));
}

#[test]
fn matches_python_cli_reverse_filter() {
    let (code, output, _) = run_cli_case(
        &[
            "--reverse-filter",
            "2",
            "10",
            "value1",
            "value2",
            "value5",
            "value10",
            "value11",
            "plain",
        ],
        "",
    );

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["plain", "value1", "value11"]));
}

#[test]
fn matches_python_cli_multiple_number_filtering() {
    let (code, output, _) = run_cli_case(
        &["--filter", "2", "10", "a1b20", "a1b5", "a20b30", "plain"],
        "",
    );

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["a1b5"]));
}

#[test]
fn matches_python_cli_integer_exclude() {
    let (code, output, _) = run_cli_case(
        &[
            "--exclude",
            "5",
            "value5",
            "value05",
            "value5.0",
            "value15",
            "plain",
        ],
        "",
    );

    assert_eq!(code, 0);
    assert_eq!(output, cli_output_lines(&["plain", "value15"]));
}

#[test]
fn matches_python_cli_reversed_range_error() {
    let (code, output, error) = run_cli_case(&["--filter", "10", "2", "value1", "value5"], "");

    assert_eq!(code, 1);
    assert!(output.is_empty());
    assert_eq!(error, cli_output_lines(&["Error in --filter: low >= high"]));
}

#[test]
fn matches_python_cli_invalid_number_type_exit_code() {
    let (code, output, error) = run_cli_case(&["--number-type", "decimal", "value1"], "");

    assert_eq!(code, 2);
    assert!(output.is_empty());
    assert!(
        String::from_utf8(error)
            .expect("UTF-8 error output")
            .contains("invalid choice: 'decimal'")
    );
}

#[test]
fn matches_python_latin1_decoder() {
    let encoded = NaturalValue::from([b'c', b'a', b'f', 0xE9].as_slice());

    assert_eq!(
        Decoder::latin1().decode(&encoded),
        Ok(NaturalValue::from("café")),
    );
}

#[test]
fn matches_python_latin1_decoder_sorting() {
    let input = vec![
        NaturalValue::from(b"caf\xe910".as_slice()),
        NaturalValue::from("café2"),
        NaturalValue::from(b"caf\xe91".as_slice()),
    ];

    assert_eq!(
        natsorted_values_with_decoder(&input, Decoder::latin1()),
        Ok(vec![
            NaturalValue::from(b"caf\xe91".as_slice()),
            NaturalValue::from("café2"),
            NaturalValue::from(b"caf\xe910".as_slice()),
        ]),
    );
}
