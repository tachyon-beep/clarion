//! JSON-RPC 2.0 protocol types for the Clarion plugin host.
//!
//! # Design choice: Option B — struct-per-method with manual dispatch
//!
//! Each method has its own typed `Params` and `Result` struct. A top-level
//! `IncomingMessage` enum dispatches on the presence of `"id"` vs `"method"` in
//! the raw JSON. This keeps the JSON-RPC 2.0 envelope (`jsonrpc`, `id`) cleanly
//! separate from the method-specific payload, matching the wire spec exactly:
//!
//! ```json
//! {"jsonrpc":"2.0","method":"initialize","params":{...},"id":1}
//! ```
//!
//! Option A (tagged enum) would conflict with the outer envelope because
//! `#[serde(tag = "method", content = "params")]` embeds `method` inside the
//! `params` layer rather than at the top level where JSON-RPC 2.0 requires it.
//!
//! # Sprint 1 scope
//!
//! Only the five L4 methods are typed here:
//! - `initialize` (request/response)
//! - `initialized` (notification — no id, no response expected)
//! - `analyze_file` (request/response)
//! - `shutdown` (request/response — empty params and result)
//! - `exit` (notification — no id, no response expected)
//!
//! Entity typing for `AnalyzeFileResult.entities` is deferred to Task 6
//! (ontology boundary enforcement). The field is `Vec<serde_json::Value>` as a
//! deliberate placeholder.
//!
//! `IncomingMessage` covers only plugin-to-core traffic (responses and
//! notifications) because Sprint 1's walking skeleton is core-initiated only.
//! Full bidirectional dispatch is Task 6's concern.

use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;

// ── JSON-RPC version wrapper ──────────────────────────────────────────────────

/// Wrapper that (de)serialises to/from the literal string `"2.0"`.
///
/// Serde impl rejects any other value with a descriptive error, catching
/// wire-format corruption early.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcVersion;

impl Serialize for JsonRpcVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s == "2.0" {
            Ok(JsonRpcVersion)
        } else {
            Err(de::Error::custom(format!(
                "unsupported JSON-RPC version {s:?}; expected \"2.0\""
            )))
        }
    }
}

// ── Envelope types ────────────────────────────────────────────────────────────

/// JSON-RPC 2.0 request envelope (core → plugin).
///
/// Wire shape: `{"jsonrpc":"2.0","method":"...","params":{...},"id":1}`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestEnvelope {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    pub params: Value,
    pub id: i64,
}

/// JSON-RPC 2.0 notification envelope (no `id`, no response expected).
///
/// Wire shape: `{"jsonrpc":"2.0","method":"...","params":{...}}`
///
/// Used for `initialized` and `exit`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationEnvelope {
    pub jsonrpc: JsonRpcVersion,
    pub method: String,
    pub params: Value,
}

/// JSON-RPC 2.0 response envelope (plugin → core).
///
/// Wire shape (success): `{"jsonrpc":"2.0","result":{...},"id":1}`
/// Wire shape (error):   `{"jsonrpc":"2.0","error":{...},"id":1}`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseEnvelope {
    pub jsonrpc: JsonRpcVersion,
    pub id: i64,
    #[serde(flatten)]
    pub payload: ResponsePayload,
}

/// The result-or-error payload inside a [`ResponseEnvelope`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponsePayload {
    Result(Value),
    Error(ProtocolError),
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// ── Method-level param and result structs ─────────────────────────────────────

// ── initialize ────────────────────────────────────────────────────────────────

/// Params for `initialize` (core → plugin).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializeParams {
    /// Protocol version the core speaks, e.g. `"1.0"`.
    pub protocol_version: String,
    /// Absolute path to the project root being analysed.
    pub project_root: PathBuf,
}

/// Result for `initialize` (plugin → core).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResult {
    /// Plugin display name.
    pub name: String,
    /// Plugin version (semver).
    pub version: String,
    /// Ontology version (semver); used in ADR-007 cache keying.
    pub ontology_version: String,
    /// Opaque capability advertisement. Shape is left to the plugin;
    /// the core forwards it without interpretation in Sprint 1.
    pub capabilities: Value,
}

// ── initialized ───────────────────────────────────────────────────────────────

/// Notification params for `initialized` (core → plugin).
///
/// Deliberately empty: the notification carries no payload. This struct exists
/// for serialisation consistency — every envelope's `params` field is a JSON
/// object, even when empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializedNotification;

