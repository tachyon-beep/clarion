//! LSP-style Content-Length framing for the Clarion plugin transport.
//!
//! Each frame is a self-describing byte sequence:
//!
//! ```text
//! Content-Length: N\r\n
//! \r\n
//! <N bytes of JSON body>
//! ```
//!
//! The `Frame` type is body-only; `Content-Length` is derived from `body.len()`
//! at write time. The transport layer is deliberately protocol-agnostic: it
//! knows nothing about `initialize`, `analyze_file`, etc. That coupling lives
//! in the supervisor (Task 6), which composes [`Frame`] with the types in
//! [`super::protocol`].
//!
//! # Size ceiling
//!
//! [`read_frame`] accepts a `max_bytes: usize` parameter. If the `Content-Length`
//! header exceeds that value, [`TransportError::FrameTooLarge`] is returned
//! **without** consuming the body bytes from the reader — the supervisor decides
//! what to do (typically disconnect). Task 4 will wrap this behind a
//! `ContentLengthCeiling` newtype; for now the raw `usize` is sufficient.
//!
//! # No async
//!
//! The framing layer is synchronous (`impl BufRead` / `impl Write`). Task 6
//! wires it over subprocess `ChildStdin`/`ChildStdout`, which implement
//! `Read`/`Write` without requiring async at this layer.

use std::io::{BufRead, Write};

use thiserror::Error;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur during frame read/write.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Underlying I/O error.
    #[error("IO error while reading/writing frame: {0}")]
    Io(#[from] std::io::Error),

    /// The header section ended without a `Content-Length` line.
    #[error("missing Content-Length header")]
    MissingContentLength,

    /// A `Content-Length` header was present but its value was not a valid
    /// non-negative decimal integer.
    #[error("malformed Content-Length header: {value:?}")]
    InvalidContentLength { value: String },

    /// The declared frame body exceeds the configured ceiling.
    ///
    /// The body bytes are **not** consumed from the reader; the supervisor
    /// must decide whether to disconnect or drain.
    #[error("frame exceeds ceiling: observed {observed} bytes, ceiling {ceiling}")]
    FrameTooLarge { observed: usize, ceiling: usize },

    /// The stream ended before the declared number of body bytes were available.
    #[error("unexpected EOF in frame body; expected {expected} bytes, got {actual}")]
    TruncatedBody { expected: usize, actual: usize },

    /// A header line did not conform to the expected `Name: Value` shape.
    #[error("malformed header line: {line:?}")]
    MalformedHeader { line: String },
}

// ── Frame type ────────────────────────────────────────────────────────────────

/// A single framed message: the raw body bytes from one `Content-Length` block.
///
/// `Content-Length` is not stored; it is derived from `body.len()` on write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The raw body bytes (typically UTF-8 JSON, but the transport does not
    /// validate encoding).
    pub body: Vec<u8>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Read one LSP-style frame from `reader`.
///
/// Protocol:
/// 1. Read header lines until a bare `\r\n` (blank line).
/// 2. Extract `Content-Length: N` (case-insensitive header name).
/// 3. If `N > max_bytes`, return [`TransportError::FrameTooLarge`] without
///    consuming any body bytes.
/// 4. Read exactly `N` bytes into the body.
/// 5. Return `Frame { body }`.
///
/// Unknown headers are silently ignored (LSP tolerance — `Content-Type` etc.).
///
/// # Errors
///
/// See [`TransportError`] variants for the full list of failure modes.
pub fn read_frame(reader: &mut impl BufRead, max_bytes: usize) -> Result<Frame, TransportError> {
    let mut content_length: Option<usize> = None;

    // ── Step 1+2: read header lines ──────────────────────────────────────────
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;

        // EOF before blank line — caller's stream ended unexpectedly.
        if n == 0 {
            return Err(TransportError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF in header section",
            )));
        }

        // Blank line signals end of headers.
        if line == "\r\n" || line == "\n" {
            break;
        }

        // Strip CRLF / LF terminator for parsing.
        let trimmed = line.trim_end_matches(['\r', '\n']);

        // Ignore empty lines inside the header block (defensive).
        if trimmed.is_empty() {
            continue;
        }

        // Split on ": " — the header must have a colon.
        let Some((name, value)) = trimmed.split_once(':') else {
            return Err(TransportError::MalformedHeader {
                line: trimmed.to_owned(),
            });
        };
        let value = value.trim_start();

        // Case-insensitive comparison per LSP spec.
        if name.trim().eq_ignore_ascii_case("content-length") {
            let n: usize = value
                .parse()
                .map_err(|_| TransportError::InvalidContentLength {
                    value: value.to_owned(),
                })?;
            content_length = Some(n);
        }
        // All other headers are silently ignored.
    }

    // ── Step 3: ceiling check ─────────────────────────────────────────────────
    let length = content_length.ok_or(TransportError::MissingContentLength)?;
    if length > max_bytes {
        // Do NOT read any body bytes.
        return Err(TransportError::FrameTooLarge {
            observed: length,
            ceiling: max_bytes,
        });
    }

    // ── Step 4: read body ─────────────────────────────────────────────────────
    let mut body = vec![0u8; length];
    let mut total_read = 0usize;
    while total_read < length {
        match reader.read(&mut body[total_read..])? {
            0 => {
                return Err(TransportError::TruncatedBody {
                    expected: length,
                    actual: total_read,
                });
            }
            n => total_read += n,
        }
    }

    Ok(Frame { body })
}

