use crate::algorithm::AlgorithmFlags;
use crate::locale::LocaleProfile;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SortOptions {
    pub ignore_case: bool,
    pub lowercase_first: bool,
    pub group_letters: bool,
    pub capital_first: bool,
    pub compatibility_normalize: bool,
    pub reverse: bool,
    pub signed: bool,
    pub float: bool,
    pub no_exp: bool,
    pub path: bool,
    pub presort: bool,
    pub nan_last: bool,
    pub num_after: bool,
    pub locale_alpha: bool,
    pub locale_numeric: bool,
    pub locale_profile: LocaleProfile,
}

impl SortOptions {
    pub const fn new() -> Self {
        Self {
            ignore_case: false,
            lowercase_first: false,
            group_letters: false,
            capital_first: false,
            compatibility_normalize: false,
            reverse: false,
            signed: false,
            float: false,
            no_exp: false,
            path: false,
            presort: false,
            nan_last: false,
            num_after: false,
            locale_alpha: false,
            locale_numeric: false,
            locale_profile: LocaleProfile::System,
        }
    }

    /// Build sorting options from Python-compatible `natsort.ns` flags.
    pub const fn from_algorithm(algorithm: AlgorithmFlags) -> Self {
        Self::new()
            .float(algorithm.intersects(AlgorithmFlags::FLOAT))
            .signed(algorithm.intersects(AlgorithmFlags::SIGNED))
            .no_exp(algorithm.intersects(AlgorithmFlags::NOEXP))
            .path(algorithm.intersects(AlgorithmFlags::PATH))
            .locale_alpha(algorithm.intersects(AlgorithmFlags::LOCALEALPHA))
            .locale_numeric(algorithm.intersects(AlgorithmFlags::LOCALENUM))
            .ignore_case(algorithm.intersects(AlgorithmFlags::IGNORECASE))
            .lowercase_first(algorithm.intersects(AlgorithmFlags::LOWERCASEFIRST))
            .group_letters(algorithm.intersects(AlgorithmFlags::GROUPLETTERS))
            .capital_first(
                algorithm.intersects(AlgorithmFlags::UNGROUPLETTERS)
                    && algorithm.intersects(AlgorithmFlags::LOCALEALPHA),
            )
            .nan_last(algorithm.intersects(AlgorithmFlags::NANLAST))
            .compatibility_normalize(algorithm.intersects(AlgorithmFlags::COMPATIBILITYNORMALIZE))
            .num_after(algorithm.intersects(AlgorithmFlags::NUMAFTER))
            .presort(algorithm.intersects(AlgorithmFlags::PRESORT))
    }

    /// Build sorting options from arbitrary Python-style integer flag bits.
    pub const fn from_algorithm_bits(bits: i64) -> Self {
        Self::from_algorithm(AlgorithmFlags::from_bits(bits))
    }

    /// Convert these options back into Python-compatible algorithm flags.
    ///
    /// `reverse` and `locale_profile` are not represented by Python's `ns`
    /// bit mask and are intentionally omitted.
    pub const fn algorithm(self) -> AlgorithmFlags {
        let mut bits = 0;

        if self.float {
            bits |= AlgorithmFlags::FLOAT.bits();
        }
        if self.signed {
            bits |= AlgorithmFlags::SIGNED.bits();
        }
        if self.no_exp {
            bits |= AlgorithmFlags::NOEXP.bits();
        }
        if self.path {
            bits |= AlgorithmFlags::PATH.bits();
        }
        if self.locale_alpha {
            bits |= AlgorithmFlags::LOCALEALPHA.bits();
        }
        if self.locale_numeric {
            bits |= AlgorithmFlags::LOCALENUM.bits();
        }
        if self.ignore_case {
            bits |= AlgorithmFlags::IGNORECASE.bits();
        }
        if self.lowercase_first {
            bits |= AlgorithmFlags::LOWERCASEFIRST.bits();
        }
        if self.group_letters {
            bits |= AlgorithmFlags::GROUPLETTERS.bits();
        }
        if self.capital_first {
            bits |= AlgorithmFlags::UNGROUPLETTERS.bits();
        }
        if self.nan_last {
            bits |= AlgorithmFlags::NANLAST.bits();
        }
        if self.compatibility_normalize {
            bits |= AlgorithmFlags::COMPATIBILITYNORMALIZE.bits();
        }
        if self.num_after {
            bits |= AlgorithmFlags::NUMAFTER.bits();
        }
        if self.presort {
            bits |= AlgorithmFlags::PRESORT.bits();
        }

        AlgorithmFlags::from_bits(bits)
    }

