use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;

use crate::locale::normalize_localized_numbers;
use crate::options::SortOptions;

fn normalize_input(input: &str, compatibility_normalize: bool) -> String {
    if compatibility_normalize {
        input.nfkd().collect()
    } else {
        input.nfd().collect()
    }
}

fn compose_input(input: &str, compatibility_normalize: bool) -> String {
    if compatibility_normalize {
        input.nfkc().collect()
    } else {
        input.nfc().collect()
    }
}

fn swap_case(input: &str) -> String {
    let mut output = String::new();

    for character in input.chars() {
        if character.is_lowercase() {
            output.extend(character.to_uppercase());
        } else if character.is_uppercase() {
            output.extend(character.to_lowercase());
        } else {
            output.push(character);
        }
    }

    output
}

fn case_fold(input: &str) -> String {
    input.case_fold().collect()
}

fn group_letters(input: &str) -> String {
    let mut output = String::new();

    for character in input.chars() {
        output.extend(character.case_fold());
        output.push(character);
    }

    output
}

pub(crate) fn prepare_input(input: &str, options: SortOptions) -> String {
    let normalized = normalize_input(input, options.compatibility_normalize);
    let dumb_locale = options.locale_alpha && options.locale_profile.uses_dumb_collation();
    let should_swap_case = options.lowercase_first ^ dumb_locale;

    let case_ordered = if should_swap_case {
        swap_case(&normalized)
    } else {
        normalized
    };

    let case_transformed = if options.ignore_case {
        case_fold(&case_ordered)
    } else {
        case_ordered
    };

    let number_transformed = if options.locale_numeric {
        normalize_localized_numbers(&case_transformed, options.locale_profile, options.float)
    } else {
        case_transformed
    };

    if options.locale_alpha {
        compose_input(&number_transformed, options.compatibility_normalize)
    } else {
        number_transformed
    }
}

pub(crate) fn transform_text_component(input: &str, options: SortOptions) -> String {
    let dumb_locale = options.locale_alpha && options.locale_profile.uses_dumb_collation();

    if options.group_letters || dumb_locale {
        group_letters(input)
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::LocaleProfile;

    #[test]
    fn applies_canonical_decomposition() {
        assert_eq!(normalize_input("café", false), "cafe\u{301}",);
    }

    #[test]
    fn applies_canonical_ring_decomposition() {
        assert_eq!(normalize_input("Å", false), "A\u{30A}",);
    }

    #[test]
    fn compatibility_normalizes_ligature() {
        assert_eq!(normalize_input("ﬀile", true), "ffile",);
    }

    #[test]
    fn compatibility_normalizes_fullwidth_letter() {
        assert_eq!(normalize_input("Ａ", true), "A",);
    }

    #[test]
    fn compatibility_normalizes_circled_letter() {
        assert_eq!(normalize_input("Ⓐ", true), "A",);
    }

    #[test]
    fn compatibility_normalizes_superscript_number() {
        assert_eq!(normalize_input("²", true), "2",);
    }

    #[test]
    fn compatibility_normalizes_circled_number() {
        assert_eq!(normalize_input("①", true), "1",);
    }

    #[test]
    fn swaps_uppercase_and_lowercase_characters() {
        assert_eq!(swap_case("Apple"), "aPPLE");
        assert_eq!(swap_case("apple"), "APPLE");
    }

    #[test]
    fn swap_case_supports_character_expansion() {
        assert_eq!(swap_case("Straße"), "sTRASSE");
    }

    #[test]
    fn folds_sharp_s() {
        assert_eq!(case_fold("Straße"), "strasse");
    }

    #[test]
    fn folds_greek_sigma_variants() {
        assert_eq!(case_fold("Σςσ"), "σσσ");
    }

    #[test]
    fn folds_kelvin_sign() {
        assert_eq!(case_fold("K"), "k");
    }

    #[test]
    fn groups_letters_by_folded_and_original_values() {
        assert_eq!(group_letters("Apple"), "aAppppllee",);
    }

    #[test]
    fn prepares_lowercase_first_input() {
        let options = SortOptions::new().lowercase_first(true);

        assert_eq!(prepare_input("Apple", options), "aPPLE",);
    }

    #[test]
    fn prepares_case_folded_input() {
        let options = SortOptions::new().ignore_case(true);

        assert_eq!(prepare_input("Straße", options), "strasse",);
    }

    #[test]
    fn c_locale_applies_python_dumb_locale_swap() {
        let options = SortOptions::new()
            .locale_alpha(true)
            .locale_profile(LocaleProfile::C);

        assert_eq!(prepare_input("Apple", options), "aPPLE");
    }

    #[test]
    fn c_locale_groups_letters_like_python() {
        let options = SortOptions::new()
            .locale_alpha(true)
            .locale_profile(LocaleProfile::C);

        let prepared = prepare_input("Apple", options);

        assert_eq!(transform_text_component(&prepared, options), "aapPpPlLeE");
    }

    #[test]
    fn locale_alpha_recomposes_unicode_before_collation() {
        let options = SortOptions::new()
            .locale_alpha(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(prepare_input("Äpfel", options), "Äpfel");
    }

    #[test]
    fn english_locale_removes_thousands_separator() {
        let options = SortOptions::new()
            .locale_numeric(true)
            .float(true)
            .locale_profile(LocaleProfile::EnglishUnitedStates);

        assert_eq!(prepare_input("1,234.50", options), "1234.50");
    }

    #[test]
    fn german_locale_switches_decimal_separator() {
        let options = SortOptions::new()
            .locale_numeric(true)
            .float(true)
            .locale_profile(LocaleProfile::GermanGermany);

        assert_eq!(prepare_input("1.234,50", options), "1234.50");
    }
}
