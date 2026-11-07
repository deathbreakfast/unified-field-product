//! `uf_product::primitives` re-exports the Orbital field validation items.

use uf_product::primitives::{FieldValidationMessage, FieldValidationState};

#[test]
fn field_validation_items_resolve_through_primitives() {
    assert!(std::any::type_name::<FieldValidationState>().ends_with("FieldValidationState"));
    // `FieldValidationMessage` is a component function, so name it by value.
    assert!(std::any::type_name_of_val(&FieldValidationMessage).contains("FieldValidationMessage"));
}
