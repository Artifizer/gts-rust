//! Test: assigning a string const to a float field must be a compile error.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.sflt.v1~",
    description = "Mismatch test base (string-to-float)",
    properties = "type,score,metadata"
)]
#[derive(Debug)]
pub struct MismatchSFltBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub score: f64,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchSFltBaseV1,
    schema_id = "gts.x.test.mismatch.sflt.v1~x.test.mismatch.string_to_float.v1~",
    description = "String const into float field",
    properties = "",
    const_values = "score='high'"
)]
#[derive(Debug)]
pub struct StringToFloatV1;

fn main() {}
