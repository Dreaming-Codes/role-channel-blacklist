use diesel::backend::Backend;
use diesel::prelude::*;
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper, pooled_connection::bb8::Pool,
    AsyncPgConnection, RunQueryDsl,
};
use diesel_migrations::{embed_migrations, MigrationHarness};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

use serenity::{async_trait, model::id::RoleId};
use std::{env, sync::Arc};

mod schema;

use schema::{blacklist_entries, exception_entries};

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations = embed_migrations!();

pub fn run_pending_migrations<T: MigrationHarness<U>, U: Backend>(conn: &mut T) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = blacklist_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlacklistEntry {
    pub id: i64,
    pub channel_id: i64,
    pub role_id: i64,
    pub custom_message: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = blacklist_entries)]
pub struct NewBlacklistEntry {
    pub channel_id: i64,
    pub role_id: i64,
    pub custom_message: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = exception_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExceptionEntry {
    pub id: i64,
    pub channel_id: i64,
    pub role_id: i64,
}

#[derive(Insertable)]
#[diesel(table_name = exception_entries)]
pub struct NewExceptionEntry {
    pub channel_id: i64,
    pub role_id: i64,
}

#[derive(Clone)]
struct Data {
    db_pool: Arc<Pool<AsyncPgConnection>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

impl Data {
    async fn new() -> Result<Self, Error> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        tokio::task::spawn_blocking({
            let database_url = database_url.clone();
            move || {
                let mut conn =
                    AsyncConnectionWrapper::<AsyncPgConnection>::establish(&database_url).unwrap();
                conn.run_pending_migrations(MIGRATIONS).unwrap();
            }
        })
        .await?;

        let config = diesel_async::pooled_connection::AsyncDieselConnectionManager::<
            AsyncPgConnection,
        >::new(database_url);

        let pool = Pool::builder().build(config).await?;

        Ok(Self {
            db_pool: Arc::new(pool),
        })
    }

