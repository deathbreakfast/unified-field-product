//! `WelcomeAdmin` permission name contract.

use uf_welcome::permissions::WelcomePermission;

#[test]
fn welcome_admin_permission_name() {
    assert_eq!(WelcomePermission::WelcomeAdmin.as_str(), "WelcomeAdmin");
}