impl InitializedNotification {
    /// Serialise to `serde_json::Value::Object({})`.
    pub fn to_value() -> Value {
        serde_json::json!({})
    }
}

// ── analyze_file ──────────────────────────────────────────────────────────────

/// Params for `analyze_file` (core → plugin).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalyzeFileParams {
    /// Path to the file to analyse (relative or absolute; plugin resolves).
    pub file_path: PathBuf,
}

/// Result for `analyze_file` (plugin → core).
///
/// `entities` is `Vec<serde_json::Value>` as a Sprint 1 placeholder.
/// Task 6 (ontology boundary enforcement) will introduce a typed `Entity`
/// struct and replace this field. The `Vec<Value>` shape matches the wire
/// contract without requiring Task 2 to know the entity schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyzeFileResult {
    /// Extracted entities. Element shape is the plugin's concern; the core
    /// stores them opaquely until Task 6 introduces the typed ontology layer.
    pub entities: Vec<Value>,
}

// ── shutdown ──────────────────────────────────────────────────────────────────

/// Params for `shutdown` (core → plugin).
///
/// Empty by design — the message carries no payload. The struct exists for
/// serialisation consistency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShutdownParams;

/// Result for `shutdown` (plugin → core).
///
/// JSON-RPC 2.0 requires a non-null response to a request, so we use an empty
/// result object `{}` rather than `null`. This signals the plugin has cleanly
/// acknowledged the shutdown request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShutdownResult;

// ── exit ──────────────────────────────────────────────────────────────────────

/// Notification params for `exit` (core → plugin).
///
/// Deliberately empty: this is a forceful termination signal sent after the
/// plugin has replied to `shutdown`. No response is expected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExitNotification;

impl ExitNotification {
    /// Serialise to `serde_json::Value::Object({})`.
    pub fn to_value() -> Value {
        serde_json::json!({})
    }
}

// ── Helpers for building common envelopes ─────────────────────────────────────

/// Convenience: build a `RequestEnvelope` from typed params.
///
/// # Panics
///
/// Panics if serialisation of `params` fails. This should never happen for the
/// well-formed Sprint 1 param types. Callers that need explicit error handling
/// should construct [`RequestEnvelope`] directly.
pub fn make_request<P: Serialize>(method: &str, params: &P, id: i64) -> RequestEnvelope {
    RequestEnvelope {
        jsonrpc: JsonRpcVersion,
        method: method.to_owned(),
        params: serde_json::to_value(params).expect("params serialisation must not fail"),
        id,
    }
}