    async fn add_blacklist_entry(
        &self,
        channel_id: u64,
        role_id: u64,
        custom_message: Option<String>,
    ) -> Result<(), Error> {
        let mut conn = self.db_pool.get().await?;

        let new_entry = NewBlacklistEntry {
            channel_id: channel_id as i64,
            role_id: role_id as i64,
            custom_message,
        };

        diesel::insert_into(blacklist_entries::table)
            .values(&new_entry)
            .on_conflict((blacklist_entries::channel_id, blacklist_entries::role_id))
            .do_update()
            .set(blacklist_entries::custom_message.eq(&new_entry.custom_message))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    async fn add_exception_entry(&self, channel_id: u64, role_id: u64) -> Result<(), Error> {
        let mut conn = self.db_pool.get().await?;

        let new_entry = NewExceptionEntry {
            channel_id: channel_id as i64,
            role_id: role_id as i64,
        };

        diesel::insert_into(exception_entries::table)
            .values(&new_entry)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    async fn check_blacklist(&self, channel_id: u64, role_ids: &[u64]) -> Option<(u64, String)> {
        let mut conn = self.db_pool.get().await.ok()?;

        let role_ids_i64: Vec<i64> = role_ids.iter().map(|&id| id as i64).collect();

        let result = blacklist_entries::table
            .filter(blacklist_entries::channel_id.eq(channel_id as i64))
            .filter(blacklist_entries::role_id.eq_any(&role_ids_i64))
            .select((
                blacklist_entries::role_id,
                blacklist_entries::custom_message,
            ))
            .first::<(i64, Option<String>)>(&mut conn)
            .await
            .ok()?;

        Some((result.0 as u64, result.1.unwrap_or_default()))
    }

    async fn has_exception(&self, channel_id: u64, role_ids: &[u64]) -> bool {
        let Ok(mut conn) = self.db_pool.get().await else {
            return false;
        };

        let role_ids_i64: Vec<i64> = role_ids.iter().map(|&id| id as i64).collect();

        exception_entries::table
            .filter(exception_entries::channel_id.eq(channel_id as i64))
            .filter(exception_entries::role_id.eq_any(&role_ids_i64))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .unwrap_or(0)
            > 0
    }

    async fn remove_blacklist_entry(&self, channel_id: u64, role_id: u64) -> Result<(), Error> {
        let mut conn = self.db_pool.get().await?;

        diesel::delete(
            blacklist_entries::table
                .filter(blacklist_entries::channel_id.eq(channel_id as i64))
                .filter(blacklist_entries::role_id.eq(role_id as i64)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    async fn remove_exception_entry(&self, channel_id: u64, role_id: u64) -> Result<(), Error> {
        let mut conn = self.db_pool.get().await?;

        diesel::delete(
            exception_entries::table
                .filter(exception_entries::channel_id.eq(channel_id as i64))
                .filter(exception_entries::role_id.eq(role_id as i64)),
        )
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    async fn is_blacklisted(&self, channel_id: u64, role_id: u64) -> bool {
        let Ok(mut conn) = self.db_pool.get().await else {
            return false;
        };

        blacklist_entries::table
            .filter(blacklist_entries::channel_id.eq(channel_id as i64))
            .filter(blacklist_entries::role_id.eq(role_id as i64))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .unwrap_or(0)
            > 0
    }

    async fn is_exception(&self, channel_id: u64, role_id: u64) -> bool {
        let Ok(mut conn) = self.db_pool.get().await else {
            return false;
        };

        exception_entries::table
            .filter(exception_entries::channel_id.eq(channel_id as i64))
            .filter(exception_entries::role_id.eq(role_id as i64))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .unwrap_or(0)
            > 0
    }

    async fn get_blacklisted_roles(&self, channel_id: u64) -> Result<Vec<(u64, String)>, Error> {
        let mut conn = self.db_pool.get().await?;

        let results = blacklist_entries::table
            .filter(blacklist_entries::channel_id.eq(channel_id as i64))
            .select((
                blacklist_entries::role_id,
                blacklist_entries::custom_message,
            ))
            .load::<(i64, Option<String>)>(&mut conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(role_id, custom_message)| (role_id as u64, custom_message.unwrap_or_default()))
            .collect())
    }

    async fn get_exception_roles(&self, channel_id: u64) -> Result<Vec<u64>, Error> {
        let mut conn = self.db_pool.get().await?;

        let results = exception_entries::table
            .filter(exception_entries::channel_id.eq(channel_id as i64))
            .select(exception_entries::role_id)
            .load::<i64>(&mut conn)
            .await?;

        Ok(results.into_iter().map(|role_id| role_id as u64).collect())
    }
}

/// Add a role to the blacklist for this channel
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn add_role_to_blacklist(
    ctx: Context<'_>,
    #[description = "Role to blacklist"] role: serenity::Role,
    #[description = "Custom message to send to users (optional)"] custom_message: Option<String>,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let role_id = role.id.get();

    // Check if already blacklisted
    if ctx.data().is_blacklisted(channel_id, role_id).await {
        ctx.say(format!(
            "⚠️ Role **{}** is already blacklisted in this channel.",
            role.name
        ))
        .await?;
        return Ok(());
    }

    ctx.data()
        .add_blacklist_entry(channel_id, role_id, custom_message.clone())
        .await?;

    let message = if let Some(msg) = custom_message {
        format!(
            "✅ Role **{}** has been blacklisted from this channel.\nCustom message: `{}`",
            role.name, msg
        )
    } else {
        format!(
            "✅ Role **{}** has been blacklisted from this channel.",
            role.name
        )
    };

    ctx.say(message).await?;
    Ok(())
}

/// Remove a role from the blacklist for this channel
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn remove_role_from_blacklist(
    ctx: Context<'_>,
    #[description = "Role to remove from blacklist"] role: serenity::Role,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let role_id = role.id.get();

    // Check if blacklisted
    if !ctx.data().is_blacklisted(channel_id, role_id).await {
        ctx.say(format!(
            "⚠️ Role **{}** is not blacklisted in this channel.",
            role.name
        ))
        .await?;
        return Ok(());
    }

    ctx.data()
        .remove_blacklist_entry(channel_id, role_id)
        .await?;

    ctx.say(format!(
        "✅ Role **{}** has been removed from the blacklist for this channel.",
        role.name
    ))
    .await?;
    Ok(())
}

/// Add a role exception that allows users to write even if they have blacklisted roles
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn add_exception_role(
    ctx: Context<'_>,
    #[description = "Role that bypasses blacklist"] role: serenity::Role,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let role_id = role.id.get();

    // Check if already an exception
    if ctx.data().is_exception(channel_id, role_id).await {
        ctx.say(format!(
            "⚠️ Role **{}** is already an exception role in this channel.",
            role.name
        ))
        .await?;
        return Ok(());
    }

    ctx.data().add_exception_entry(channel_id, role_id).await?;

    ctx.say(format!(
        "✅ Role **{}** has been added as an exception role for this channel.\n\
        Users with this role can write even if they have blacklisted roles.",
        role.name
    ))
    .await?;
    Ok(())
}

/// Remove a role exception
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn remove_exception_role(
    ctx: Context<'_>,
    #[description = "Role to remove from exceptions"] role: serenity::Role,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let role_id = role.id.get();

    // Check if is an exception
    if !ctx.data().is_exception(channel_id, role_id).await {
        ctx.say(format!(
            "⚠️ Role **{}** is not an exception role in this channel.",
            role.name
        ))
        .await?;
        return Ok(());
    }

    ctx.data()
        .remove_exception_entry(channel_id, role_id)
        .await?;

    ctx.say(format!(
        "✅ Role **{}** has been removed from exception roles for this channel.",
        role.name
    ))
    .await?;
    Ok(())
}

/// List all blacklisted roles for this channel
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn list_blacklisted_roles(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    let mut response = String::from("**Channel Configuration:**\n\n");

    // Get guild roles for lookup
    let guild_roles = guild_id.roles(ctx.http()).await?;

    // List blacklisted roles
    let blacklisted_roles = ctx.data().get_blacklisted_roles(channel_id).await?;

    if !blacklisted_roles.is_empty() {
        response.push_str("📛 **Blacklisted Roles:**\n");
        for (role_id, custom_message) in blacklisted_roles {
            if let Some(role) = guild_roles.get(&RoleId::new(role_id)) {
                response.push_str(&format!("• {}", role.name));
                if !custom_message.is_empty() {
                    response.push_str(&format!(" - Message: `{custom_message}`"));
                }
                response.push('\n');
            } else {
                response.push_str(&format!("• Role ID {role_id} (deleted role)\n"));
            }
        }
    } else {
        response.push_str("📛 **No blacklisted roles**\n");
    }

    response.push('\n');

    // List exception roles
    let exception_roles = ctx.data().get_exception_roles(channel_id).await?;

    if !exception_roles.is_empty() {
        response.push_str("✅ **Exception Roles:**\n");
        for role_id in exception_roles {
            if let Some(role) = guild_roles.get(&RoleId::new(role_id)) {
                response.push_str(&format!("• {}\n", role.name));
            } else {
                response.push_str(&format!("• Role ID {role_id} (deleted role)\n"));
            }
        }
    } else {
        response.push_str("✅ **No exception roles**\n");
    }

    ctx.say(response).await?;
    Ok(())
}

#[async_trait]
impl serenity::EventHandler for Data {
    async fn message(&self, ctx: serenity::Context, msg: serenity::Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        let channel_id = msg.channel_id.get();

        // Get user's roles
        let member = match msg.member(&ctx.http).await {
            Ok(member) => member,
            Err(_) => return,
        };

        let role_ids: Vec<u64> = member.roles.iter().map(|r| r.get()).collect();

        // Check if user has any exception roles first
        if self.has_exception(channel_id, &role_ids).await {
            return;
        }

        // Check if user has any blacklisted roles
        if let Some((_, custom_message)) = self.check_blacklist(channel_id, &role_ids).await {
            // Delete the message
            if let Err(e) = msg.delete(&ctx.http).await {
                eprintln!("Failed to delete message: {e}");
            }

            // Send DM to user
            let dm_message = if custom_message.is_empty() {
                format!(
                    "Your message in <#{channel_id}> was deleted because you have a blacklisted role."
                )
            } else {
                custom_message
            };

            if let Ok(dm_channel) = msg.author.create_dm_channel(&ctx.http).await {
                let _ = dm_channel
                    .send_message(
                        &ctx.http,
                        serenity::CreateMessage::new().content(dm_message),
                    )
                    .await;
            }
        }
    }

    async fn ready(&self, _: serenity::Context, ready: serenity::Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    // Initialize data
    let data = Data::new().await.expect("Failed to initialize data");
    let data_clone = data.clone();

    let intents = serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                add_role_to_blacklist(),
                remove_role_from_blacklist(),
                add_exception_role(),
                remove_exception_role(),
                list_blacklisted_roles(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data_clone)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(data)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
