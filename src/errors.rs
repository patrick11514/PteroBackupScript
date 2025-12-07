#[derive(Debug)]
pub enum AppError {
    Request(reqwest::Error),
    Response(String),
    Parse {
        err: serde_json::Error,
        body: String,
    },
    YupOauth2(yup_oauth2::Error),
    Other(String),
    Created,
    Format(serde_json::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Request(err) => write!(f, "Request error: {}", err),
            AppError::Response(msg) => write!(f, "Response error: {}", msg),
            AppError::Parse { err, body } => {
                body.lines().enumerate().for_each(|(i, line)| {
                    if i + 1 == err.line() {
                        writeln!(f, ">{:4} | {}", i + 1, line).unwrap();
                    } else {
                        writeln!(f, " {:4} | {}", i + 1, line).unwrap();
                    }
                });
                write!(f, "JSON error: {}", err)
            }
            AppError::YupOauth2(err) => write!(f, "OAuth2 error: {}", err),
            AppError::Other(msg) => write!(f, "Error: {}", msg),
            AppError::Created => write!(
                f,
                "The example config file created. Please edit it, and run program again."
            ),
            AppError::Format(msg) => write!(f, "Format error: {}", msg),
            AppError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for AppError {}

#[allow(dead_code)]
trait ToAppError<T> {
    fn to_app_error(self) -> Result<T, AppError>;
}

impl<T> ToAppError<T> for Result<T, yup_oauth2::Error> {
    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::YupOauth2(err)),
        }
    }
}

impl<T> ToAppError<T> for Result<T, reqwest::Error> {
    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(AppError::Request(err)),
        }
    }
}

impl<T> ToAppError<T> for Result<T, String> {
    fn to_app_error(self) -> Result<T, AppError> {
        match self {
            Ok(value) => Ok(value),
            Err(msg) => Err(AppError::Other(msg)),
        }
    }
}
