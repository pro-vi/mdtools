use schemars::schema_for;

use crate::patch::{Patch, PatchReceipt};

pub fn patch_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(Patch)).expect("generated patch schema serializes")
}

pub fn patch_receipt_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(PatchReceipt))
        .expect("generated patch receipt schema serializes")
}

pub fn protocol_schema() -> serde_json::Value {
    serde_json::json!({
        "patch": patch_schema(),
        "patch_receipt": patch_receipt_schema(),
    })
}
