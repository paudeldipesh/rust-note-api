// @generated automatically by Diesel CLI.

diesel::table! {
    notes (id) {
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        content -> Text,
        created_by -> Int4,
        created_on -> Nullable<Timestamptz>,
        updated_on -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 30]
        username -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        password -> Varchar,
    }
}

diesel::joinable!(notes -> users (created_by));

diesel::allow_tables_to_appear_in_same_query!(notes, users,);
