//! Test: assigning a string const to a bool field must be a compile error.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.mismatch.sbool.v1~",
    description = "Mismatch test base (string-to-bool)",
    properties = "type,enabled,metadata"
)]
#[derive(Debug)]
pub struct MismatchSBoolBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub enabled: bool,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = MismatchSBoolBaseV1,
    schema_id = "gts.x.test.mismatch.sbool.v1~x.test.mismatch.string_to_bool.v1~",
    description = "String const into bool field",
    properties = "",
    const_values = "enabled='yes'"
)]
#[derive(Debug)]
pub struct StringToBoolV1;

fn main() {}
