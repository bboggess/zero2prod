use validator::validate_email;

/// An email address to send the newsletter to. Enforces validity of the email
/// address, so any instance of this is guaranteed to have a valid email address.
///
/// # Examples
/// Use the `parse` function to build a `SubscriberEmail` from a string.
/// We can then get the email address back out using the `AsRef<str>` implementation.
/// ```
/// use zero2prod::domain::SubscriberEmail;
///
/// let name = SubscriberEmail::parse("valid@domain.com".to_string()).unwrap();
/// assert_eq!("valid@domain.com", name.as_ref());
/// ```
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    /// Return `Ok` with a valid `SubscriberEmail` when `s` is a valid email address.
    /// Otherwise, returns `Err` with an error message describing the problem.
    pub fn parse(s: String) -> Result<Self, String> {
        if validate_email(&s) {
            Ok(SubscriberEmail(s))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::{assert_err, assert_ok};

    #[test]
    fn basic_valid_email_is_accepted() {
        let email = "valid@domain.com".to_string();
        assert_ok!(SubscriberEmail::parse(email));
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
