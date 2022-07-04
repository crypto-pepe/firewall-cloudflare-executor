table! {
    use diesel::sql_types::*;
    use crate::models::*;

    filters (id) {
        id -> Text,
        rule_id -> Text,
        filter_type -> Filter_type,
        expression -> Text,
    }
}

table! {
    nongratas (id) {
        id -> Int8,
        filter_id -> Text,
        reason -> Text,
        restriction_value -> Text,
        restriction_type -> Text,
        expires_at -> Timestamptz,
        is_global -> Bool,
        analyzer_id -> Text,
    }
}

joinable!(nongratas -> filters (filter_id));

allow_tables_to_appear_in_same_query!(filters, nongratas,);
