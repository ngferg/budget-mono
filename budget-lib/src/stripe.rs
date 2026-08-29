//! Thin client for the handful of Stripe REST calls the budget service makes,
//! plus verification/parsing for the incoming webhook.
//!
//! Managed Payments lets Stripe act as the merchant of record and handle tax
//! collection, so every request here pins the preview API version that feature
//! ships in and every Checkout Session opts in via `managed_payments[enabled]`.
//!
//! Configuration comes from the environment:
//!   STRIPE_SECRET_KEY      required for every outbound call (sk_test_… / sk_live_…)
//!   STRIPE_PRICE_ID        the recurring Price the Checkout Session sells; obtain
//!                          it by running the `create_subscription_product` binary
//!   STRIPE_WEBHOOK_SECRET  used to verify the signature on the webhook (whsec_…)
//!   PUBLIC_BASE_URL        where Checkout sends the customer back afterwards

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

const STRIPE_API_BASE: &str = "https://api.stripe.com";

/// Managed Payments requires this preview version (or newer). Sent on every
/// request as the `Stripe-Version` header so behaviour is pinned regardless of
/// the account's dashboard default.
const STRIPE_VERSION: &str = "2026-02-25.preview";

/// How far the timestamp on a webhook signature may drift from our clock before
/// we reject it as a replay. Matches Stripe's own default tolerance.
const WEBHOOK_TOLERANCE_SECONDS: i64 = 300;

#[derive(thiserror::Error, Debug)]
pub enum StripeError {
    #[error("STRIPE_SECRET_KEY is not set")]
    MissingApiKey,
    #[error("Could not reach Stripe: {0}")]
    Transport(String),
    #[error("Stripe returned {status}: {body}")]
    Api { status: u16, body: String },
    #[error("Could not understand Stripe's response: {0}")]
    Decode(String),
    #[error("Webhook signature verification failed: {0}")]
    BadSignature(String),
}

/// Authenticated handle to the Stripe REST API.
pub struct StripeClient {
    http: reqwest::Client,
    secret_key: String,
}

impl StripeClient {
    /// Builds a client from `STRIPE_SECRET_KEY`.
    pub fn from_env() -> Result<Self, StripeError> {
        let secret_key =
            std::env::var("STRIPE_SECRET_KEY").map_err(|_| StripeError::MissingApiKey)?;
        Ok(Self {
            http: reqwest::Client::new(),
            secret_key,
        })
    }

    async fn post_form(
        &self,
        path: &str,
        form: &[(&str, String)],
    ) -> Result<serde_json::Value, StripeError> {
        let resp = self
            .http
            .post(format!("{STRIPE_API_BASE}{path}"))
            .basic_auth(&self.secret_key, None::<&str>)
            .header("Stripe-Version", STRIPE_VERSION)
            .form(form)
            .send()
            .await
            .map_err(|e| StripeError::Transport(e.to_string()))?;
        Self::read_json(resp).await
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value, StripeError> {
        let resp = self
            .http
            .get(format!("{STRIPE_API_BASE}{path}"))
            .basic_auth(&self.secret_key, None::<&str>)
            .header("Stripe-Version", STRIPE_VERSION)
            .send()
            .await
            .map_err(|e| StripeError::Transport(e.to_string()))?;
        Self::read_json(resp).await
    }

    async fn delete(&self, path: &str) -> Result<serde_json::Value, StripeError> {
        let resp = self
            .http
            .delete(format!("{STRIPE_API_BASE}{path}"))
            .basic_auth(&self.secret_key, None::<&str>)
            .header("Stripe-Version", STRIPE_VERSION)
            .send()
            .await
            .map_err(|e| StripeError::Transport(e.to_string()))?;
        Self::read_json(resp).await
    }

    async fn read_json(resp: reqwest::Response) -> Result<serde_json::Value, StripeError> {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| StripeError::Transport(e.to_string()))?;
        if !status.is_success() {
            return Err(StripeError::Api {
                status: status.as_u16(),
                body,
            });
        }
        serde_json::from_str(&body).map_err(|e| StripeError::Decode(e.to_string()))
    }

