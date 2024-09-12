use reqwest::Client;
use serde::Deserialize;

use crate::{api_call::call, error::Error};

pub async fn check_quota(threshold: u16, base_url: &String, client: &Client) -> Result<(), Error> {
    let url = format!("{}/api/v1/get/mailbox/all", base_url);
    let text = call(&url, client).await?;

    let mailboxes: Vec<Mailbox> = match serde_json::from_str(text.as_str()) {
        Ok(result) => result,
        Err(e) => return Err(Error::from(e.to_string())),
    };

    let mut error_message = String::new();
    for mailbox in mailboxes {
        if mailbox.percent_in_use > threshold {
            let message = format!("{}, which is above the {}% threshold", mailbox, threshold);
            error_message.push_str(message.as_str());
        }
    }

    match error_message.is_empty() {
        true => Ok(()),
        false => Err(Error::from(error_message)),
    }
}

#[derive(Deserialize)]
struct Mailbox {
    pub username: String,
    pub percent_in_use: u16,
}

impl std::fmt::Display for Mailbox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Mailbox {} is {}% utilized",
            self.username, self.percent_in_use
        )
    }
}
