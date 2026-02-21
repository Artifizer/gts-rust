//! Test: assigning an integer const to a bool field must be a compile error.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.ibool.v1~",
    description = "Mismatch test base (int-to-bool)",
    properties = "type,enabled,metadata"
)]
#[derive(Debug)]
pub struct MismatchIBoolBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub enabled: bool,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchIBoolBaseV1,
    schema_id = "gts.x.test.mismatch.ibool.v1~x.test.mismatch.int_to_bool.v1~",
    description = "Integer const into bool field",
    properties = "",
    const_values = "enabled=1"
)]
#[derive(Debug)]
pub struct IntToBoolV1;

fn main() {}
