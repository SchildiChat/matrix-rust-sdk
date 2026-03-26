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

pub fn get_sort_by_vec(sort_order: ScSortOrder) -> Vec<BoxedSorterFn> {
    let mut result: Vec<BoxedSorterFn> = Vec::new();
    tracing::info!("SC_SORT_DBG: sort by {} {} {} {}", sort_order.by_unread, sort_order.pin_favorites, sort_order.bury_low_priority, sort_order.client_generated_unread);
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
    // Sort by latest event's kind, i.e. put the rooms with a
    // **local** latest event first.
    result.push(Box::new(new_sorter_latest_event()));
    // Sort rooms by their recency (either by looking
    // at their latest event's timestamp, or their
    // `recency_stamp`).
    result.push(Box::new(new_sorter_recency()));
    // Finally, sort by name.
    result.push(Box::new(new_sorter_name()));
    result
}

impl From<ScSortOrder> for BoxedSorterFn {
    fn from(value: ScSortOrder) -> Self {
        Box::new(new_sorter_lexicographic(
            get_sort_by_vec(value)
        ))
    }
}
