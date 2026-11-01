//! Fail-closed detector for common secrets in optional text capture/export.

const SECRET_MARKER: &str = "[REDACTED]";

pub fn contains_secret(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    // Any PEM private-key header, including encrypted PKCS#8 and DSA.
    if lower
        .lines()
        .any(|l| l.contains("-----begin") && l.contains("private key-----"))
    {
        return true;
    }
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
        if has_credential_assignment(&lower_line)
            || has_sensitive_argv(&lower_line)
            || has_auth_header(&lower_line)
        {
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
        let keyed = has_credential_assignment(&lower)
            || has_sensitive_argv(&lower)
            || has_auth_header(&lower);
        if keyed {
            out.push(redact_value(line));
        } else if token_line(&lower) {
            out.push(redact_tokens(line));
        } else {
            out.push(line.to_owned());
        }
    }
    if in_pem {
        out.push(SECRET_MARKER.to_owned());
    }
    out.join("\n")
}

/// A known prefix at a word boundary followed by a long run of token characters.
/// Hyphens and underscores count, so `sk-proj-...`, `sk-ant-api03-...` and
/// `xoxb-1234-5678-...` are caught; the boundary keeps ordinary words such as
/// `risk-assessment-...` from matching `sk-`.
fn contains_token_prefix(text: &str, prefix: &str) -> bool {
    text.match_indices(prefix).any(|(at, _)| {
        let at_boundary = text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_');
        let suffix = &text[at + prefix.len()..];
        at_boundary
            && suffix
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
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

/// `Authorization: Bearer ...` and `Basic ...` headers.
fn has_auth_header(line: &str) -> bool {
    line.contains("authorization")
        && (line.contains(':') || line.contains('='))
        && (line.contains("bearer ") || line.contains("basic "))
}

/// Replace only the token-looking words, so text after a token is kept and a
/// later `:` or `=` cannot leave the token in place.
fn redact_tokens(line: &str) -> String {
    line.split(' ')
        .map(|word| {
            if token_line(&word.to_ascii_lowercase()) {
                SECRET_MARKER
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn authorization_headers_and_encrypted_keys_are_detected() {
        assert!(contains_secret("Authorization: Bearer abc123"));
        assert!(contains_secret("authorization = basic dXNlcjpwYXNz"));
        assert!(contains_secret("-----BEGIN ENCRYPTED PRIVATE KEY-----"));
        assert!(contains_secret("-----BEGIN DSA PRIVATE KEY-----"));
        assert!(!contains_secret("the authorization flow is documented"));
    }

    #[test]
    fn a_token_is_redacted_even_when_a_colon_follows_it() {
        let out = redact_secrets("ghp_0123456789abcdef0123456789abcdef01234567 see https://example.com");
        assert!(!out.contains("ghp_0123"), "{out}");
        assert!(out.contains("see"), "{out}");
    }
}

#[cfg(test)]
mod hyphenated_token_tests {
    use super::*;

    #[test]
    fn hyphenated_modern_keys_are_detected() {
        assert!(contains_secret("sk-proj-AbCdEfGhIjKlMnOpQrStUvWxYz0123456789"));
        assert!(contains_secret("use sk-ant-api03-AbCdEfGhIjKlMnOpQrStUvWxYz012345 here"));
    }

    #[test]
    fn ordinary_hyphenated_words_are_not_flagged() {
        assert!(!contains_secret("the risk-assessment-for-the-new-service-rollout-plan is ready"));
        assert!(!contains_secret("task-force-for-the-quarterly-review-of-open-items"));
    }
}
