use unicode_segmentation::UnicodeSegmentation;

/// Captures all of the information we need to register a new subscriber.
pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

/// The name of a subscriber. Enforces invariants of a valid subscriber name, so
/// if you have an instance of this, the name is guaranteed to be valid.
///
/// # Examples
/// Use the `parse` function to build a `SubscriberName` from a string.
/// We can then get the name back out using the `AsRef<str>` implementation.
/// ```
/// use zero2prod::domain::SubscriberName;
///
/// let name = SubscriberName::parse("A valid name".to_string()).unwrap();
/// assert_eq!("A valid name", name.as_ref());
/// ```
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    /// Returns `Ok` with a `SubscriberName` if the name is valid, otherwise returns
    /// `Err` with an error message.
    ///
    /// A name is invalid if:
    /// * It is all whitespace (or empty)
    /// * It has more than 256 characters
    /// * Contains any of `/`, `(`, `)`, `"`, `<`, `>`, `\`, `{`, or `}`
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        // graphemes are the visible characters in a unicode string
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = "  \t".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for c in ['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = c.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
