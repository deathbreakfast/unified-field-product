// Apps a person pinned to their Welcome page.

use valence::prelude::*;
use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;

valence_schema! {
    WelcomePinnedApp {
        repository: "https://github.com/unified-field-dev/unified-field-product",
        table: "welcome_pinned_app",
        version: "0.1.0",
        database: crate::embedded_surreal::DEFAULT_STORAGE,
        description: "Per-user pinned apps for /welcome (app_id = AppRegistration.id, ordered by ordinal)",

        privacy: {
            gdpr_compliant: true,
        },

        // Private to the owner; WelcomeAdmin and other users get no access.
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
            ordinal: {
                r#type: FieldType::Integer,
                required: true,
            },
            updated_at: {
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