    pub const fn num_after(mut self, enabled: bool) -> Self {
        self.num_after = enabled;
        self
    }

    pub const fn nan_last(mut self, enabled: bool) -> Self {
        self.nan_last = enabled;
        self
    }

    pub const fn ignore_case(mut self, enabled: bool) -> Self {
        self.ignore_case = enabled;
        self
    }

    pub const fn lowercase_first(mut self, enabled: bool) -> Self {
        self.lowercase_first = enabled;
        self
    }

    pub const fn group_letters(mut self, enabled: bool) -> Self {
        self.group_letters = enabled;
        self
    }

    pub const fn capital_first(mut self, enabled: bool) -> Self {
        self.capital_first = enabled;
        self
    }

    pub const fn compatibility_normalize(mut self, enabled: bool) -> Self {
        self.compatibility_normalize = enabled;
        self
    }

    pub const fn reverse(mut self, enabled: bool) -> Self {
        self.reverse = enabled;
        self
    }

    pub const fn signed(mut self, enabled: bool) -> Self {
        self.signed = enabled;
        self
    }

    pub const fn float(mut self, enabled: bool) -> Self {
        self.float = enabled;
        self
    }

    pub const fn no_exp(mut self, enabled: bool) -> Self {
        self.no_exp = enabled;
        self
    }

    pub const fn path(mut self, enabled: bool) -> Self {
        self.path = enabled;
        self
    }

    pub const fn presort(mut self, enabled: bool) -> Self {
        self.presort = enabled;
        self
    }

    pub const fn locale_alpha(mut self, enabled: bool) -> Self {
        self.locale_alpha = enabled;
        self
    }

    pub const fn locale_numeric(mut self, enabled: bool) -> Self {
        self.locale_numeric = enabled;
        self
    }

    pub const fn locale(mut self, enabled: bool) -> Self {
        self.locale_alpha = enabled;
        self.locale_numeric = enabled;
        self
    }

    pub const fn locale_profile(mut self, profile: LocaleProfile) -> Self {
        self.locale_profile = profile;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_options_from_python_flags() {
        let algorithm = AlgorithmFlags::REAL
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

        let options = SortOptions::from_algorithm(algorithm);

        assert!(options.float);
        assert!(options.signed);
        assert!(!options.no_exp);
        assert!(options.path);
        assert!(options.locale_alpha);
        assert!(options.locale_numeric);
        assert!(options.ignore_case);
        assert!(options.lowercase_first);
        assert!(options.group_letters);
        assert!(options.capital_first);
        assert!(options.nan_last);
        assert!(options.compatibility_normalize);
        assert!(options.num_after);
        assert!(options.presort);
        assert!(!options.reverse);
    }

    #[test]
    fn round_trips_all_known_algorithm_bits() {
        let algorithm = AlgorithmFlags::from_bits(AlgorithmFlags::KNOWN_MASK);

        assert_eq!(
            SortOptions::from_algorithm(algorithm).algorithm(),
            algorithm
        );
    }

    #[test]
    fn ignores_unknown_algorithm_bits() {
        let algorithm = AlgorithmFlags::from_bits(AlgorithmFlags::KNOWN_MASK | (1_i64 << 40));

        assert_eq!(
            SortOptions::from_algorithm(algorithm).algorithm().bits(),
            AlgorithmFlags::KNOWN_MASK
        );
    }

    #[test]
    fn maps_real_and_locale_aliases() {
        let options = SortOptions::from_algorithm(AlgorithmFlags::REAL | AlgorithmFlags::LOCALE);

        assert!(options.float);
        assert!(options.signed);
        assert!(options.locale_alpha);
        assert!(options.locale_numeric);
        assert_eq!(
            options.algorithm(),
            AlgorithmFlags::REAL | AlgorithmFlags::LOCALE
        );
    }

    #[test]
    fn raw_negative_bits_enable_every_known_option() {
        assert_eq!(
            SortOptions::from_algorithm_bits(-1).algorithm().bits(),
            AlgorithmFlags::KNOWN_MASK
        );
    }

    #[test]
    fn reverse_and_locale_profile_are_not_algorithm_flags() {
        let options = SortOptions::new()
            .reverse(true)
            .locale_profile(LocaleProfile::GermanGermany);

        assert_eq!(options.algorithm(), AlgorithmFlags::DEFAULT);
    }
}
