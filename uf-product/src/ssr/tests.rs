#[cfg(test)]
mod tests {
    use crate::ssr::{current_operation, session_user_record_id, with_operation};

    #[tokio::test]
    async fn with_operation_sets_context_happy_path() {
        let result = with_operation("test_op", async {
            assert_eq!(current_operation(), Some("test_op"));
            42
        })
        .await;

        assert_eq!(result, 42);
        // Outside the scope, operation should be None
        assert_eq!(current_operation(), None);
    }

    #[tokio::test]
    async fn nested_operations_restore_outer_happy_path() {
        let result = with_operation("outer", async {
            assert_eq!(current_operation(), Some("outer"));

            let inner_result = with_operation("inner", async {
                assert_eq!(current_operation(), Some("inner"));
                100
            })
            .await;

            // After inner scope, should be back to outer
            assert_eq!(current_operation(), Some("outer"));
            inner_result
        })
        .await;

        assert_eq!(result, 100);
        assert_eq!(current_operation(), None);
    }

    #[tokio::test]
    async fn current_operation_none_when_unset_sad() {
        assert_eq!(current_operation(), None);
    }

    #[test]
    fn session_user_record_id_parses_table_id_happy_path() {
        let rid = session_user_record_id("user:alice").expect("parse");
        assert_eq!(rid.to_string(), "user:alice");
    }

    #[test]
    fn session_user_record_id_rejects_bare_id_sad() {
        assert!(session_user_record_id("alice").is_err());
    }
}

#[cfg(all(test, feature = "ssr"))]
mod privacy_policy_tests {
    use serde_json::json;
    use valence::privacy::PrivacyPolicy;
    use valence::privacy_policies::common::SYSTEM_ONLY;
    use valence::privacy_policies::owner::OWNER_BY_USER_FIELD;
    use valence::{Actor, PrivacyEvaluator};

    fn owner_policy() -> PrivacyPolicy {
        PrivacyPolicy {
            always_allow: vec![],
            allow: vec![OWNER_BY_USER_FIELD],
            block: vec![],
            always_block: vec![],
        }
    }

    fn system_only_policy() -> PrivacyPolicy {
        PrivacyPolicy {
            always_allow: vec![],
            allow: vec![SYSTEM_ONLY],
            block: vec![],
            always_block: vec![],
        }
    }

    #[test]
    fn owner_policy_allows_owner_denies_cross_user() {
        let policy = owner_policy();
        let owner = Actor::User {
            user_id: "alice".into(),
        };
        let other = Actor::User {
            user_id: "bob".into(),
        };
        let record = json!({ "user": "user:alice" });

        assert!(PrivacyEvaluator::evaluate(&policy, &record, &owner).is_ok());
        assert!(
            PrivacyEvaluator::evaluate(&policy, &record, &other).is_err(),
            "cross-user must be denied"
        );
    }

    #[test]
    fn session_schema_read_is_owner_scoped_not_authenticated() {
        use crate::generated::Session;

        let schema = Session::get_schema();
        let read = schema
            .policies
            .as_ref()
            .and_then(|p| p.read.as_ref())
            .expect("session read policy");
        let names: Vec<_> = read.allow.iter().map(|r| r.name.as_str()).collect();
        assert!(
            names.contains(&"owner_by_user_field"),
            "expected OWNER_BY_USER_FIELD, got {names:?}"
        );
        assert!(
            !names.iter().any(|n| *n == "authenticated"),
            "AUTHENTICATED must not remain on session read: {names:?}"
        );
    }

    #[test]
    fn user_appearance_schema_read_update_create_owner_delete_system() {
        use crate::generated::UserAppearance;

        let schema = UserAppearance::get_schema();
        let policies = schema.policies.as_ref().expect("appearance policies");

        let read_names: Vec<_> = policies
            .read
            .as_ref()
            .expect("read")
            .allow
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(read_names, ["owner_by_user_field"]);

        let update_names: Vec<_> = policies
            .update
            .as_ref()
            .expect("update")
            .allow
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(update_names, ["owner_by_user_field"]);

        let create_names: Vec<_> = policies
            .create
            .as_ref()
            .expect("create")
            .allow
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(
            create_names,
            ["owner_by_user_field"],
            "create must match lepton-identity owner bootstrap (session Valence)"
        );

        let delete_names: Vec<_> = policies
            .delete
            .as_ref()
            .expect("delete")
            .allow
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(delete_names, ["system_only"]);
    }

    #[test]
    fn system_only_create_denies_authenticated_user() {
        let policy = system_only_policy();
        let user = Actor::User {
            user_id: "alice".into(),
        };
        let system = Actor::System {
            operation: "test".into(),
        };
        let record = json!({ "user": "user:alice" });

        assert!(PrivacyEvaluator::evaluate(&policy, &record, &user).is_err());
        assert!(PrivacyEvaluator::evaluate(&policy, &record, &system).is_ok());
    }

    #[test]
    fn notification_create_is_system_only() {
        // Mirrors uf-notifications-core Notification schema after UP-06.
        let policy = system_only_policy();
        let user = Actor::User {
            user_id: "alice".into(),
        };
        let other = Actor::User {
            user_id: "bob".into(),
        };
        let record = json!({ "user": "user:bob" });

        assert!(
            PrivacyEvaluator::evaluate(&policy, &record, &user).is_err(),
            "authenticated users must not create notifications for arbitrary recipients"
        );
        assert!(
            PrivacyEvaluator::evaluate(&policy, &record, &other).is_err(),
            "recipient also cannot self-create under SYSTEM_ONLY"
        );
    }
}
