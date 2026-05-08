// @generated automatically by Diesel CLI.

diesel::table! {
    jks (row_id) {
        row_id -> Nullable<Integer>,
        id -> Integer,
        cislo_stofy -> Integer,
        text -> Text,
    }
}

diesel::table! {
    jks_types (song_id, type_id) {
        song_id -> Integer,
        type_id -> Integer,
    }
}

diesel::table! {
    song_types (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::joinable!(jks_types -> jks (song_id));
diesel::joinable!(jks_types -> song_types (type_id));

diesel::allow_tables_to_appear_in_same_query!(jks, jks_types, song_types,);
