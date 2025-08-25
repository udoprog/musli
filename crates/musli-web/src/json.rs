use alloc::string::{String, ToString};

#[cfg(feature = "axum-core05")]
use axum_core05::extract as extract05;
#[cfg(feature = "axum-core05")]
use axum_core05::extract::rejection as rejection05;
#[cfg(feature = "axum-core05")]
use axum_core05::response as response05;
use bytes::{BufMut, Bytes, BytesMut};
use http::header::{self, HeaderValue};
use http::{HeaderMap, StatusCode};
use musli::Encode;
use musli::alloc::Global;
use musli::context::ErrorMarker;
use musli::de::DecodeOwned;
use musli::json::Encoding;
use musli::mode::Text;

const ENCODING: Encoding = Encoding::new();

/// A rejection from the JSON extractor.
pub struct JsonRejection {
    kind: JsonRejectionKind,
}

impl JsonRejection {
    #[inline]
    pub(crate) fn report(report: String) -> Self {
        Self {
            kind: JsonRejectionKind::Report(report),
        }
    }
}

enum JsonRejectionKind {
    ContentType,
    Report(String),
    #[cfg(feature = "axum-core05")]
    BytesRejection05(rejection05::BytesRejection),
}

#[cfg(feature = "axum-core05")]
impl From<rejection05::BytesRejection> for JsonRejection {
    #[inline]
    fn from(rejection: rejection05::BytesRejection) -> Self {
        JsonRejection {
            kind: JsonRejectionKind::BytesRejection05(rejection),
        }
    }
}

#[cfg(feature = "axum-core05")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "axum-core05")))]
impl response05::IntoResponse for JsonRejection {
    fn into_response(self) -> response05::Response {
        let status;
        let body;

        match self.kind {
            JsonRejectionKind::ContentType => {
                status = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                body = String::from("Expected request with `Content-Type: application/json`");
            }
            JsonRejectionKind::Report(report) => {
                status = StatusCode::BAD_REQUEST;
                body = report;
            }
            JsonRejectionKind::BytesRejection05(rejection) => {
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

#[cfg(feature = "axum-core05")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "axum-core05")))]
impl<T, S> extract05::FromRequest<S> for Json<T>
where
    T: DecodeOwned<Text, Global>,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(req: extract05::Request, state: &S) -> Result<Self, Self::Rejection> {
        if !json_content_type(req.headers()) {
            return Err(JsonRejection {
                kind: JsonRejectionKind::ContentType,
            });
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

#[cfg(feature = "axum-core05")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "axum-core05")))]
impl<T> response05::IntoResponse for Json<T>
where
    T: Encode<Text>,
{
    fn into_response(self) -> response05::Response {
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
    T: DecodeOwned<Text, Global>,
{
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self, JsonRejection> {
        let cx = musli::context::new().with_trace();

        if let Ok(value) = ENCODING.from_slice_with(&cx, bytes) {
            return Ok(Json(value));
        }

        let report = cx.report();
        let report = report.to_string();
        Err(JsonRejection::report(report))
    }
}
