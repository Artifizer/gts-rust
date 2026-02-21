//! Test: const_values referencing a field that doesn't exist on the parent struct
//! must be a compile error

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.const.base.v1~",
    description = "Base for unknown field test",
    properties = "type,value,metadata"
)]
#[derive(Debug)]
pub struct ConstBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub value: String,
    pub metadata: M,
}

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = ConstBaseV1,
    schema_id = "gts.x.test.const.base.v1~x.test.const.derived.v1~",
    description = "Derived with a non-existent field in const_values",
    properties = "",
    const_values = "nonexistent_field=42"
)]
#[derive(Debug)]
pub struct ConstDerivedV1;

fn main() {}
