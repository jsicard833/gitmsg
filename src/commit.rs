use anyhow::{Result, bail};

pub fn clean_commit_message(raw_message: &str) -> Result<String> {
    let mut message = raw_message
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned();

    if let Some(stripped) = strip_code_fence(&message) {
        message = stripped;
    }

    message = message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned();

    if let Some(stripped) = message.strip_prefix("- ") {
        message = stripped.trim().to_owned();
    }

    if message.is_empty() {
        bail!("the model returned an empty commit message");
    }

    if message.contains('\n') {
        bail!("the model returned a multi-line commit message");
    }

    Ok(message)
}

fn strip_code_fence(message: &str) -> Option<String> {
    let trimmed = message.trim();
    if !trimmed.starts_with("```") {
        return None;
    }

    let mut lines = trimmed.lines();
    let first = lines.next()?;
    if !first.starts_with("```") {
        return None;
    }

    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            break;
        }
        body.push(line);
    }

    Some(body.join("\n").trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::clean_commit_message;

    #[test]
    fn trims_quotes_and_whitespace() {
        let message = clean_commit_message("  \"feat(parser): add support\"  ").unwrap();
        assert_eq!(message, "feat(parser): add support");
    }

    #[test]
    fn strips_code_fence() {
        let message = clean_commit_message("```\nfix(api): handle retries\n```").unwrap();
        assert_eq!(message, "fix(api): handle retries");
    }

    #[test]
    fn rejects_empty_message() {
        let error = clean_commit_message("   ").unwrap_err();
        assert!(error.to_string().contains("empty commit message"));
    }
}
