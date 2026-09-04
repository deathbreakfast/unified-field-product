use leptos::prelude::*;
use uf_product::components::StatCard;
use uf_product::primitives::{Grid, GridConfig, GridItem, MessageBar, MessageBarIntent};

use crate::server::{get_notification_count, get_today_count, get_unread_count};

/// Unread / total / today-count summary cards above the notification list.
#[component]
pub fn NotificationStatsGrid(
    /// Bumped after bulk operations (e.g. "Mark all read") so stats refetch.
    trigger: RwSignal<u64>,
) -> impl IntoView {
    let total_res = Resource::new(move || trigger.get(), |_| get_notification_count());
    let unread_res = Resource::new(move || trigger.get(), |_| get_unread_count());
    let today_res = Resource::new(move || trigger.get(), |_| get_today_count());

    view! {
        <Suspense fallback=move || view! { <StatsGridSkeleton /> }>
            {move || match (total_res.get(), unread_res.get(), today_res.get()) {
                (Some(Ok(total)), Some(Ok(unread)), Some(Ok(today))) => {
                    view! {
                        <Grid config=GridConfig::with_gaps(3, 16, 0)>
                            <GridItem><StatCard label="Unread" value=Signal::derive(move || unread.to_string()) /></GridItem>
                            <GridItem><StatCard label="Total" value=Signal::derive(move || total.to_string()) /></GridItem>
                            <GridItem><StatCard label="Today" value=Signal::derive(move || today.to_string()) /></GridItem>
                        </Grid>
                    }.into_any()
                }
                (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => view! {
                    <MessageBar intent=MessageBarIntent::Error>
                        "Failed to load stats: " {e.to_string()}
                    </MessageBar>
                }.into_any(),
                _ => view! { <StatsGridSkeleton /> }.into_any(),
            }}
        </Suspense>
    }
}

/// Skeleton for stats grid while loading
#[component]
pub fn StatsGridSkeleton() -> impl IntoView {
    view! {
        <Grid config=GridConfig::with_gaps(3, 16, 0)>
            <GridItem><StatCard label="Unread" value=Signal::derive(|| "—".to_string()) /></GridItem>
            <GridItem><StatCard label="Total" value=Signal::derive(|| "—".to_string()) /></GridItem>
            <GridItem><StatCard label="Today" value=Signal::derive(|| "—".to_string()) /></GridItem>
        </Grid>
    }
}
