use std::fmt::Display;

use chrono::DateTime;
use reqwest::Client;
use serde::Deserialize;

use crate::{api_call::call, error::Error};

pub async fn check_queue(base_url: &String, client: &Client) -> Result<(), Error> {
    let url = format!("{}/api/v1/get/mailq/all", base_url);
    let text = call(&url, client).await?;
    let queue_items: Vec<QueueItem> = serde_json::from_str(&text)?;

    check_queue_items(queue_items)?;

    Ok(())
}

fn check_queue_items(queue_items: Vec<QueueItem>) -> Result<(), Error> {
    if queue_items.len() == 0 {
        return Ok(());
    }

    let mut message = "Outbound message queue is not empty:\n\n".to_string();
    for queue_item in queue_items {
        message.push_str(&format!("{}\n", queue_item));
    }

    return Err(Error::from(message));
}

#[derive(Deserialize)]
struct QueueItem {
    pub queue_name: String,
    pub queue_id: String,
    pub arrival_time: i64,
    pub message_size: usize,
    pub forced_expire: bool,
    pub sender: String,
    pub recipients: Vec<String>,
}

impl Display for QueueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let receive_time = DateTime::from_timestamp(self.arrival_time, 0);
        let receive_time = match receive_time {
            Some(time) => time.format("%m/%d/%Y %l:%M:%S%p").to_string(),
            None => self.arrival_time.to_string(),
        };

        write!(
            f,
            "ID: {}\n├─Sender: {}\n├─Arrival time: {}\n├─Message size: {}\n├─Queue type: {}\n├─Forced expire? {}\n├─Message:\n",
            self.queue_id,
            self.sender,
            receive_time,
            self.message_size,
            self.queue_name,
            self.forced_expire
        )?;

        for recipient in &self.recipients {
            let recipient_lines = recipient.split("  ");
            for line in recipient_lines {
                line.split_inclusive("said: ")
                    .for_each(|final_split_line| write!(f, "├────{}\n", final_split_line).unwrap());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_queue_items_returns_ok_when_queue_items_is_empty() {
        let queue_items: Vec<super::QueueItem> = vec![];
        let result = super::check_queue_items(queue_items);
        assert!(result.is_ok());
    }

    #[test]
    fn check_queue_items_returns_err_when_queue_items_is_not_empty() {
        let queue_items: Vec<super::QueueItem> = vec![super::QueueItem {
            queue_name: "test".to_string(),
            queue_id: "test".to_string(),
            arrival_time: 0,
            message_size: 0,
            forced_expire: false,
            sender: "test".to_string(),
            recipients: vec!["test".to_string()],
        }];
        let result = super::check_queue_items(queue_items);
        assert!(result.is_err());
    }

    #[test]
    fn check_queue_items_returns_expected_output_when_queue_items_is_not_empty() {
        let queue_items: Vec<super::QueueItem> = vec![
            super::QueueItem {
                queue_name: "queue name1".to_string(),
                queue_id: "queue ID1".to_string(),
                arrival_time: 1,
                message_size: 2,
                forced_expire: false,
                sender: "me".to_string(),
                recipients: vec!["test".to_string(), "test2".to_string()],
            },
            super::QueueItem {
                queue_name: "queue name2".to_string(),
                queue_id: "queue ID2".to_string(),
                arrival_time: 1234235236,
                message_size: 5,
                forced_expire: true,
                sender: "someone".to_string(),
                recipients: vec!["test3".to_string(), "test4".to_string()],
            },
        ];
        let result = super::check_queue_items(queue_items);
        insta::assert_snapshot!(result.unwrap_err().to_string());
    }
}
