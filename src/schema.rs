table! {
    nongratas (id) {
        id -> Int4,
        reason -> Text,
        restriction_type -> Text,
        restriction_value -> Text,
        expires_at -> Timestamptz,
        is_global -> Bool,
    }
}
