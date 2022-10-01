use reqwest::Client;

use crate::{api_call::call, error::Error};

pub async fn check_queue(base_url: &String, client: &Client) -> Result<(), Error> {
    let url = format!("{}/api/v1/get/mailq/all", base_url);
    let text = call(&url, client).await?;

    if text != "[]" {
        let message = format!("Outbound message queue is not empty: {}", text);
        return Err(Error::from(message));
    }

    Ok(())
}
