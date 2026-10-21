//! A dialog on wide screens and a bottom drawer on narrow ones.
//!
//! Forms and confirmations read better as a centred `Dialog` at the Md
//! breakpoint and up, and as a sheet sliding up from the bottom of the screen
//! below it. [`AdaptiveSheet`] keeps one body and one set of actions and picks
//! the chrome from `use_breakpoint_down(Breakpoint::Md)`.
//!
//! The breakpoint is read from a media query on the client. The sheet renders
//! nothing visible until `open` is true, so the first paint has no chrome to
//! mismatch.

use leptos::prelude::*;

use crate::primitives::{
    use_breakpoint_down, Breakpoint, Dialog, DialogActions, DialogBody, DialogContent,
    DialogSurface, DialogTitle, DrawerBody, DrawerHeader, DrawerHeaderTitle, DrawerPosition,
    DrawerSize, Flex, FlexGap, FlexJustify, FlexWrap, OverlayDrawer,
};

/// A form or confirmation that is a `Dialog` at Md and up and a bottom sheet
/// below Md.
///
/// Forms use the default full-height sheet; short confirmations pass
/// `size=DrawerSize::Small`. Body and actions are built again when the
/// breakpoint changes, so keep form state in signals the caller owns.
/// Loading, empty and error states belong in the body.
///
/// ```rust,ignore
/// use uf_product::AdaptiveSheet;
/// use uf_product::primitives::{Button, ButtonAppearance};
///
/// view! {
///     <AdaptiveSheet
///         open=open
///         title=|| "Create project"
///         testid="project-create-sheet"
///         actions=move || view! {
///             <Button on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
///             <Button appearance=ButtonAppearance::Primary on_click=confirm>"Create"</Button>
///         }
///     >
///         <Field label="Name">"…"</Field>
///     </AdaptiveSheet>
/// }
/// ```
#[component]
pub fn AdaptiveSheet(
    /// Open bind. The caller owns the signal.
    open: RwSignal<bool>,
    /// Heading content.
    #[prop(into)]
    title: ViewFn,
    /// Sets `data-testid` on the content element at every width: the dialog
    /// surface at Md and up, the drawer body below Md. Required, so tests
    /// always have a handle.
    #[prop(into)]
    testid: String,
    /// Sheet height below Md; `DrawerSize::Full` unless set.
    #[prop(default = DrawerSize::Full)]
    size: DrawerSize,
    /// Buttons under the body, right-aligned.
    #[prop(into)]
    actions: ViewFn,
    /// Form or message body.
    children: ChildrenFn,
) -> impl IntoView {
    let narrow = use_breakpoint_down(Breakpoint::Md);
    let testid = StoredValue::new(testid);

    move || {
        let is_narrow = narrow.get();
        let title = title.clone();
        let actions = actions.clone();
        let children = children.clone();
        untrack(move || {
            if is_narrow {
                view! {
                    <OverlayDrawer
                        open=open
                        position=DrawerPosition::Bottom
                        size=size
                        close_on_esc=true
                    >
                        <DrawerHeader>
                            <DrawerHeaderTitle>{move || title.run()}</DrawerHeaderTitle>
                        </DrawerHeader>
                        <DrawerBody>
                            <Flex
                                vertical=true
                                gap=FlexGap::Medium
                                attr:data-testid=testid.get_value()
                            >
                                {children()}
                                <Flex
                                    justify=FlexJustify::End
                                    gap=FlexGap::Small
                                    wrap=FlexWrap::Wrap
                                >
                                    {actions.run()}
                                </Flex>
                            </Flex>
                        </DrawerBody>
                    </OverlayDrawer>
                }
                .into_any()
            } else {
                view! {
                    <Dialog open=open>
                        <DialogSurface attr:data-testid=testid.get_value()>
                            <DialogBody>
                                <DialogTitle>{move || title.run()}</DialogTitle>
                                <DialogContent>{children()}</DialogContent>
                                <DialogActions>{actions.run()}</DialogActions>
                            </DialogBody>
                        </DialogSurface>
                    </Dialog>
                }
                .into_any()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    /// Compile-level guard: the crate-root re-export keeps resolving.
    #[test]
    fn crate_root_reexport_resolves() {
        let _ = crate::AdaptiveSheet;
    }
}
