use poise::serenity_prelude as serenity;
use serenity::CreateEmbed;

use crate::config::AutomodConfig;
use crate::{Context, Error};

// ── /automod ──────────────────────────────────────────────────────────────────

/// Automod-System konfigurieren
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    guild_only,
    subcommands("setup", "status")
)]
pub async fn automod_cmd(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Automod-Einstellungen konfigurieren
#[poise::command(slash_command, required_permissions = "MANAGE_GUILD", guild_only, rename = "setup")]
pub async fn setup(
    ctx: Context<'_>,
    #[description = "Anti-Spam aktivieren (>5 Nachrichten in 5s = Auto-Mute)"] anti_spam: Option<bool>,
    #[description = "Anti-Invite aktivieren (Discord-Einladungslinks löschen)"] anti_invite: Option<bool>,
    #[description = "Anti-Caps aktivieren (>80% Großbuchstaben löschen)"] anti_caps: Option<bool>,
    #[description = "Spam-Limit: Nachrichten pro Zeitfenster (Standard: 5)"]
    #[min = 2]
    #[max = 20]
    spam_limit: Option<i64>,
    #[description = "Spam-Zeitfenster in Sekunden (Standard: 5)"]
    #[min = 2]
    #[max = 30]
    spam_window: Option<i64>,
    #[description = "Log-Kanal für Automod-Aktionen"] log_channel: Option<serenity::GuildChannel>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap();

    let mut cfg = {
        let configs = ctx.data().automod_configs.lock().await;
        configs.get(&guild_id).cloned().unwrap_or_default()
    };

    if cfg.spam_limit == 0 { cfg.spam_limit = 5; }
    if cfg.spam_window == 0 { cfg.spam_window = 5; }

    let mut changed: Vec<String> = Vec::new();

    if let Some(v) = anti_spam   { cfg.anti_spam   = v; changed.push(format!("Anti-Spam: **{}**", if v { "✅ An" } else { "❌ Aus" })); }
    if let Some(v) = anti_invite { cfg.anti_invite = v; changed.push(format!("Anti-Invite: **{}**", if v { "✅ An" } else { "❌ Aus" })); }
    if let Some(v) = anti_caps   { cfg.anti_caps   = v; changed.push(format!("Anti-Caps: **{}**",   if v { "✅ An" } else { "❌ Aus" })); }
    if let Some(v) = spam_limit  { cfg.spam_limit  = v; changed.push(format!("Spam-Limit: **{}**", v)); }
    if let Some(v) = spam_window { cfg.spam_window = v; changed.push(format!("Spam-Fenster: **{}s**", v)); }
    if let Some(ch) = &log_channel {
        cfg.log_channel = Some(ch.id);
        changed.push(format!("Log-Kanal: <#{}>", ch.id));
    }

    crate::db::save_automod_config(&ctx.data().db, guild_id, &cfg).await;

    {
        let mut configs = ctx.data().automod_configs.lock().await;
        configs.insert(guild_id, cfg);
    }

    let desc = if changed.is_empty() {
        "Keine Änderungen vorgenommen.".to_string()
    } else {
        changed.join("\n")
    };

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🛡️ Automod konfiguriert")
                    .description(desc)
                    .color(0x57F287u32),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Aktuelle Automod-Einstellungen anzeigen
#[poise::command(slash_command, required_permissions = "MANAGE_GUILD", guild_only, rename = "status")]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap();
    let cfg = {
        let configs = ctx.data().automod_configs.lock().await;
        configs.get(&guild_id).cloned().unwrap_or(AutomodConfig {
            spam_limit:  5,
            spam_window: 5,
            ..Default::default()
        })
    };

    let on_off = |b: bool| if b { "✅ An" } else { "❌ Aus" };
    let log_ch = cfg.log_channel.map(|c| format!("<#{}>", c)).unwrap_or_else(|| "Nicht konfiguriert".to_string());

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🛡️ Automod Status")
                    .color(0x5865F2u32)
                    .field("Anti-Spam",   on_off(cfg.anti_spam),   true)
                    .field("Anti-Invite", on_off(cfg.anti_invite), true)
                    .field("Anti-Caps",   on_off(cfg.anti_caps),   true)
                    .field("Spam-Limit",  format!("{} Nachrichten", cfg.spam_limit),  true)
                    .field("Spam-Fenster", format!("{}s", cfg.spam_window), true)
                    .field("Log-Kanal",   log_ch, false),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
