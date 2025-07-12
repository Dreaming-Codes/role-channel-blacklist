// @generated automatically by Diesel CLI.

diesel::table! {
    blacklist_entries (id) {
        id -> Int8,
        channel_id -> Int8,
        role_id -> Int8,
        custom_message -> Nullable<Text>,
    }
}

diesel::table! {
    exception_entries (id) {
        id -> Int8,
        channel_id -> Int8,
        role_id -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blacklist_entries,
    exception_entries,
);
