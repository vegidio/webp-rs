//! Small helpers shared by the encoder and decoder for talking to libwebp's C API.
//!
//! Unlike libavif, libwebp has no `*ResultToString` helper, so the human-readable
//! messages for its status/error enums are spelled out here.

use crate::sys;

/// `true` when a decode status is `VP8_STATUS_OK`.
pub(crate) fn is_ok(status: sys::VP8StatusCode) -> bool {
    status == sys::VP8StatusCode_VP8_STATUS_OK
}

/// Human-readable message for a libwebp decode status (`VP8StatusCode`).
pub(crate) fn status_message(status: sys::VP8StatusCode) -> String {
    let msg = match status {
        sys::VP8StatusCode_VP8_STATUS_OK => "ok",
        sys::VP8StatusCode_VP8_STATUS_OUT_OF_MEMORY => "out of memory",
        sys::VP8StatusCode_VP8_STATUS_INVALID_PARAM => "invalid parameter",
        sys::VP8StatusCode_VP8_STATUS_BITSTREAM_ERROR => "bitstream error",
        sys::VP8StatusCode_VP8_STATUS_UNSUPPORTED_FEATURE => "unsupported feature",
        sys::VP8StatusCode_VP8_STATUS_SUSPENDED => "suspended",
        sys::VP8StatusCode_VP8_STATUS_USER_ABORT => "user abort",
        sys::VP8StatusCode_VP8_STATUS_NOT_ENOUGH_DATA => "not enough data",
        _ => "unknown decode error",
    };
    msg.to_string()
}

/// Human-readable message for a libwebp encode error (`WebPEncodingError`, from
/// `WebPPicture.error_code`).
pub(crate) fn encode_error_message(code: sys::WebPEncodingError) -> String {
    let msg = match code {
        sys::WebPEncodingError_VP8_ENC_OK => "ok",
        sys::WebPEncodingError_VP8_ENC_ERROR_OUT_OF_MEMORY => "out of memory",
        sys::WebPEncodingError_VP8_ENC_ERROR_BITSTREAM_OUT_OF_MEMORY => "bitstream out of memory",
        sys::WebPEncodingError_VP8_ENC_ERROR_NULL_PARAMETER => "null parameter",
        sys::WebPEncodingError_VP8_ENC_ERROR_INVALID_CONFIGURATION => "invalid configuration",
        sys::WebPEncodingError_VP8_ENC_ERROR_BAD_DIMENSION => "bad dimension (max 16383x16383)",
        sys::WebPEncodingError_VP8_ENC_ERROR_PARTITION0_OVERFLOW => "partition 0 overflow",
        sys::WebPEncodingError_VP8_ENC_ERROR_PARTITION_OVERFLOW => "partition overflow",
        sys::WebPEncodingError_VP8_ENC_ERROR_BAD_WRITE => "bad write",
        sys::WebPEncodingError_VP8_ENC_ERROR_FILE_TOO_BIG => "file too big",
        sys::WebPEncodingError_VP8_ENC_ERROR_USER_ABORT => "user abort",
        _ => "unknown encode error",
    };
    msg.to_string()
}
