//! Test: A base struct with 2 GtsInstanceId fields compiles fine, but
//! new_instance_with_defaults() is NOT generated — it would be ambiguous
//! which field to populate. Calling it must produce a compile error.

use gts::gts::{GtsInstanceId, GtsSchemaId};
use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.test.dupid.ambiguous.v1~",
    description = "Struct with two GtsInstanceId fields — ambiguous",
    properties = "type,primary_id,secondary_id,name"
)]
#[derive(Debug)]
pub struct TwoInstanceIdBaseV1<M> {
    pub r#type: GtsSchemaId,
    pub primary_id: GtsInstanceId,
    pub secondary_id: GtsInstanceId,
    pub name: String,
    pub metadata: M,
}

fn main() {
    // The struct itself compiles OK.
    // new_instance_with_defaults() is not generated because it is ambiguous
    // (two GtsInstanceId fields — unclear which one represents the instance ID).
    let _ = TwoInstanceIdBaseV1::<()>::new_instance_with_defaults();
}
