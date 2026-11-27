//! When a notification arrived: day groups, readable time labels in the
//! reader's zone, and the `<time>` element the bell and inbox render (to be
//! replaced by Orbital `Timestamp`).

use chrono::{DateTime, Datelike, Local, SecondsFormat, TimeZone, Utc};
use leptos::prelude::*;

/// Which day heading a notification sits under in the inbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DayGroup {
    /// Arrived today (local time).
    Today,
    /// Arrived yesterday (local time).
    Yesterday,
    /// Anything older.
    Earlier,
}

impl DayGroup {
    /// Heading shown above the group.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Yesterday => "Yesterday",
            Self::Earlier => "Earlier",
        }
    }
}

/// The day group `then` falls in, seen from `now` (both in the reader's zone).
pub(crate) fn day_group<Tz: TimeZone>(now: &DateTime<Tz>, then: &DateTime<Tz>) -> DayGroup {
    let today = now.date_naive();
    let day = then.date_naive();
    if day >= today {
        DayGroup::Today
    } else if today.pred_opt() == Some(day) {
        DayGroup::Yesterday
    } else {
        DayGroup::Earlier
    }
}

/// Label for when a notification arrived: "Just now", "47 min ago", "2 h ago"
/// (same day), "Yesterday, 4:10 PM", or "Nov 21, 9:00 AM PST" (with the year
/// when it isn't this year).
pub(crate) fn time_label<Tz: TimeZone>(
    now: &DateTime<Tz>,
    then: &DateTime<Tz>,
    tz_abbrev: Option<&str>,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let elapsed = now.clone().signed_duration_since(then.clone());
    let minutes = elapsed.num_minutes();
    if minutes < 1 {
        return "Just now".to_string();
    }
    if minutes < 60 {
        return format!("{minutes} min ago");
    }
    let clock = then.format("%-I:%M %p");
    match day_group(now, then) {
        DayGroup::Today => format!("{} h ago", elapsed.num_hours()),
        DayGroup::Yesterday => format!("Yesterday, {clock}"),
        DayGroup::Earlier => {
            let date = if then.year() == now.year() {
                then.format("%b %-d").to_string()
            } else {
                then.format("%b %-d, %Y").to_string()
            };
            match tz_abbrev {
                Some(tz) => format!("{date}, {clock} {tz}"),
                None => format!("{date}, {clock}"),
            }
        }
    }
}

/// `true` once the page has hydrated in the browser, so times and day groups
/// can switch from UTC (what the server rendered) to the reader's local zone
/// without a hydration mismatch.
pub(crate) fn use_local_time() -> Signal<bool> {
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    ready.into()
}

/// Day group for a row; rows without a time go under Earlier.
pub(crate) fn row_day_group(created_at: Option<DateTime<Utc>>, local: bool) -> DayGroup {
    let Some(then) = created_at else {
        return DayGroup::Earlier;
    };
    if local {
        day_group(&Local::now(), &then.with_timezone(&Local))
    } else {
        day_group(&Utc::now(), &then)
    }
}

fn row_time_label(then: DateTime<Utc>, local: bool) -> String {
    if local {
        time_label(
            &Local::now(),
            &then.with_timezone(&Local),
            local_tz_abbrev().as_deref(),
        )
    } else {
        time_label(&Utc::now(), &then, Some("UTC"))
    }
}

/// The reader's short zone name, such as "PST", from the browser.
#[cfg(feature = "hydrate")]
fn local_tz_abbrev() -> Option<String> {
    use leptos::wasm_bindgen::JsValue;
    use leptos::web_sys::js_sys::{Date, Object, Reflect};

    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("timeZoneName"),
        &JsValue::from_str("short"),
    )
    .ok()?;
    let text: String = Date::new_0()
        .to_locale_date_string("en-US", &options)
        .into();
    text.rsplit(", ")
        .next()
        .filter(|tz| !tz.is_empty())
        .map(str::to_string)
}

#[cfg(not(feature = "hydrate"))]
fn local_tz_abbrev() -> Option<String> {
    None
}

/// When a notification arrived, as `<time datetime>` with a readable label.
// TODO: switch to Orbital `Timestamp` once the pinned Orbital exports it
// (it's on `feat/orbital-wave2`).
#[component]
pub(crate) fn NotificationTime(
    /// When the notification was created; `None` uses `fallback`.
    created_at: Option<DateTime<Utc>>,
    /// The API's relative label, shown when `created_at` is unknown.
    fallback: String,
    /// Whether to render in the reader's local zone (see [`use_local_time`]).
    local: Signal<bool>,
) -> impl IntoView {
    match created_at {
        Some(then) => {
            let datetime = then.to_rfc3339_opts(SecondsFormat::Secs, true);
            view! {
                <time datetime=datetime>{move || row_time_label(then, local.get())}</time>
            }
            .into_any()
        }
        None => view! { <span>{fallback}</span> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn pst() -> FixedOffset {
        FixedOffset::west_opt(8 * 3600).expect("valid offset")
    }

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<FixedOffset> {
        pst()
            .with_ymd_and_hms(y, m, d, h, min, 0)
            .single()
            .expect("valid time")
    }

    #[test]
    fn time_label_recent_happy() {
        let now = at(2026, 11, 24, 17, 29);
        assert_eq!(time_label(&now, &now, None), "Just now");
        assert_eq!(
            time_label(&now, &at(2026, 11, 24, 16, 42), None),
            "47 min ago"
        );
        assert_eq!(time_label(&now, &at(2026, 11, 24, 15, 20), None), "2 h ago");
    }

    #[test]
    fn time_label_older_days_happy() {
        let now = at(2026, 11, 24, 17, 29);
        assert_eq!(
            time_label(&now, &at(2026, 11, 23, 16, 10), Some("PST")),
            "Yesterday, 4:10 PM"
        );
        assert_eq!(
            time_label(&now, &at(2026, 11, 21, 9, 0), Some("PST")),
            "Nov 21, 9:00 AM PST"
        );
        assert_eq!(
            time_label(&now, &at(2026, 11, 21, 9, 0), None),
            "Nov 21, 9:00 AM"
        );
        assert_eq!(
            time_label(&now, &at(2025, 12, 30, 21, 5), Some("PST")),
            "Dec 30, 2025, 9:05 PM PST"
        );
    }

    #[test]
    fn time_label_future_clock_skew_sad() {
        let now = at(2026, 11, 24, 17, 29);
        assert_eq!(
            time_label(&now, &at(2026, 11, 24, 17, 35), None),
            "Just now"
        );
    }

    #[test]
    fn day_group_buckets_happy() {
        let now = at(2026, 11, 24, 0, 30);
        assert_eq!(day_group(&now, &at(2026, 11, 24, 0, 5)), DayGroup::Today);
        assert_eq!(
            day_group(&now, &at(2026, 11, 23, 23, 50)),
            DayGroup::Yesterday
        );
        assert_eq!(day_group(&now, &at(2026, 11, 22, 12, 0)), DayGroup::Earlier);
        assert_eq!(DayGroup::Earlier.label(), "Earlier");
    }

    #[test]
    fn row_day_group_without_time_is_earlier_sad() {
        assert_eq!(row_day_group(None, false), DayGroup::Earlier);
        assert_eq!(row_day_group(Some(Utc::now()), false), DayGroup::Today);
    }
}
