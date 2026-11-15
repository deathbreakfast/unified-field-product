//! Product app bar: branding, breadcrumbs, search, and host-controlled utilities.
//!
//! On viewports below [`Breakpoint::Md`], wraps the sticky bar in Orbital
//! [`HideOnScroll`] so chrome tucks on scroll-down and returns on scroll-up.
//!
//! Session and appearance preference storage live in `uf-product`. The auth menu
//! widget is supplied by the host via [`crate::ShellAuthMenu`]
//! (`lepton_shell::AppBarUserMenu`). Help, Apps, and Appearance buttons come
//! from sibling offering crates when the default utilities pack is enabled.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Top app bar for a product shell | [`UnifiedFieldAppBar`] |
//! | Breadcrumb trail | [`BreadcrumbLink`] |
//! | Host-provided auth menu region | [`ShellAuthMenu`] |
//! | Host-controlled trailing utilities | [`AppBarUtilities`] slot (omit for [`DefaultAppBarUtilities`]) |
//! | Optional search region | [`AppBarSearchSlot`] |
//!
//! # Example
//!
//! Crate-root Getting started shows full shell composition. For the search
//! combobox alone, see [`crate::SearchSourcePicker`].

use crate::shell_layout::ShellSidebarToggle;
use crate::{HostNotificationBell, WorkspaceSearch, WorkspaceSearchMobileTrigger};
use leptos::prelude::*;
use orbital_base_components::use_breakpoint_down;
use orbital_theme::Breakpoint;
use uf_product::collect_app_bar_utilities;
use uf_product::components::{
    AppBar, AppBarDensity, AppBarLeading, AppBarMaterial, AppBarPosition, AppBarTrailing,
    HideOnScroll, LayoutSidebarToggle, MaterialCorners, MaterialElevation, MaterialVariant, Title3,
};
use uf_product::primitives::{
    Avatar, AvatarColor, AvatarConfig, AvatarShape, Box, Breadcrumb, BreadcrumbButton,
    BreadcrumbDivider, BreadcrumbItem, Flex, FlexAlign, FlexGap, FlexJustify, Link,
};
use uf_product::provide_app_bar_menu_extras;
use uf_product::theme::product_avatar_letter;

/// Optional auth menu slot — host passes `lepton_shell::AppBarUserMenu` here.
#[slot]
pub struct ShellAuthMenu {
    /// The auth menu content, typically a user avatar/dropdown component.
    pub children: Children,
}

/// Optional trailing utilities slot — host children replace the default offering pack.
#[slot]
pub struct AppBarUtilities {
    /// Host-controlled trailing utilities (buttons, menus, etc.).
    pub children: ChildrenFn,
}

/// Breadcrumb navigation item for [`UnifiedFieldAppBar`].
#[derive(Clone, PartialEq, Eq)]
pub struct BreadcrumbLink {
    /// Display text for the breadcrumb.
    pub title: String,
    /// Link target the breadcrumb navigates to when clicked.
    pub url: String,
}

impl BreadcrumbLink {
    /// Construct a breadcrumb entry from a title and link target.
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
        }
    }
}

#[component]
fn AppBarBranding(
    /// App name.
    app_name: Signal<String>,
    /// Avatar letter.
    avatar_letter: Signal<String>,
    /// Homepage URL.
    homepage_url: String,
) -> impl IntoView {
    view! {
        <Link href=homepage_url>
            <Flex align=FlexAlign::Center gap=FlexGap::Medium>
                {move || {
                    let name = app_name.get();
                    view! {
                        <Avatar config=AvatarConfig {
                            initials: Some(avatar_letter.get()),
                            name: Some(name.clone()),
                            shape: AvatarShape::Square,
                            size: Some(32),
                            color: AvatarColor::Brand,
                            ..Default::default()
                        } />
                        <Title3>{name}</Title3>
                    }
                }}
            </Flex>
        </Link>
    }
}

