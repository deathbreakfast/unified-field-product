//! Compile-time guard: Orbital's field validation message stays reachable
//! through `uf_product::primitives`, next to the state type it takes.

use uf_product::primitives::{FieldValidationMessage, FieldValidationState};

#[test]
fn field_validation_message_is_exported_with_its_state() {
    let _component = FieldValidationMessage;
    let _state = FieldValidationState::Error("Name is required.".to_string());
}
