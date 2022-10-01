use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::from(message.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::from(e.to_string())
    }
}

impl From<Vec<Error>> for Error {
    fn from(errors: Vec<Error>) -> Self {
        let mut message = String::new();
        for error in errors {
            message.push_str(error.to_string().as_str());
            message.push_str("\n");
        }

        Self { message }
    }
}
