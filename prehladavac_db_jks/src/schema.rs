// @generated automatically by Diesel CLI.

diesel::table! {
    jks (row_id) {
        row_id -> Nullable<Integer>,
        id -> Integer,
        cislo_stofy -> Integer,
        text -> Text,
    }
}
