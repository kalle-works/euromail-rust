//! Verification for EuroMail webhook payload signatures.
//!
//! Every webhook delivery is signed with HMAC-SHA256 and carries an
//! `X-Euromail-Signature` header shaped like Stripe's: `t=<unix_ts>,v1=<hex_hmac>`.
//! The signed input is `"{timestamp}.{raw_request_body}"`, keyed with the
//! webhook's signing secret (see `crates/euromail-worker/src/processors/fire_webhook.rs`
//! in the main API repo for the server-side implementation this mirrors).
//!
//! Always verify against the *raw* bytes of the request body — re-serializing
//! a parsed JSON value can reorder keys or change whitespace and break the
//! signature.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Replay-protection window, in seconds, matching the server's own tolerance.
pub const DEFAULT_TOLERANCE_SECONDS: i64 = 300;

/// Errors returned by [`verify_webhook_signature`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WebhookSignatureError {
    /// The header did not match the expected `t=<ts>,v1=<hex>` shape.
    #[error("malformed signature header: {0}")]
    MalformedHeader(String),

    /// The computed signature did not match the one in the header.
    #[error("signature does not match payload")]
    SignatureMismatch,

    /// The header's timestamp is outside the allowed tolerance window,
    /// suggesting a replayed request (or a badly skewed clock).
    #[error(
        "timestamp {timestamp} is outside the {tolerance}s tolerance window (reference time {now})"
    )]
    TimestampOutOfTolerance {
        timestamp: i64,
        now: i64,
        tolerance: i64,
    },
}

/// Verify a webhook payload's signature using the current system time as the
/// replay-protection reference and [`DEFAULT_TOLERANCE_SECONDS`] as the window.
///
/// `payload` must be the exact raw request body bytes EuroMail sent — not a
/// re-serialized copy. `signature_header` is the full value of the
/// `X-Euromail-Signature` header. `secret` is the webhook's signing secret,
/// shown once when the webhook is created.
///
/// # Example
///
/// ```rust
/// use euromail::verify_webhook_signature;
///
/// # fn handle_request(body: &[u8], signature_header: &str, secret: &str) {
/// match verify_webhook_signature(body, signature_header, secret) {
///     Ok(()) => { /* process the event */ }
///     Err(e) => { /* reject with 400 — not a genuine EuroMail delivery */
///         eprintln!("rejected webhook: {e}");
///     }
/// }
/// # }
/// ```
pub fn verify_webhook_signature(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<(), WebhookSignatureError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    verify_webhook_signature_at(payload, signature_header, secret, now, DEFAULT_TOLERANCE_SECONDS)
}

/// Like [`verify_webhook_signature`], but with an explicit reference time and
/// tolerance window instead of the system clock. Exists mainly so tests can
/// verify against a fixed timestamp deterministically; production code should
/// normally call [`verify_webhook_signature`].
pub fn verify_webhook_signature_at(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
    now: i64,
    tolerance_seconds: i64,
) -> Result<(), WebhookSignatureError> {
    let (timestamp, signature_hex) = parse_header(signature_header)?;

    if (now - timestamp).abs() > tolerance_seconds {
        return Err(WebhookSignatureError::TimestampOutOfTolerance {
            timestamp,
            now,
            tolerance: tolerance_seconds,
        });
    }

    let signature_bytes = hex::decode(signature_hex)
        .map_err(|_| WebhookSignatureError::MalformedHeader("v1 is not valid hex".to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| WebhookSignatureError::MalformedHeader("invalid secret".to_string()))?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);

    // `verify_slice` compares in constant time, so this cannot be used as a
    // timing oracle to brute-force the signature byte by byte.
    mac.verify_slice(&signature_bytes)
        .map_err(|_| WebhookSignatureError::SignatureMismatch)
}

