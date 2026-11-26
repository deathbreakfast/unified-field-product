// Last page each person opened in each app, for Welcome's recent pages (Valence).

use valence::prelude::*;
use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;

valence_schema! {
    WelcomeRecentPage {
        repository: "https://github.com/unified-field-dev/unified-field-product",
        table: "welcome_recent_page",
        version: "0.1.0",
        database: crate::embedded_surreal::IDENTITY_DEFAULT_STORAGE,
        description: "Per-user last page opened in each app (one row per user and app) for Welcome recent pages",

        ttl: {
            seconds: 2_592_000,
            mode: "backend_capability",
        },

        privacy: {
            gdpr_compliant: true,
        },

        policies: {
            read: {
                always_allow: [],
                allow: [OWNER_BY_USER_FIELD],
                block: [],
                always_block: [],
            },
            create: {
                always_allow: [],
                allow: [OWNER_BY_USER_FIELD],
                block: [],
                always_block: [],
            },
            update: {
                always_allow: [],
                allow: [OWNER_BY_USER_FIELD],
                block: [],
                always_block: [],
            },
            delete: {
                always_allow: [],
                allow: [OWNER_BY_USER_FIELD],
                block: [],
                always_block: [],
            },
        },

        fields: [
            id: {
                r#type: FieldType::String,
                primary_key: true,
                required: true,
            },
            user: {
                r#type: FieldType::Record("user"),
                required: true,
            },
            app_id: {
                r#type: FieldType::String,
                required: true,
            },
            title: {
                r#type: FieldType::String,
                required: true,
            },
            route: {
                r#type: FieldType::String,
                required: true,
            },
            viewed_at: {
                r#type: FieldType::DateTime,
                required: true,
            },
        ],

        connections: [
            user: {
                table: "user",
                cardinality: HasOne,
                on_delete: Cascade,
                model: "lepton_identity::generated::User",
            },
        ],
    }
}
