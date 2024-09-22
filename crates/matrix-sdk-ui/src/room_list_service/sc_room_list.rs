use super::sorters::{
    BoxedSorterFn,
    new_sorter_name,
    new_sorter_recency,
    new_sorter_unread,
    new_sorter_tag,
    new_sorter_lexicographic,
};

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ScSortOrder {
    by_unread: bool,
    pin_favorites: bool,
    bury_low_priority: bool,
    client_generated_unread: bool,
}

pub fn get_sort_by_vec(sort_order: ScSortOrder) -> Vec<BoxedSorterFn> {
    let mut result: Vec<BoxedSorterFn> = Vec::new();
    tracing::info!("SC_DBG: sort by {} {} {} {}", sort_order.by_unread, sort_order.pin_favorites, sort_order.bury_low_priority, sort_order.client_generated_unread);
    // Always sort by tag: also sorts invites on top
    //if sort_order.pin_favorites || sort_order.bury_low_priority {
    result.push(Box::new(new_sorter_tag(
        sort_order.pin_favorites,
        sort_order.bury_low_priority
    )));
    //}
    if sort_order.by_unread {
        result.push(Box::new(new_sorter_unread(sort_order.client_generated_unread)));
    }
    result.push(Box::new(new_sorter_recency()));
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
