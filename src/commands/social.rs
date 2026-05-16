use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, Timestamp};

use crate::{Context, Error};
use crate::commands::moderation::{err, info, ok};

// ── /profil ───────────────────────────────────────────────────────────────────

/// Profil eines Nutzers anzeigen
#[poise::command(slash_command, guild_only)]
pub async fn profil(
    ctx: Context<'_>,
    #[description = "Nutzer (leer lassen für dein eigenes Profil)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let target   = user.as_ref().unwrap_or(ctx.author());

    if target.bot {
        ctx.send(poise::CreateReply::default().embed(err("Kein Profil", "Bots haben kein Profil.")).ephemeral(true)).await?;
        return Ok(());
    }

    let bio     = crate::db::get_bio(&ctx.data().db, guild_id, target.id).await;
    let coins   = crate::db::get_coins(&ctx.data().db, guild_id, target.id).await;
    let xp      = crate::db::get_xp(&ctx.data().db, guild_id, target.id).await;
    let level   = crate::xp::level_from_xp(xp);
    let prestige= crate::db::get_prestige(&ctx.data().db, guild_id, target.id).await;
    let rep     = crate::db::get_rep(&ctx.data().db, guild_id, target.id).await;
    let partner = crate::db::get_partner(&ctx.data().db, guild_id, target.id).await;
    let bday    = crate::db::get_birthday(&ctx.data().db, guild_id, target.id).await;

    let bio_display = if bio.is_empty() { "*Keine Bio gesetzt.*".to_string() } else { bio };
    let partner_display = match partner {
        Some(p) => format!("💍 <@{}>", p),
        None    => "Ledig".to_string(),
    };
    let bday_display = match bday {
        Some((m, d)) => format!("{:02}.{:02}.", d, m),
        None         => "Nicht gesetzt".to_string(),
    };
    let prestige_stars = "⭐".repeat(prestige as usize);

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .author(serenity::CreateEmbedAuthor::new(target.tag()).icon_url(target.face()))
                    .thumbnail(target.face())
                    .description(&bio_display)
                    .color(0x5865F2u32)
                    .field("⭐ Level",    format!("{} {}", level, prestige_stars), true)
                    .field("💰 Coins",    format!("{}", coins),                    true)
                    .field("👍 Rep",      format!("{}", rep.rep),                  true)
                    .field("💑 Partner",  partner_display,                         true)
                    .field("🎂 Geburtstag", bday_display,                          true)
                    .timestamp(Timestamp::now()),
            ),
    )
    .await?;

    Ok(())
}

/// Deine Bio setzen
#[poise::command(slash_command, guild_only)]
pub async fn bio(
    ctx: Context<'_>,
    #[description = "Deine neue Bio (max 200 Zeichen)"] text: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if text.chars().count() > 200 {
        ctx.send(poise::CreateReply::default().embed(err("Zu lang", "Bio darf maximal 200 Zeichen haben.")).ephemeral(true)).await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    crate::db::set_bio(&ctx.data().db, guild_id, ctx.author().id, &text).await;

    ctx.send(poise::CreateReply::default().embed(ok("Bio gesetzt", "Deine Bio wurde aktualisiert.")).ephemeral(true)).await?;
    Ok(())
}

// ── /heiraten & /scheiden ─────────────────────────────────────────────────────

/// Einen Nutzer heiraten (kostet 5000 Coins)
#[poise::command(slash_command, guild_only)]
pub async fn heiraten(
    ctx: Context<'_>,
    #[description = "Der Nutzer, den du heiraten möchtest"] user: serenity::User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let me       = ctx.author();

    if user.bot {
        ctx.send(poise::CreateReply::default().embed(err("Kein Bot-Eheschluss", "Bots können nicht heiraten.")).ephemeral(true)).await?;
        return Ok(());
    }
    if user.id == me.id {
        ctx.send(poise::CreateReply::default().embed(err("Narzisstisch", "Du kannst dich nicht selbst heiraten.")).ephemeral(true)).await?;
        return Ok(());
    }

    let cost: i64 = 5_000;
    let balance   = crate::db::get_coins(&ctx.data().db, guild_id, me.id).await;
    if balance < cost {
        ctx.send(poise::CreateReply::default().embed(err(
            "Nicht genug Coins",
            &format!("Eine Hochzeit kostet **{} Coins**. Du hast nur **{}**.", cost, balance),
        )).ephemeral(true)).await?;
        return Ok(());
    }

    let ok_marriage = crate::db::create_marriage(&ctx.data().db, guild_id, me.id, user.id).await;
    if !ok_marriage {
        ctx.send(poise::CreateReply::default().embed(err(
            "Nicht möglich",
            "Einer von euch ist bereits verheiratet. Nutze `/scheiden` zuerst.",
        )).ephemeral(true)).await?;
        return Ok(());
    }

    crate::db::add_coins(&ctx.data().db, guild_id, me.id, -cost).await;

    ctx.send(poise::CreateReply::default().embed(
        CreateEmbed::new()
            .title("💍 Hochzeit!")
            .description(format!("💕 <@{}> und <@{}> haben sich das Ja-Wort gegeben!", me.id, user.id))
            .color(0xFF73FAu32)
            .field("Kosten", format!("{} Coins", cost), true),
    )).await?;

    Ok(())
}

