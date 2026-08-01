use crate::options::SortOptions;

/// Python natsort uses twenty maximum Unicode code points as the
/// separator for numeric-first keys when NUMAFTER is enabled.
pub(crate) const NUM_AFTER_SEPARATOR: &str = concat!(
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
    "\u{10FFFF}",
);

pub(crate) fn numeric_prefix(options: SortOptions) -> &'static str {
    if options.num_after {
        NUM_AFTER_SEPARATOR
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_numeric_prefix_is_empty() {
        assert_eq!(numeric_prefix(SortOptions::new()), "",);
    }

    #[test]
    fn num_after_prefix_has_twenty_characters() {
        let prefix = numeric_prefix(SortOptions::new().num_after(true));

        assert_eq!(prefix.chars().count(), 20);
    }

    #[test]
    fn num_after_prefix_uses_maximum_unicode_character() {
        let prefix = numeric_prefix(SortOptions::new().num_after(true));

        assert!(prefix.chars().all(|character| character == '\u{10FFFF}'));
    }
}
