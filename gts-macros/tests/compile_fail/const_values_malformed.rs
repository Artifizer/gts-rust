//! Test: malformed const_values (entry with no '=') must be a compile error

use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.const.malformed.v1~",
    description = "Malformed const_values test",
    properties = "value",
    const_values = "value_without_equals"
)]
pub struct MalformedConstV1 {
    pub value: String,
}

fn main() {}