/// Ehe auflösen (kostet 1000 Coins)
#[poise::command(slash_command, guild_only)]
pub async fn scheiden(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let me       = ctx.author();

    let partner = crate::db::get_partner(&ctx.data().db, guild_id, me.id).await;
    if partner.is_none() {
        ctx.send(poise::CreateReply::default().embed(info("Nicht verheiratet", "Du bist nicht verheiratet.")).ephemeral(true)).await?;
        return Ok(());
    }

    let cost: i64 = 1_000;
    let balance   = crate::db::get_coins(&ctx.data().db, guild_id, me.id).await;
    if balance < cost {
        ctx.send(poise::CreateReply::default().embed(err(
            "Nicht genug Coins",
            &format!("Eine Scheidung kostet **{} Coins**. Du hast nur **{}**.", cost, balance),
        )).ephemeral(true)).await?;
        return Ok(());
    }

    crate::db::divorce(&ctx.data().db, guild_id, me.id).await;
    crate::db::add_coins(&ctx.data().db, guild_id, me.id, -cost).await;

    ctx.send(poise::CreateReply::default().embed(
        CreateEmbed::new()
            .title("💔 Scheidung")
            .description(format!("<@{}> ist nun wieder ledig.", me.id))
            .color(0xED4245u32)
            .field("Scheidungskosten", format!("{} Coins", cost), true),
    )).await?;

    Ok(())
}

// ── /rep ─────────────────────────────────────────────────────────────────────

/// Einem Nutzer +1 Reputation geben (einmal täglich)
#[poise::command(slash_command, guild_only)]
pub async fn rep(
    ctx: Context<'_>,
    #[description = "Nutzer, dem du Reputation geben möchtest"] user: serenity::User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let me       = ctx.author();

    if user.bot {
        ctx.send(poise::CreateReply::default().embed(err("Kein Bot-Rep", "Bots können kein Reputation erhalten.")).ephemeral(true)).await?;
        return Ok(());
    }
    if user.id == me.id {
        ctx.send(poise::CreateReply::default().embed(err("Nein", "Du kannst dir nicht selbst Reputation geben.")).ephemeral(true)).await?;
        return Ok(());
    }

    let my_rep   = crate::db::get_rep(&ctx.data().db, guild_id, me.id).await;
    let now      = chrono::Utc::now().timestamp();
    let cooldown = 86400i64;

    if now - my_rep.last_gave_at < cooldown {
        let remaining = cooldown - (now - my_rep.last_gave_at);
        let h = remaining / 3600;
        let m = (remaining % 3600) / 60;
        ctx.send(poise::CreateReply::default().embed(err(
            "Cooldown",
            &format!("Du kannst erst in **{}h {}m** wieder Rep geben.", h, m),
        )).ephemeral(true)).await?;
        return Ok(());
    }

    if my_rep.last_gave_to == user.id.get() as i64 {
        ctx.send(poise::CreateReply::default().embed(err(
            "Dieselbe Person",
            "Du kannst nicht zweimal hintereinander derselben Person Rep geben.",
        )).ephemeral(true)).await?;
        return Ok(());
    }

    crate::db::give_rep(&ctx.data().db, guild_id, me.id, user.id).await;
    let new_rep = crate::db::get_rep(&ctx.data().db, guild_id, user.id).await;

    ctx.send(poise::CreateReply::default().embed(
        CreateEmbed::new()
            .title("👍 Reputation gegeben!")
            .description(format!("<@{}> hat <@{}> +1 Rep gegeben!", me.id, user.id))
            .color(0x57F287u32)
            .field("Neue Rep von", format!("<@{}>", user.id), true)
            .field("Gesamt-Rep",   format!("{}", new_rep.rep),  true),
    )).await?;

    Ok(())
}

/// Rep-Rangliste anzeigen
#[poise::command(slash_command, guild_only, rename = "rep-rangliste")]
pub async fn rep_rangliste(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let board    = crate::db::get_rep_leaderboard(&ctx.data().db, guild_id).await;

    if board.is_empty() {
        ctx.send(poise::CreateReply::default().embed(info("Leer", "Noch niemand hat Reputation erhalten.")).ephemeral(true)).await?;
        return Ok(());
    }

    let medals = ["🥇", "🥈", "🥉"];
    let lines: Vec<String> = board.iter().enumerate().map(|(i, (uid, rep))| {
        let prefix = medals.get(i).copied().unwrap_or("▪️");
        format!("{} <@{}>: **{} Rep**", prefix, uid, rep)
    }).collect();

    ctx.send(poise::CreateReply::default().embed(
        CreateEmbed::new()
            .title("👍 Reputation-Rangliste")
            .description(lines.join("\n"))
            .color(0x5865F2u32),
    )).await?;

    Ok(())
}

// ── /geburtstag ───────────────────────────────────────────────────────────────

/// Deinen Geburtstag eintragen
#[poise::command(slash_command, guild_only)]
pub async fn geburtstag(
    ctx: Context<'_>,
    #[description = "Monat (1-12)"]
    #[min = 1]
    #[max = 12]
    monat: u8,
    #[description = "Tag (1-31)"]
    #[min = 1]
    #[max = 31]
    tag: u8,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap();
    crate::db::set_birthday(&ctx.data().db, guild_id, ctx.author().id, monat, tag).await;

    ctx.send(poise::CreateReply::default().embed(ok(
        "Geburtstag gesetzt",
        &format!("Dein Geburtstag wurde auf **{:02}.{:02}.** gesetzt. 🎂", tag, monat),
    )).ephemeral(true)).await?;

    Ok(())
}