/// Write one LSP-style frame to `writer`.
///
/// Produces:
/// ```text
/// Content-Length: N\r\n
/// \r\n
/// <body bytes>
/// ```
///
/// # Errors
///
/// Returns [`TransportError::Io`] on write failure.
pub fn write_frame(writer: &mut impl Write, frame: &Frame) -> Result<(), TransportError> {
    let len = frame.body.len();
    write!(writer, "Content-Length: {len}\r\n\r\n")?;
    writer.write_all(&frame.body)?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    // ── Transport test 1: round-trip a single frame ───────────────────────────

    #[test]
    fn transport_01_single_frame_round_trip() {
        let body = b"{\"jsonrpc\":\"2.0\",\"method\":\"initialized\",\"params\":{}}".to_vec();
        let frame = Frame { body: body.clone() };

        // Write
        let mut buf: Vec<u8> = Vec::new();
        write_frame(&mut buf, &frame).expect("write_frame must succeed");

        // Read back
        let mut cursor = Cursor::new(buf);
        let decoded = read_frame(&mut cursor, usize::MAX).expect("read_frame must succeed");

        assert_eq!(decoded.body, body);
    }

    // ── Transport test 2: exact Content-Length boundary ───────────────────────

    #[test]
    fn transport_02_exact_ceiling_boundary_succeeds() {
        let body = b"hello".to_vec();
        let frame = Frame { body: body.clone() };
        let max = body.len(); // exactly at the ceiling

        let mut buf: Vec<u8> = Vec::new();
        write_frame(&mut buf, &frame).expect("write");

        let mut cursor = Cursor::new(buf);
        let decoded = read_frame(&mut cursor, max).expect("read at exact boundary must succeed");
        assert_eq!(decoded.body, body);
    }

    // ── Transport test 3: Content-Length above ceiling — body not consumed ────

    #[test]
    fn transport_03_content_length_above_ceiling_returns_frame_too_large_without_consuming_body() {
        let body = b"hello world".to_vec();
        let frame = Frame { body: body.clone() };
        let max = body.len() - 1; // one byte below declared length

        let mut buf: Vec<u8> = Vec::new();
        write_frame(&mut buf, &frame).expect("write");

        // Record position after headers so we can assert the body was not consumed.
        // The header is "Content-Length: 11\r\n\r\n" — 22 bytes for len=11.
        let header_len = format!("Content-Length: {}\r\n\r\n", body.len()).len();

        let mut cursor = Cursor::new(buf);
        let err = read_frame(&mut cursor, max).expect_err("should fail with FrameTooLarge");

        assert!(
            matches!(
                err,
                TransportError::FrameTooLarge {
                    observed,
                    ceiling,
                } if observed == body.len() && ceiling == max
            ),
            "unexpected error: {err}"
        );

        // Cursor position must be exactly at the start of the body, not past it.
        let pos =
            usize::try_from(cursor.position()).expect("cursor position fits in usize on test host");
        assert_eq!(
            pos, header_len,
            "body must not have been consumed; cursor should be at position {header_len}, got {pos}"
        );
    }

    // ── Transport test 4: two back-to-back frames ─────────────────────────────

    #[test]
    fn transport_04_two_consecutive_frames_round_trip() {
        let body1 =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"initialize\",\"params\":{},\"id\":1}".to_vec();
        let body2 =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\",\"params\":{},\"id\":2}".to_vec();

        let mut buf: Vec<u8> = Vec::new();
        write_frame(
            &mut buf,
            &Frame {
                body: body1.clone(),
            },
        )
        .expect("write 1");
        write_frame(
            &mut buf,
            &Frame {
                body: body2.clone(),
            },
        )
        .expect("write 2");

        let mut cursor = Cursor::new(buf);
        let f1 = read_frame(&mut cursor, usize::MAX).expect("read frame 1");
        let f2 = read_frame(&mut cursor, usize::MAX).expect("read frame 2");

        assert_eq!(f1.body, body1, "first frame body mismatch");
        assert_eq!(f2.body, body2, "second frame body mismatch");
    }

    // ── Transport test 5: missing Content-Length ──────────────────────────────

    #[test]
    fn transport_05_missing_content_length_returns_error() {
        // Headers end without Content-Length.
        let raw = b"X-Custom: stuff\r\n\r\n{\"key\":\"value\"}";
        let mut cursor = Cursor::new(raw.as_ref());
        let err = read_frame(&mut cursor, usize::MAX).expect_err("should fail");
        assert!(
            matches!(err, TransportError::MissingContentLength),
            "expected MissingContentLength, got: {err}"
        );
    }

    // ── Transport test 6: malformed Content-Length ────────────────────────────

    #[test]
    fn transport_06_malformed_content_length_returns_invalid_content_length() {
        let raw = b"Content-Length: abc\r\n\r\n";
        let mut cursor = Cursor::new(raw.as_ref());
        let err = read_frame(&mut cursor, usize::MAX).expect_err("should fail");
        assert!(
            matches!(
                err,
                TransportError::InvalidContentLength { ref value } if value == "abc"
            ),
            "expected InvalidContentLength, got: {err}"
        );
    }

    // ── Transport test 7: truncated body ──────────────────────────────────────

    #[test]
    fn transport_07_truncated_body_returns_truncated_body_error() {
        // Header says 10, body has only 5 bytes.
        let raw = b"Content-Length: 10\r\n\r\nhello";
        let mut cursor = Cursor::new(raw.as_ref());
        let err = read_frame(&mut cursor, usize::MAX).expect_err("should fail");
        assert!(
            matches!(
                err,
                TransportError::TruncatedBody {
                    expected: 10,
                    actual: 5
                }
            ),
            "expected TruncatedBody, got: {err}"
        );
    }
}
