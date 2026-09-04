use valence::prelude::*;
use valence::privacy_policies::common::SYSTEM_ONLY;
use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;

valence_schema! {
    Session {
        table: "session",
        version: "0.1.0",
        database: crate::embedded_surreal::SESSION_DEFAULT_STORAGE,
        description: "Active user session with device metadata",

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
                allow: [SYSTEM_ONLY],
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
            session_token: {
                r#type: FieldType::String,
                required: true,
                validations: [Validator::NonEmpty],
                policies: {
                    read: { allow: [SYSTEM_ONLY] },
                },
            },
            device_name: {
                r#type: FieldType::String,
                required: false,
                policies: {
                    read: { allow: [OWNER_BY_USER_FIELD] },
                },
            },
            ip_address: {
                r#type: FieldType::String,
                required: false,
                policies: {
                    read: { allow: [OWNER_BY_USER_FIELD] },
                },
            },
            user_agent: {
                r#type: FieldType::String,
                required: false,
                policies: {
                    read: { allow: [OWNER_BY_USER_FIELD] },
                },
            },
            expires_at: {
                r#type: FieldType::DateTime,
                required: true,
            },
            last_active_at: {
                r#type: FieldType::DateTime,
                required: true,
            },
            created_at: {
                r#type: FieldType::DateTime,
                required: true,
                policies: {
                    read: { allow: [OWNER_BY_USER_FIELD] },
                },
            }
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
