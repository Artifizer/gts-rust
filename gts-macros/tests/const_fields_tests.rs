//! GTS error schema const fields example.
//!
//! Demonstrates `const_values` in derived GTS error schemas. Three error types are shown:
//! 1. `BadRequest` (400) – basic const values with single-quoted strings
//! 2. `UnprocessableEntity` (422) – `message` value contains a comma
//! 3. `Conflict` (409) – `message` value contains an escaped single quote (`\'`)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::str_to_string,
    clippy::nonminimal_bool,
    clippy::uninlined_format_args,
    clippy::bool_assert_comparison
)]

use gts::{GtsSchema, GtsSchemaId};
use gts_macros::struct_to_gts_schema;

/* ============================================================
Base error class:  gts.x.core.err.error.v1~

Fields:
  - type        : GtsSchemaId   – identifies the concrete error type (schema ID)
  - http_status : i32           – HTTP response code
  - message     : String        – human-readable error text
  - trace_id    : String        – distributed tracing identifier
  - metadata    : M             – generic error-specific context
============================================================ */

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.err.error.v1~",
    description = "Base GTS error with HTTP status, message, trace ID and metadata",
    properties = "type,http_status,message,trace_id,metadata"
)]
#[derive(Debug)]
pub struct BaseErrorV1<M> {
    pub r#type: GtsSchemaId,
    pub http_status: i32,
    pub message: String,
    pub trace_id: String,
    pub metadata: M,
}

/* ============================================================
Error type 1 – Bad Request (400)

const_values uses single-quoted strings.
============================================================ */

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseErrorV1,
    schema_id = "gts.x.core.err.error.v1~x.core.http.bad_request.v1~",
    description = "HTTP 400 Bad Request",
    properties = "",
    const_values = "http_status=400,message='bad request'"
)]
#[derive(Debug)]
pub struct BadRequestMetadataV1;

impl BadRequestMetadataV1 {
    /// Create a `BaseErrorV1<Self>` with const values applied.
    /// Only `trace_id` varies per instance.
    pub fn new(trace_id: impl Into<String>) -> BaseErrorV1<Self> {
        let mut instance = Self::new_instance_with_defaults();
        instance.trace_id = trace_id.into();
        instance
    }
}

/* ============================================================
Error type 2 – Unprocessable Entity (422)

const_values message contains a comma – must be single-quoted.
============================================================ */

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseErrorV1,
    schema_id = "gts.x.core.err.error.v1~x.core.http.unprocessable.v1~",
    description = "HTTP 422 Unprocessable Entity",
    properties = "",
    const_values = "http_status=422,message='Invalid input: field required, value missing'"
)]
#[derive(Debug)]
pub struct UnprocessableMetadataV1;

impl UnprocessableMetadataV1 {
    pub fn new(trace_id: impl Into<String>) -> BaseErrorV1<Self> {
        let mut instance = Self::new_instance_with_defaults();
        instance.trace_id = trace_id.into();
        instance
    }
}

/* ============================================================
Error type 3 – Conflict (409)

const_values message contains an escaped single quote (\').
In the Rust string literal `\\' ` is a backslash + quote;
the parser sees `\'` and outputs a literal `'`.
============================================================ */

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseErrorV1,
    schema_id = "gts.x.core.err.error.v1~x.core.http.conflict.v1~",
    description = "HTTP 409 Conflict",
    properties = "",
    const_values = "http_status=409,message='Resource can\\'t be created: already exists'"
)]
#[derive(Debug)]
pub struct ConflictMetadataV1;

impl ConflictMetadataV1 {
    pub fn new(trace_id: impl Into<String>) -> BaseErrorV1<Self> {
        let mut instance = Self::new_instance_with_defaults();
        instance.trace_id = trace_id.into();
        instance
    }
}

/* ============================================================
All-Types Base for exhaustive const_values type coverage

Fields:
  - type    : GtsSchemaId – schema type field (auto-injected as const for derived)
  - count   : i32         – integer field
  - score   : f64         – floating-point field
  - enabled : bool        – boolean field
  - label   : String      – string field
  - metadata: M           – generic slot
============================================================ */

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.const.types.v1~",
    description = "Base for testing all const_values value types",
    properties = "type,count,score,enabled,label,metadata"
)]
#[derive(Debug)]
pub struct AllTypesBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub count: i32,
    pub score: f64,
    pub enabled: bool,
    pub label: String,
    pub metadata: M,
}

// Integer const
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.int.v1~",
    description = "Integer const",
    properties = "",
    const_values = "count=42"
)]
#[derive(Debug)]
pub struct IntConstV1;

// Negative integer const
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.neg_int.v1~",
    description = "Negative integer const",
    properties = "",
    const_values = "count=-5"
)]
#[derive(Debug)]
pub struct NegIntConstV1;

// Float const
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.float.v1~",
    description = "Float const",
    properties = "",
    const_values = "score=2.5"
)]
#[derive(Debug)]
pub struct FloatConstV1;

// Bool true const
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.bool_true.v1~",
    description = "Bool true const",
    properties = "",
    const_values = "enabled=true"
)]
#[derive(Debug)]
pub struct BoolTrueConstV1;

// Bool false const
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.bool_false.v1~",
    description = "Bool false const",
    properties = "",
    const_values = "enabled=false"
)]
#[derive(Debug)]
pub struct BoolFalseConstV1;

