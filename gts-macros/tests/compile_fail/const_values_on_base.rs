//! Test: `const_values` must not be used on base structs (`base = true`).
//! Base structs have no parent to override, so const_values is meaningless.

use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.const.base.v1~",
    description = "Base with const_values (invalid)",
    properties = "type,name",
    const_values = "name='fixed'"
)]
pub struct BaseWithConstV1 {
    pub r#type: GtsSchemaId,
    pub name: String,
}

fn main() {}