    /// Creates the "Basic subscription" product together with its recurring
    /// $5/month Price, tagged with the digital-goods tax code so Stripe can
    /// calculate tax under Managed Payments. Returns both identifiers; the Price
    /// id is what the running service needs in `STRIPE_PRICE_ID`.
    pub async fn create_subscription_product(&self) -> Result<SubscriptionProduct, StripeError> {
        let form = [
            ("name", "Montly Subscription".to_string()),
            (
                "description",
                "Grants access to the febudget.com platform on a month to month basis".to_string(),
            ),
            ("tax_code", "txcd_10103100".to_string()),
            ("default_price_data[unit_amount]", "500".to_string()),
            ("default_price_data[currency]", "usd".to_string()),
            (
                "default_price_data[recurring][interval]",
                "month".to_string(),
            ),
        ];
        let body = self.post_form("/v1/products", &form).await?;
        let product_id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StripeError::Decode("product response had no id".to_string()))?
            .to_string();
        let price_id = body
            .get("default_price")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                StripeError::Decode("product response had no default_price".to_string())
            })?
            .to_string();
        Ok(SubscriptionProduct {
            product_id,
            price_id,
        })
    }

    /// Opens a subscription-mode Checkout Session for `price_id`, letting Stripe
    /// run the payment as merchant of record (`managed_payments[enabled]=true`).
    /// `client_reference_id` is echoed back on the webhook so we can find the
    /// account that paid.
    pub async fn create_checkout_session(
        &self,
        price_id: &str,
        client_reference_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, StripeError> {
        let form = [
            ("mode", "subscription".to_string()),
            ("line_items[0][price]", price_id.to_string()),
            ("line_items[0][quantity]", "1".to_string()),
            ("managed_payments[enabled]", "true".to_string()),
            ("client_reference_id", client_reference_id.to_string()),
            // Stamp the account onto the subscription itself so later
            // `customer.subscription.*` webhooks (which don't carry
            // `client_reference_id`) can still be routed back to this database.
            (
                "subscription_data[metadata][hashed_email]",
                client_reference_id.to_string(),
            ),
            ("success_url", success_url.to_string()),
            ("cancel_url", cancel_url.to_string()),
        ];
        let body = self.post_form("/v1/checkout/sessions", &form).await?;
        let id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StripeError::Decode("session response had no id".to_string()))?
            .to_string();
        let url = body
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StripeError::Decode("session response had no url".to_string()))?
            .to_string();
        Ok(CheckoutSession { id, url })
    }

    /// Reads the paid-through date off a subscription so entitlement checks can
    /// run without calling Stripe. Best-effort: the field moved onto the
    /// subscription item in recent API versions, so both spots are consulted,
    /// and `None` simply means "leave the stored value alone".
    pub async fn subscription_period_end(&self, subscription_id: &str) -> Option<String> {
        let body = self
            .get(&format!("/v1/subscriptions/{subscription_id}"))
            .await
            .ok()?;
        let unix = body
            .get("current_period_end")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                body.get("items")?
                    .get("data")?
                    .get(0)?
                    .get("current_period_end")?
                    .as_i64()
            })?;
        chrono::DateTime::from_timestamp(unix, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }

    /// Tells Stripe to stop the subscription renewing. It stays active (and
    /// usable) until the current paid period ends, then Stripe closes it and
    /// sends `customer.subscription.deleted`. Returns the updated subscription so
    /// the caller can mirror the cancel flag and paid-through date locally.
    pub async fn cancel_subscription_at_period_end(
        &self,
        subscription_id: &str,
    ) -> Result<SubscriptionLifecycle, StripeError> {
        let body = self
            .post_form(
                &format!("/v1/subscriptions/{subscription_id}"),
                &[("cancel_at_period_end", "true".to_string())],
            )
            .await?;
        serde_json::from_value(body).map_err(|e| StripeError::Decode(e.to_string()))
    }

    /// Cancels a subscription immediately, with no remaining paid period. Used
    /// when the account itself is being deleted, so there is nobody left to keep
    /// access for. A subscription Stripe will not cancel because it is already
    /// canceled or unknown (`400`/`404`) is treated as success.
    pub async fn cancel_subscription_now(&self, subscription_id: &str) -> Result<(), StripeError> {
        match self
            .delete(&format!("/v1/subscriptions/{subscription_id}"))
            .await
        {
            Ok(_) => Ok(()),
            Err(StripeError::Api {
                status: 400 | 404, ..
            }) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// The product/price pair created by [`StripeClient::create_subscription_product`].
#[derive(Debug)]
pub struct SubscriptionProduct {
    pub product_id: String,
    pub price_id: String,
}

/// A Checkout Session the customer can be redirected to.
#[derive(Debug)]
pub struct CheckoutSession {
    pub id: String,
    pub url: String,
}

/// A verified webhook event. Only the envelope is typed; `data.object` is left
/// as raw JSON for the caller to interpret per `event_type`.
#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: WebhookData,
}

#[derive(Debug, Deserialize)]
pub struct WebhookData {
    pub object: serde_json::Value,
}

/// The fields we care about on a `checkout.session.completed` object.
#[derive(Debug, Deserialize)]
pub struct CompletedCheckoutSession {
    pub id: String,
    #[serde(default)]
    pub client_reference_id: Option<String>,
    #[serde(default)]
    pub customer: Option<String>,
    #[serde(default)]
    pub subscription: Option<String>,
    #[serde(default)]
    pub payment_intent: Option<String>,
}

/// The fields we read off a `customer.subscription.*` webhook object (and the
/// response to a cancel call, which has the same shape).
#[derive(Debug, Deserialize)]
pub struct SubscriptionLifecycle {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub cancel_at_period_end: bool,
    #[serde(default)]
    pub current_period_end: Option<i64>,
    /// Left raw: recent API versions moved `current_period_end` onto the items.
    #[serde(default)]
    pub items: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl SubscriptionLifecycle {
    /// The account this subscription belongs to, stamped into `metadata` when the
    /// Checkout Session was created. `None` for subscriptions made before that
    /// stamping existed.
    pub fn hashed_email(&self) -> Option<&str> {
        self.metadata.get("hashed_email").map(String::as_str)
    }

    /// Paid-through date formatted the way the `subscription` table stores
    /// timestamps, checking both the top-level field and the per-item location.
    pub fn period_end_rfc3339(&self) -> Option<String> {
        let unix = self.current_period_end.or_else(|| {
            self.items
                .as_ref()?
                .get("data")?
                .get(0)?
                .get("current_period_end")?
                .as_i64()
        })?;
        chrono::DateTime::from_timestamp(unix, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
    }

    /// True when the subscription is over for good, not merely set to cancel.
    pub fn has_ended(&self) -> bool {
        matches!(
            self.status.as_deref(),
            Some("canceled") | Some("incomplete_expired")
        )
    }
}

/// Verifies the `Stripe-Signature` header against `secret` (Stripe's scheme:
/// HMAC-SHA256 over `"{timestamp}.{payload}"`) and, on success, parses the
/// event envelope. Mirrors `stripe.Webhook.constructEvent` in the SDKs.
pub fn construct_event(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<WebhookEvent, StripeError> {
    verify_signature(payload, signature_header, secret)?;
    serde_json::from_slice(payload).map_err(|e| StripeError::Decode(e.to_string()))
}

fn verify_signature(
    payload: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<(), StripeError> {
    let mut timestamp: Option<i64> = None;
    let mut signatures: Vec<&str> = Vec::new();
    for part in signature_header.split(',') {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| StripeError::BadSignature("malformed header".to_string()))?;
        match key.trim() {
            "t" => timestamp = value.trim().parse().ok(),
            "v1" => signatures.push(value.trim()),
            _ => {}
        }
    }

    let timestamp =
        timestamp.ok_or_else(|| StripeError::BadSignature("no timestamp in header".to_string()))?;
    if signatures.is_empty() {
        return Err(StripeError::BadSignature(
            "no v1 signatures in header".to_string(),
        ));
    }

    let age = (chrono::Utc::now().timestamp() - timestamp).abs();
    if age > WEBHOOK_TOLERANCE_SECONDS {
        return Err(StripeError::BadSignature(format!(
            "timestamp outside tolerance ({age}s)"
        )));
    }

    let mut signed_payload = timestamp.to_string().into_bytes();
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(payload);
    let expected = hex_lower(&hmac_sha256(secret.as_bytes(), &signed_payload));

    if signatures
        .iter()
        .any(|sig| constant_time_eq(sig.as_bytes(), expected.as_bytes()))
    {
        Ok(())
    } else {
        Err(StripeError::BadSignature(
            "no signature matched".to_string(),
        ))
    }
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts keys of any length");
    mac.update(message);
    mac.finalize().into_bytes().into()
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        out.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
    }
    out
}

/// Length-independent, non-short-circuiting byte comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // Signature fixture from Stripe's own webhook-signing documentation.
    const SECRET: &str = "whsec_test_secret";

    fn sign(payload: &[u8], timestamp: i64) -> String {
        let mut signed = timestamp.to_string().into_bytes();
        signed.push(b'.');
        signed.extend_from_slice(payload);
        let sig = hex_lower(&hmac_sha256(SECRET.as_bytes(), &signed));
        format!("t={timestamp},v1={sig}")
    }

    #[test]
    fn accepts_a_correctly_signed_recent_payload() {
        let payload = br#"{"id":"evt_1","type":"checkout.session.completed","data":{"object":{"id":"cs_1"}}}"#;
        let header = sign(payload, chrono::Utc::now().timestamp());
        let event = construct_event(payload, &header, SECRET).expect("should verify");
        assert_eq!(event.id, "evt_1");
        assert_eq!(event.event_type, "checkout.session.completed");
    }

    #[test]
    fn rejects_a_tampered_payload() {
        let payload = br#"{"id":"evt_1","type":"checkout.session.completed","data":{"object":{}}}"#;
        let header = sign(payload, chrono::Utc::now().timestamp());
        let tampered =
            br#"{"id":"evt_2","type":"checkout.session.completed","data":{"object":{}}}"#;
        assert!(matches!(
            construct_event(tampered, &header, SECRET),
            Err(StripeError::BadSignature(_))
        ));
    }

    #[test]
    fn rejects_a_stale_timestamp() {
        let payload = br#"{"id":"evt_1","type":"x","data":{"object":{}}}"#;
        let header = sign(payload, chrono::Utc::now().timestamp() - 10_000);
        assert!(matches!(
            construct_event(payload, &header, SECRET),
            Err(StripeError::BadSignature(_))
        ));
    }

    #[test]
    fn rejects_the_wrong_secret() {
        let payload = br#"{"id":"evt_1","type":"x","data":{"object":{}}}"#;
        let header = sign(payload, chrono::Utc::now().timestamp());
        assert!(matches!(
            construct_event(payload, &header, "whsec_not_it"),
            Err(StripeError::BadSignature(_))
        ));
    }

    #[test]
    fn parses_the_completed_session_object() {
        let object = serde_json::json!({
            "id": "cs_test_123",
            "client_reference_id": "abc123hash",
            "customer": "cus_123",
            "subscription": "sub_123",
            "payment_intent": null,
        });
        let session: CompletedCheckoutSession = serde_json::from_value(object).unwrap();
        assert_eq!(session.client_reference_id.as_deref(), Some("abc123hash"));
        assert_eq!(session.subscription.as_deref(), Some("sub_123"));
        assert_eq!(session.payment_intent, None);
    }

    #[test]
    fn parses_a_subscription_lifecycle_object() {
        let object = serde_json::json!({
            "id": "sub_123",
            "status": "active",
            "cancel_at_period_end": true,
            "metadata": { "hashed_email": "abc123hash" },
            "items": { "data": [{ "current_period_end": 1_800_000_000 }] },
        });
        let sub: SubscriptionLifecycle = serde_json::from_value(object).unwrap();
        assert_eq!(sub.id, "sub_123");
        assert!(sub.cancel_at_period_end);
        assert!(!sub.has_ended());
        assert_eq!(sub.hashed_email(), Some("abc123hash"));
        let expected = chrono::DateTime::from_timestamp(1_800_000_000, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        assert_eq!(sub.period_end_rfc3339(), Some(expected));
    }

    #[test]
    fn subscription_lifecycle_reads_top_level_period_end_and_detects_end() {
        let object = serde_json::json!({
            "id": "sub_9",
            "status": "canceled",
            "current_period_end": 1_800_000_000,
        });
        let sub: SubscriptionLifecycle = serde_json::from_value(object).unwrap();
        assert!(sub.has_ended());
        assert!(!sub.cancel_at_period_end);
        assert_eq!(sub.hashed_email(), None);
        let expected = chrono::DateTime::from_timestamp(1_800_000_000, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        assert_eq!(sub.period_end_rfc3339(), Some(expected));
    }
}
