use super::sorters::{
    BoxedSorterFn,
    new_sorter_name,
    new_sorter_recency,
    new_sorter_unread,
    new_sorter_tag,
    new_sorter_lexicographic,
    new_sorter_latest_event,
};

use matrix_sdk::schildi::ScSortOrder;

// TODO is that `enable_latest_event_sorter` temporary upstream stuff?
pub fn get_sort_by_vec(sort_order: ScSortOrder, enable_latest_event_sorter: bool) -> Vec<BoxedSorterFn> {
    let mut result: Vec<BoxedSorterFn> = Vec::new();
    tracing::info!("SC_SORT_DBG: sort by {} {} {} {} {}", sort_order.by_unread, sort_order.pin_favorites, sort_order.bury_low_priority, sort_order.client_generated_unread, enable_latest_event_sorter);
    // Always sort by tag: also sorts invites on top
    //if sort_order.pin_favorites || sort_order.bury_low_priority {
    result.push(Box::new(new_sorter_tag(
        sort_order.pin_favorites,
        sort_order.bury_low_priority
    )));
    //}
    if sort_order.by_unread {
        result.push(Box::new(new_sorter_unread(sort_order.client_generated_unread, sort_order.with_silent_unread)));
    }
    if enable_latest_event_sorter { // TODO is this an upstream flag or should we integrate better?
        // Sort by latest event's kind, i.e. put the rooms with a
        // **local** latest event first.
        result.push(Box::new(new_sorter_latest_event()));
    }
    result.push(Box::new(new_sorter_recency()));
    result.push(Box::new(new_sorter_name()));
    result
}

impl From<ScSortOrder> for BoxedSorterFn {
    fn from(value: ScSortOrder) -> Self {
        Box::new(new_sorter_lexicographic(
            get_sort_by_vec(value, false) // TODO upstream seems to default to false on this one
        ))
    }
}

// TODO delete when upstream drops `enable_latest_event_sorter`?
pub fn get_sc_sort_box(setting: ScSortOrder, enable_latest_event_sorter: bool) -> BoxedSorterFn {
    Box::new(new_sorter_lexicographic(
        get_sort_by_vec(setting, enable_latest_event_sorter)
    ))
}
