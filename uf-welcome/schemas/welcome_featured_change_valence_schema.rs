// Published changes to the Welcome featured list (history + undo).

use crate::privacy_policies::WELCOME_ADMIN_GATE;
use valence::prelude::*;
use valence::privacy_policies::common::SYSTEM_ONLY;

valence_schema! {
    WelcomeFeaturedChange {
        repository: "https://github.com/unified-field-dev/unified-field-product",
        table: "welcome_featured_change",
        version: "0.1.0",
        database: crate::embedded_surreal::DEFAULT_STORAGE,
        description: "One published change to the /welcome featured list: who, what in words, and the list before and after",

        privacy: {
            gdpr_compliant: true,
        },

        policies: {
            // Only featured curators (or System jobs) see or write the history.
            read: {
                allow: [WELCOME_ADMIN_GATE, SYSTEM_ONLY],
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
            actor: {
                r#type: FieldType::Record("user"),
                required: false,
            },
            summary: {
                r#type: FieldType::String,
                required: true,
            },
            before: {
                r#type: FieldType::JsonAs("Vec<String>"),
                required: true,
            },
            after: {
                r#type: FieldType::JsonAs("Vec<String>"),
                required: true,
            },
            created_at: {
                r#type: FieldType::DateTime,
                required: true,
            },
        ],

        connections: [
            // The change stays in history when the curator's account goes; only who is cleared.
            actor: {
                table: "user",
                cardinality: HasOne,
                required: false,
                on_delete: SetNull,
                model: "lepton_identity::generated::User",
            },
        ],
    }
}
