//! Small pieces the access views share: name lists, the sent time, and principal rows.

use leptos::prelude::*;

use super::{PrincipalKind, PrincipalRef};
use crate::components::{Persona, PersonaConfig, PersonaSecondaryText, PersonaSize};
use crate::primitives::{AvatarColor, Badge, BadgeAppearance, BadgeSize};

/// "A", "A and B", "A, B and C".
#[must_use]
pub fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// "Sent at 14:02 UTC to Chronon admins", from unix seconds.
#[must_use]
pub fn sent_caption(sent_at: i64, owners: &[String]) -> String {
    let secs = sent_at.rem_euclid(86_400);
    let time = format!("{:02}:{:02} UTC", secs / 3600, (secs % 3600) / 60);
    if owners.is_empty() {
        format!("Sent at {time}")
    } else {
        format!("Sent at {time} to {}", join_names(owners))
    }
}

/// A Persona for one principal, with its secondary line (team size, service-user owner).
pub fn principal_persona(principal: &PrincipalRef) -> AnyView {
    let color = match principal.kind {
        PrincipalKind::User | PrincipalKind::ServiceUser { .. } => AvatarColor::Colorful,
        PrincipalKind::Group | PrincipalKind::Team { .. } => AvatarColor::Neutral,
    };
    let config = PersonaConfig {
        size: PersonaSize::Medium,
        color,
        ..PersonaConfig::named(principal.label.clone())
    };
    match principal.secondary() {
        Some(secondary) => view! {
            <Persona config=config>
                <PersonaSecondaryText slot>{secondary}</PersonaSecondaryText>
            </Persona>
        }
        .into_any(),
        None => view! { <Persona config=config /> }.into_any(),
    }
}

/// A small tint badge, such as Team, Default owner or Kind-wide.
pub fn tint_badge(text: &'static str, testid: &'static str) -> impl IntoView {
    view! {
        <span data-testid=testid>
            <Badge appearance=BadgeAppearance::Tint size=BadgeSize::Small>
                {text}
            </Badge>
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_names_reads_as_a_list() {
        let names = |v: &[&str]| v.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(join_names(&[]), "");
        assert_eq!(join_names(&names(&["A"])), "A");
        assert_eq!(join_names(&names(&["A", "B"])), "A and B");
        assert_eq!(join_names(&names(&["A", "B", "C"])), "A, B and C");
    }

    #[test]
    fn sent_caption_formats_utc_time_and_owners() {
        // 2026-11-23 14:02:09 UTC
        let at = 1_795_442_529;
        assert_eq!(
            sent_caption(at, &["Chronon admins".to_string()]),
            "Sent at 14:02 UTC to Chronon admins"
        );
        assert_eq!(sent_caption(at, &[]), "Sent at 14:02 UTC");
    }
}
