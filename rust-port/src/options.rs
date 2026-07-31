#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SortOptions {
    pub ignore_case: bool,
    pub reverse: bool,
    pub signed: bool,
    pub float: bool,
}

impl SortOptions {
    pub const fn new() -> Self {
        Self {
            ignore_case: false,
            reverse: false,
            signed: false,
            float: false,
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
}
