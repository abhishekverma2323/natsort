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
