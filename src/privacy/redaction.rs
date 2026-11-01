//! Fail-closed detector for common secrets in optional text capture/export.

const SECRET_MARKER: &str = "[REDACTED]";

pub fn contains_secret(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if [
        "-----begin private key-----",
        "-----begin rsa private key-----",
        "-----begin ec private key-----",
        "-----begin open ssh private key-----",
        "-----begin openssh private key-----",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return true;
    }
    if [
        "ghp_",
        "github_pat_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xoxr-",
        "sk-",
        "akia",
    ]
    .iter()
    .any(|prefix| contains_token_prefix(&lower, prefix))
    {
        return true;
    }
    for line in text.lines() {
        let lower_line = line.to_ascii_lowercase();
        if has_credential_assignment(&lower_line) || has_sensitive_argv(&lower_line) {
            return true;
        }
    }
    false
}

pub fn redact_secrets(text: &str) -> String {
    let mut out = Vec::new();
    let mut in_pem = false;
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("-----begin") && lower.contains("private key-----") {
            in_pem = true;
            out.push(SECRET_MARKER.to_owned());
            continue;
        }
        if in_pem {
            if lower.contains("-----end") && lower.contains("private key-----") {
                in_pem = false;
            }
            continue;
        }
        if has_credential_assignment(&lower) || has_sensitive_argv(&lower) || token_line(&lower) {
            out.push(redact_value(line));
        } else {
            out.push(line.to_owned());
        }
    }
    if in_pem {
        out.push(SECRET_MARKER.to_owned());
    }
    out.join("\n")
}

fn contains_token_prefix(text: &str, prefix: &str) -> bool {
    text.match_indices(prefix).any(|(at, _)| {
        let suffix = &text[at + prefix.len()..];
        suffix
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .count()
            >= 16
    })
}

fn has_credential_assignment(line: &str) -> bool {
    [
        "password",
        "passwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "private_key",
        "client_secret",
    ]
    .iter()
    .any(|key| {
        line.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .any(|part| part == *key)
            && (line.contains('=') || line.contains(':'))
    })
}

fn has_sensitive_argv(line: &str) -> bool {
    [
        "--password",
        "--token",
        "--secret",
        "--api-key",
        "--apikey",
        "--client-secret",
    ]
    .iter()
    .any(|flag| line.contains(flag))
}

fn token_line(line: &str) -> bool {
    [
        "ghp_",
        "github_pat_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xoxr-",
        "sk-",
        "akia",
    ]
    .iter()
    .any(|prefix| contains_token_prefix(line, prefix))
}

fn redact_value(line: &str) -> String {
    if let Some(i) = line.find(['=', ':']) {
        format!("{}{}", &line[..=i], SECRET_MARKER)
    } else if let Some(i) = line.find("--") {
        let flag_end = line[i..]
            .find(char::is_whitespace)
            .map(|n| i + n)
            .unwrap_or(line.len());
        format!("{} {}", &line[..flag_end], SECRET_MARKER)
    } else {
        SECRET_MARKER.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_detector_does_not_flag_unrelated_words() {
        assert!(!contains_secret("the token bucket is full"));
        assert!(contains_secret("API_KEY=abc"));
    }
}