#[component]
fn AppBarBreadcrumbs(
    /// List of breadcrumbs.
    breadcrumbs: Vec<BreadcrumbLink>,
) -> impl IntoView {
    if breadcrumbs.is_empty() {
        let _: () = view! { <></> };
        return ().into_any();
    }

    // Parents are BreadcrumbButton links and the last crumb is the current
    // page, with a divider between each.
    let last = breadcrumbs.len() - 1;
    view! {
        <Breadcrumb>
            {breadcrumbs.into_iter().enumerate().map(|(i, breadcrumb)| {
                let item = if i == last {
                    view! {
                        <BreadcrumbItem>
                            <BreadcrumbButton current=true>{breadcrumb.title}</BreadcrumbButton>
                        </BreadcrumbItem>
                    }
                    .into_any()
                } else {
                    view! {
                        <BreadcrumbItem>
                            <BreadcrumbButton href=breadcrumb.url>{breadcrumb.title}</BreadcrumbButton>
                        </BreadcrumbItem>
                    }
                    .into_any()
                };
                view! {
                    {(i > 0).then(|| view! { <BreadcrumbDivider /> })}
                    {item}
                }
            }).collect::<Vec<_>>()}
        </Breadcrumb>
    }
    .into_any()
}

/// Default workspace content-index search for product shells.
#[component]
pub fn AppBarSearchSlot() -> impl IntoView {
    view! { <WorkspaceSearch /> }
}

/// Default trailing utilities from linked product offerings (inventory).
///
/// With `uf-integrations` feature `full` (and a host that links `uf-apps`), this
/// typically renders Help → Apps → Appearance. Hosts may replace it via
/// [`AppBarUtilities`].
#[component]
pub fn DefaultAppBarUtilities() -> impl IntoView {
    #[cfg(feature = "offering-help")]
    uf_help::ensure_linked();
    #[cfg(feature = "offering-appearance")]
    uf_appearance::ensure_linked();
    #[cfg(feature = "offering-apps")]
    {
        // `uf-apps` is not a dep of this crate (cycle with AppsLayout). Hosts that
        // enable `offering-apps` / `full` should also depend on `uf-apps` so its
        // inventory row is linked (shell-chrome-host and product hosts already do).
    }
    #[cfg(feature = "offering-notifications")]
    {
        // `uf-notifications` is not a dep here (cycle with shell). Hosts that
        // enable `offering-notifications` / `full` should depend on
        // `uf-notifications` so `HostNotificationBell` inventory fallback links.
    }

    let items = collect_app_bar_utilities();
    view! {
        {items
            .into_iter()
            .map(|c| (c.render)())
            .collect_view()}
    }
}

/// Alias for [`DefaultAppBarUtilities`] (legacy name).
#[component]
pub fn AppBarTrailingSlot() -> impl IntoView {
    view! { <DefaultAppBarUtilities /> }
}

