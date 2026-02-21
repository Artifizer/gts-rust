//! Test: assigning a bool const to an integer field must be a compile error.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.bint.v1~",
    description = "Mismatch test base (bool-to-int)",
    properties = "type,count,metadata"
)]
#[derive(Debug)]
pub struct MismatchBIntBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub count: i32,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchBIntBaseV1,
    schema_id = "gts.x.test.mismatch.bint.v1~x.test.mismatch.bool_to_int.v1~",
    description = "Bool const into integer field",
    properties = "",
    const_values = "count=true"
)]
#[derive(Debug)]
pub struct BoolToIntV1;

fn main() {}
