use chrono::DateTime;
use reqwest::Client;
use serde::Deserialize;

use crate::{api_call::call, error::Error};

pub async fn check_postfix_rejections(
    base_url: &String,
    messages_to_retrieve: &usize,
    verbose: bool,
    client: &Client,
) -> Result<(), Error> {
    let url = format!(
        "{}/api/v1/get/logs/postfix/{}",
        base_url, messages_to_retrieve
    );
    let text = call(&url, client).await?;
    let log_messages: Vec<LogMessage> = serde_json::from_str(&text)?;

    check_rejections(log_messages, verbose)
}

fn check_rejections(log_messages: Vec<LogMessage>, verbose: bool) -> Result<(), Error> {
    if verbose {
        match log_messages.last() {
            Some(last_message) => {
                let last_message_time: i64 = last_message.time.parse().unwrap_or(0);
                let last_message_time =
                    DateTime::from_timestamp(last_message_time, 0).unwrap_or_default();
                let current_time = chrono::Utc::now();
                let time_delta = current_time - last_message_time;

                println!(
                    "DEBUG: check_postfix_rejections: Last message: \"{}\"",
                    last_message.message
                );
                println!(
                    "DEBUG: check_postfix_rejections: Last message time: {} ({} minute(s) ago)",
                    last_message_time,
                    time_delta.num_minutes()
                );
            }
            None => println!("DEBUG: check_postfix_rejections: log is empty."),
        }
    }

    let ssl_rejections: Vec<LogMessage> = log_messages
        .into_iter()
        .filter(|l| {
            l.message
                .contains("550 5.7.1 Session encryption is required")
        })
        .collect();

    match ssl_rejections.len() {
        0 => Ok(()),
        _ => {
            let ssl_rejections: Vec<String> =
                ssl_rejections.into_iter().map(|s| s.message).collect();
            let ssl_rejections = ssl_rejections.join("\n");

            Err(Error::from(format!(
                "SSL rejections were found:\n{}",
                ssl_rejections
            )))
        }
    }
}

#[derive(Deserialize)]
struct LogMessage {
    // More properties exist, but the script isn't using them. https://demo.mailcow.email/api/
    pub message: String,
    pub time: String,
}

#[cfg(test)]
mod tests {
    use crate::check_postfix_rejections::{check_rejections, LogMessage};

    #[tokio::test]
    async fn test_ssl_rejections() {
        let log_message = LogMessage {
            message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
            time: "0".to_string()
        };
        let result = check_rejections(vec![log_message], false);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_ssl_rejections() {
        let log_message = LogMessage {
            message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
            time: "0".to_string()
        };
        let result = check_rejections(vec![log_message], false);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_messages_without_error() {
        let messages = vec![
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
                time: "0".to_string()
            },
            LogMessage {
                message: "some random message".to_string(),
                time: "0".to_string()
            },
        ];
        let result = check_rejections(messages, false);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_messages_with_error() {
        let messages = vec![
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
                time: "0".to_string()
            },
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
                time: "0".to_string()
            },
        ];
        let result = check_rejections(messages, false);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_messages_with_multiple_errors() {
        let messages = vec![
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
                time: "0".to_string()
            },
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
                time: "0".to_string()
            },
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
                time: "0".to_string()
            },
        ];
        let result = check_rejections(messages, false);
        assert!(result.is_err());
    }
}
