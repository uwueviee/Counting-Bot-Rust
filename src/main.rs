use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

extern crate openssl;
#[macro_use] extern crate diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use tokio_diesel::*;
use crate::structs::servers::{Servers, GlobalStats};
use serenity::model::gateway::Activity;
use serenity::model::id::GuildId;
use std::any::Any;

pub mod structs;
pub mod schema;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        use crate::schema::servers::dsl::*;

        let data = &ctx.data.write().await;
        let db = data.get::<DbConn>().unwrap();

        if msg.author.bot || data.get::<BannedUsers>().unwrap().contains(&msg.author.id.0) {
            return;
        }

        let msg_arguments: Vec<&str> = msg.content.split(" ").collect();

        let org_channel_id: String = msg.channel_id.0.to_string();
        let org_guild_id: String = msg.guild_id.unwrap().0.to_string();

        if msg_arguments[0] == "~help"{
            if let Err(why) = msg.channel_id.send_message(&ctx.http, |c| {
                c.embed(|e| {
                    e.title("Counting Help");
                    e.description("Count to skies! Just don't mess up the chain....\nNeed support? Join Counting's home server [here](https://discord.gg/Jp4yMWZ7jk)!");
                    e.color(12522619);
                    e.thumbnail("https://cdn.discordapp.com/avatars/786911411792117770/c74bd0d6860e287e2aade5753eeeeafd.png?size=512");
                    e.fields(vec![
                        ("~help", "Shows this help message!", true),
                        ("~stats [server_id]", "Shows the server's statistics!", true),
                        ("~global_stats", "Shows the global statistics!", true),
                        ("~set_channel [channel_id]", "Sets the dedicated counting channel (Admins Only)", false),
                        ("~set_gamemode <gamemode_id>", "Sets the gamemode (Admins only)\n\n0 = No punishments\n1 = Resets progress if wrong number\n2 = Resets progress if wrong number or same user\n", false)
                    ]);
                    e
                })
            }).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg_arguments[0] == "<:7_:770356395261952020>"{
            if let Err(why) = msg.channel_id.say(&ctx.http, "<:7_:770356395261952020>").await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg_arguments[0] == "~stats"{
            let mut server_lookup = org_guild_id.clone();
            if msg_arguments.len() > 1 {
                server_lookup = msg_arguments[1].to_string();
            }

            let results = servers.filter(guild_id.eq(server_lookup.clone()))
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

            if guild_info.guild_id == "" {
                return;
            }

            let u64_guild_id = GuildId(guild_info.guild_id.parse::<u64>().expect("Error converting into ID"));
            let guild_name = &ctx.cache.guild_field(u64_guild_id, |guild| guild.name.clone())
                .await
                .expect("Guild not in cache!");

            let mut fields = vec![
                ("Current Count", guild_info.current_count.to_string(), true),
                ("Highest Count", guild_info.highest_count.to_string(), true),
                ("Times Failed", guild_info.times_failed.to_string(), true)
            ];

            if guild_info.last_failed_user != "" {
                fields.push(("Last Failure", format!("<@{}>", guild_info.last_failed_user), true))
            }
            if guild_info.last_submission_user != "" {
                fields.push(("Last Success", format!("<@{}>", guild_info.last_submission_user), true))
            }
            fields.push(("Current Gamemode", guild_info.gamemode.to_string(), false));

            if let Err(why) = msg.channel_id.send_message(&ctx.http, |c| {
                c.embed(|e| {
                    e.title("Server Stats");
                    e.description(format!("Statistics for `{}`", guild_name));
                    e.color(12522619);
                    e.thumbnail("https://cdn.discordapp.com/avatars/786911411792117770/c74bd0d6860e287e2aade5753eeeeafd.png?size=512");
                    e.fields(fields);
                    e
                })
            }).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg_arguments[0] == "~global_stats"{
            let total_highest_count_result = diesel::sql_query("SELECT SUM(highest_count) FROM servers;")
                .load_async::<crate::structs::servers::SumStats>(db)
                .await
                .expect("Error getting info from the database");
            let total_current_count_result = diesel::sql_query("SELECT SUM(current_count) FROM servers;")
                .load_async::<crate::structs::servers::SumStats>(db)
                .await
                .expect("Error getting info from the database");
            let total_times_failed_result = diesel::sql_query("SELECT SUM(times_failed) FROM servers;")
                .load_async::<crate::structs::servers::SumStats>(db)
                .await
                .expect("Error getting info from the database");
            let highest_count_result = diesel::sql_query("SELECT MAX(highest_count) FROM servers;")
                .load_async::<crate::structs::servers::MaxStats>(db)
                .await
                .expect("Error getting info from the database");

            let mut global_stats = GlobalStats{
                total_highest_count: 0,
                total_current_count: 0,
                total_times_failed: 0,
                highest_count: 0
            };

            for result in total_highest_count_result {
                global_stats.total_highest_count = result.sum;
            }
            for result in total_current_count_result {
                global_stats.total_current_count = result.sum;
            }
            for result in total_times_failed_result {
                global_stats.total_times_failed = result.sum;
            }
            for result in highest_count_result {
                global_stats.highest_count = result.max;
            }

            if let Err(why) = msg.channel_id.send_message(&ctx.http, |c| {
                c.embed(|e| {
                    e.title("Global Stats");
                    e.description("Global Statistics for Counting");
                    e.color(12522619);
                    e.thumbnail("https://cdn.discordapp.com/avatars/786911411792117770/c74bd0d6860e287e2aade5753eeeeafd.png?size=512");
                    e.fields(vec![
                        ("Highest Count", global_stats.highest_count.to_string(), true),
                        ("Total Highest Count", global_stats.total_highest_count.to_string(), true),
                        ("Total Current Count", global_stats.total_current_count.to_string(), false),
                        ("Total Times Failed", global_stats.total_times_failed.to_string(), true)
                        ]);
                    e
                })
            }).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg_arguments[0] == "~set_channel" {
            // Check to see if the message author has the "Manage Channels" permission
            if msg.guild(&ctx.cache).await.unwrap().member_permissions(msg.author.id).bits & 0x00000010 != 0x00000010 {
                return;
            }

            let mut new_channel = org_channel_id.clone();
            if msg_arguments.len() > 1 {
                new_channel = msg_arguments[1].to_string();
            }

            diesel::insert_into(crate::schema::servers::table)
                .values((
                    channel_id.eq(new_channel.clone()),
                    guild_id.eq(org_guild_id)
                ))
                .on_conflict(guild_id)
                .do_update()
                .set(channel_id.eq(new_channel.clone()))
                .execute_async(db)
                .await
                .unwrap();

            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Setting <#{}> as counting channel!", new_channel)).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg_arguments[0] == "~set_gamemode" {
            // Check to see if the message author has the "Manage Channels" permission
            if msg.guild(&ctx.cache).await.unwrap().member_permissions(msg.author.id).bits & 0x00000010 != 0x00000010 {
                return;
            }

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

        } else if msg_arguments[0].parse::<i32>().is_ok() {
            let submission = msg_arguments[0].parse::<i32>().unwrap();
            let mut submission_passed = true;

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

            let mut same_auth_in_two = false;
            if guild_info.gamemode == 2 {
                if msg.author.id.0.to_string() == guild_info.last_submission_user {
                    same_auth_in_two = true;
                }

                guild_info.last_submission_user = msg.author.id.0.to_string();
            }

            let mut failure_emoji = serenity::model::channel::ReactionType::Unicode("⚠️".to_string());

            if submission != guild_info.current_count + 1 || same_auth_in_two {
                if guild_info.gamemode != 0 {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("<@{}> has broken the chain! Next number was {}!", msg.author.id.0, guild_info.current_count + 1)).await {
                        println!("Error sending message: {:?}", why);
                    }

                    guild_info.times_failed += 1;
                    guild_info.last_failed_user = msg.author.id.0.to_string();
                    guild_info.current_count = 0;
                    guild_info.last_submission_user = "".to_string();

                    failure_emoji = serenity::model::channel::ReactionType::Unicode("❌".to_string());
                }
                submission_passed = false;
            } else {
                guild_info.current_count = submission.clone();

                if submission > guild_info.highest_count {
                    guild_info.highest_count = submission.clone();
                }
            }

            if submission_passed {
                if let Err(why) = msg.react(&ctx.http, serenity::model::channel::ReactionType::Unicode("✅".to_string())).await {
                    println!("Error reacting to message: {:?}", why);
                }
            } else {
                if let Err(why) = msg.react(&ctx.http, failure_emoji).await {
                    println!("Error reacting to message: {:?}", why);
                }
            }

            diesel::update(servers.filter(guild_id.eq(org_guild_id.clone())))
                .set((
                        guild_id.eq(guild_info.guild_id),
                        channel_id.eq(guild_info.channel_id),
                        current_count.eq(guild_info.current_count),
                        last_submission_user.eq(guild_info.last_submission_user),
                        highest_count.eq(guild_info.highest_count),
                        times_failed.eq(guild_info.times_failed),
                        last_failed_user.eq(guild_info.last_failed_user),
                        gamemode.eq(guild_info.gamemode)
                    ))
                .execute_async(db)
                .await
                .expect("Error updating the database");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::competing("the CMO | Do ~help")).await;
    }
}

// Banned user handling
struct BannedUsers;

impl TypeMapKey for BannedUsers {
    type Value = Vec<u64>;
}

struct DbConn;

impl TypeMapKey for DbConn {
    type Value = Pool<ConnectionManager<PgConnection>>;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let db_url = env::var("DATABASE_URL")
        .expect("Expected DATABASE_URL to be populated");

    // Hardcoded banned users
    let banned_users: Vec<u64> = vec![211230498398273537, 238805922997075971];

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
        {
            let mut data = client.data.write().await;
            data.insert::<DbConn>(Pool::builder().build(ConnectionManager::<PgConnection>::new(db_url)).unwrap());
            data.insert::<BannedUsers>(banned_users.clone());
        }

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
}