/// Convenience: build a `NotificationEnvelope` from typed params.
///
/// # Panics
///
/// Panics if serialisation of `params` fails. This should never happen for the
/// well-formed Sprint 1 param types. Callers that need explicit error handling
/// should construct [`NotificationEnvelope`] directly.
pub fn make_notification<P: Serialize>(method: &str, params: &P) -> NotificationEnvelope {
    NotificationEnvelope {
        jsonrpc: JsonRpcVersion,
        method: method.to_owned(),
        params: serde_json::to_value(params).expect("params serialisation must not fail"),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Protocol test 1: round-trip InitializeParams ──────────────────────────

    #[test]
    fn proto_01_initialize_params_round_trips() {
        let params = InitializeParams {
            protocol_version: "1.0".to_owned(),
            project_root: PathBuf::from("/home/user/project"),
        };
        let json = serde_json::to_string(&params).expect("serialise");
        let back: InitializeParams = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(params, back);
    }

    // ── Protocol test 2: JsonRpcVersion rejects wrong versions ────────────────

    #[test]
    fn proto_02_json_rpc_version_rejects_non_2_0() {
        // "3.0"
        let bad_30 = r#"{"jsonrpc":"3.0","method":"initialize","params":{},"id":1}"#;
        let err = serde_json::from_str::<RequestEnvelope>(bad_30);
        assert!(err.is_err(), "should reject version 3.0");

        // "1.0"
        let bad_10 = r#"{"jsonrpc":"1.0","method":"initialize","params":{},"id":1}"#;
        let err = serde_json::from_str::<RequestEnvelope>(bad_10);
        assert!(err.is_err(), "should reject version 1.0");

        // "" (empty)
        let bad_empty = r#"{"jsonrpc":"","method":"initialize","params":{},"id":1}"#;
        let err = serde_json::from_str::<RequestEnvelope>(bad_empty);
        assert!(err.is_err(), "should reject empty version string");
    }

    // ── Protocol test 3: request envelope serialises to expected shape ─────────

    #[test]
    fn proto_03_request_envelope_serialises_correctly() {
        let params = InitializeParams {
            protocol_version: "1.0".to_owned(),
            project_root: PathBuf::from("/proj"),
        };
        let env = make_request("initialize", &params, 1);
        let json = serde_json::to_string(&env).expect("serialise");
        let v: Value = serde_json::from_str(&json).expect("parse");

        assert_eq!(v["jsonrpc"], "2.0", "jsonrpc field must be \"2.0\"");
        assert_eq!(v["method"], "initialize", "method field");
        assert_eq!(v["id"], 1, "id field");
        // params object is present and contains protocol_version
        assert_eq!(v["params"]["protocol_version"], "1.0");
    }

    // ── Protocol test 4: response envelope — both payload variants ────────────

    #[test]
    fn proto_04_response_envelope_result_variant_round_trips() {
        let env = ResponseEnvelope {
            jsonrpc: JsonRpcVersion,
            id: 7,
            payload: ResponsePayload::Result(serde_json::json!({"ok": true})),
        };
        let json = serde_json::to_string(&env).expect("serialise");
        let v: Value = serde_json::from_str(&json).expect("parse");

        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 7);
        assert_eq!(v["result"]["ok"], true);
        assert!(v.get("error").is_none(), "no error key on success response");

        // Full round-trip
        let back: ResponseEnvelope = serde_json::from_str(&json).expect("deserialise back");
        assert_eq!(env, back);
    }

    #[test]
    fn proto_04_response_envelope_error_variant_round_trips() {
        let env = ResponseEnvelope {
            jsonrpc: JsonRpcVersion,
            id: 7,
            payload: ResponsePayload::Error(ProtocolError {
                code: -32_600,
                message: "Invalid Request".to_owned(),
                data: None,
            }),
        };
        let json = serde_json::to_string(&env).expect("serialise");
        let v: Value = serde_json::from_str(&json).expect("parse");

        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 7);
        assert_eq!(v["error"]["code"], -32_600);
        assert_eq!(v["error"]["message"], "Invalid Request");
        assert!(v.get("result").is_none(), "no result key on error response");

        // Full round-trip
        let back: ResponseEnvelope = serde_json::from_str(&json).expect("deserialise back");
        assert_eq!(env, back);
    }

    // ── Protocol test 5: notification envelope has no id field ────────────────

    #[test]
    fn proto_05_notification_envelope_has_no_id_field() {
        let env = make_notification("initialized", &serde_json::json!({}));
        let json = serde_json::to_string(&env).expect("serialise");
        let v: Value = serde_json::from_str(&json).expect("parse");

        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "initialized");
        assert!(
            v.get("id").is_none(),
            "notification must not carry an id field"
        );
    }

    // ── Protocol test 6: round-trip for all 5 methods' param+result structs ───

    #[test]
    fn proto_06_all_method_param_result_structs_round_trip() {
        // initialize params
        let p = InitializeParams {
            protocol_version: "1.0".to_owned(),
            project_root: PathBuf::from("/proj"),
        };
        let back: InitializeParams =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(p, back);

        // initialize result
        let r = InitializeResult {
            name: "clarion-plugin-python".to_owned(),
            version: "0.1.0".to_owned(),
            ontology_version: "0.1.0".to_owned(),
            capabilities: serde_json::json!({"wardline_aware": true}),
        };
        let back: InitializeResult =
            serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(r, back);

        // initialized notification — serialises to empty object
        let notif_val = InitializedNotification::to_value();
        assert_eq!(notif_val, serde_json::json!({}));

        // analyze_file params
        let p = AnalyzeFileParams {
            file_path: PathBuf::from("src/main.py"),
        };
        let back: AnalyzeFileParams =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(p, back);

        // analyze_file result
        let r = AnalyzeFileResult {
            entities: vec![serde_json::json!({"kind": "function", "name": "main"})],
        };
        let back: AnalyzeFileResult =
            serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(r, back);

        // shutdown params (empty)
        let p = ShutdownParams;
        let back: ShutdownParams =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(p, back);

        // shutdown result (empty)
        let r = ShutdownResult;
        let back: ShutdownResult =
            serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(r, back);

        // exit notification — serialises to empty object
        let notif_val = ExitNotification::to_value();
        assert_eq!(notif_val, serde_json::json!({}));
    }
}
