use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

#[macro_use] extern crate diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use tokio_diesel::*;
use crate::structs::servers::Servers;

pub mod structs;
pub mod schema;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        use crate::schema::servers::dsl::*;

        let data = &ctx.data.write().await;
        let db = data.get::<DbConn>().unwrap();

        let msg_arguments: Vec<&str> = msg.content.split(" ").collect();

        let org_channel_id: String = msg.channel_id.0.to_string();
        let org_guild_id: String = msg.guild_id.unwrap().0.to_string();

        if msg_arguments[0] == "~set_channel" {
            diesel::insert_into(crate::schema::servers::table)
                .values((
                    channel_id.eq(org_channel_id),
                    guild_id.eq(org_guild_id)
                ))
                .execute_async(db)
                .await
                .unwrap();

            if let Err(why) = msg.channel_id.edit(&ctx.http, |c| c.topic(format!("Next number: {} | Last Submission User: <@{}> | Highest Count: {} | Times Failed: {} | Last Failed Submission User: <@{}>",
                    1,
                    "",
                    0,
                    0,
                    ""
                ))).await {
                println!("Error editing channel topic: {:?}", why);
            } 

            if let Err(why) = msg.channel_id.say(&ctx.http, "Setting current channel as counting channel!").await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg_arguments[0] == "~set_gamemode" { 
            if msg_arguments.len() == 1 || msg_arguments[1].parse::<i32>().unwrap() > 2 {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Please enter a valid gamemode id using `~set_gamemode <gamemode id>`").await {
                    println!("Error sending message: {:?}", why);
                }

                return;
            }

            diesel::update(servers.filter(guild_id.eq(org_guild_id.clone())))
                .set((
                    current_count.eq(0),
                    last_submission_user.eq("".to_string()),
                    highest_count.eq(0),
                    times_failed.eq(0),
                    last_failed_user.eq("".to_string()),
                    gamemode.eq(msg_arguments[1].parse::<i32>().unwrap())
                ))
                .execute_async(db)
                .await
                .expect("Error updating the database");

            if let Err(why) = msg.channel_id.say(&ctx.http, "Gamemode changed!").await {
                println!("Error sending message: {:?}", why);
            }

            if let Err(why) = msg.channel_id.edit(&ctx.http, |c| c.topic(format!("Next number: {} | Last Submission User: <@{}> | Highest Count: {} | Times Failed: {} | Last Failed Submission User: <@{}>",
                    1,
                    "",
                    0,
                    0,
                     ""
                ))).await {
                 println!("Error editing channel topic: {:?}", why);
             } 
        } else if msg_arguments[0].parse::<i32>().is_ok() {
            let submission = msg_arguments[0].parse::<i32>().unwrap();

            let results = servers.filter(guild_id.eq(org_guild_id.clone()))
                .load_async::<Servers>(db)
                .await
                .expect("Server not registered");
            
            let mut guild_info = Servers{
                guild_id: "".to_string(),
                channel_id: "".to_string(),
                current_count: 0,
                last_submission_user: "".to_string(),
                highest_count: 0,
                times_failed: 0,
                last_failed_user: "".to_string(),
                gamemode: 0
            };

            for result in results {
                guild_info = Servers{
                    guild_id: result.guild_id,
                    channel_id: result.channel_id,
                    current_count: result.current_count,
                    last_submission_user: result.last_submission_user,
                    highest_count: result.highest_count,
                    times_failed: result.times_failed,
                    last_failed_user: result.last_failed_user,
                    gamemode: result.gamemode
                }
            }

            if guild_info.guild_id == "" || guild_info.channel_id != org_channel_id {
                return;
            }

            if submission != guild_info.current_count + 1 || msg.author.id.0.to_string() == guild_info.last_submission_user {
                if guild_info.gamemode == 1 || guild_info.gamemode == 2 {
                    guild_info.times_failed += 1;

                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("<@{}> has broken the chain! Next number was {}!", msg.author.id.0, guild_info.current_count + 1)).await {
                        println!("Error sending message: {:?}", why);
                    }    
                    diesel::update(servers.filter(guild_id.eq(org_guild_id.clone())))
                        .set((
                            times_failed.eq(guild_info.times_failed),
                            last_failed_user.eq(msg.author.id.0.to_string()),
                            current_count.eq(0),
                            last_submission_user.eq("".to_string())
                        ))
                        .execute_async(db)
                        .await
                        .expect("Error updating the database");
                    
                        if let Err(why) = msg.channel_id.edit(&ctx.http, |c| c.topic(format!("Next number: {} | Last Submission User: <@{}> | Highest Count: {} | Times Failed: {} | Last Failed Submission User: <@{}>",
                                1,
                                "",
                                &guild_info.highest_count,
                                &guild_info.times_failed,
                                msg.author.id.0
                            ))).await {
                            println!("Error editing channel topic: {:?}", why);
                        } 

                    return;
                } else if guild_info.gamemode == 0 {
                    return;
                }
            }

            if submission > guild_info.highest_count {
                guild_info.highest_count = submission.clone();

                diesel::update(servers.filter(guild_id.eq(org_guild_id.clone())))
                    .set(highest_count.eq(guild_info.highest_count))
                    .execute_async(db)
                    .await
                    .expect("Error updating the database");
            }

            if guild_info.gamemode == 2 {
                guild_info.last_submission_user = msg.author.id.0.to_string();

                diesel::update(servers.filter(guild_id.eq(org_guild_id.clone())))
                .set(last_submission_user.eq(guild_info.last_submission_user.clone()))
                .execute_async(db)
                .await
                .expect("Error updating the database");
            }

            guild_info.current_count = submission.clone();
            
            diesel::update(servers.filter(guild_id.eq(org_guild_id.clone())))
                .set(current_count.eq(guild_info.current_count))
                .execute_async(db)
                .await
                .expect("Error updating the database");

            if let Err(why) = msg.react(&ctx.http, serenity::model::channel::ReactionType::Unicode("✅".to_string())).await {
                println!("Error reacting to message: {:?}", why);
            } 

            if let Err(why) = msg.channel_id.edit(&ctx.http, |c| c.topic(format!("Next number: {} | Last Submission User: <@{}> | Highest Count: {} | Times Failed: {} | Last Failed Submission User: <@{}>",
                    &guild_info.current_count,
                    guild_info.last_submission_user.clone(),
                    &guild_info.highest_count,
                    &guild_info.times_failed,
                    guild_info.last_failed_user.clone()
                ))).await {
                println!("Error editing channel topic: {:?}", why);
            } 
        }
    }
    
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}


struct DbConn;

impl TypeMapKey for DbConn {
    type Value = Pool<ConnectionManager<SqliteConnection>>;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let db_url = env::var("DATABASE_URL")
        .expect("Expected DATABASE_URL to be populated");
    
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
        {
            let mut data = client.data.write().await;
            data.insert::<DbConn>(Pool::builder().build(ConnectionManager::<SqliteConnection>::new(db_url)).unwrap());
        }
    
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