/// Parse `"t=<unix_ts>,v1=<hex>"` into `(timestamp, signature_hex)`.
fn parse_header(header: &str) -> Result<(i64, &str), WebhookSignatureError> {
    let mut timestamp: Option<i64> = None;
    let mut signature: Option<&str> = None;

    for part in header.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = value.parse().ok();
        } else if let Some(value) = part.strip_prefix("v1=") {
            signature = Some(value);
        }
    }

    match (timestamp, signature) {
        (Some(t), Some(s)) if !s.is_empty() => Ok((t, s)),
        _ => Err(WebhookSignatureError::MalformedHeader(header.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vector computed independently with Python's `hmac`/`hashlib`
    // (not derived from this implementation):
    //
    //   secret = "whsec_test_secret_a1b2c3"
    //   timestamp = 1717000000
    //   payload = b'{"event":"delivered","email_id":"em_01JQ5K8WMFZ9XRTVB3GH6EDCN4",
    //               "account_id":"acc_01JQ5K8WMFZ9XRTVB3GH6EDCN4","to":"user@example.com"}'
    //   signed_input = f"{timestamp}.".encode() + payload
    //   hmac.new(secret.encode(), signed_input, hashlib.sha256).hexdigest()
    //   == "5e6f21d632ea84232cb5bb5ac75b70f3d68609d28d37e17054a9db5f60d8d654"[..64]
    const SECRET: &str = "whsec_test_secret_a1b2c3";
    const TIMESTAMP: i64 = 1717000000;
    const PAYLOAD: &[u8] = br#"{"event":"delivered","email_id":"em_01JQ5K8WMFZ9XRTVB3GH6EDCN4","account_id":"acc_01JQ5K8WMFZ9XRTVB3GH6EDCN4","to":"user@example.com"}"#;
    const HEADER: &str =
        "t=1717000000,v1=5e6f21d632ea84232cb5bb5ac75b70f3d68609d28d37e17054a9db5f60d8d654";

    #[test]
    fn verifies_the_exact_server_test_vector() {
        // `now` set equal to the vector's timestamp so the tolerance check
        // never enters into whether the signature itself is correct.
        let result = verify_webhook_signature_at(PAYLOAD, HEADER, SECRET, TIMESTAMP, 300);
        assert_eq!(result, Ok(()));
    }

    /// The exact test vector specified for this cross-SDK effort (the same
    /// one asserted by the Python, TypeScript, Go, and PHP SDKs), so all
    /// five verify identically against one shared, independently-computed
    /// HMAC — not just internally consistent with this crate's own signing.
    #[test]
    fn verifies_the_shared_cross_sdk_contract_vector() {
        const SECRET: &str = "whsec_test_secret_do_not_use";
        const TIMESTAMP: i64 = 1735689600;
        const PAYLOAD: &[u8] = br#"{"event":"delivered","email_id":"018f2c3a-7b1e-7c3e-8b1a-2f6e9d4c5a01","account_id":"018f2c3a-7b1e-7c3e-8b1a-2f6e9d4c5a02","timestamp":"2025-01-01T00:00:00Z"}"#;
        const HEADER: &str =
            "t=1735689600,v1=d571fbef13b9e524d460f6f2c88f8d8dc7df3c50ff7aabdedd8a3656abb96dd0";

        let result = verify_webhook_signature_at(PAYLOAD, HEADER, SECRET, TIMESTAMP, 300);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_a_tampered_payload() {
        let mut tampered = PAYLOAD.to_vec();
        // Flip one byte inside the payload — a real attacker modifying an
        // intercepted, replayed, or forged webhook body.
        let last = tampered.len() - 2;
        tampered[last] = if tampered[last] == b'm' { b'n' } else { b'm' };

        let result = verify_webhook_signature_at(&tampered, HEADER, SECRET, TIMESTAMP, 300);
        assert_eq!(result, Err(WebhookSignatureError::SignatureMismatch));
    }

    #[test]
    fn rejects_the_wrong_secret() {
        let result =
            verify_webhook_signature_at(PAYLOAD, HEADER, "whsec_wrong_secret", TIMESTAMP, 300);
        assert_eq!(result, Err(WebhookSignatureError::SignatureMismatch));
    }

    #[test]
    fn rejects_a_signature_replayed_outside_the_tolerance_window() {
        // Six minutes after the signed timestamp, with the default 5-minute window.
        let result = verify_webhook_signature_at(
            PAYLOAD,
            HEADER,
            SECRET,
            TIMESTAMP + 360,
            DEFAULT_TOLERANCE_SECONDS,
        );
        assert_eq!(
            result,
            Err(WebhookSignatureError::TimestampOutOfTolerance {
                timestamp: TIMESTAMP,
                now: TIMESTAMP + 360,
                tolerance: DEFAULT_TOLERANCE_SECONDS,
            })
        );
    }

    #[test]
    fn accepts_a_signature_at_the_edge_of_the_tolerance_window() {
        let result = verify_webhook_signature_at(
            PAYLOAD,
            HEADER,
            SECRET,
            TIMESTAMP + DEFAULT_TOLERANCE_SECONDS,
            DEFAULT_TOLERANCE_SECONDS,
        );
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_a_header_missing_the_v1_component() {
        let result = verify_webhook_signature_at(PAYLOAD, "t=1717000000", SECRET, TIMESTAMP, 300);
        assert!(matches!(
            result,
            Err(WebhookSignatureError::MalformedHeader(_))
        ));
    }

    #[test]
    fn rejects_a_header_missing_the_timestamp_component() {
        let result = verify_webhook_signature_at(
            PAYLOAD,
            "v1=5e6f21d632ea84232cb5bb5ac75b70f3d68609d28d37e17054a9db5f60d8d654",
            SECRET,
            TIMESTAMP,
            300,
        );
        assert!(matches!(
            result,
            Err(WebhookSignatureError::MalformedHeader(_))
        ));
    }

    #[test]
    fn rejects_non_hex_signature() {
        let result =
            verify_webhook_signature_at(PAYLOAD, "t=1717000000,v1=not-hex", SECRET, TIMESTAMP, 300);
        assert!(matches!(
            result,
            Err(WebhookSignatureError::MalformedHeader(_))
        ));
    }

    #[test]
    fn verify_webhook_signature_uses_the_system_clock_and_accepts_a_fresh_signature() {
        // Build a signature for "right now" the same way the server does, and
        // confirm the public, clock-driven entry point accepts it — this is
        // the one test that exercises `verify_webhook_signature` itself
        // rather than the `_at` variant.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut mac = HmacSha256::new_from_slice(SECRET.as_bytes()).unwrap();
        mac.update(now.to_string().as_bytes());
        mac.update(b".");
        mac.update(PAYLOAD);
        let sig = hex::encode(mac.finalize().into_bytes());
        let header = format!("t={now},v1={sig}");

        assert_eq!(
            verify_webhook_signature(PAYLOAD, &header, SECRET),
            Ok(())
        );
    }
}
