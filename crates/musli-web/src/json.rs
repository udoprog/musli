use alloc::string::{String, ToString};

use axum_core::extract::rejection::BytesRejection;
use axum_core::extract::{FromRequest, Request};
use axum_core::response::{IntoResponse, Response};
use bytes::{BufMut, Bytes, BytesMut};
use http::header::{self, HeaderValue};
use http::{HeaderMap, StatusCode};
use musli::alloc::System;
use musli::context::ErrorMarker;
use musli::de::DecodeOwned;
use musli::json::Encoding;
use musli::mode::Text;
use musli::Encode;

const ENCODING: Encoding = Encoding::new();

/// A rejection from the JSON extractor.
pub enum JsonRejection {
    ContentType,
    Report(String),
    BytesRejection(BytesRejection),
}

impl From<BytesRejection> for JsonRejection {
    #[inline]
    fn from(rejection: BytesRejection) -> Self {
        JsonRejection::BytesRejection(rejection)
    }
}

impl IntoResponse for JsonRejection {
    fn into_response(self) -> Response {
        let status;
        let body;

        match self {
            JsonRejection::ContentType => {
                status = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                body = String::from("Expected request with `Content-Type: application/json`");
            }
            JsonRejection::Report(report) => {
                status = StatusCode::BAD_REQUEST;
                body = report;
            }
            JsonRejection::BytesRejection(rejection) => {
                return rejection.into_response();
            }
        }

        (
            status,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
            )],
            body,
        )
            .into_response()
    }
}

/// Encode the given value as JSON.
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DecodeOwned<Text, System>,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if !json_content_type(req.headers()) {
            return Err(JsonRejection::ContentType);
        }

        let bytes = Bytes::from_request(req, state).await?;
        Self::from_bytes(&bytes)
    }
}

fn json_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().is_some_and(|name| name == "json"))
}

impl<T> IntoResponse for Json<T>
where
    T: Encode<Text>,
{
    fn into_response(self) -> Response {
        let cx = musli::context::new().with_trace();

        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();

        match ENCODING.to_writer_with(&cx, &mut buf, &self.0) {
            Ok(()) => {
                let content_type = [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )];
                let report = buf.into_inner().freeze();
                (content_type, report).into_response()
            }
            Err(ErrorMarker { .. }) => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                let content_type = [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )];
                let report = cx.report().to_string();
                (status, content_type, report).into_response()
            }
        }
    }
}

impl<T> Json<T>
where
    T: DecodeOwned<Text, System>,
{
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self, JsonRejection> {
        let cx = musli::context::new().with_trace();

        if let Ok(value) = ENCODING.from_slice_with(&cx, bytes) {
            return Ok(Json(value));
        }

        let report = cx.report();
        let report = report.to_string();
        Err(JsonRejection::Report(report))
    }
}
