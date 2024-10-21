use reqwest::StatusCode;
use serde::Deserialize;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request send error")]
    RequestSend(#[source] reqwest::Error),

    #[error("too many requests")]
    TooManyRequests,
    #[error("too many attempts")]
    TooManyAttempts,

    #[error("api error")]
    ApiError(ApiError),

    #[error("error retrieving api error")]
    RetrievingApiError(#[source] reqwest::Error),

    #[error("error retrieving response content")]
    ResponseContent(#[source] reqwest::Error),

    #[error("error while deserializing: {source}")]
    Deserialize {
        #[source]
        err: serde_json::Error,
        source: String,
    },
}

impl Error {
    pub(crate) async fn api_error(response: reqwest::Response) -> Self {
        match ApiError::from_response(response).await {
            Ok(good_error) => Self::ApiError(good_error),
            Err(bad_error) => bad_error,
        }
    }
}

#[derive(thiserror::Error, Debug, Deserialize)]
#[error("({status_code}) {}", status.as_ref().map(|s| s.message.as_str()).or_else(|| status_code.canonical_reason()).unwrap_or("<no message>"))]
pub struct ApiError {
    /// Status reported in response headers.
    #[serde(deserialize_with = "deserialize_status")]
    pub status_code: StatusCode,
    pub status: Option<ApiErrorStatus>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorStatus {
    pub message: String,
    /// Status code reported by the Riot API JSON body.
    #[serde(deserialize_with = "deserialize_status")]
    pub status_code: StatusCode,
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let status_code = u16::deserialize(deserializer)?;
    StatusCode::from_u16(status_code).map_err(serde::de::Error::custom)
}

impl ApiError {
    pub async fn from_response(response: reqwest::Response) -> Result<Self> {
        let status = response.status();
        let text = response.text().await.map_err(Error::RetrievingApiError)?;
        Ok(Self::from_response_text(status, text))
    }

    pub fn from_response_text(status: StatusCode, text: String) -> Self {
        if let Ok(error) = serde_json::from_str::<ApiError>(&text) {
            error
        } else {
            ApiError {
                status_code: status,
                status: None,
            }
        }
    }
}
