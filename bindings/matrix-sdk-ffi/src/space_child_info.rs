#[derive(uniffi::Record)]
pub struct SpaceChildInfo {
    room_id: String,
    order: Option<String>,
    suggested: bool,
}

impl SpaceChildInfo {
    pub(crate) fn new(
        room_id: String,
        order: Option<String>,
        suggested: bool,
    ) -> Self {
        Self {
            room_id: room_id,
            order: order,
            suggested: suggested,
        }
    }
}


pub fn space_children_info(room: &matrix_sdk::Room) -> Vec<SpaceChildInfo> {
    let mut space_children = Vec::new();
    if !room.is_space() {
        return space_children;
    }
    for (r, s) in room.space_children().iter() {
        // Has room been removed from space again?
        if let Some(ev) = s.as_original() {
            // Hasn't been replaced by empty state event?
            // The spec tells us to ignore children without `via`
            if !ev.content.via.is_empty() {
                space_children.push(
                    SpaceChildInfo::new(
                        r.to_string(),
                        ev.content.order.clone(),
                        ev.content.suggested,
                    )
                );
            }
        }
    }
    return space_children;
}
