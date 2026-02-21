//! Test: assigning a string const to an integer field must be a compile error.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.int.v1~",
    description = "Mismatch test base (int field)",
    properties = "type,count,metadata"
)]
#[derive(Debug)]
pub struct MismatchIntBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub count: i32,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchIntBaseV1,
    schema_id = "gts.x.test.mismatch.int.v1~x.test.mismatch.string_to_int.v1~",
    description = "String const into integer field",
    properties = "",
    const_values = "count='bad'"
)]
#[derive(Debug)]
pub struct StringToIntV1;

fn main() {}
