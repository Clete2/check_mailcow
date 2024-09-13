use std::process::ExitCode;

use check_mailcow::{
    check_postfix_rejections::check_postfix_rejections, check_queue::check_queue,
    check_quota::check_quota,
};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue};

#[derive(Parser, Debug)]
#[clap(
    author = "Clete Blackwell II",
    version = "1.0.0",
    about = "Run various checks on Mailcow",
    long_about = "Run various checks on self-hosted Mailcow-Dockerized installations (https://github.com/mailcow/mailcow-dockerized)\n\nNOTE: You MUST set an environment variable 'MAILCOW_API_KEY' that contains the secret used to invoke the Mailcow API."
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
        help = "Mailbox quota threshold % for erroring",
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
        short = 'p',
        long,
        value_parser,
        help = "Check postfix rejections for SSL requirements"
    )]
    postfix_rejections: bool,

    #[clap(
        short = 'l',
        long,
        value_parser,
        default_value_t = 1000,
        help = "Number of messages to retrieve from logs for checking. You should set this to a number where there are less logs than the specified number are emitted in the period between runs of this program."
    )]
    logs_to_retrieve: usize,

    #[clap(
        value_parser,
        default_value = "https://localhost",
        help = "URL running Mailcow API"
    )]
    base_url: String,
}

#[tokio::main]
async fn main() -> ExitCode {
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

    if args.all || args.postfix_rejections {
        if let Err(e) =
            check_postfix_rejections(&args.base_url, &args.logs_to_retrieve, &client).await
        {
            errors.push(e);
        }
    }

    match errors.is_empty() {
        true => ExitCode::SUCCESS,
        false => {
            for error in errors {
                eprintln!("Error:\n{}\n", error);
            }

            ExitCode::FAILURE
        }
    }
}
