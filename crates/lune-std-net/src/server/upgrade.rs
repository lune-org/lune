use async_tungstenite::tungstenite::{error::ProtocolError, handshake::derive_accept_key};

use hyper::{
    HeaderMap, Request as HyperRequest, Response as HyperResponse, StatusCode,
    body::Incoming,
    header::{CONNECTION, HeaderName, UPGRADE},
};

use crate::body::ReadableBody;

const SEC_WEBSOCKET_VERSION: HeaderName = HeaderName::from_static("sec-websocket-version");
const SEC_WEBSOCKET_KEY: HeaderName = HeaderName::from_static("sec-websocket-key");
const SEC_WEBSOCKET_ACCEPT: HeaderName = HeaderName::from_static("sec-websocket-accept");

pub fn is_upgrade_request(request: &HyperRequest<Incoming>) -> bool {
    fn check_header_contains(headers: &HeaderMap, header_name: HeaderName, value: &str) -> bool {
        headers.get(header_name).is_some_and(|header| {
            header.to_str().map_or_else(
                |_| false,
                |header_str| {
                    header_str
                        .split(',')
                        .any(|part| part.trim().eq_ignore_ascii_case(value))
                },
            )
        })
    }

    check_header_contains(request.headers(), CONNECTION, "Upgrade")
        && check_header_contains(request.headers(), UPGRADE, "websocket")
}

pub fn make_upgrade_response(
    request: &HyperRequest<Incoming>,
) -> Result<HyperResponse<ReadableBody>, ProtocolError> {
    let key = request
        .headers()
        .get(SEC_WEBSOCKET_KEY)
        .ok_or(ProtocolError::MissingSecWebSocketKey)?;

    if request
        .headers()
        .get(SEC_WEBSOCKET_VERSION)
        .is_none_or(|v| v.as_bytes() != b"13")
    {
        return Err(ProtocolError::MissingSecWebSocketVersionHeader);
    }

    Ok(HyperResponse::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(CONNECTION, "upgrade")
        .header(UPGRADE, "websocket")
        .header(SEC_WEBSOCKET_ACCEPT, derive_accept_key(key.as_bytes()))
        .body(ReadableBody::from("switching to websocket protocol"))
        .unwrap())
}
