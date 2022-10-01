use crate::error::Error;
use reqwest::Client;

pub async fn call(url: &String, client: &Client) -> Result<String, Error> {
    let result = client.get(url).send().await?.error_for_status()?;
    let text = result.text().await?;

    Ok(text)
}