/// Unified Field product app bar — branding, search, and shell utilities.
#[component]
pub fn UnifiedFieldAppBar(
    /// App name. Pass a signal to change the name without remounting the bar.
    #[prop(into)]
    app_name: Signal<String>,
    /// Optional app ID. Pass a signal to follow the current app.
    #[prop(optional, into)]
    app_id: Option<Signal<&'static str>>,
    /// Optional app logo initial.
    #[prop(optional)]
    app_logo_initial: Option<String>,
    /// Optional homepage URL.
    #[prop(optional)]
    homepage_url: Option<String>,
    /// Optional breadcrumbs.
    #[prop(optional)]
    breadcrumbs: Option<Vec<BreadcrumbLink>>,
    /// Whether interactive is enabled.
    #[prop(default = true)]
    interactive: bool,
    /// Optional flag for whether to show sidebar toggle.
    #[prop(optional)]
    show_sidebar_toggle: Option<bool>,
    /// Optional auth menu.
    #[prop(optional)]
    auth_menu: Option<ShellAuthMenu>,
    /// Optional trailing utilities (omit to use [`DefaultAppBarUtilities`]).
    #[prop(optional)]
    app_bar_utilities: Option<AppBarUtilities>,
) -> impl IntoView {
    let avatar_letter = Signal::derive(move || {
        app_id
            .map(|id| product_avatar_letter(id.get()).to_string())
            .or_else(|| app_logo_initial.clone())
            .unwrap_or_else(|| {
                app_name
                    .get()
                    .chars()
                    .next()
                    .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string())
            })
    });
    let homepage_url = homepage_url.unwrap_or_else(|| "/".to_string());
    let breadcrumbs = breadcrumbs.unwrap_or_default();
    let show_sidebar_toggle = show_sidebar_toggle
        .or_else(|| use_context::<ShellSidebarToggle>().map(|ctx| ctx.0))
        .unwrap_or(false);

    // Same breakpoint as Layout sidebar overlay — one avatar/login menu on phones.
    let compact = use_breakpoint_down(Breakpoint::Md);
    provide_app_bar_menu_extras(compact.into());

    let utilities_children = app_bar_utilities.map(|AppBarUtilities { children }| children);

    view! {
        <HideOnScroll enabled=Signal::derive(move || compact.get())>
            <AppBar
                attr:style=move || {
                    if interactive {
                        String::new()
                    } else {
                        "pointer-events: none".to_string()
                    }
                }
                position=AppBarPosition::Sticky
                density=AppBarDensity::Compact
            >
                <AppBarMaterial
                    variant=MaterialVariant::Frost
                    elevation=MaterialElevation::Flat
                    corners=MaterialCorners::Square
                    slot
                />
                <AppBarLeading slot>
                    <Flex align=FlexAlign::Center gap=FlexGap::Medium full_width=true>
                        {show_sidebar_toggle.then(|| view! {
                            <LayoutSidebarToggle />
                        })}
                        <AppBarBranding
                            app_name=app_name
                            avatar_letter=avatar_letter
                            homepage_url=homepage_url
                        />
                        <AppBarBreadcrumbs breadcrumbs=breadcrumbs />
                    </Flex>
                </AppBarLeading>
                <AppBarTrailing slot>
                    <Flex
                        align=FlexAlign::Center
                        gap=FlexGap::Small
                        justify=FlexJustify::End
                        full_width=true
                    >
                        {move || {
                            let utilities_view = match &utilities_children {
                                Some(children) => children().into_any(),
                                None => view! { <DefaultAppBarUtilities /> }.into_any(),
                            };
                            if compact.get() {
                                view! {
                                    <div data-testid="app-bar-trailing-compact">
                                        <Flex align=FlexAlign::Center gap=FlexGap::Small>
                                            <WorkspaceSearchMobileTrigger />
                                            {utilities_view}
                                        </Flex>
                                    </div>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <>
                                        <Box
                                            width="min(250px, 100%)"
                                            style="flex: 1 1 auto; min-width: 0; max-width: 250px; margin-right: auto;"
                                        >
                                            <div data-testid="app-bar-search">
                                                <AppBarSearchSlot />
                                            </div>
                                        </Box>
                                        <div data-testid="app-bar-trailing">
                                            <Flex align=FlexAlign::Center gap=FlexGap::Small>
                                                {utilities_view}
                                            </Flex>
                                        </div>
                                    </>
                                }
                                .into_any()
                            }
                        }}
                        <div data-testid="app-bar-notification-bell">
                            <HostNotificationBell />
                        </div>
                        <div data-testid="app-bar-user-menu">
                            {auth_menu.map(|ShellAuthMenu { children }| children())}
                        </div>
                    </Flex>
                </AppBarTrailing>
            </AppBar>
        </HideOnScroll>
    }
}

/// App bar that follows the current route, for hosts that keep one shell mounted.
///
/// Resolves the app from the location pathname with
/// [`uf_product::telemetry::resolve_app_for_path`] and the host's route table, so
/// moving between apps updates the name and avatar without remounting the bar
/// (no data refetch, no notifications socket reconnect). Mount it inside
/// [`crate::UnifiedFieldShellLayout`]; the stock app layouts then skip their own shell.
#[component]
pub fn RoutedAppBar(
    /// Route table, the same one passed to `PageViewTracker`.
    routes: &'static [uf_product::telemetry::UfAppRouteEntry],
    /// Name shown when no entry in `routes` matches the path.
    #[prop(into)]
    fallback_name: String,
) -> impl IntoView {
    let pathname = leptos_router::hooks::use_location().pathname;
    let current = Signal::derive(move || {
        let path = pathname.get();
        let entry = uf_product::telemetry::resolve_app_for_path(&path, routes);
        (entry.route_prefix != "/").then_some(entry)
    });
    let app_name = Signal::derive(move || {
        current
            .get()
            .map_or_else(|| fallback_name.clone(), |e| e.app_name.to_string())
    });
    let app_id = Signal::derive(move || current.get().map_or("shell", |e| e.app_id));

    view! {
        <UnifiedFieldAppBar app_name=app_name app_id=app_id homepage_url="/".to_string()>
            <ShellAuthMenu slot:auth_menu>
                <crate::HostAuthMenu />
            </ShellAuthMenu>
        </UnifiedFieldAppBar>
    }
}
