#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SortOptions {
    pub ignore_case: bool,
    pub reverse: bool,
    pub signed: bool,
    pub float: bool,
    pub no_exp: bool,
    pub path: bool,
    pub presort: bool,
}

impl SortOptions {
    pub const fn new() -> Self {
        Self {
            ignore_case: false,
            reverse: false,
            signed: false,
            float: false,
            no_exp: false,
            path: false,
            presort: false,
        }
    }

    pub const fn ignore_case(mut self, enabled: bool) -> Self {
        self.ignore_case = enabled;
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
}
