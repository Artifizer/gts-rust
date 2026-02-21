//! Test: assigning an integer const to a String field must be a compile error.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.base.v1~",
    description = "Mismatch test base",
    properties = "type,label,metadata"
)]
#[derive(Debug)]
pub struct MismatchBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub label: String,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchBaseV1,
    schema_id = "gts.x.test.mismatch.base.v1~x.test.mismatch.int_to_string.v1~",
    description = "Integer const into String field",
    properties = "",
    const_values = "label=400"
)]
#[derive(Debug)]
pub struct IntToStringV1;

fn main() {}
