use super::{super::Room, Filter};

struct IsSpaceRoomMatcher<F>
where
    F: Fn(&Room) -> bool,
{
    is_space: F,
}

impl<F> IsSpaceRoomMatcher<F>
where
    F: Fn(&Room) -> bool,
{
    fn matches(&self, room: &Room) -> bool {
        (self.is_space)(room)
    }
}

/// Create a new filter that will filter out rooms that are not spaces, i.e.
/// room with a `room_type` of `m.space` as defined in <https://spec.matrix.org/latest/client-server-api/#spaces>
pub fn new_filter() -> impl Filter {
    let matcher = IsSpaceRoomMatcher { is_space: move |room| room.is_space() };

    move |room| -> bool { matcher.matches(room) }
}
