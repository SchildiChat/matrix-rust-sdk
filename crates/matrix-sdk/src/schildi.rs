/// SchildiChat's user-controlled settings for inbox sorting and filtering.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScInboxSettings {
    /// The sort order to apply to the inbox.
    pub sort_order: ScSortOrder,
}

/// SchildiChat's user-controlled inbox sort-order settings.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScSortOrder {
    /// Whether to sort unread chats above read chats.
    pub by_unread: bool,
    /// Whether to sort favorite chat above anything else.
    pub pin_favorites: bool,
    /// Whether to sort low-priority chats at the bottom.
    pub bury_low_priority: bool,
    /// Whether to sort by client-generated or server-reported unread counts,
    /// when sorting by unread.
    pub client_generated_unread: bool,
    /// Whether to include non-notification/mention unread counts when sorting by unread.
    pub with_silent_unread: bool,
}
