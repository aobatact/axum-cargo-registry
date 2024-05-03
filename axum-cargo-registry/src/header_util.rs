use axum::http::{header, HeaderMap};

/// Get the value of the `If-None-Match` header.
pub fn get_if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|e| e.to_str().ok().map(|o| o.into()))
}

/// Get the value of the `If-Modified-Since` header.
///
/// This function is only available when the `parse-time` feature is enabled.
pub fn get_modified_since(headers: &HeaderMap) -> Option<std::time::SystemTime> {
    #[cfg(feature = "parse-time")]
    {
        use time::format_description::well_known::Rfc2822;
        headers
            .get(header::IF_MODIFIED_SINCE)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| {
                time::OffsetDateTime::parse(x, &Rfc2822)
                    .ok()
                    .map(|x| x.into())
            })
    }
    #[cfg(not(feature = "parse-time"))]
    {
        let _ = headers;
        None
    }
}

/// Convert a `SystemTime` to a `Last-Modified` header.
///
/// This function is only available when the `parse-time` feature is enabled.
pub fn to_modified(last_modified: std::time::SystemTime) -> Option<(header::HeaderName, String)> {
    #[cfg(feature = "parse-time")]
    {
        use time::format_description::well_known::Rfc2822;
        let time: time::OffsetDateTime = last_modified.into();
        time.format(&Rfc2822)
            .ok()
            .map(|t| (header::LAST_MODIFIED, t))
    }
    #[cfg(not(feature = "parse-time"))]
    {
        let _ = last_modified;
        None
    }
}
