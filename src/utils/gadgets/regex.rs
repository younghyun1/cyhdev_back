use regex::Regex;

pub const EMAIL_VALIDATION_REGEX: &str = r"^[a-z0-9!#$%&'*+/=?^_`{|}~-]+(\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@[a-z0-9]+([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]+([a-z0-9-]*[a-z0-9])?)+$";

pub fn compile_regex(regex: &'static str) -> anyhow::Result<Regex> {
    Regex::new(regex).map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation_regex() {
        let regex = compile_regex(EMAIL_VALIDATION_REGEX).expect("Failed to compile regex");

        let valid_emails = vec![
            "test@example.com",
            "user.name+tag+sorting@example.com",
            "x@x.au",
            "example-indeed@strange-example.com",
        ];

        for email in valid_emails {
            assert!(regex.is_match(email), "Should match: {}", email);
        }

        let invalid_emails = vec![
            "plainaddress",
            "@example.com",
            "Joe Smith <email@example.com>",
            "email.example.com",
            "email@example@example.com",
        ];

        for email in invalid_emails {
            assert!(!regex.is_match(email), "Should not match: {}", email);
        }
    }
}
