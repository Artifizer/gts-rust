//! Test: assigning a float const to an integer field must be a compile error.
//! (After parser fix, `3.14` is parsed as f64 Number, not as string "3.14".)

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.fint.v1~",
    description = "Mismatch test base (float-to-int)",
    properties = "type,count,metadata"
)]
#[derive(Debug)]
pub struct MismatchFIntBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub count: i32,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchFIntBaseV1,
    schema_id = "gts.x.test.mismatch.fint.v1~x.test.mismatch.float_to_int.v1~",
    description = "Float const into integer field",
    properties = "",
    const_values = "count=3.14"
)]
#[derive(Debug)]
pub struct FloatToIntV1;

fn main() {}
