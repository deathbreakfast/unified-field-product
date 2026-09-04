//! Permission manifest for the Welcome app.

use uf_product_macros::UfPermissionManifest;

/// Welcome app permission domain. Admin mutations gate on [`Self::WelcomeAdmin`].
#[allow(clippy::expl_impl_clone_on_copy)]
#[derive(UfPermissionManifest)]
#[permission_manifest(
    domain_key = "welcome",
    domain_name = "Welcome",
    domain_description = "Signed-in welcome landing and featured-app administration"
)]
pub enum WelcomePermission {
    /// Curate featured apps on `/welcome` (add / remove / reorder).
    #[permission(description = "Manage featured apps on the welcome page")]
    WelcomeAdmin,
}
