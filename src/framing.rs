use std::io::{self, BufRead, Write};

use crate::error::{ErrorCode, ProtocolError};

pub const MAX_FRAME_SIZE: usize = 4 * 1024 * 1024;
pub const MAX_HEADER_SIZE: usize = 8 * 1024;

pub fn write_frame(mut writer: impl Write, body: &[u8]) -> Result<(), ProtocolError> {
    if body.len() > MAX_FRAME_SIZE {
        return Err(frame_error(
            ErrorCode::OversizedFrame,
            "frame exceeds limit",
        ));
    }
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())
        .and_then(|_| writer.write_all(body))
        .and_then(|_| writer.flush())
        .map_err(io_error)
}

pub fn read_frame(mut reader: impl BufRead) -> Result<Option<Vec<u8>>, ProtocolError> {
    let mut header = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(io_error)?;
        if available.is_empty() {
            return if header.is_empty() {
                Ok(None)
            } else {
                Err(frame_error(
                    ErrorCode::MalformedFrame,
                    "incomplete frame header",
                ))
            };
        }
        let byte = available[0];
        reader.consume(1);
        header.push(byte);
        if header.len() > MAX_HEADER_SIZE {
            return Err(frame_error(
                ErrorCode::OversizedFrame,
                "frame header exceeds limit",
            ));
        }
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    let header = std::str::from_utf8(&header)
        .map_err(|_| frame_error(ErrorCode::MalformedFrame, "header is not UTF-8"))?;
    let mut length = None;
    for line in header[..header.len() - 4].split("\r\n") {
        let Some(value) = line.strip_prefix("Content-Length:") else {
            return Err(frame_error(
                ErrorCode::MalformedFrame,
                "unsupported frame header",
            ));
        };
        if length.is_some() {
            return Err(frame_error(
                ErrorCode::MalformedFrame,
                "duplicate Content-Length",
            ));
        }
        length = Some(
            value
                .trim()
                .parse::<usize>()
                .map_err(|_| frame_error(ErrorCode::MalformedFrame, "invalid Content-Length"))?,
        );
    }
    let length =
        length.ok_or_else(|| frame_error(ErrorCode::MalformedFrame, "missing Content-Length"))?;
    if length > MAX_FRAME_SIZE {
        return Err(frame_error(
            ErrorCode::OversizedFrame,
            "frame exceeds limit",
        ));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).map_err(io_error)?;
    Ok(Some(body))
}

fn io_error(error: io::Error) -> ProtocolError {
    frame_error(ErrorCode::MalformedFrame, error.to_string())
}

fn frame_error(code: ErrorCode, message: impl Into<String>) -> ProtocolError {
    ProtocolError::new(code, message, false, "false-agent-protocol")
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use super::*;

    #[test]
    fn frames_round_trip_and_reject_oversize() {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, br#"{"jsonrpc":"2.0"}"#).unwrap();
        assert_eq!(
            read_frame(BufReader::new(Cursor::new(bytes))).unwrap(),
            Some(br#"{"jsonrpc":"2.0"}"#.to_vec())
        );
        let oversized = format!("Content-Length: {}\r\n\r\n", MAX_FRAME_SIZE + 1);
        assert!(read_frame(BufReader::new(Cursor::new(oversized))).is_err());
    }
}
