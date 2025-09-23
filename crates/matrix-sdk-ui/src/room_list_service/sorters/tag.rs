use std::cmp::Ordering;
use matrix_sdk::RoomState;

use super::{RoomListItem, Sorter};

struct TagMatcher<F>
where
    F: Fn(&RoomListItem, &RoomListItem) -> (u8, u8),
{
    order_key: F,
}

impl<F> TagMatcher<F>
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

pub fn new_sorter(pin_favorites: bool, bury_low_priority: bool) -> impl Sorter {
    let matcher = TagMatcher {
        order_key: move |left, right| (room_to_tag_weight(left, pin_favorites, bury_low_priority), room_to_tag_weight(right, pin_favorites, bury_low_priority)),
    };

    move |left, right| -> Ordering { matcher.matches(left, right) }
}

fn room_to_tag_weight(room: &RoomListItem, pin_favorites: bool, bury_low_priority: bool) -> u8 {
    if room.state() == RoomState::Invited {
        0
    } else if pin_favorites && room.is_favourite() {
        1
    } else if bury_low_priority && room.is_low_priority() {
        3
    } else {
        2
    }
}
