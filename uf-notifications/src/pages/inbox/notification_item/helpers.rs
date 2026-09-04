//! Turf class names for inbox notification row read/unread affordances.

pub(super) struct ItemClassNames {
    pub item_read: String,
    pub unread_dot: String,
    pub unread_dot_hidden: String,
    pub row: String,
    pub hit_fill: String,
    pub side_footer: String,
}

pub(super) fn item_style_sheet() -> (String, ItemClassNames) {
    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .ItemRead {
            opacity: 0.72;
        }

        .UnreadDot {
            flex-shrink: 0;
        }

        .UnreadDotHidden {
            opacity: 0;
        }

        .Row {
            min-width: 0;
        }

        .HitFill {
            flex: 1 1 0%;
            min-width: 0;
            align-self: stretch;
        }

        .SideFooter {
            flex: 0 0 auto;
            flex-direction: column;
            justify-content: flex-end;
            align-items: flex-end;
            padding: 0 16px 16px 0;
        }
    };

    (
        style_sheet.to_string(),
        ItemClassNames {
            item_read: class_names.item_read.to_string(),
            unread_dot: class_names.unread_dot.to_string(),
            unread_dot_hidden: class_names.unread_dot_hidden.to_string(),
            row: class_names.row.to_string(),
            hit_fill: class_names.hit_fill.to_string(),
            side_footer: class_names.side_footer.to_string(),
        },
    )
}

pub(super) fn mark_read_label(is_read: bool) -> &'static str {
    if is_read {
        "Mark unread"
    } else {
        "Mark read"
    }
}
