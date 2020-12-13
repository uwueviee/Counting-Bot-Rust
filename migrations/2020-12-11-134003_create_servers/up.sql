-- Your SQL goes here
CREATE TABLE servers (
    guild_id TEXT PRIMARY KEY NOT NULL,
    channel_id TEXT NOT NULL,
    current_count INT NOT NULL DEFAULT 0,
    last_submission_user TEXT NOT NULL DEFAULT '',
    highest_count INT NOT NULL DEFAULT 0,
    times_failed INT NOT NULL DEFAULT 0,
    last_failed_user TEXT NOT NULL DEFAULT '',
    gamemode INT NOT NULL DEFAULT 0
);