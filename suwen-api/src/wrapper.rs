use std::borrow::Cow;

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T: Serialize> {
    status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<Cow<'static, str>>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            status_code: 200,
            data: Some(data),
            message: None,
        }
    }

    pub fn not_found(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            status_code: 404,
            data: None,
            message: Some(message.into()),
        }
    }

    pub fn internal_server_error(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            status_code: 500,
            data: None,
            message: Some(message.into()),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::from_u16(self.status_code).expect("invalid Http Status Code"),
            Json(self),
        )
            .into_response()
    }
}

pub type ApiError = ApiResponse<()>;

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self::internal_server_error(value.into().to_string())
    }
}
