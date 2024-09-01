use std::cmp::Ordering;

use super::{Room, Sorter};

struct UnreadMatcher<F>
where
    F: Fn(&Room, &Room) -> (u8, u8),
{
    order_key: F,
}

impl<F> UnreadMatcher<F>
where
    F: Fn(&Room, &Room) -> (u8, u8),
{
    fn matches(&self, left: &Room, right: &Room) -> Ordering {
        // Same workaround as for recency sorter - not sure if required?
        if left.id() == right.id() {
            return Ordering::Greater;
        }

        let (left_key, right_key) = (self.order_key)(left, right);
        left_key.cmp(&right_key)
    }
}

pub fn new_sorter(client_generated_counts: bool) -> impl Sorter {
    let matcher = UnreadMatcher {
        order_key: move |left, right| (room_to_unread_weight(left, client_generated_counts), room_to_unread_weight(right, client_generated_counts)),
    };

    move |left, right| -> Ordering { matcher.matches(left, right) }
}

fn room_to_unread_weight(room: &Room, client_generated_counts: bool) -> u8 {
    let inner_room = room.inner_room();
    if client_generated_counts {
        counts_to_unread_weight(
            inner_room.is_marked_unread(),
            inner_room.num_unread_mentions(),
            inner_room.num_unread_notifications(),
            inner_room.num_unread_messages(),
        )
    } else {
        // Note: always use client-generated mention counts, server cannot know for encrypted rooms
        counts_to_unread_weight(
            inner_room.is_marked_unread(),
            inner_room.num_unread_mentions(),
            inner_room.unread_notification_counts().notification_count,
            inner_room.unread_count().unwrap_or_default(),
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
