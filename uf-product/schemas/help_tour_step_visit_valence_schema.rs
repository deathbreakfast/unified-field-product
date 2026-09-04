// Per-user per-step help tour progress (Valence).

use valence::prelude::*;
use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;

valence_schema! {
    HelpTourStepVisit {
        table: "help_tour_step_visit",
        version: "0.1.0",
        database: crate::embedded_surreal::IDENTITY_DEFAULT_STORAGE,
        description: "Per-user per-step help tour progress for product spotlight tours",

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
            route: {
                r#type: FieldType::String,
                required: true,
            },
            feature_highlight: {
                r#type: FieldType::String,
                required: true,
            },
            spotlight: {
                r#type: FieldType::String,
                required: false,
            },
            replay: {
                r#type: FieldType::Bool,
                required: true,
                default: false,
            },
            first_seen_at: {
                r#type: FieldType::DateTime,
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
