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
