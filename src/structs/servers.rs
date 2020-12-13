use crate::schema::*;
use diesel::sql_types::{Integer, BigInt};

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

#[derive(QueryableByName)]
pub struct SumStats {
    #[sql_type = "BigInt"]
    pub sum: i64
}

#[derive(QueryableByName)]
pub struct MaxStats {
    #[sql_type = "Integer"]
    pub max: i32
}

pub struct GlobalStats {
    pub total_highest_count: i64,
    pub total_current_count: i64,
    pub total_times_failed: i64,
    pub highest_count: i32,
}