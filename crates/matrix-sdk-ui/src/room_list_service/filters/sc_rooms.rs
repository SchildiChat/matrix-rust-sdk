use super::{super::Room, Filter};

struct ScRoomsMatcher<F>
where
    F: Fn(&Room) -> bool,
{
    is_included: F,
}

impl<F> ScRoomsMatcher<F>
where
    F: Fn(&Room) -> bool,
{
    fn matches(&self, room: &Room) -> bool {
        (self.is_included)(room)
    }
}

/// Create a new filter that will filter out rooms that are not marked as
/// favourite (see [`matrix_sdk_base::Room::is_favourite`]).
pub fn new_filter(rooms: Vec<String>) -> impl Filter {
    //let matcher = ScRoomsMatcher { is_included: move |room| rooms.contains(room.room_id()) };
    let matcher = ScRoomsMatcher { is_included: move |room| rooms.iter().any(|room_id| room_id == room.room_id()) };

    move |room| -> bool { matcher.matches(room) }
}


