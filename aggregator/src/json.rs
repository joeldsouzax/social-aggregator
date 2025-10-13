use crate::error::Error;
use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidJson<T>(pub T);

impl<T> IntoResponse for ValidJson<T>
where
    T: Serialize,
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

impl<S, T> FromRequest<S> for ValidJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidJson(value))
    }
}
