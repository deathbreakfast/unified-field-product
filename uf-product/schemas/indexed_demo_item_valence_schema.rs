// Teaching / e2e source model for workspace search indexing.

#[allow(unused_imports)]
use crate::workspace_search::demo::{IndexedDemoBackfillIter, IndexedDemoIndexer};
use valence::prelude::*;
use valence::privacy_policies::common::SYSTEM_ONLY;
use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;

valence_schema! {
    IndexedDemoItem {
        table: "indexed_demo_item",
        version: "0.1.0",
        database: crate::embedded_surreal::IDENTITY_DEFAULT_STORAGE,
        description: "Teaching source row for workspace content index SideEffect / Iter",

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
                allow: [OWNER_BY_USER_FIELD, SYSTEM_ONLY],
                block: [],
                always_block: [],
            },
            update: {
                always_allow: [],
                allow: [OWNER_BY_USER_FIELD, SYSTEM_ONLY],
                block: [],
                always_block: [],
            },
            delete: {
                always_allow: [],
                allow: [OWNER_BY_USER_FIELD, SYSTEM_ONLY],
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
            title: {
                r#type: FieldType::String,
                required: true,
            },
            link: {
                r#type: FieldType::String,
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

        side_effects: [IndexedDemoIndexer],
        iters: [IndexedDemoBackfillIter],
    }
}
