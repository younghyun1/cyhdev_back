use regex::Regex;

pub const EMAIL_VALIDATION_REGEX: &str = r"^[a-z0-9!#$%&'*+/=?^_`{|}~-]+(\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@[a-z0-9]+([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]+([a-z0-9-]*[a-z0-9])?)+$";

pub fn compile_regex(regex: &'static str) -> anyhow::Result<Regex> {
    Regex::new(regex).map_err(anyhow::Error::from)
}

pub fn pw_regex_custom(pw: &str) -> bool {
    if pw.len() < 8 {
        return false;
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in pw.chars() {
        if c.is_uppercase() {
            has_upper = true;
        } else if c.is_lowercase() {
            has_lower = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if matches!(c, '@' | '$' | '!' | '%' | '*' | '?' | '&' | '#') {
            has_special = true;
        }

        if has_upper && has_lower && has_digit && has_special {
            return true;
        }
    }

    false
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
