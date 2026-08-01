use grift_unicode::digit_value;

use crate::unicode_numeric::unicode_numeric_value;

pub(crate) fn is_path_separator(character: char) -> bool {
    matches!(character, '/' | '\\')
}

fn is_decimal_digit(character: char) -> bool {
    digit_value(character).is_some() && unicode_numeric_value(character).is_none()
}

fn is_dot_only_path(input: &str) -> bool {
    if input.is_empty() {
        return true;
    }

    let mut saw_dot = false;

    for component in input.split(is_path_separator) {
        if component.is_empty() {
            continue;
        }

        if component != "." {
            return false;
        }

        saw_dot = true;
    }

    saw_dot
}

fn suffix_candidates(base: &str) -> Vec<String> {
    let dot_positions: Vec<usize> = base
        .char_indices()
        .filter_map(|(index, character)| {
            let is_valid_dot =
                character == '.' && index != 0 && index + character.len_utf8() < base.len();

            is_valid_dot.then_some(index)
        })
        .collect();

    dot_positions
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = dot_positions.get(index + 1).copied().unwrap_or(base.len());

            base[*start..end].to_string()
        })
        .collect()
}

fn starts_with_decimal_number(suffix: &str) -> bool {
    suffix.chars().nth(1).is_some_and(is_decimal_digit)
}

fn split_base_extensions(base: &str) -> (String, Vec<String>) {
    let candidates = suffix_candidates(base);
    let mut selected_reversed = Vec::new();

    for (index, suffix) in candidates.iter().rev().enumerate() {
        let suffix_too_long = suffix.chars().count() > 5;
        let too_many_suffixes = index > 1;
        let starts_with_number = starts_with_decimal_number(suffix);

        if suffix_too_long || too_many_suffixes || starts_with_number {
            break;
        }

        selected_reversed.push(suffix.clone());
    }

    selected_reversed.reverse();

    let suffix_byte_length: usize = selected_reversed.iter().map(String::len).sum();

    let stem_length = base.len() - suffix_byte_length;
    let stem = base[..stem_length].to_string();

    (stem, selected_reversed)
}

pub(crate) fn path_components(input: &str) -> Vec<String> {
    if is_dot_only_path(input) {
        return vec![".".to_string()];
    }

    let mut components = Vec::new();

    if let Some(first) = input.chars().next()
        && is_path_separator(first)
    {
        components.push(first.to_string());
    }

    let mut raw_components: Vec<&str> = input
        .split(is_path_separator)
        .filter(|component| !component.is_empty())
        .collect();

    let Some(base) = raw_components.pop() else {
        return components;
    };

    components.extend(raw_components.into_iter().map(str::to_string));

    let (stem, suffixes) = split_base_extensions(base);

    if !stem.is_empty() {
        components.push(stem);
    }

    components.extend(suffixes);
    components
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_regular_path_components() {
        assert_eq!(
            path_components("this/is/a/path"),
            vec!["this", "is", "a", "path"]
        );
    }

    #[test]
    fn splits_windows_path_components() {
        assert_eq!(
            path_components(r"this\is\a\path"),
            vec!["this", "is", "a", "path"]
        );
    }

    #[test]
    fn preserves_root_component() {
        assert_eq!(
            path_components("/this/is/a/path"),
            vec!["/", "this", "is", "a", "path"]
        );
    }

    #[test]
    fn normalizes_empty_path_to_dot() {
        assert_eq!(path_components(""), vec!["."]);
    }

    #[test]
    fn normalizes_dot_paths() {
        for input in [".", "./", "./././", ".\\"] {
            assert_eq!(path_components(input), vec!["."]);
        }
    }

    #[test]
    fn preserves_parent_components() {
        assert_eq!(
            path_components("../folder/file.txt"),
            vec!["..", "folder", "file", ".txt"]
        );
    }

    #[test]
    fn separates_single_extension() {
        assert_eq!(
            path_components("folder/file.x1.10.tar"),
            vec!["folder", "file.x1.10", ".tar"]
        );
    }

    #[test]
    fn separates_two_extensions() {
        assert_eq!(
            path_components("/this/is/a/path/file.x1.10.tar.gz"),
            vec!["/", "this", "is", "a", "path", "file.x1.10", ".tar", ".gz",]
        );
    }

    #[test]
    fn stops_before_numeric_suffix() {
        assert_eq!(path_components("file.123.txt"), vec!["file.123", ".txt"]);
    }

    #[test]
    fn leaves_long_extension_attached() {
        assert_eq!(
            path_components("file.longextension"),
            vec!["file.longextension"]
        );
    }

    #[test]
    fn separates_at_most_two_extensions() {
        assert_eq!(path_components("file.a.b.c"), vec!["file.a", ".b", ".c"]);
    }

    #[test]
    fn handles_hidden_file_extension() {
        assert_eq!(path_components(".hidden.txt"), vec![".hidden", ".txt"]);
    }

    #[test]
    fn does_not_treat_hidden_file_name_as_extension() {
        assert_eq!(path_components(".bashrc"), vec![".bashrc"]);
    }

    #[test]
    fn applies_extension_length_heuristic() {
        assert_eq!(
            path_components("Try.Me.Bug - 09 - One.Two.Three.[text].mkv"),
            vec!["Try.Me.Bug - 09 - One.Two.Three.[text]", ".mkv",]
        );
    }
}
