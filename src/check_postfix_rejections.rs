use reqwest::Client;
use serde::Deserialize;

use crate::{api_call::call, error::Error};

pub async fn check_postfix_rejections(base_url: &String, messages_to_retrieve: &usize, client: &Client) -> Result<(), Error> {
    let url = format!("{}/api/v1/get/logs/postfix/{}", base_url, messages_to_retrieve);
    let text = call(&url, client).await?;
    let log_messages: Vec<LogMessage> = serde_json::from_str(&text)?;

    check_rejections(log_messages)
}

fn check_rejections(log_messages: Vec<LogMessage>) -> Result<(), Error> {
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
    // More properties exist, but the script isn't using them.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use crate::check_postfix_rejections::{check_rejections, LogMessage};

    #[tokio::test]
    async fn test_ssl_rejections() {
        let log_message = LogMessage {
            message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
        };
        let result = check_rejections(vec![log_message]);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_ssl_rejections() {
        let log_message = LogMessage {
            message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
        };
        let result = check_rejections(vec![log_message]);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_messages_without_error() {
        let messages = vec![
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
            },
            LogMessage {
                message: "some random message".to_string(),
            },
        ];
        let result = check_rejections(messages);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_messages_with_error() {
        let messages = vec![
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
            },
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
            },
        ];
        let result = check_rejections(messages);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_messages_with_multiple_errors() {
        let messages = vec![
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.1.1 Authentication failed".to_string(),
            },
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
            },
            LogMessage {
                message: "NOQUEUE: reject: RCPT from smtp.example.com[1.2.3.4]: 550 5.7.1 Session encryption is required; from=<noreply@example.com> to=<me@mydomain.com> proto=ESMTP helo=<smtp.example.com>".to_string(),
            },
        ];
        let result = check_rejections(messages);
        assert!(result.is_err());
    }
}
