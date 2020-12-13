table! {
    servers (guild_id) {
        guild_id -> Text,
        channel_id -> Text,
        current_count -> Int4,
        last_submission_user -> Text,
        highest_count -> Int4,
        times_failed -> Int4,
        last_failed_user -> Text,
        gamemode -> Int4,
    }
}
