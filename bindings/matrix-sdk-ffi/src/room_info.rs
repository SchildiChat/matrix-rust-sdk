use std::collections::HashMap;

use matrix_sdk::{EncryptionState, RoomState};
use tracing::warn;

use crate::{
    client::JoinRule,
    error::ClientError,
    notification_settings::RoomNotificationMode,
    room::{Membership, RoomHero, RoomHistoryVisibility, SuccessorRoom},
    room_member::RoomMember,
    space_child_info::{SpaceChildInfo, space_children_info},
    event::StateEventType,
};

#[derive(uniffi::Record)]
pub struct RoomInfo {
    id: String,
    encryption_state: EncryptionState,
    creator: Option<String>,
    /// The room's name from the room state event if received from sync, or one
    /// that's been computed otherwise.
    display_name: Option<String>,
    /// Room name as defined by the room state event only.
    raw_name: Option<String>,
    topic: Option<String>,
    avatar_url: Option<String>,
    is_direct: bool,
    is_public: bool,
    is_space: bool,
    /// If present, it means the room has been archived/upgraded.
    successor_room: Option<SuccessorRoom>,
    is_favourite: bool,
    is_low_priority: bool,
    canonical_alias: Option<String>,
    alternative_aliases: Vec<String>,
    membership: Membership,
    /// Member who invited the current user to a room that's in the invited
    /// state.
    ///
    /// Can be missing if the room membership invite event is missing from the
    /// store.
    inviter: Option<RoomMember>,
    heroes: Vec<RoomHero>,
    active_members_count: u64,
    invited_members_count: u64,
    joined_members_count: u64,
    user_power_levels: HashMap<String, i64>,
    highlight_count: u64,
    notification_count: u64,
    unread_count: u64,
    cached_user_defined_notification_mode: Option<RoomNotificationMode>,
    has_room_call: bool,
    active_room_call_participants: Vec<String>,
    /// SC: Space-specific fields
    space_children: Vec<SpaceChildInfo>,
    can_user_manage_spaces: bool,
    /// Whether this room has been explicitly marked as unread
    is_marked_unread: bool,
    /// "Interesting" messages received in that room, independently of the
    /// notification settings.
    num_unread_messages: u64,
    /// Events that will notify the user, according to their
    /// notification settings.
    num_unread_notifications: u64,
    /// Events causing mentions/highlights for the user, according to their
    /// notification settings.
    num_unread_mentions: u64,
    /// The currently pinned event ids.
    pinned_event_ids: Vec<String>,
    /// The join rule for this room, if known.
    join_rule: Option<JoinRule>,
    /// The history visibility for this room, if known.
    history_visibility: RoomHistoryVisibility,
}

impl RoomInfo {
    pub(crate) async fn new(room: &matrix_sdk::Room) -> Result<Self, ClientError> {
        let unread_notification_counts = room.unread_notification_counts();
        let unread_count = room.unread_count();

        let power_levels_map = room.users_with_power_levels().await;
        let mut user_power_levels = HashMap::<String, i64>::new();
        for (id, level) in power_levels_map.iter() {
            user_power_levels.insert(id.to_string(), *level);
        }
        let pinned_event_ids =
            room.pinned_event_ids().unwrap_or_default().iter().map(|id| id.to_string()).collect();

        let join_rule = room.join_rule().try_into();
        if let Err(e) = &join_rule {
            warn!("Failed to parse join rule: {e:?}");
        }

        // Some SC spaces things
        let is_space = room.is_space();
        let can_user_manage_spaces = if is_space {
            match room.can_user_send_state(room.own_user_id(), StateEventType::SpaceChild.into()).await {
                Ok(can_send) => can_send,
                Err(e) => {
                    warn!("Failed to check if user can manage space: {:?}", e);
                    false
                }
            }
        } else {
            false
        };

        Ok(Self {
            id: room.room_id().to_string(),
            encryption_state: room.encryption_state(),
            creator: room.creator().as_ref().map(ToString::to_string),
            display_name: room.cached_display_name().map(|name| name.to_string()),
            raw_name: room.name(),
            topic: room.topic(),
            avatar_url: room.avatar_url().map(Into::into),
            is_direct: room.is_direct().await?,
            is_public: room.is_public(),
            is_space,
            successor_room: room.successor_room().map(Into::into),
            is_favourite: room.is_favourite(),
            is_low_priority: room.is_low_priority(),
            canonical_alias: room.canonical_alias().map(Into::into),
            alternative_aliases: room.alt_aliases().into_iter().map(Into::into).collect(),
            membership: room.state().into(),
            inviter: match room.state() {
                RoomState::Invited => room
                    .invite_details()
                    .await
                    .ok()
                    .and_then(|details| details.inviter)
                    .map(TryInto::try_into)
                    .transpose()
                    .ok()
                    .flatten(),
                _ => None,
            },
            heroes: room.heroes().into_iter().map(Into::into).collect(),
            active_members_count: room.active_members_count(),
            invited_members_count: room.invited_members_count(),
            joined_members_count: room.joined_members_count(),
            user_power_levels,
            highlight_count: unread_notification_counts.highlight_count,
            notification_count: unread_notification_counts.notification_count,
            unread_count: unread_count.unwrap_or_default(),
            cached_user_defined_notification_mode: room
                .cached_user_defined_notification_mode()
                .map(Into::into),
            has_room_call: room.has_active_room_call(),
            active_room_call_participants: room
                .active_room_call_participants()
                .iter()
                .map(|u| u.to_string())
                .collect(),
            is_marked_unread: room.is_marked_unread(),
            space_children: space_children_info(&room),
            can_user_manage_spaces,
            num_unread_messages: room.num_unread_messages(),
            num_unread_notifications: room.num_unread_notifications(),
            num_unread_mentions: room.num_unread_mentions(),
            pinned_event_ids,
            join_rule: join_rule.ok(),
            history_visibility: room.history_visibility_or_default().try_into()?,
        })
    }
}
