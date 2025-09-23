use std::cmp::Ordering;

use super::{RoomListItem, Sorter};

struct UnreadMatcher<F>
where
    F: Fn(&RoomListItem, &RoomListItem) -> (u8, u8),
{
    order_key: F,
}

impl<F> UnreadMatcher<F>
where
    F: Fn(&RoomListItem, &RoomListItem) -> (u8, u8),
{
    fn matches(&self, left: &RoomListItem, right: &RoomListItem) -> Ordering {
        // Same workaround as for recency sorter - not sure if required?
        if left.room_id() == right.room_id() {
            return Ordering::Greater;
        }

        let (left_key, right_key) = (self.order_key)(left, right);
        left_key.cmp(&right_key)
    }
}

pub fn new_sorter(client_generated_counts: bool, with_silent_unread: bool) -> impl Sorter {
    let matcher = UnreadMatcher {
        order_key: move |left, right| (room_to_unread_weight(left, client_generated_counts, with_silent_unread), room_to_unread_weight(right, client_generated_counts, with_silent_unread)),
    };

    move |left, right| -> Ordering { matcher.matches(left, right) }
}

fn room_to_unread_weight(room: &RoomListItem, client_generated_counts: bool, with_silent_unread: bool) -> u8 {
    if client_generated_counts {
        counts_to_unread_weight(
            room.is_marked_unread(),
            room.num_unread_mentions(),
            room.num_unread_notifications(),
            if with_silent_unread {
                room.num_unread_messages()
            } else {
                0
            },
        )
    } else {
        // Note: always use client-generated mention counts, server cannot know for encrypted rooms
        counts_to_unread_weight(
            room.is_marked_unread(),
            room.num_unread_mentions(),
            room.unread_notification_counts().notification_count,
            if with_silent_unread {
                room.unread_count().unwrap_or_default()
            } else {
                0
            },
        )
    }
}

fn counts_to_unread_weight(marked_unread: bool, highlight_count: u64, notification_count: u64, unread_count: u64) -> u8 {
    if marked_unread || notification_count > 0 || highlight_count > 0 {
        0
    } else if unread_count > 0 {
        1
    } else {
        2
    }
}
