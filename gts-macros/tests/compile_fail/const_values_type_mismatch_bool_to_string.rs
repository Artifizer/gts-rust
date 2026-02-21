//! Test: assigning a bool const to a String field must be a compile error.
//! (After parser fix, `true` is parsed as Bool, not as string "true".)

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.bstr.v1~",
    description = "Mismatch test base (bool-to-string)",
    properties = "type,label,metadata"
)]
#[derive(Debug)]
pub struct MismatchBStrBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub label: String,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchBStrBaseV1,
    schema_id = "gts.x.test.mismatch.bstr.v1~x.test.mismatch.bool_to_string.v1~",
    description = "Bool const into String field",
    properties = "",
    const_values = "label=true"
)]
#[derive(Debug)]
pub struct BoolToStringV1;

fn main() {}
