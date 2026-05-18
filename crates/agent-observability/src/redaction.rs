#[cfg(test)]
mod tests;

pub const REDACTED: &str = "[REDACTED]";

pub fn should_redact_key(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    key.contains("TOKEN")
        || key.contains("SECRET")
        || key.contains("PASSWORD")
        || key.ends_with("_KEY")
        || key == "AUTHORIZATION"
}

pub fn redact_value(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else {
        REDACTED.to_string()
    }
}

pub fn redact_key_value(key: &str, value: &str) -> String {
    if should_redact_key(key) {
        redact_value(value)
    } else {
        value.to_string()
    }
}