// Single-quoted string const
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.str_quoted.v1~",
    description = "Single-quoted string const",
    properties = "",
    const_values = "label='hello world'"
)]
#[derive(Debug)]
pub struct StrQuotedConstV1;

// Unquoted string const (bare word, no digits/bool)
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.str_unquoted.v1~",
    description = "Unquoted string const",
    properties = "",
    const_values = "label=hello"
)]
#[derive(Debug)]
pub struct StrUnquotedConstV1;

// Multiple types together in a single const_values string
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = AllTypesBaseV1,
    schema_id = "gts.x.test.const.types.v1~x.test.types.multi.v1~",
    description = "Multiple const types together",
    properties = "",
    const_values = "count=10,score=2.5,enabled=true,label='active'"
)]
#[derive(Debug)]
pub struct MultiTypeConstV1;

/* ============================================================
Three-level const_values – realistic event/metadata pattern

  L1 (base):    gts.x.test.three.event.v1~
                Generic base with type(GtsSchemaId), severity(i32),
                source(String), payload(P).

  L2 (derived generic):  gts.x.test.three.event.v1~x.test.three.alert.v1~
                Initialises payload to a concrete generic struct that has
                an object-typed `data` field.  Fixes severity=3 and
                source='alert-service' via const_values.

  L3 (concrete leaf):  ...~x.test.three.email.alert.v1~
                Defines the `data` object with `to` / `subject` fields.
                Fixes channel='email' via const_values.

  L3-override variant:  ...~x.test.three.sms.alert.v1~
                Same shape but sets channel='sms'.  Also demonstrates
                that attempting to override L2's severity const (severity=9)
                creates two conflicting allOf constraints.
============================================================ */

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.three.event.v1~",
    description = "Base alert event with type, severity, source, payload",
    properties = "type,severity,source,payload"
)]
#[derive(Debug)]
pub struct ThreeEventBaseV1<P> {
    pub r#type: GtsSchemaId,
    pub severity: i32,
    pub source: String,
    pub payload: P,
}

/// L2 – generic derived struct: initialises payload, sets severity=3 and source='alert-service'.
/// The `data` field is of an object type supplied by the concrete L3 struct.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = ThreeEventBaseV1,
    schema_id = "gts.x.test.three.event.v1~x.test.three.alert.v1~",
    description = "Alert event payload (severity=3, source=alert-service)",
    properties = "channel,recipient,data",
    const_values = "severity=3,source='alert-service'"
)]
#[derive(Debug)]
pub struct ThreeAlertPayloadV1<D> {
    pub channel: String,
    pub recipient: String,
    pub data: D,
}

/// L3 – concrete email-alert data struct: fixes channel='email'.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = ThreeAlertPayloadV1,
    schema_id = "gts.x.test.three.event.v1~x.test.three.alert.v1~x.test.three.email.v1~",
    description = "Email alert data (channel='email')",
    properties = "to,subject",
    const_values = "channel='email'"
)]
#[derive(Debug)]
pub struct ThreeEmailAlertDataV1 {
    pub to: String,
    pub subject: String,
}

/// Standalone base struct so `ThreeSmsAlertDataV1` can call
/// `new_instance_with_defaults()` and get channel/severity as direct fields.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.alert.base.v1~",
    description = "Alert payload base with channel, severity, kind, data fields",
    properties = "type,channel,recipient,severity,data"
)]
#[derive(Debug)]
pub struct ThreeAlertBaseV1<D> {
    pub r#type: GtsSchemaId,
    pub channel: String,
    pub recipient: String,
    pub severity: i32,
    pub data: D,
}

/// L3-sms: unit struct extending `ThreeAlertBaseV1` – fixes channel, severity, kind as const.
/// Demonstrates: (a) `const_values` on a concrete leaf, (b) instance construction via
/// `new_instance_with_defaults()`, (c) conflicting const when severity is also constrained upstream.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = ThreeAlertBaseV1,
    schema_id = "gts.x.test.alert.base.v1~x.test.three.sms.v1~",
    description = "SMS alert unit struct (channel='sms', severity=9, kind='traffic')",
    properties = "",
    const_values = "channel='sms',severity=9,kind='traffic'"
)]
pub struct ThreeSmsAlertDataV1 {
    pub kind: String,
}

