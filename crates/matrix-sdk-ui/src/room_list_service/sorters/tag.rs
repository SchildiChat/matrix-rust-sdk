use std::cmp::Ordering;
use matrix_sdk::RoomState;

use super::{Room, Sorter};

struct TagMatcher<F>
where
    F: Fn(&Room, &Room) -> (u8, u8),
{
    order_key: F,
}

impl<F> TagMatcher<F>
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

pub fn new_sorter(pin_favorites: bool, bury_low_priority: bool) -> impl Sorter {
    let matcher = TagMatcher {
        order_key: move |left, right| (room_to_tag_weight(left, pin_favorites, bury_low_priority), room_to_tag_weight(right, pin_favorites, bury_low_priority)),
    };

    move |left, right| -> Ordering { matcher.matches(left, right) }
}

fn room_to_tag_weight(room: &Room, pin_favorites: bool, bury_low_priority: bool) -> u8 {
    let inner_room = room.inner_room();
    if inner_room.state() == RoomState::Invited {
        0
    } else if pin_favorites && inner_room.is_favourite() {
        1
    } else if bury_low_priority && inner_room.is_low_priority() {
        3
    } else {
        2
    }
}
