use axum::http::{header, HeaderMap};
use time::format_description::well_known::Rfc2822;

pub fn get_if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|e| e.to_str().ok().map(|o| o.into()))
}

pub fn get_modified_since(headers: &HeaderMap) -> Option<std::time::SystemTime> {
    #[cfg(feature = "parse-time")]
    {
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
        None
    }
}

pub fn to_modified(last_modified: std::time::SystemTime) -> Option<(header::HeaderName, String)> {
    #[cfg(feature = "parse-time")]
    {
        let time: time::OffsetDateTime = last_modified.into();
        time.format(&Rfc2822)
            .ok()
            .map(|t| (header::LAST_MODIFIED, t))
    }
    #[cfg(not(feature = "parse-time"))]
    {
        None
    }
}
