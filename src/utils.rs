use bytes::BytesMut;

pub fn extract_tid(data: &BytesMut) -> Option<&str> {
    if data.len() <= 17 {
        return None;
    }

    if let Ok(str) = std::str::from_utf8(&data[..]) {
        Some(&str[9..17])
    } else {
        None
    }
}
