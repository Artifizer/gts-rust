//! Test: A base struct with 2 GtsSchemaId fields compiles fine, but
//! new_instance_with_defaults() is NOT generated — it would be ambiguous
//! which field to populate. Calling it must produce a compile error.

use gts::gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.duptype.ambiguous.v1~",
    description = "Struct with two GtsSchemaId fields — ambiguous",
    properties = "type,alt_type,name"
)]
#[derive(Debug)]
pub struct TwoSchemaIdBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub alt_type: GtsSchemaId,
    pub name: String,
    pub metadata: M,
}

fn main() {
    // The struct itself compiles OK.
    // new_instance_with_defaults() is not generated because it is ambiguous
    // (two GtsSchemaId fields — unclear which one holds the schema ID).
    let _ = TwoSchemaIdBaseV1::<()>::new_instance_with_defaults();
}