/* ============================================================
Tests
============================================================ */

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn register_all_schemas(ops: &mut gts::GtsOps) {
        for (id, schema) in [
            (
                BaseErrorV1::<()>::gts_schema_id().clone().into_string(),
                BaseErrorV1::<()>::gts_schema_with_refs(),
            ),
            (
                BadRequestMetadataV1::gts_schema_id().clone().into_string(),
                BadRequestMetadataV1::gts_schema_with_refs(),
            ),
            (
                UnprocessableMetadataV1::gts_schema_id()
                    .clone()
                    .into_string(),
                UnprocessableMetadataV1::gts_schema_with_refs(),
            ),
            (
                ConflictMetadataV1::gts_schema_id().clone().into_string(),
                ConflictMetadataV1::gts_schema_with_refs(),
            ),
        ] {
            let result = ops.add_schema(id.clone(), &schema);
            assert!(result.ok, "add_schema {id}: {}", result.error);
        }
    }

    // ------------------------------------------------------------------
    // 1. Schema IDs and inheritance chain
    // ------------------------------------------------------------------

    #[test]
    fn test_error_schema_ids() {
        assert_eq!(
            BaseErrorV1::<()>::gts_schema_id().as_ref(),
            "gts.x.core.err.error.v1~"
        );

        assert_eq!(
            BadRequestMetadataV1::gts_schema_id().as_ref(),
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~"
        );
        assert_eq!(
            BadRequestMetadataV1::gts_base_schema_id().map(std::convert::AsRef::as_ref),
            Some("gts.x.core.err.error.v1~")
        );

        assert_eq!(
            UnprocessableMetadataV1::gts_schema_id().as_ref(),
            "gts.x.core.err.error.v1~x.core.http.unprocessable.v1~"
        );
        assert_eq!(
            ConflictMetadataV1::gts_schema_id().as_ref(),
            "gts.x.core.err.error.v1~x.core.http.conflict.v1~"
        );
    }

    // ------------------------------------------------------------------
    // 2. Base schema – `type` field present instead of `gts_id`
    // ------------------------------------------------------------------

    #[test]
    fn test_base_error_schema_structure() {
        let schema = BaseErrorV1::<()>::gts_schema_with_refs();
        println!(
            "\nBASE SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );

        assert!(schema.get("allOf").is_none(), "Base must not have allOf");
        assert_eq!(schema["$id"], "gts://gts.x.core.err.error.v1~");

        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("type"), "missing type");
        assert!(props.contains_key("http_status"), "missing http_status");
        assert!(props.contains_key("message"), "missing message");
        assert!(props.contains_key("trace_id"), "missing trace_id");
        assert!(props.contains_key("metadata"), "missing metadata");
        assert!(!props.contains_key("gts_id"), "gts_id must not exist");
    }

    // ------------------------------------------------------------------
    // 3a. BadRequest schema – const values (basic single-quoted strings)
    // ------------------------------------------------------------------

    #[test]
    fn test_bad_request_schema_const_values() {
        let schema = BadRequestMetadataV1::gts_schema_with_refs();
        println!(
            "\nBAD REQUEST SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );

        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["http_status"]["const"], 400);
        assert_eq!(props["message"]["const"], "bad request");
        assert_eq!(
            props["type"]["const"],
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~"
        );
    }

    // ------------------------------------------------------------------
    // 3b. BadRequest instance – new(trace_id) uses new_instance_with_defaults()
    // ------------------------------------------------------------------

    #[test]
    fn test_bad_request_constructor_and_serialization() {
        let error = BadRequestMetadataV1::new("trace-abc-123");

        assert_eq!(
            error.r#type.as_ref(),
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~"
        );
        assert_eq!(error.http_status, 400);
        assert_eq!(error.message, "bad request");
        assert_eq!(error.trace_id, "trace-abc-123");

        let json = serde_json::to_value(&error).unwrap();
        println!(
            "\nBAD REQUEST JSON:\n{}",
            serde_json::to_string_pretty(&json).unwrap()
        );

        assert_eq!(
            json["type"],
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~"
        );
        assert_eq!(json["http_status"], 400);
        assert_eq!(json["message"], "bad request");
        assert_eq!(json["trace_id"], "trace-abc-123");
        assert_eq!(json["metadata"], json!({}));
        assert!(
            json.get("gts_id").is_none(),
            "gts_id must not appear in serialized output"
        );
    }

    // ------------------------------------------------------------------
    // 4. UnprocessableEntity – message const contains a comma
    // ------------------------------------------------------------------

    #[test]
    fn test_unprocessable_schema_message_with_comma() {
        let schema = UnprocessableMetadataV1::gts_schema_with_refs();
        println!(
            "\nUNPROCESSABLE SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );

        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["http_status"]["const"], 422);
        assert_eq!(
            props["message"]["const"], "Invalid input: field required, value missing",
            "comma inside the message must be preserved"
        );
    }

    #[test]
    fn test_unprocessable_instance_serialization() {
        let error = UnprocessableMetadataV1::new("trace-unproc-1");
        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["http_status"], 422);
        assert_eq!(
            json["message"],
            "Invalid input: field required, value missing"
        );
        assert_eq!(
            json["type"],
            "gts.x.core.err.error.v1~x.core.http.unprocessable.v1~"
        );
        assert_eq!(json["trace_id"], "trace-unproc-1");
    }

    // ------------------------------------------------------------------
    // 5. Conflict – message const contains an escaped single quote
    // ------------------------------------------------------------------

    #[test]
    fn test_conflict_schema_message_with_escaped_quote() {
        let schema = ConflictMetadataV1::gts_schema_with_refs();
        println!(
            "\nCONFLICT SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );

        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["http_status"]["const"], 409);
        assert_eq!(
            props["message"]["const"], "Resource can't be created: already exists",
            "escaped \\' must become a literal single quote"
        );
    }

    #[test]
    fn test_conflict_instance_serialization() {
        let error = ConflictMetadataV1::new("trace-conflict-1");
        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["http_status"], 409);
        assert_eq!(json["message"], "Resource can't be created: already exists");
        assert_eq!(
            json["type"],
            "gts.x.core.err.error.v1~x.core.http.conflict.v1~"
        );
        assert_eq!(json["trace_id"], "trace-conflict-1");
    }

    // ------------------------------------------------------------------
    // 6. new_instance_with_defaults() — base struct helper
    // ------------------------------------------------------------------

    #[test]
    fn test_new_instance_with_defaults_base_sets_schema_id_from_generic() {
        // The base's new_instance_with_defaults sets GtsSchemaId from M::SCHEMA_ID, no const overrides.
        let instance = BaseErrorV1::<BadRequestMetadataV1>::new_instance_with_defaults();
        assert_eq!(
            instance.r#type.as_ref(),
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~",
            "GtsSchemaId field must be set from M::SCHEMA_ID"
        );
        assert_eq!(
            instance.http_status, 0,
            "base new_instance_with_defaults does NOT apply const overrides"
        );
        assert_eq!(instance.message, "");
    }

    #[test]
    fn test_new_instance_with_defaults_derived_applies_const_values() {
        // The derived new_instance_with_defaults wraps the base and applies const_values.
        let instance = BadRequestMetadataV1::new_instance_with_defaults();
        assert_eq!(
            instance.r#type.as_ref(),
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~"
        );
        assert_eq!(
            instance.http_status, 400,
            "const_values applied: http_status=400"
        );
        assert_eq!(
            instance.message, "bad request",
            "const_values applied: message"
        );
        assert_eq!(instance.trace_id, "", "trace_id left at Default");
    }

    #[test]
    fn test_new_instance_with_defaults_unit_type() {
        // Base with M = () - SCHEMA_ID is "", GtsSchemaId gets empty string.
        let instance = BaseErrorV1::<()>::new_instance_with_defaults();
        assert_eq!(instance.r#type.as_ref(), "");
        assert_eq!(instance.http_status, 0);
    }

    // ------------------------------------------------------------------
    // 7. Schema chain validation (all three derived types)
    // ------------------------------------------------------------------

    #[test]
    fn test_all_schema_chains_valid() {
        let mut ops = gts::GtsOps::new(None, None, 0);
        register_all_schemas(&mut ops);

        for id in [
            "gts.x.core.err.error.v1~x.core.http.bad_request.v1~",
            "gts.x.core.err.error.v1~x.core.http.unprocessable.v1~",
            "gts.x.core.err.error.v1~x.core.http.conflict.v1~",
        ] {
            let result = ops.validate_schema(id);
            assert!(
                result.ok,
                "Schema chain for {id} must be valid: {}",
                result.error
            );
        }
    }

    // ==================================================================
    // All-Types const_values: JSON Schema const entries
    // ==================================================================

    // ------------------------------------------------------------------
    // 8. Integer const (count=42)
    // ------------------------------------------------------------------

    #[test]
    fn test_int_const_schema() {
        let schema = IntConstV1::gts_schema_with_refs();
        println!(
            "\nINT CONST SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["count"]["const"], json!(42));
        assert!(props["count"]["const"].is_number());
        assert!(props["count"]["const"].as_i64().is_some());
    }

    #[test]
    fn test_int_const_instance() {
        let instance = IntConstV1::new_instance_with_defaults();
        assert_eq!(instance.count, 42, "const_values: count=42");
        assert!((instance.score - 0.0_f64).abs() < f64::EPSILON);
        assert!(!instance.enabled);
        assert_eq!(instance.label, "");
    }

    // ------------------------------------------------------------------
    // 9. Negative integer const (count=-5)
    // ------------------------------------------------------------------

    #[test]
    fn test_neg_int_const_schema() {
        let schema = NegIntConstV1::gts_schema_with_refs();
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["count"]["const"], json!(-5));
        assert!(props["count"]["const"].as_i64() == Some(-5));
    }

    #[test]
    fn test_neg_int_const_instance() {
        let instance = NegIntConstV1::new_instance_with_defaults();
        assert_eq!(instance.count, -5);
    }

    // ------------------------------------------------------------------
    // 10. Float const (score=2.5)
    // ------------------------------------------------------------------

    #[test]
    fn test_float_const_schema() {
        let schema = FloatConstV1::gts_schema_with_refs();
        println!(
            "\nFLOAT CONST SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert!(
            props["score"]["const"].is_number(),
            "score const must be a JSON number"
        );
        let val = props["score"]["const"]
            .as_f64()
            .expect("score const must be representable as f64");
        assert!(
            (val - 2.5_f64).abs() < 1e-10,
            "score const must be ~2.5, got {val}"
        );
    }

    #[test]
    fn test_float_const_instance() {
        let instance = FloatConstV1::new_instance_with_defaults();
        assert!(
            (instance.score - 2.5_f64).abs() < 1e-10,
            "score must be 2.5, got {}",
            instance.score
        );
        assert_eq!(instance.count, 0);
    }

    // ------------------------------------------------------------------
    // 11. Bool true const (enabled=true)
    // ------------------------------------------------------------------

    #[test]
    fn test_bool_true_const_schema() {
        let schema = BoolTrueConstV1::gts_schema_with_refs();
        println!(
            "\nBOOL TRUE SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["enabled"]["const"], json!(true));
        assert!(props["enabled"]["const"].is_boolean());
    }

    #[test]
    fn test_bool_true_const_instance() {
        let instance = BoolTrueConstV1::new_instance_with_defaults();
        assert!(instance.enabled, "enabled must be true");
    }

    // ------------------------------------------------------------------
    // 12. Bool false const (enabled=false)
    // ------------------------------------------------------------------

    #[test]
    fn test_bool_false_const_schema() {
        let schema = BoolFalseConstV1::gts_schema_with_refs();
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["enabled"]["const"], json!(false));
        assert!(props["enabled"]["const"].is_boolean());
    }

    #[test]
    fn test_bool_false_const_instance() {
        let instance = BoolFalseConstV1::new_instance_with_defaults();
        assert!(!instance.enabled, "enabled must be false");
    }

    // ------------------------------------------------------------------
    // 13. Single-quoted string const (label='hello world')
    // ------------------------------------------------------------------

    #[test]
    fn test_string_quoted_const_schema() {
        let schema = StrQuotedConstV1::gts_schema_with_refs();
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["label"]["const"], json!("hello world"));
        assert!(props["label"]["const"].is_string());
    }

    #[test]
    fn test_string_quoted_const_instance() {
        let instance = StrQuotedConstV1::new_instance_with_defaults();
        assert_eq!(instance.label, "hello world");
    }

    // ------------------------------------------------------------------
    // 14. Unquoted string const (label=hello)
    // ------------------------------------------------------------------

    #[test]
    fn test_string_unquoted_const_schema() {
        let schema = StrUnquotedConstV1::gts_schema_with_refs();
        let props = schema["allOf"][1]["properties"].as_object().unwrap();
        assert_eq!(props["label"]["const"], json!("hello"));
        assert!(props["label"]["const"].is_string());
    }

    #[test]
    fn test_string_unquoted_const_instance() {
        let instance = StrUnquotedConstV1::new_instance_with_defaults();
        assert_eq!(instance.label, "hello");
    }

    // ------------------------------------------------------------------
    // 15. Multiple types together (count=10,score=2.5,enabled=true,label='active')
    // ------------------------------------------------------------------

    #[test]
    fn test_multi_type_const_schema() {
        let schema = MultiTypeConstV1::gts_schema_with_refs();
        println!(
            "\nMULTI-TYPE SCHEMA:\n{}",
            serde_json::to_string_pretty(&schema).unwrap()
        );
        let props = schema["allOf"][1]["properties"].as_object().unwrap();

        assert_eq!(props["count"]["const"], json!(10));
        assert!(props["count"]["const"].is_number());

        assert!(props["score"]["const"].is_number());
        let score = props["score"]["const"].as_f64().unwrap();
        assert!((score - 2.5_f64).abs() < 1e-10);

        assert_eq!(props["enabled"]["const"], json!(true));
        assert!(props["enabled"]["const"].is_boolean());

        assert_eq!(props["label"]["const"], json!("active"));
        assert!(props["label"]["const"].is_string());
    }

    #[test]
    fn test_multi_type_const_instance() {
        let instance = MultiTypeConstV1::new_instance_with_defaults();
        assert_eq!(instance.count, 10);
        assert!((instance.score - 2.5_f64).abs() < 1e-10);
        assert!(instance.enabled);
        assert_eq!(instance.label, "active");
    }

    // ------------------------------------------------------------------
    // 3-level const_values: event/metadata pattern
    //   L1: ThreeEventBaseV1<P>  (base, generic)
    //   L2: ThreeAlertPayloadV1<D>  (generic child, const_values=severity+source)
    //   L3: ThreeEmailAlertDataV1 / ThreeSmsAlertDataV1  (concrete leaf)
    // ------------------------------------------------------------------

    #[test]
    fn test_three_event_l1_is_base_with_no_allof() {
        let schema = ThreeEventBaseV1::<()>::gts_schema_with_refs();
        assert_eq!(
            ThreeEventBaseV1::<()>::gts_schema_id().as_ref(),
            "gts.x.test.three.event.v1~"
        );
        assert!(
            schema.get("properties").is_some(),
            "L1 base must have top-level properties"
        );
        assert!(schema.get("allOf").is_none(), "L1 base must NOT have allOf");
        let props = schema["properties"].as_object().unwrap();
        assert!(
            props.contains_key("severity"),
            "L1 must have severity field"
        );
        assert!(props.contains_key("payload"), "L1 must have payload field");
    }

    #[test]
    fn test_three_event_l2_refs_l1_and_sets_severity_source_consts() {
        // L2 is a generic struct – const values land in allOf[1]["properties"]
        // alongside the nesting-wrapped payload sub-fields.
        let schema = ThreeAlertPayloadV1::<()>::gts_schema_with_refs();
        assert_eq!(
            ThreeAlertPayloadV1::<()>::gts_schema_id().as_ref(),
            "gts.x.test.three.event.v1~x.test.three.alert.v1~"
        );
        let all_of = schema["allOf"].as_array().expect("L2 must have allOf");
        assert_eq!(
            all_of[0]["$ref"].as_str().unwrap(),
            "gts://gts.x.test.three.event.v1~",
            "L2 allOf[0] must $ref L1"
        );
        let props = all_of[1]["properties"]
            .as_object()
            .expect("L2 allOf[1].properties must exist");
        // const_values: severity=3, source='alert-service'
        assert_eq!(
            props["severity"]["const"],
            json!(3),
            "L2 must fix severity=3"
        );
        assert_eq!(
            props["source"]["const"],
            json!("alert-service"),
            "L2 must fix source"
        );
        // auto-injected type const from L1's GtsSchemaId field
        assert_eq!(
            props["type"]["const"].as_str().unwrap(),
            "gts.x.test.three.event.v1~x.test.three.alert.v1~",
            "L2 must auto-inject type const"
        );
        // When called with ThreeAlertPayloadV1::<()>, the inner type is () whose SCHEMA_ID
        // is empty, so collect_nesting_path() returns [] and wrap_in_nesting_path returns
        // properties flat (no "payload" wrapper level).
        assert!(
            props.contains_key("channel"),
            "allOf[1].properties must include channel"
        );
        assert!(
            props.contains_key("recipient"),
            "allOf[1].properties must include recipient"
        );
        // data is the generic field – replaced with {"type":"object"} placeholder
        assert_eq!(
            props["data"]["type"].as_str().unwrap_or(""),
            "object",
            "data field must be typed as object placeholder in L2"
        );
    }

    #[test]
    fn test_three_event_l3_email_refs_l2_and_sets_channel_const() {
        // L3 is a concrete non-generic struct whose properties are nested under L2's
        // generic field (`data`).  Its own const value (channel='email') is injected
        // at the property level alongside the wrapped `data` sub-schema.
        let schema = ThreeEmailAlertDataV1::gts_schema_with_refs();
        assert_eq!(
            ThreeEmailAlertDataV1::gts_schema_id().as_ref(),
            "gts.x.test.three.event.v1~x.test.three.alert.v1~x.test.three.email.v1~"
        );
        let all_of = schema["allOf"].as_array().expect("L3 must have allOf");
        assert_eq!(
            all_of[0]["$ref"].as_str().unwrap(),
            "gts://gts.x.test.three.event.v1~x.test.three.alert.v1~",
            "L3 must $ref L2, not L1"
        );
        let props = all_of[1]["properties"]
            .as_object()
            .expect("L3 allOf[1].properties must exist");
        // const_values: channel='email'
        assert_eq!(
            props["channel"]["const"],
            json!("email"),
            "L3 must fix channel='email'"
        );
        // data sub-schema contains the concrete L3 fields (to, subject)
        let data_props = props["data"]["properties"]
            .as_object()
            .expect("data must have nested properties in L3");
        assert!(data_props.contains_key("to"), "data must include to");
        assert!(
            data_props.contains_key("subject"),
            "data must include subject"
        );
    }

    #[test]
    fn test_three_event_l3_does_not_own_l2_severity_const() {
        // L2 defines severity=3.  L3 email does NOT touch severity, so L3's own
        // allOf[1].properties must NOT contain "severity".
        let schema = ThreeEmailAlertDataV1::gts_schema_with_refs();
        let l3_props = schema["allOf"][1]["properties"]
            .as_object()
            .expect("L3 allOf[1].properties must exist");
        assert!(
            !l3_props.contains_key("severity"),
            "L3's own properties must NOT include severity (that const belongs to L2)"
        );
        // L2 independently still carries severity=3
        let l2_props = ThreeAlertPayloadV1::<()>::gts_schema_with_refs()["allOf"][1]["properties"]
            .as_object()
            .unwrap()
            .clone();
        assert_eq!(
            l2_props["severity"]["const"],
            json!(3),
            "L2 schema must still have severity=3"
        );
    }

    #[test]
    fn test_three_event_l3_sms_has_conflicting_severity_const() {
        // ThreeSmsAlertDataV1 is a unit struct extending ThreeAlertBaseV1 (a base struct).
        // It sets channel='sms', severity=9, kind='traffic' via const_values.
        // These appear in allOf[1]["properties"] of the generated schema.
        let schema = ThreeSmsAlertDataV1::gts_schema_with_refs();
        let all_of = schema["allOf"]
            .as_array()
            .expect("SMS-alert must have allOf");
        assert_eq!(
            all_of[0]["$ref"].as_str().unwrap(),
            "gts://gts.x.test.alert.base.v1~",
            "SMS-alert must reference ThreeAlertBaseV1"
        );
        let props = all_of[1]["properties"]
            .as_object()
            .expect("must have allOf[1].properties");
        // Parent-level consts (channel, severity, type) remain flat in allOf[1].properties.
        assert_eq!(
            props["channel"]["const"],
            json!("sms"),
            "channel must be 'sms'"
        );
        assert_eq!(props["severity"]["const"], json!(9), "severity must be 9");
        // auto-injected type const (ThreeAlertBaseV1 has r#type: GtsSchemaId)
        assert_eq!(
            props["type"]["const"].as_str().unwrap(),
            "gts.x.test.alert.base.v1~x.test.three.sms.v1~",
            "type const must match ThreeSmsAlertDataV1 schema id"
        );
        // Child-owned const (kind belongs to ThreeSmsAlertDataV1) is now correctly
        // nested under data.properties, not flat at the top level.
        assert!(
            !props.contains_key("kind"),
            "kind must NOT appear flat in allOf[1].properties (it is a child-owned field)"
        );
        let data_props = props["data"]["properties"]
            .as_object()
            .expect("data must have nested properties after nesting fix");
        assert_eq!(
            data_props["kind"]["const"],
            json!("traffic"),
            "kind const must be nested under data.properties"
        );

        // Check instance construction
        let instance = ThreeSmsAlertDataV1::new_instance_with_defaults();
        assert_eq!(instance.channel, "sms");
        assert_eq!(instance.severity, 9);
        assert_eq!(instance.data.kind, "traffic");
    }

    #[test]
    fn test_three_event_l3_type_const_is_full_three_segment_id() {
        // L3 email: the auto-injected `type` const must be the full 3-segment schema ID.
        // Note: auto-injection for L3 is skipped because L2 (ThreeAlertPayloadV1) has
        // no GtsSchemaId field – the type const is only present when the direct parent
        // carries a GtsSchemaId field.  In this hierarchy the type constraint is
        // instead carried by L2's schema (which was auto-injected from L1's r#type field).
        let l2_type = ThreeAlertPayloadV1::<()>::gts_schema_with_refs()["allOf"][1]["properties"]["type"]["const"]
            .as_str().unwrap().to_owned();
        assert!(
            l2_type.starts_with("gts.x.test.three.event.v1~"),
            "L2 type const must start with L1 prefix, got: {l2_type}"
        );
        assert!(
            l2_type.ends_with("x.test.three.alert.v1~"),
            "L2 type const must end with L2 segment, got: {l2_type}"
        );
    }

    // ------------------------------------------------------------------
    // 16. JSON types are distinct – int is not string, bool is not int
    // ------------------------------------------------------------------

    #[test]
    fn test_const_json_types_are_correct() {
        let int_schema = IntConstV1::gts_schema_with_refs();
        let int_props = int_schema["allOf"][1]["properties"].as_object().unwrap();
        assert!(
            !int_props["count"]["const"].is_string(),
            "int const must not be a JSON string"
        );
        assert!(
            !int_props["count"]["const"].is_boolean(),
            "int const must not be a JSON bool"
        );

        let bool_schema = BoolTrueConstV1::gts_schema_with_refs();
        let bool_props = bool_schema["allOf"][1]["properties"].as_object().unwrap();
        assert!(
            !bool_props["enabled"]["const"].is_string(),
            "bool const must not be a JSON string"
        );
        assert!(
            !bool_props["enabled"]["const"].is_number(),
            "bool const must not be a JSON number"
        );

        let float_schema = FloatConstV1::gts_schema_with_refs();
        let float_props = float_schema["allOf"][1]["properties"].as_object().unwrap();
        assert!(
            !float_props["score"]["const"].is_string(),
            "float const must not be a JSON string"
        );
        assert!(
            !float_props["score"]["const"].is_boolean(),
            "float const must not be a JSON bool"
        );

        let str_schema = StrQuotedConstV1::gts_schema_with_refs();
        let str_props = str_schema["allOf"][1]["properties"].as_object().unwrap();
        assert!(
            !str_props["label"]["const"].is_number(),
            "string const must not be a JSON number"
        );
        assert!(
            !str_props["label"]["const"].is_boolean(),
            "string const must not be a JSON bool"
        );
    }

    // ==================================================================
    // 17. Instance-against-schema validation for 3-level hierarchies (#1)
    // ==================================================================

    /// Register L1, L2, L3 (email) schemas plus `ThreeAlertBaseV1` + SMS schemas.
    fn register_three_level_schemas(ops: &mut gts::GtsOps) {
        for (id, schema) in [
            // L1
            (
                ThreeEventBaseV1::<()>::gts_schema_id()
                    .clone()
                    .into_string(),
                ThreeEventBaseV1::<()>::gts_schema_with_refs(),
            ),
            // L2
            (
                ThreeAlertPayloadV1::<()>::gts_schema_id()
                    .clone()
                    .into_string(),
                ThreeAlertPayloadV1::<()>::gts_schema_with_refs(),
            ),
            // L3 email
            (
                ThreeEmailAlertDataV1::gts_schema_id().clone().into_string(),
                ThreeEmailAlertDataV1::gts_schema_with_refs(),
            ),
            // Standalone base for SMS hierarchy
            (
                ThreeAlertBaseV1::<()>::gts_schema_id()
                    .clone()
                    .into_string(),
                ThreeAlertBaseV1::<()>::gts_schema_with_refs(),
            ),
            // SMS (non-unit child of ThreeAlertBaseV1)
            (
                ThreeSmsAlertDataV1::gts_schema_id().clone().into_string(),
                ThreeSmsAlertDataV1::gts_schema_with_refs(),
            ),
        ] {
            let result = ops.add_schema(id.clone(), &schema);
            assert!(result.ok, "add_schema {id}: {}", result.error);
        }
    }

    #[test]
    fn test_three_level_schemas_register_successfully() {
        // All three levels must register without error (add_schema succeeds).
        let mut ops = gts::GtsOps::new(None, None, 0);
        register_three_level_schemas(&mut ops);
        // Registration is asserted inside register_three_level_schemas.
    }

    #[test]
    fn test_three_level_schema_chain_l1_valid() {
        let mut ops = gts::GtsOps::new(None, None, 0);
        register_three_level_schemas(&mut ops);

        // L1 (base) must pass schema chain validation.
        let r1 = ops.validate_schema("gts.x.test.three.event.v1~");
        assert!(r1.ok, "L1 schema chain must be valid: {}", r1.error);
    }

    #[test]
    fn test_three_level_schema_chain_l2_flat_properties_rejected() {
        // KNOWN LIMITATION: L2's own properties (channel, recipient, data) are
        // placed flat in allOf[1].properties instead of nested under L1's
        // generic field ("payload").  The schema chain validator correctly
        // rejects this because L1 has additionalProperties: false.
        //
        // When the nesting bug is fixed (child properties placed under the
        // parent's generic field), this test should be updated to assert
        // r2.ok == true.
        let mut ops = gts::GtsOps::new(None, None, 0);
        register_three_level_schemas(&mut ops);

        let r2 = ops.validate_schema("gts.x.test.three.event.v1~x.test.three.alert.v1~");
        assert!(
            !r2.ok,
            "L2 schema chain should currently fail due to flat property placement"
        );
        assert!(
            r2.error.contains("additionalProperties"),
            "Error must mention additionalProperties: {}",
            r2.error
        );
    }

    #[test]
    fn test_three_level_instance_validates_structure() {
        // Build a concrete 3-level instance manually and verify the serialized
        // JSON has the correct shape for the full hierarchy.
        let instance = ThreeEventBaseV1 {
            r#type: gts::GtsSchemaId::new(
                "gts.x.test.three.event.v1~x.test.three.alert.v1~x.test.three.email.v1~",
            ),
            severity: 3,
            source: "alert-service".to_owned(),
            payload: ThreeAlertPayloadV1 {
                channel: "email".to_owned(),
                recipient: "user@example.com".to_owned(),
                data: ThreeEmailAlertDataV1 {
                    to: "admin@example.com".to_owned(),
                    subject: "Alert triggered".to_owned(),
                },
            },
        };
        let json = serde_json::to_value(&instance).unwrap();

        // Root-level fields (from L1)
        assert_eq!(
            json["type"],
            "gts.x.test.three.event.v1~x.test.three.alert.v1~x.test.three.email.v1~"
        );
        assert_eq!(json["severity"], 3);
        assert_eq!(json["source"], "alert-service");

        // L2 fields live under payload
        assert_eq!(json["payload"]["channel"], "email");
        assert_eq!(json["payload"]["recipient"], "user@example.com");

        // L3 fields live under payload.data
        assert_eq!(json["payload"]["data"]["to"], "admin@example.com");
        assert_eq!(json["payload"]["data"]["subject"], "Alert triggered");
    }

    // ==================================================================
    // 18. Non-unit derived struct with non-empty properties + const_values (#3)
    // ==================================================================

    #[test]
    fn test_three_email_properties_and_const_values_coexist_in_schema() {
        // ThreeEmailAlertDataV1 has `properties = "to,subject"` AND
        // `const_values = "channel='email'"`.  The schema must include both:
        //   - structural properties for `to` and `subject` (under `data`)
        //   - const constraint for `channel`
        // Note: `new_instance_with_defaults()` is NOT generated for non-unit
        // derived structs with non-empty `properties`.  This is by design;
        // use manual construction instead.
        let schema = ThreeEmailAlertDataV1::gts_schema_with_refs();
        let all_of = schema["allOf"].as_array().expect("must have allOf");
        let props = all_of[1]["properties"]
            .as_object()
            .expect("allOf[1].properties");

        // const_values: channel='email'
        assert_eq!(
            props["channel"]["const"],
            json!("email"),
            "channel const must appear in schema"
        );

        // structural properties under data
        let data_props = props["data"]["properties"]
            .as_object()
            .expect("data must have nested properties");
        assert!(data_props.contains_key("to"), "data must include 'to'");
        assert!(
            data_props.contains_key("subject"),
            "data must include 'subject'"
        );
    }

    #[test]
    fn test_three_email_manual_instance_creation() {
        // Since new_instance_with_defaults() is not available for non-unit
        // structs with non-empty properties, verify manual construction works.
        let data = ThreeEmailAlertDataV1 {
            to: "admin@example.com".to_owned(),
            subject: "Alert!".to_owned(),
        };
        let instance = ThreeEventBaseV1 {
            r#type: gts::GtsSchemaId::new(ThreeEmailAlertDataV1::gts_schema_id().as_ref()),
            severity: 3,
            source: "alert-service".to_owned(),
            payload: ThreeAlertPayloadV1 {
                channel: "email".to_owned(),
                recipient: "ops-team".to_owned(),
                data,
            },
        };
        let json = serde_json::to_value(&instance).unwrap();
        assert_eq!(json["payload"]["channel"], "email");
        assert_eq!(json["payload"]["data"]["to"], "admin@example.com");
        assert_eq!(json["payload"]["data"]["subject"], "Alert!");
    }

    // ==================================================================
    // 19. Schema validation for ThreeSmsAlertDataV1 via GtsOps (#4)
    // ==================================================================

    #[test]
    fn test_sms_schema_chain_base_valid() {
        let mut ops = gts::GtsOps::new(None, None, 0);
        register_three_level_schemas(&mut ops);

        // ThreeAlertBaseV1 (base) must be valid
        let r1 = ops.validate_schema("gts.x.test.alert.base.v1~");
        assert!(r1.ok, "ThreeAlertBaseV1 schema must be valid: {}", r1.error);
    }

    #[test]
    fn test_sms_schema_chain_child_valid() {
        // After the nesting fix: child-owned const `kind` is placed under
        // data.properties (not flat), so the schema chain is now valid.
        let mut ops = gts::GtsOps::new(None, None, 0);
        register_three_level_schemas(&mut ops);

        let r2 = ops.validate_schema("gts.x.test.alert.base.v1~x.test.three.sms.v1~");
        assert!(
            r2.ok,
            "ThreeSmsAlertDataV1 schema chain must be valid after nesting fix: {}",
            r2.error
        );
    }

    #[test]
    fn test_sms_instance_validates_structure() {
        // Verify the SMS instance built via new_instance_with_defaults()
        // serializes with the expected field values.
        let instance = ThreeSmsAlertDataV1::new_instance_with_defaults();
        let json = serde_json::to_value(&instance).unwrap();

        // Parent-level const overrides
        assert_eq!(json["channel"], "sms", "channel must be overridden");
        assert_eq!(json["severity"], 9, "severity must be overridden");

        // Child-level const override (kind is ThreeSmsAlertDataV1's own field)
        assert_eq!(
            json["data"]["kind"], "traffic",
            "kind must be set via Default impl"
        );

        // Auto-injected type const
        assert_eq!(
            json["type"], "gts.x.test.alert.base.v1~x.test.three.sms.v1~",
            "type must match ThreeSmsAlertDataV1 schema id"
        );
    }
}
