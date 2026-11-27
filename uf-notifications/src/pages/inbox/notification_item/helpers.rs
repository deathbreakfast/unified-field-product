//! Turf class names for notification rows (inbox and bell).

pub(crate) struct ItemClassNames {
    pub unread_dot: String,
    pub unread_dot_hidden: String,
    pub content: String,
    pub message_text: String,
}

pub(crate) fn item_style_sheet() -> (String, ItemClassNames) {
    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .UnreadDot {
            flex-shrink: 0;
        }

        .UnreadDotHidden {
            visibility: hidden;
        }

        .Content {
            min-width: 0;
            flex: 1 1 0%;
        }

        .MessageText {
            display: -webkit-box;
            -webkit-line-clamp: 2;
            -webkit-box-orient: vertical;
            overflow: hidden;
            min-width: 0;
        }
    };

    (
        style_sheet.to_string(),
        ItemClassNames {
            unread_dot: class_names.unread_dot.to_string(),
            unread_dot_hidden: class_names.unread_dot_hidden.to_string(),
            content: class_names.content.to_string(),
            message_text: class_names.message_text.to_string(),
        },
    )
}

pub(crate) fn mark_read_label(is_read: bool) -> &'static str {
    if is_read {
        "Mark unread"
    } else {
        "Mark read"
    }
}
