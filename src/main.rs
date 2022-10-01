use check_mailcow::{check_queue::check_queue, check_quota::check_quota, error::Error};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue};

#[derive(Parser, Debug)]
#[clap(
    author = "Clete Blackwell II",
    version = "1.0.0",
    about = "Run various checks on Mailcow",
    long_about = "Run various checks on self-hosted Mailcow-Dockerized installations (https://github.com/mailcow/mailcow-dockerized)"
)]
struct Args {
    #[clap(
        short = 'a',
        long,
        value_parser,
        default_value_t = true,
        help = "(default) Run all checks"
    )]
    all: bool,

    #[clap(short = 'o', long, value_parser, help = "Check all mailbox quotas")]
    quotas: bool,

    #[clap(
        short = 't',
        long,
        value_parser,
        help = "Mailbox quota threshold for erroring",
        default_value_t = 70
    )]
    quota_threshold: u16,

    #[clap(
        short = 'q',
        long,
        value_parser,
        help = "Check the outbound message queue"
    )]
    queue: bool,

    #[clap(
        value_parser,
        default_value = "https://localhost",
        help = "URL running Mailcow API"
    )]
    base_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    if !args.all && !args.queue && !args.quotas {
        panic!("Must enable at least one check.");
    }

    let api_key =
        std::env::var("MAILCOW_API_KEY").expect("Environment variable MAILCOW_API_KEY missing!");
    let api_key =
        HeaderValue::from_str(&api_key).expect("Could not convert MAILCOW_API_KEY to header.");
    let mut headers = HeaderMap::new();
    headers.append("x-api-key", api_key);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Could not create HTTP client.");

    let mut errors = vec![];

    if args.all || args.queue {
        if let Err(e) = check_queue(&args.base_url, &client).await {
            errors.push(e);
        }
    }

    if args.all || args.quotas {
        if let Err(e) = check_quota(args.quota_threshold, &args.base_url, &client).await {
            errors.push(e);
        }
    }

    match errors.is_empty() {
        true => Ok(()),
        false => Err(Error::from(errors)),
    }
}
