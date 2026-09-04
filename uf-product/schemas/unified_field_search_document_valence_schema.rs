// Per-user workspace content index row (Valence).

use valence::prelude::*;
use valence::privacy_policies::common::SYSTEM_ONLY;
use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;

valence_schema! {
    UnifiedFieldSearchDocument {
        table: "unified_field_search_document",
        version: "0.1.0",
        database: crate::embedded_surreal::IDENTITY_DEFAULT_STORAGE,
        description: "Per-user denormalized workspace search index row",

        ttl: {
            seconds: 7_776_000,
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
                allow: [SYSTEM_ONLY],
                block: [],
                always_block: [],
            },
            update: {
                always_allow: [],
                allow: [SYSTEM_ONLY],
                block: [],
                always_block: [],
            },
            delete: {
                always_allow: [],
                allow: [SYSTEM_ONLY],
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
            source_table: {
                r#type: FieldType::String,
                required: true,
            },
            source_id: {
                r#type: FieldType::String,
                required: true,
            },
            title: {
                r#type: FieldType::String,
                required: true,
            },
            searchable_text: {
                r#type: FieldType::String,
                required: true,
            },
            link: {
                r#type: FieldType::String,
                required: true,
            },
            kind: {
                r#type: FieldType::String,
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
