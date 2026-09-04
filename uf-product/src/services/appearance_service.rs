//! Server and client helpers for user appearance preferences.

use crate::theme::{
    read_local_appearance, write_local_appearance, AppearanceContext, AppearancePreferences,
};
use leptos::prelude::*;

/// Serializable appearance payload returned by server functions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppearanceData {
    pub color_mode: String,
    pub brand_source: String,
    pub brand_seed_color: Option<String>,
}

impl From<AppearanceData> for AppearancePreferences {
    fn from(data: AppearanceData) -> Self {
        Self {
            color_mode: data.color_mode,
            brand_source: data.brand_source,
            brand_seed_color: data.brand_seed_color,
        }
    }
}

impl From<AppearancePreferences> for AppearanceData {
    fn from(prefs: AppearancePreferences) -> Self {
        Self {
            color_mode: prefs.color_mode,
            brand_source: prefs.brand_source,
            brand_seed_color: prefs.brand_seed_color,
        }
    }
}

// Used from `#[server]` bodies (compiled under `ssr`) and unit tests; hydrate
// stubs omit the server-fn body so this looks unused without the cfg.
#[cfg(any(feature = "ssr", test))]
fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

/// Load the signed-in user's appearance preferences (creating a default row if needed).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller is not authenticated, Higgs/Valence setup fails,
/// the appearance query/create path fails, or the function is invoked without the `ssr` feature.
#[server(GetMyAppearance)]
pub async fn get_my_appearance() -> Result<AppearanceData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::generated::UserAppearance;
        use chrono::Utc;
        use valence::{Model, RecordPredicate};

        let ctx = crate::ssr::higgs().await?;
        let user_id_str = ctx
            .session_user_id()
            .ok_or_else(|| ServerFnError::new("Not authenticated"))?
            .clone();
        let user = crate::ssr::session_user_record_id(&user_id_str)?;

        // Session Valence enforces OWNER_BY_USER_FIELD on read/update.
        let v = crate::ssr::valence(&ctx)?;

        let existing = UserAppearance::query(&v)
            .where_user(RecordPredicate::Equals(user.clone()))
            .first()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to query appearance: {e}")))?;

        let row = match existing {
            Some(row) => row,
            None => {
                let new_row = UserAppearance::new(
                    user,
                    "light".to_string(),
                    "product".to_string(),
                    None,
                    Utc::now(),
                )
                .map_err(|e| ServerFnError::new(format!("Failed to build appearance: {e}")))?;

                UserAppearance::create(new_row, &v)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Failed to create appearance: {e}")))?
            }
        };

        Ok(AppearanceData {
            color_mode: row.color_mode().to_string(),
            brand_source: row.brand_source().to_string(),
            brand_seed_color: row.brand_seed_color().map(|s| s.to_string()),
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("SSR required"))
    }
}

/// Persist appearance preferences for the signed-in user.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product::save_my_appearance;
///
/// // In a signed-in settings handler on SSR:
/// save_my_appearance("dark".into(), "product".into(), None).await?;
/// assert_eq!("dark", "dark");
/// ```
///
/// # Errors
///
/// Returns [`ServerFnError::Args`] for invalid `color_mode`, `brand_source`, or custom seed
/// (must be `#RRGGBB` when `brand_source` is `"custom"`). Returns other [`ServerFnError`]
/// variants when the caller is unauthenticated, Valence I/O fails, or SSR is unavailable.
#[server(SaveMyAppearance)]
pub async fn save_my_appearance(
    /// Desired color mode; must be `"light"` or `"dark"`.
    color_mode: String,
    /// Desired brand source; must be `"product"` or `"custom"`.
    brand_source: String,
    /// Custom brand seed color as `#RRGGBB`, required when `brand_source` is `"custom"`.
    brand_seed_color: Option<String>,
) -> Result<(), ServerFnError> {
    if color_mode != "light" && color_mode != "dark" {
        return Err(ServerFnError::Args("Invalid color_mode".into()));
    }
    if brand_source != "product" && brand_source != "custom" {
        return Err(ServerFnError::Args("Invalid brand_source".into()));
    }
    if brand_source == "custom" {
        let seed = brand_seed_color.as_deref().unwrap_or("");
        if !is_hex_color(seed) {
            return Err(ServerFnError::Args("Custom brand requires #RRGGBB".into()));
        }
    }

    #[cfg(feature = "ssr")]
    {
        use crate::generated::UserAppearance;
        use chrono::Utc;
        use valence::RecordPredicate;

        let ctx = crate::ssr::higgs().await?;
        let user_id_str = ctx
            .session_user_id()
            .ok_or_else(|| ServerFnError::new("Not authenticated"))?
            .clone();
        let user = crate::ssr::session_user_record_id(&user_id_str)?;

        let v = crate::ssr::valence(&ctx)?;

        let row = UserAppearance::query(&v)
            .where_user(RecordPredicate::Equals(user))
            .first()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to query appearance: {e}")))?
            .ok_or_else(|| ServerFnError::new("Appearance not found"))?;

        let mut mutable = row
            .get_mutable(&v)
            .set_color_mode(color_mode)
            .map_err(|e| ServerFnError::new(format!("Validation error: {e}")))?
            .set_brand_source(brand_source)
            .map_err(|e| ServerFnError::new(format!("Validation error: {e}")))?;

        if let Some(seed) = brand_seed_color {
            mutable = mutable
                .set_brand_seed_color(seed)
                .map_err(|e| ServerFnError::new(format!("Validation error: {e}")))?;
        }

        mutable
            .set_updated_at(Utc::now())
            .map_err(|e| ServerFnError::new(format!("Validation error: {e}")))?
            .commit()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to save appearance: {e}")))?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (color_mode, brand_source, brand_seed_color);
        Err(ServerFnError::new("SSR required"))
    }
}

/// Load appearance preferences and keep [`AppearanceContext`] synchronized.
pub fn init_appearance_resource(
    appearance_ctx: AppearanceContext,
) -> Resource<Result<AppearanceData, ServerFnError>> {
    let appearance_resource = Resource::new(|| (), |()| get_my_appearance());

    let prefs_signal = appearance_ctx.prefs;
    Effect::new(move |_| {
        if let Some(result) = appearance_resource.get() {
            match result {
                Ok(data) => {
                    let prefs: AppearancePreferences = data.into();
                    prefs_signal.set(prefs.clone());
                    write_local_appearance(&prefs);
                }
                Err(_) => {
                    if let Some(local) = read_local_appearance() {
                        prefs_signal.set(local);
                    }
                }
            }
        }
    });

    appearance_resource
}

/// Provide appearance context with anonymous/local defaults before auth resolves.
pub fn provide_appearance_context() -> AppearanceContext {
    let initial = read_local_appearance().unwrap_or_else(AppearancePreferences::light_product);
    AppearanceContext::provide(initial)
}

#[cfg(test)]
mod tests {
    use super::is_hex_color;

    #[test]
    fn is_hex_color_accepts_six_digit_rgb_values() {
        assert!(is_hex_color("#aB12fE"));
    }

    #[test]
    fn is_hex_color_rejects_malformed_values() {
        for value in ["", "#fff", "#1234567", "123456", "#12gg56"] {
            assert!(!is_hex_color(value), "{value} should be rejected");
        }
    }
}
