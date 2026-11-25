//! Tests for the access module: a fake backend drives every state the views draw.

use std::sync::{Arc, Mutex};

use leptos::prelude::*;

use super::access_card::{card_body, CardActions};
use super::access_denied::{denied_action_view, denied_view};
use super::add_access_dialog::add_access_fields;
use super::backend::{require_with, UnwiredBackend};
use super::remove_access_dialog::remove_access_body;
use super::request_access_dialog::request_access_fields;
use super::*;

// ---- fixtures ----------------------------------------------------------------------------

fn job() -> ResourceRef {
    ResourceRef::new("chronon_job", "billing-rollup")
        .label("Billing rollup")
        .noun("job")
}

fn principal(kind: PrincipalKind, id: &str, label: &str) -> PrincipalRef {
    PrincipalRef {
        kind,
        id: id.to_string(),
        label: label.to_string(),
    }
}

fn admins() -> PrincipalRef {
    principal(PrincipalKind::Group, "chronon_admins", "Chronon admins")
}

fn billing_team() -> PrincipalRef {
    principal(
        PrincipalKind::Team {
            org: None,
            member_count: 6,
        },
        "polaron_team_billing",
        "Billing engineering",
    )
}

fn noor() -> PrincipalRef {
    principal(PrincipalKind::User, "u_noor", "Noor Haddad")
}

fn entry(principal: PrincipalRef, role: Role) -> AccessEntry {
    AccessEntry {
        principal,
        role,
        default_owner: false,
        umbrella: false,
    }
}

/// What the default owner sees on a code-declared job.
fn owner_view() -> AccessView {
    AccessView {
        entries: vec![
            AccessEntry {
                default_owner: true,
                ..entry(admins(), Role::Owner)
            },
            entry(billing_team(), Role::Edit),
            entry(
                principal(PrincipalKind::Group, "billing_analysts", "Billing Analysts"),
                Role::Run,
            ),
            entry(noor(), Role::View),
            AccessEntry {
                umbrella: true,
                ..entry(
                    principal(
                        PrincipalKind::Group,
                        "chronon_viewers",
                        "Everyone with Chronon access",
                    ),
                    Role::View,
                )
            },
        ],
        roles: Role::STANDARD.to_vec(),
        viewer_role: Some(Role::Owner),
        can_maintain: true,
    }
}

/// What a team member who can edit sees.
fn reader_view() -> AccessView {
    AccessView {
        viewer_role: Some(Role::Edit),
        can_maintain: false,
        ..owner_view()
    }
}

/// The owner's view after the kind-wide row is removed.
fn restricted_view() -> AccessView {
    let mut view = owner_view();
    view.entries.retain(|e| !e.umbrella);
    view
}

fn no_actions() -> CardActions {
    CardActions {
        retry: Callback::new(|()| {}),
        add: Callback::new(|()| {}),
        set_role: Callback::new(|_| {}),
        remove: Callback::new(|_| {}),
    }
}

/// Render under the theme provider, which Orbital's overlays (Select, Menu, Tooltip) need.
fn render(f: impl FnOnce() -> AnyView + Send + 'static) -> String {
    use crate::primitives::OrbitalThemeProvider;
    Owner::new().with(|| view! { <OrbitalThemeProvider>{f()}</OrbitalThemeProvider> }.to_html())
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime")
        .block_on(f)
}

// ---- fake backend ------------------------------------------------------------------------

/// A scripted backend: an access view (or a load error), allowed actions, search results,
/// and the owner guard the real one enforces.
#[derive(Default)]
struct FakeBackend {
    view: Mutex<Option<AccessView>>,
    load_error: Option<&'static str>,
    allowed: Vec<&'static str>,
    check_error: bool,
    search: PrincipalResults,
    /// Every resource's rows, as `(kind, id, rows)`, for `owned_by`.
    resources: Vec<(&'static str, &'static str, Vec<AccessEntry>)>,
    /// The groups and team groups each user is in, by user id.
    memberships: Vec<(&'static str, Vec<PrincipalRef>)>,
}

impl FakeBackend {
    fn with_view(view: AccessView) -> Self {
        Self {
            view: Mutex::new(Some(view)),
            ..Self::default()
        }
    }

