use crate::schema::*;

#[derive(Insertable, Queryable)]
#[table_name = "servers"]
pub struct Servers {
    pub guild_id: String,
    pub channel_id: String,
    pub current_count: i32,
    pub last_submission_user: String,
    pub highest_count: i32,
    pub times_failed: i32,
    pub last_failed_user: String,
    pub gamemode: i32
}