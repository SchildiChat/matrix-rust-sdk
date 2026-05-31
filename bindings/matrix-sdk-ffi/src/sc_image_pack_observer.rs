use std::collections::HashSet;
use std::sync::Arc;

use futures_util::{StreamExt, pin_mut};
use matrix_sdk::{
    Client as MatrixClient, Room, RoomState,
    deserialized_responses::RawSyncOrStrippedState,
    ruma::{
        OwnedRoomId, RoomId,
        events::{
            GlobalAccountDataEvent, SyncStateEvent,
            image_pack::{ImagePackRoomsEventContent, RoomImagePackEventContent},
        },
    },
};
use matrix_sdk_common::{SendOutsideWasm, SyncOutsideWasm};
use tracing::error;

use crate::{TaskHandle, runtime::get_runtime_handle};

#[derive(Clone, uniffi::Record)]
pub struct RoomImagePackStateEvent {
    pub room_id: String,
    pub state_key: String,
    pub raw: String,
}

#[matrix_sdk_ffi_macros::export(callback_interface)]
pub trait RoomImagePackStateEventsListener: SendOutsideWasm + SyncOutsideWasm {
    fn on_update(&self, events: Vec<RoomImagePackStateEvent>);
}

pub fn subscribe_to_image_pack_state_events(
    client: MatrixClient,
    primary_room_ids: HashSet<OwnedRoomId>,
    listener: Box<dyn RoomImagePackStateEventsListener>,
) -> Arc<TaskHandle> {
    let listener: Arc<dyn RoomImagePackStateEventsListener> = listener.into();
    let event_listener = listener.clone();
    let event_client = client.clone();
    let event_primary_room_ids = primary_room_ids.clone();

    // Listen to image pack state events
    let handler = client.add_event_handler(
        move |event: SyncStateEvent<RoomImagePackEventContent>, room: Room| {
            let listener = event_listener.clone();
            let client = event_client.clone();
            let primary_room_ids = event_primary_room_ids.clone();

            async move {
                if room.state() != RoomState::Joined {
                    return;
                }

                if primary_room_ids.contains(room.room_id())
                    || is_globally_enabled(&client, room.room_id(), event.state_key()).await
                {
                    emit_snapshot(&client, &primary_room_ids, listener.as_ref()).await;
                }
            }
        },
    );
    let drop_guard = client.event_handler_drop_guard(handler);

    // Listen to globally enabled packs account data
    Arc::new(TaskHandle::new(get_runtime_handle().spawn(async move {
        let _drop_guard = drop_guard;
        let account_data_observer = Arc::new(
            client.observe_events::<GlobalAccountDataEvent<ImagePackRoomsEventContent>, ()>(),
        );
        let mut account_data_subscriber = account_data_observer.subscribe();

        let (_rooms, rooms_stream) = client.rooms_stream();
        pin_mut!(rooms_stream);

        emit_snapshot(&client, &primary_room_ids, listener.as_ref()).await;

        loop {
            tokio::select! {
                // Listen to room joins / leaves that may affect globally enabled data
                Some(_) = rooms_stream.next() => {
                    emit_snapshot(&client, &primary_room_ids, listener.as_ref()).await;
                }
                // Listen to actual globally enabled account data
                Some(_) = account_data_subscriber.next() => {
                    emit_snapshot(&client, &primary_room_ids, listener.as_ref()).await;
                }
                else => break,
            }
        }
    })))
}

async fn emit_snapshot(
    client: &MatrixClient,
    primary_room_ids: &HashSet<OwnedRoomId>,
    listener: &dyn RoomImagePackStateEventsListener,
) {
    listener.on_update(collect_image_pack_events(client, primary_room_ids).await);
}

async fn collect_image_pack_events(
    client: &MatrixClient,
    primary_room_ids: &HashSet<OwnedRoomId>,
) -> Vec<RoomImagePackStateEvent> {
    let mut events = Vec::new();

    // Rooms the caller is specifically interested in (e.g. currently opened room with its parent spaces)
    for room_id in primary_room_ids {
        if let Some(room) =
            client.get_room(room_id).filter(|room| room.state() == RoomState::Joined)
        {
            events.extend(collect_room_image_pack_events(&room, None).await);
        }
    }

    // Globally enabled packs from account data
    if let Some(image_pack_rooms) = global_image_pack_rooms(client).await {
        for (pack_room_id, packs) in image_pack_rooms.rooms {
            if primary_room_ids.contains(&pack_room_id) {
                // Already added previously
                continue;
            }

            if let Some(room) =
                client.get_room(&pack_room_id).filter(|room| room.state() == RoomState::Joined)
            {
                let state_keys = packs.keys().cloned().collect();
                events.extend(collect_room_image_pack_events(&room, Some(&state_keys)).await);
            }
        }
    }

    events
}

async fn collect_room_image_pack_events(
    room: &Room,
    state_keys: Option<&HashSet<String>>,
) -> Vec<RoomImagePackStateEvent> {
    let state_events = match state_keys {
        // For globally enabled rooms, collect only enabled packs
        Some(state_keys) => {
            room.get_state_events_for_keys_static::<RoomImagePackEventContent, _, _>(state_keys)
                .await
        }
        // For primary room, collect all
        None => room.get_state_events_static::<RoomImagePackEventContent>().await,
    };

    let state_events = match state_events {
        Ok(state_events) => state_events,
        Err(error) => {
            error!(
                ?error,
                room_id = ?room.room_id(),
                "Failed to get room image pack state events"
            );
            return Vec::new();
        }
    };

    state_events
        .into_iter()
        .filter_map(|raw| match raw {
            RawSyncOrStrippedState::Sync(raw) => {
                let event = raw.deserialize().ok()?;
                let state_key = event.state_key().to_owned();

                Some(RoomImagePackStateEvent {
                    room_id: room.room_id().to_string(),
                    state_key,
                    raw: raw.json().get().to_owned(),
                })
            }
            RawSyncOrStrippedState::Stripped(_) => None,
        })
        .collect()
}

async fn is_globally_enabled(client: &MatrixClient, room_id: &RoomId, state_key: &str) -> bool {
    global_image_pack_rooms(client).await.is_some_and(|image_pack_rooms| {
        image_pack_rooms.rooms.get(room_id).is_some_and(|packs| packs.contains_key(state_key))
    })
}

async fn global_image_pack_rooms(client: &MatrixClient) -> Option<ImagePackRoomsEventContent> {
    client
        .account()
        .account_data::<ImagePackRoomsEventContent>()
        .await
        .ok()
        .flatten()
        .and_then(|raw| raw.deserialize().ok())
}
