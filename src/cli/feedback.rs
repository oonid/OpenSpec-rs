use crate::core::error::{OpenSpecError, Result};
use std::process::Command;

/// GitHub repository that feedback issues are filed against.
const FEEDBACK_REPO: &str = "oonid/OpenSpec-rs";

/// Submit feedback as a GitHub issue, mirroring upstream `FeedbackCommand`:
/// format a title/body, file it via the `gh` CLI when available + authenticated,
/// otherwise fall back to printing the formatted feedback and a manual issue URL.
/// (This is an explicit user action — not telemetry — so it is NOT gated by the
/// telemetry opt-out.)
pub fn run_feedback(message: &str, body: Option<&str>) -> Result<()> {
    let title = format_title(message);
    let body = format_body(body);

    if !gh_installed() {
        return handle_fallback(&title, &body, Fallback::Missing);
    }
    if !gh_authenticated() {
        return handle_fallback(&title, &body, Fallback::Unauthenticated);
    }
    submit_via_gh(&title, &body)
}

fn format_title(message: &str) -> String {
    format!("Feedback: {}", message)
}

fn format_body(body_text: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(text) = body_text.filter(|t| !t.trim().is_empty()) {
        parts.push(text.to_string());
        parts.push(String::new()); // blank line before metadata
    }
    parts.push(generate_metadata());
    parts.join("\n")
}

fn generate_metadata() -> String {
    format!(
        "---\nSubmitted via OpenSpec CLI\n- Version: {}\n- Platform: {}\n- Timestamp: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        chrono::Utc::now().to_rfc3339()
    )
}

fn gh_installed() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn gh_authenticated() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn submit_via_gh(title: &str, body: &str) -> Result<()> {
    let output = Command::new("gh")
        .args([
            "issue",
            "create",
            "--repo",
            FEEDBACK_REPO,
            "--title",
            title,
            "--body",
            body,
            "--label",
            "feedback",
        ])
        .output()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to run gh: {}", e)))?;

    if output.status.success() {
        let issue_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("\n✓ Feedback submitted successfully!");
        println!("Issue URL: {}\n", issue_url);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(OpenSpecError::Custom(if stderr.is_empty() {
            "gh issue create failed".to_string()
        } else {
            stderr
        }))
    }
}

enum Fallback {
    Missing,
    Unauthenticated,
}

fn handle_fallback(title: &str, body: &str, reason: Fallback) -> Result<()> {
    match reason {
        Fallback::Missing => println!("⚠️  GitHub CLI not found. Manual submission required."),
        Fallback::Unauthenticated => {
            println!("⚠️  GitHub authentication required. Manual submission required.")
        }
    }

    println!("\n--- FORMATTED FEEDBACK ---");
    println!("Title: {}", title);
    println!("Labels: feedback");
    println!("\nBody:");
    println!("{}", body);
    println!("--- END FEEDBACK ---\n");

    println!("Please submit your feedback manually:");
    println!("{}", manual_submission_url(title, body));

    if matches!(reason, Fallback::Unauthenticated) {
        println!("\nTo auto-submit in the future: gh auth login");
    }
    Ok(())
}

fn manual_submission_url(title: &str, body: &str) -> String {
    format!(
        "https://github.com/{}/issues/new?title={}&body={}&labels={}",
        FEEDBACK_REPO,
        encode_uri_component(title),
        encode_uri_component(body),
        encode_uri_component("feedback"),
    )
}

/// Percent-encode like JavaScript's `encodeURIComponent`: leave unreserved
/// characters (A-Z a-z 0-9 - _ . ! ~ * ' ( )) as-is, encode everything else as
/// UTF-8 bytes.
fn encode_uri_component(input: &str) -> String {
    const UNRESERVED: &[u8] = b"-_.!~*'()";
    let mut out = String::with_capacity(input.len());
    for &byte in input.as_bytes() {
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{:02X}", byte));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_and_body_formatting() {
        assert_eq!(format_title("hello"), "Feedback: hello");

        let with_body = format_body(Some("details here"));
        assert!(with_body.starts_with("details here\n"));
        assert!(with_body.contains("Submitted via OpenSpec CLI"));
        assert!(with_body.contains("- Version:"));

        // No body text → just the metadata footer (no leading blank line).
        let no_body = format_body(None);
        assert!(no_body.starts_with("---"));
        assert!(no_body.contains("Submitted via OpenSpec CLI"));
    }

    #[test]
    fn manual_url_is_percent_encoded_against_repo() {
        let url = manual_submission_url("Feedback: a b/c", "x&y");
        assert!(url.starts_with("https://github.com/oonid/OpenSpec-rs/issues/new?"));
        assert!(url.contains("title=Feedback%3A%20a%20b%2Fc"));
        assert!(url.contains("body=x%26y"));
        assert!(url.contains("labels=feedback"));
    }

    #[test]
    fn encode_uri_component_matches_js_semantics() {
        assert_eq!(encode_uri_component("a b"), "a%20b");
        assert_eq!(encode_uri_component("a/b?c=d&e"), "a%2Fb%3Fc%3Dd%26e");
        assert_eq!(encode_uri_component("keep-_.!~*'()"), "keep-_.!~*'()");
    }
}
