table! {
    nongratas (id) {
        id -> Int4,
        rule_id -> Text,
        reason -> Text,
        restriction_type -> Text,
        restriction_value -> Text,
        expires_at -> Timestamptz,
        is_global -> Bool,
    }
}