    fn owner_count_without(&self, principal: &PrincipalRef) -> usize {
        self.view.lock().unwrap().as_ref().map_or(0, |v| {
            v.entries
                .iter()
                .filter(|e| e.role == Role::Owner && e.principal.key() != principal.key())
                .count()
        })
    }
}

#[async_trait::async_trait]
impl ResourceAccessBackend for FakeBackend {
    async fn access(&self, _resource: &ResourceRef) -> Result<AccessView, AccessError> {
        if let Some(message) = self.load_error {
            return Err(AccessError::Backend {
                op: "load access".to_string(),
                message: message.to_string(),
            });
        }
        Ok(self.view.lock().unwrap().clone().expect("fake view"))
    }

    async fn set_role(
        &self,
        resource: &ResourceRef,
        principal: &PrincipalRef,
        role: Role,
    ) -> Result<(), AccessError> {
        if role != Role::Owner && self.owner_count_without(principal) == 0 {
            return Err(AccessError::LastOwner {
                resource: resource.id.clone(),
            });
        }
        let mut view = self.view.lock().unwrap();
        let view = view.as_mut().expect("fake view");
        match view
            .entries
            .iter_mut()
            .find(|e| e.principal.key() == principal.key())
        {
            Some(row) => row.role = role,
            None => view.entries.push(entry(principal.clone(), role)),
        }
        Ok(())
    }

    async fn remove(
        &self,
        resource: &ResourceRef,
        principal: &PrincipalRef,
    ) -> Result<(), AccessError> {
        if self.owner_count_without(principal) == 0 {
            return Err(AccessError::LastOwner {
                resource: resource.id.clone(),
            });
        }
        let mut view = self.view.lock().unwrap();
        view.as_mut()
            .expect("fake view")
            .entries
            .retain(|e| e.principal.key() != principal.key());
        Ok(())
    }

    async fn search_principals(&self, _query: &str) -> Result<PrincipalResults, AccessError> {
        Ok(self.search.clone())
    }

    async fn request(
        &self,
        _resource: &ResourceRef,
        _role: Role,
        _reason: &str,
    ) -> Result<RequestId, AccessError> {
        Ok(RequestId("req-1".to_string()))
    }

    async fn can(&self, _resource: &ResourceRef, action: &str) -> Result<bool, AccessError> {
        if self.check_error {
            return Err(AccessError::Backend {
                op: "check access".to_string(),
                message: "valence: connection refused".to_string(),
            });
        }
        Ok(self.allowed.contains(&action))
    }

