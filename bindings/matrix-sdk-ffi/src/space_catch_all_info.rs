#[derive(uniffi::Record)]
pub struct SpaceCatchAllInfo {
    state_key: String,
    include_orphans: bool,
    filter_is_dm: Option<bool>,
}

impl SpaceCatchAllInfo {
    pub(crate) fn new(
        state_key: String,
        include_orphans: bool,
        filter_is_dm: Option<bool>,
    ) -> Self {
        Self { state_key, include_orphans, filter_is_dm }
    }
}

pub fn space_catch_all_info(room: &matrix_sdk::Room) -> Option<SpaceCatchAllInfo> {
    let event = room.space_catch_all()?;

    Some(SpaceCatchAllInfo::new(
        event.state_key,
        event.event.content.include_orphans.unwrap_or(false),
        event.event.content.filter_is_dm,
    ))
}
