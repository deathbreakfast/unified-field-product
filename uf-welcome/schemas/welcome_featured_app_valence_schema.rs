// Featured apps catalog for the signed-in welcome page.

use crate::privacy_policies::WELCOME_ADMIN_GATE;
use valence::prelude::*;
use valence::privacy_policies::common::{AUTHENTICATED, SYSTEM_ONLY};

valence_schema! {
    WelcomeFeaturedApp {
        table: "welcome_featured_app",
        version: "0.1.1",
        database: crate::embedded_surreal::DEFAULT_STORAGE,
        description: "Admin-curated featured app rows for /welcome (app_id = AppRegistration.id)",

        policies: {
            // Authenticated welcome readers; CUD for WelcomeAdmin (or System jobs).
            read: {
                allow: [AUTHENTICATED],
            },
            create: {
                allow: [WELCOME_ADMIN_GATE, SYSTEM_ONLY],
            },
            update: {
                allow: [WELCOME_ADMIN_GATE, SYSTEM_ONLY],
            },
            delete: {
                allow: [WELCOME_ADMIN_GATE, SYSTEM_ONLY],
            },
        },

        fields: [
            id: {
                r#type: FieldType::String,
                primary_key: true,
                required: true,
            },
            app_id: {
                r#type: FieldType::String,
                required: true,
            },
            ordinal: {
                r#type: FieldType::Integer,
                required: true,
            },
            created_at: {
                r#type: FieldType::DateTime,
                required: true,
            },
            updated_at: {
                r#type: FieldType::DateTime,
                required: true,
            },
        ],
    }
}
