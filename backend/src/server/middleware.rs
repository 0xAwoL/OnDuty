use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: Deserialize<'static> + Validate,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(payload) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                (StatusCode::BAD_REQUEST, rejection.body_text()).into_response()
            })?;

        if let Err(errors) = payload.validate() {
            let body = serde_json::json!({
                "errors": errors
                    .field_errors()
                    .iter()
                    .map(|(field, errs)| {
                        (
                            field.to_string(),
                            serde_json::json!(errs.iter().map(|e| e.code.to_string()).collect::<Vec<_>>()),
                        )
                    })
                    .collect::<serde_json::Value>()
            });
            return Err((StatusCode::BAD_REQUEST, axum::Json(body)).into_response());
        }

        Ok(ValidatedJson(payload))
    }
}