    async fn owned_by(&self, actor: &PrincipalRef, kind: &str) -> Result<Vec<String>, AccessError> {
        let mut principals = vec![actor.clone()];
        principals.extend(
            self.memberships
                .iter()
                .filter(|(user, _)| *user == actor.id)
                .flat_map(|(_, groups)| groups.iter().cloned()),
        );
        let of_kind = self
            .resources
            .iter()
            .filter(|(k, _, _)| *k == kind)
            .map(|(_, id, rows)| (*id, rows));
        Ok(owned_resource_ids(of_kind, &principals))
    }
}

// ---- backend, require and roles ----------------------------------------------------------

#[test]
fn unwired_backend_fails_closed() {
    block_on(async {
        let r = job();
        assert_eq!(UnwiredBackend.can(&r, "View").await, Ok(false));
        assert_eq!(UnwiredBackend.access(&r).await, Err(AccessError::Unwired));
        assert_eq!(
            UnwiredBackend.set_role(&r, &noor(), Role::Run).await,
            Err(AccessError::Unwired)
        );
        assert_eq!(
            UnwiredBackend.search_principals("no").await,
            Err(AccessError::Unwired)
        );
    });
}

// ---- owned_by ----------------------------------------------------------------------------

fn sam() -> PrincipalRef {
    principal(PrincipalKind::User, "u_sam", "Sam Okafor")
}

fn ines() -> PrincipalRef {
    principal(PrincipalKind::User, "u_ines", "Ines Duarte")
}

fn default_owner(principal: PrincipalRef) -> AccessEntry {
    AccessEntry {
        default_owner: true,
        ..entry(principal, Role::Owner)
    }
}

/// Four jobs and a table: Noor owns one directly, the billing team edits one, the analysts
/// run one, and the admin group owns everything code-declared. Sam is on the billing team,
/// Ines is in the analysts group, and Noor is an admin.
fn ownership_backend() -> FakeBackend {
    let analysts = principal(PrincipalKind::Group, "billing_analysts", "Billing Analysts");
    FakeBackend {
        resources: vec![
            (
                "chronon_job",
                "noor-export",
                vec![default_owner(admins()), entry(noor(), Role::Owner)],
            ),
            (
                "chronon_job",
                "billing-rollup",
                vec![default_owner(admins()), entry(billing_team(), Role::Edit)],
            ),
            (
                "chronon_job",
                "invoice-sync",
                vec![default_owner(admins()), entry(analysts.clone(), Role::Run)],
            ),
            (
                "chronon_job",
                "nightly-vacuum",
                vec![default_owner(admins())],
            ),
            (
                "spectra_table",
                "billing_events",
                vec![entry(billing_team(), Role::Owner)],
            ),
        ],
        memberships: vec![
            ("u_noor", vec![admins()]),
            ("u_sam", vec![billing_team()]),
            ("u_ines", vec![analysts]),
        ],
        ..FakeBackend::default()
    }
}

#[test]
fn owned_by_includes_a_direct_owner_but_not_admin_default_ownership() {
    let backend = ownership_backend();
    let owned = block_on(backend.owned_by(&noor(), "chronon_job")).unwrap();
    assert_eq!(owned, vec!["noor-export".to_string()]);
}

#[test]
fn owned_by_includes_what_the_actors_team_can_edit() {
    let backend = ownership_backend();
    assert_eq!(
        block_on(backend.owned_by(&sam(), "chronon_job")).unwrap(),
        vec!["billing-rollup".to_string()]
    );
    assert_eq!(
        block_on(backend.owned_by(&sam(), "spectra_table")).unwrap(),
        vec!["billing_events".to_string()],
        "only the asked kind comes back"
    );
}

#[test]
fn owned_by_leaves_out_can_run() {
    let backend = ownership_backend();
    assert!(block_on(backend.owned_by(&ines(), "chronon_job"))
        .unwrap()
        .is_empty());
}

#[test]
fn owned_by_defaults_to_empty_and_the_unwired_backend_returns_empty() {
    // FakeBackend with no resources behaves like the trait's default body.
    assert_eq!(
        block_on(FakeBackend::default().owned_by(&noor(), "chronon_job")),
        Ok(Vec::new())
    );
    assert_eq!(
        block_on(UnwiredBackend.owned_by(&noor(), "chronon_job")),
        Ok(Vec::new())
    );
}

#[test]
fn trait_default_owned_by_returns_empty() {
    struct Minimal;

    #[async_trait::async_trait]
    impl ResourceAccessBackend for Minimal {
        async fn access(&self, _r: &ResourceRef) -> Result<AccessView, AccessError> {
            Err(AccessError::Unwired)
        }
        async fn set_role(
            &self,
            _r: &ResourceRef,
            _p: &PrincipalRef,
            _role: Role,
        ) -> Result<(), AccessError> {
            Ok(())
        }
        async fn remove(&self, _r: &ResourceRef, _p: &PrincipalRef) -> Result<(), AccessError> {
            Ok(())
        }
        async fn search_principals(&self, _q: &str) -> Result<PrincipalResults, AccessError> {
            Ok(PrincipalResults::default())
        }
        async fn request(
            &self,
            _r: &ResourceRef,
            _role: Role,
            _reason: &str,
        ) -> Result<RequestId, AccessError> {
            Ok(RequestId("r".to_string()))
        }
        async fn can(&self, _r: &ResourceRef, _action: &str) -> Result<bool, AccessError> {
            Ok(true)
        }
    }

    assert_eq!(
        block_on(Minimal.owned_by(&noor(), "chronon_job")),
        Ok(Vec::new())
    );
}

#[test]
fn counts_as_owned_follows_the_role_and_skips_default_owner() {
    assert!(entry(noor(), Role::Owner).counts_as_owned());
    assert!(entry(noor(), Role::Edit).counts_as_owned());
    assert!(entry(noor(), Role::Custom(vec!["View".into(), "Edit".into()])).counts_as_owned());
    assert!(!entry(noor(), Role::Custom(vec!["View".into(), "Run".into()])).counts_as_owned());
    assert!(!entry(noor(), Role::Run).counts_as_owned());
    assert!(!entry(noor(), Role::View).counts_as_owned());
    assert!(!entry(noor(), Role::ReadUnmasked).counts_as_owned());
    assert!(!default_owner(admins()).counts_as_owned());
}

#[test]
fn require_refuses_when_no_backend_is_wired() {
    let owner = Owner::new();
    owner.set();
    let err = block_on(require(&job(), "Run")).expect_err("no backend must refuse");
    assert!(
        err.to_string().contains("chronon_job.billing-rollup.Run"),
        "denial names the permission: {err}"
    );
}

#[test]
fn require_uses_the_wired_backend() {
    let owner = Owner::new();
    owner.set();
    let backend: Arc<dyn ResourceAccessBackend> = Arc::new(FakeBackend {
        allowed: vec!["View", "Run"],
        ..FakeBackend::default()
    });
    provide_resource_access_backend(backend);
    assert!(block_on(require(&job(), "Run")).is_ok());
    assert!(block_on(require(&job(), "Edit")).is_err());
}

#[test]
fn require_with_tells_denied_from_check_failed() {
    let r = job();
    let allowed = FakeBackend {
        allowed: vec!["View"],
        ..FakeBackend::default()
    };
    assert!(block_on(require_with(&allowed, &r, "View")).is_ok());
    let denied = block_on(require_with(&allowed, &r, "Run")).unwrap_err();

    let broken = FakeBackend {
        check_error: true,
        ..FakeBackend::default()
    };
    let failed = block_on(require_with(&broken, &r, "Run")).unwrap_err();
    assert_ne!(denied.to_string(), failed.to_string());
    assert!(
        failed.to_string().contains("permission_check_failed"),
        "{failed}"
    );
}

#[test]
fn role_labels_and_keys_round_trip() {
    let labels: Vec<String> = Role::STANDARD.iter().map(Role::label).collect();
    assert_eq!(labels, ["Can view", "Can run", "Can edit", "Owner"]);
    assert_eq!(Role::ReadUnmasked.label(), "Can read unmasked");
    assert_eq!(
        Role::Custom(vec!["Edit".to_string(), "Delete".to_string()]).label(),
        "Custom: Edit, Delete"
    );
    for role in [
        Role::View,
        Role::Run,
        Role::ReadUnmasked,
        Role::Edit,
        Role::Owner,
    ] {
        assert_eq!(Role::from_key(role.key()), Some(role));
    }
    assert_eq!(Role::from_key("custom"), None);
    assert_eq!(Role::from_key("admin"), None);
}

#[test]
fn role_descriptions_name_the_kind() {
    assert_eq!(
        Role::Run.describe("job"),
        "Can run: see the job and its runs, and start, retry or cancel a run."
    );
    assert_eq!(
        Role::ReadUnmasked.describe("table"),
        "Can read unmasked: see the table with personal fields shown."
    );
    assert_eq!(Role::Edit.you_can("topic"), "You can edit this topic.");
    assert_eq!(Role::Owner.you_can("queue"), "You own this queue.");
}

#[test]
fn role_for_action_picks_the_smallest_role() {
    assert_eq!(Role::for_action("View"), Role::View);
    assert_eq!(Role::for_action("Run"), Role::Run);
    assert_eq!(Role::for_action("Unmask"), Role::ReadUnmasked);
    assert_eq!(Role::for_action("Edit"), Role::Edit);
    assert_eq!(Role::for_action("Maintain"), Role::Owner);
    assert_eq!(Role::for_action("Delete"), Role::Owner);
}

#[test]
fn default_role_prefers_run() {
    assert_eq!(default_role(&Role::STANDARD), Role::Run);
    assert_eq!(default_role(&[Role::View, Role::Owner]), Role::View);
}

#[test]
fn resource_ref_builds_names() {
    let r = job();
    assert_eq!(r.permission_name("Run"), "chronon_job.billing-rollup.Run");
    assert_eq!(r.label_with_id(), "Billing rollup (billing-rollup)");
    assert_eq!(ResourceRef::new("k", "x").label_with_id(), "x");
    assert_eq!(ResourceRef::new("k", "x").noun, "resource");
}

#[test]
fn access_errors_carry_stable_prefixes() {
    let last = AccessError::LastOwner {
        resource: "billing-rollup".to_string(),
    };
    assert!(last.to_string().starts_with(LAST_OWNER_PREFIX));
    assert!(is_last_owner(&last));
    assert!(AccessError::Unwired.to_string().starts_with(UNWIRED_PREFIX));
    let denied = AccessError::Denied {
        action: "Run".to_string(),
        resource: "billing-rollup".to_string(),
    };
    assert!(denied
        .to_string()
        .contains("You need Run on billing-rollup."));
    let step_up = AccessError::StepUp("STEP_UP:step_up_required: code needed".to_string());
    assert_eq!(step_up.to_string(), "STEP_UP:step_up_required: code needed");
    assert!(is_step_up_challenge(&step_up));
    assert_eq!(
        AccessError::Backend {
            op: "load access".to_string(),
            message: "valence: connection refused".to_string()
        }
        .to_string(),
        "Failed to load access: valence: connection refused"
    );
}

#[test]
fn principal_secondary_lines() {
    assert_eq!(billing_team().secondary().as_deref(), Some("6 members"));
    let su = principal(
        PrincipalKind::ServiceUser {
            owner: Some("Ines".to_string()),
        },
        "su",
        "spectra-agent",
    );
    assert_eq!(
        su.secondary().as_deref(),
        Some("Service user · owned by Ines")
    );
    assert_eq!(noor().secondary(), None);
    assert_eq!(noor().key(), "user:u_noor");
}

#[test]
fn access_view_helpers() {
    let view = owner_view();
    assert_eq!(view.owner_names(), ["Chronon admins"]);
    assert!(!view.restricted());
    assert!(restricted_view().restricted());
    assert_eq!(view.role_of(&noor()), Some(&Role::View));
}

#[test]
fn fake_backend_refuses_to_drop_the_last_owner() {
    let backend = FakeBackend::with_view(owner_view());
    let r = job();
    let err = block_on(backend.set_role(&r, &admins(), Role::Edit)).unwrap_err();
    assert!(is_last_owner(&err));
    assert_eq!(
        write_error_text(&err.to_string(), &r.noun),
        "A job needs at least one owner. Make someone else an owner first."
    );
    assert!(block_on(backend.remove(&r, &admins())).is_err());
    // Another owner first, then the default owner can go.
    block_on(backend.set_role(&r, &noor(), Role::Owner)).unwrap();
    block_on(backend.remove(&r, &admins())).unwrap();
    let view = block_on(backend.access(&r)).unwrap();
    assert_eq!(view.owner_names(), ["Noor Haddad"]);
}

#[test]
fn reader_notice_names_role_and_owners() {
    assert_eq!(
        reader_notice(&reader_view(), "job"),
        "You can edit this job. To change who has access, ask an owner: Chronon admins."
    );
    let mut no_role = reader_view();
    no_role.viewer_role = None;
    assert_eq!(
        reader_notice(&no_role, "job"),
        "To change who has access, ask an owner: Chronon admins."
    );
}

#[test]
fn denied_text_helpers() {
    let r = job();
    assert_eq!(
        denied_sentence("View", &r, Some("see this job's runs and logs")),
        "You need View on billing-rollup to see this job's runs and logs."
    );
    assert_eq!(
        denied_sentence("Run", &r, None),
        "You need Run on billing-rollup."
    );
    assert_eq!(
        denied_tooltip("Run", &r, &["Chronon admins".to_string()]),
        "You need Run on billing-rollup. Owners: Chronon admins."
    );
    assert_eq!(
        goes_to_caption(&r, &["Chronon admins".to_string()]),
        "Goes to the owners of billing-rollup: Chronon admins. You will get a notification when they decide."
    );
    let info = DeniedInfo::from_load(&Ok(AccessLoad::Ready(owner_view())));
    assert_eq!(info.owners, ["Chronon admins"]);
    assert!(DeniedInfo::from_load(&Ok(AccessLoad::Unwired)).unwired);
    assert_eq!(
        DeniedInfo::from_load(&Err(ServerFnError::new("down"))),
        DeniedInfo::default()
    );
}

// ---- picker listing ----------------------------------------------------------------------

fn search_results(with_teams: bool) -> PrincipalResults {
    PrincipalResults {
        teams: if with_teams {
            vec![
                principal(
                    PrincipalKind::Team {
                        org: None,
                        member_count: 4,
                    },
                    "polaron_team_release",
                    "Release engineering",
                ),
                billing_team(),
            ]
        } else {
            Vec::new()
        },
        groups: vec![principal(
            PrincipalKind::Group,
            "release_captains",
            "Release Captains",
        )],
        people: vec![principal(PrincipalKind::User, "u_ravi", "Ravi Menon")],
    }
}

#[test]
fn picker_lists_teams_groups_people_in_order() {
    let results = search_results(true);
    let listing = build_listing("rel", Some(&results), &owner_view().entries);
    let keys: Vec<_> = listing.groups.iter().map(|g| g.label).collect();
    assert_eq!(keys, ["Teams", "Groups", "People"]);
    assert_eq!(listing.status, None);
    // Billing engineering is already on the card, as Can edit.
    let billing = &listing.groups[0].rows[1];
    assert_eq!(billing.principal, billing_team());
    assert_eq!(billing.existing, Some(Role::Edit));
    assert_eq!(listing.groups[0].rows[0].existing, None);
}

#[test]
fn picker_has_no_teams_heading_without_managed_teams() {
    let results = search_results(false);
    let listing = build_listing("rel", Some(&results), &[]);
    let keys: Vec<_> = listing.groups.iter().map(|g| g.key).collect();
    assert_eq!(keys, ["groups", "people"]);
}

#[test]
fn picker_status_lines() {
    assert_eq!(
        build_listing("r", None, &[]).status,
        Some("Type 2 or more letters to search.")
    );
    assert_eq!(build_listing("rel", None, &[]).status, Some("Searching…"));
    assert_eq!(
        build_listing("zzz", Some(&PrincipalResults::default()), &[]).status,
        Some("No one matches.")
    );
}

// ---- SSR renders: the Access card --------------------------------------------------------

#[test]
fn card_owner_state_has_controls() {
    let html =
        render(move || card_body(&job(), CardState::Ready(owner_view()), false, no_actions()));
    assert!(html.contains("access-card-add"), "Add people for owners");
    assert_eq!(html.matches("data-testid=\"access-row\"").count(), 5);
    assert_eq!(html.matches("access-row-role-select").count(), 5);
    assert_eq!(html.matches("access-row-menu").count(), 5);
    assert!(html.contains("Default owner"));
    assert!(html.contains("Kind-wide"));
    assert!(html.contains("access-row-team"));
    assert!(html.contains("6 members"));
    assert!(html.contains("Super User can always manage every job."));
    assert!(!html.contains("access-card-restricted"));
    assert!(!html.contains("access-card-reader-notice"));
}

#[test]
fn card_reader_state_is_read_only_with_own_role() {
    let html =
        render(move || card_body(&job(), CardState::Ready(reader_view()), false, no_actions()));
    assert!(!html.contains("access-card-add"));
    assert!(!html.contains("access-row-role-select"));
    assert!(!html.contains("access-row-menu"));
    assert!(html.contains(
        "You can edit this job. To change who has access, ask an owner: Chronon admins."
    ));
    assert_eq!(html.matches("data-testid=\"access-row-role\"").count(), 5);
    assert!(html.contains("Can edit"));
}

#[test]
fn card_restricted_state_says_only_listed_rows_see_it() {
    let html = render(move || {
        card_body(
            &job(),
            CardState::Ready(restricted_view()),
            false,
            no_actions(),
        )
    });
    assert!(html.contains("Only the people and groups above can see this job."));
    assert_eq!(html.matches("data-testid=\"access-row\"").count(), 4);
}

#[test]
fn card_loading_state_shows_skeleton_rows() {
    let html = render(move || card_body(&job(), CardState::Loading, false, no_actions()));
    assert!(html.contains("access-card-loading"));
    assert!(!html.contains("access-card-add"));
    assert!(html.contains("Who can view, run, edit and own this job."));
}

#[test]
fn card_error_state_keeps_prefix_and_offers_retry() {
    let backend = FakeBackend {
        load_error: Some("valence: connection refused (os error 111)"),
        ..FakeBackend::default()
    };
    let message = block_on(backend.access(&job())).unwrap_err().to_string();
    let state = CardState::from_load(Err(ServerFnError::new(message)));
    let html = render(move || card_body(&job(), state, false, no_actions()));
    assert!(
        html.contains("Couldn&#x27;t load who has access:")
            || html.contains("Couldn't load who has access:")
    );
    assert!(html.contains("Failed to load access: valence: connection refused (os error 111)"));
    assert!(html.contains("access-card-retry"));
}

#[test]
fn card_unwired_state_says_why() {
    let state = CardState::from_load(Ok(AccessLoad::Unwired));
    assert_eq!(state, CardState::Unwired);
    let html = render(move || card_body(&job(), state, false, no_actions()));
    assert!(html.contains("access-card-unwired"));
    assert!(html.contains("set up on this host."));
    assert!(!html.contains("data-testid=\"access-row\""));
}

#[test]
fn card_rows_stack_when_narrow() {
    let wide =
        render(move || card_body(&job(), CardState::Ready(owner_view()), false, no_actions()));
    let narrow =
        render(move || card_body(&job(), CardState::Ready(owner_view()), true, no_actions()));
    assert_ne!(wide, narrow, "stacked rows render a vertical layout");
}

// ---- SSR renders: dialogs ----------------------------------------------------------------

#[test]
fn add_dialog_fields_show_role_help() {
    let html = render(move || {
        let who = RwSignal::new(None);
        let role_key = RwSignal::new(Role::Run.key().to_string());
        let error = RwSignal::new(None);
        add_access_fields(
            "job".to_string(),
            who,
            role_key,
            Role::STANDARD.to_vec(),
            Signal::derive(|| owner_view().entries),
            error,
        )
        .into_any()
    });
    assert!(html.contains("principal-picker"));
    assert!(html.contains("Can run: see the job and its runs, and start, retry or cancel a run."));
    assert!(!html.contains("add-access-error"));
}

#[test]
fn add_dialog_keeps_choice_on_save_error() {
    let html = render(move || {
        let who = RwSignal::new(Some(billing_team()));
        let role_key = RwSignal::new(Role::Run.key().to_string());
        let error = RwSignal::new(Some(
            "error running server function: Failed to set access: valence: connection refused"
                .to_string(),
        ));
        add_access_fields(
            "job".to_string(),
            who,
            role_key,
            Role::STANDARD.to_vec(),
            Signal::derive(Vec::new),
            error,
        )
        .into_any()
    });
    assert!(
        html.contains("principal-picker-pick"),
        "the pick stays as a tag"
    );
    assert!(html.contains("Billing engineering"));
    assert!(html.contains("Failed to set access"));
}

#[test]
fn remove_dialog_explains_last_owner_refusal() {
    let r = job();
    let err = AccessError::LastOwner {
        resource: r.id.clone(),
    }
    .to_string();
    let html = render(move || remove_access_body("Chronon admins", &r, Some(err)).into_any());
    assert!(html.contains("Chronon admins loses access to billing-rollup right away."));
    assert!(html.contains("A job needs at least one owner. Make someone else an owner first."));
}

#[test]
fn request_dialog_fills_in_resource_and_owners() {
    let r = job();
    let html = render(move || {
        let role_key = RwSignal::new(Role::View.key().to_string());
        let reason = RwSignal::new("I'm on call tonight.".to_string());
        let error = RwSignal::new(None);
        request_access_fields(
            &r,
            role_key,
            &Role::STANDARD,
            reason,
            &["Chronon admins".to_string()],
            error,
        )
        .into_any()
    });
    assert!(html.contains("Why do you need it?"));
    assert!(html.contains("Goes to the owners of billing-rollup: Chronon admins."));
    assert!(
        html.contains("Job"),
        "the resource field is labelled by the kind"
    );
}

// ---- SSR renders: denied states ----------------------------------------------------------

#[test]
fn denied_state_names_action_and_reviewers() {
    let r = job();
    let info = DeniedInfo::from_load(&Ok(AccessLoad::Ready(owner_view())));
    let html = render(move || {
        denied_view(
            &r,
            "View",
            Some("see this job's runs and logs"),
            &info,
            None,
            Callback::new(|()| {}),
        )
    });
    assert!(html.contains("You need View on billing-rollup to see this job"));
    assert!(html.contains("Owners review requests: Chronon admins."));
    assert!(html.contains("access-denied-request"));
    assert!(!html.contains("access-denied-sent"));
}

#[test]
fn denied_state_after_request_shows_sent_time() {
    let r = job();
    let info = DeniedInfo::from_load(&Ok(AccessLoad::Ready(owner_view())));
    let backend = FakeBackend::default();
    let id = block_on(backend.request(&r, Role::View, "on call")).unwrap();
    let receipt = AccessRequestReceipt {
        id,
        sent_at: 1_795_442_529,
    };
    let html = render(move || {
        denied_view(
            &r,
            "View",
            None,
            &info,
            Some(receipt),
            Callback::new(|()| {}),
        )
    });
    assert!(html.contains("Request sent"));
    assert!(html.contains("Sent at 14:02 UTC to Chronon admins"));
    assert!(!html.contains("access-denied-request"), "no second request");
}

#[test]
fn denied_state_on_unwired_host_offers_no_request() {
    let r = job();
    let info = DeniedInfo::from_load(&Ok(AccessLoad::Unwired));
    let html = render(move || denied_view(&r, "Run", None, &info, None, Callback::new(|()| {})));
    assert!(html.contains("so nobody can grant this yet."));
    assert!(!html.contains("access-denied-request"));
}

#[test]
fn denied_button_stays_visible_with_ask_link() {
    let html = render(move || {
        denied_action_view(
            "Run now",
            "Run",
            Signal::derive(|| "You need Run on billing-rollup.".to_string()),
            false,
            Callback::new(|()| {}),
        )
    });
    assert!(html.contains("denied-action-button"));
    assert!(html.contains("Run now"));
    assert!(html.contains("disabled") || html.contains("aria-disabled"));
    assert!(html.contains("Ask for Run access"));

    let sent = render(move || {
        denied_action_view(
            "Run now",
            "Run",
            Signal::derive(String::new),
            true,
            Callback::new(|()| {}),
        )
    });
    assert!(sent.contains("denied-action-sent"));
    assert!(!sent.contains("Ask for Run access"));
}
