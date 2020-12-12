table! {
    servers (guild_id) {
        guild_id -> Text,
        channel_id -> Text,
        current_count -> Integer,
        last_submission_user -> Text,
        highest_count -> Integer,
        times_failed -> Integer,
        last_failed_user -> Text,
        gamemode -> Integer,
    }
}
