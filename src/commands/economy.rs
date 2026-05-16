use rand::Rng;
use chrono::Timelike;

use poise::serenity_prelude as serenity;
use serenity::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter};

use crate::commands::moderation::info;
use crate::lang::lang;
use crate::{Context, Error};

const ARBEIT_COOLDOWN_SECS: i64 = 3600;

struct WorkTierConfig {
    min_coins: i64,
    max_coins: i64,
    xp_reward: u64,
}

static TIER_CONFIGS: &[WorkTierConfig] = &[
    WorkTierConfig { min_coins: 40,  max_coins: 80,  xp_reward: 15 },
    WorkTierConfig { min_coins: 70,  max_coins: 130, xp_reward: 20 },
    WorkTierConfig { min_coins: 110, max_coins: 200, xp_reward: 30 },
    WorkTierConfig { min_coins: 180, max_coins: 350, xp_reward: 50 },
];

const KLAUEN_COOLDOWN_SECS: i64 = 1800;

/// Arbeite und verdiene Coins - einmal pro Stunde (Belohnung steigt mit deinem Level)
#[poise::command(slash_command, guild_only)]
pub async fn arbeit(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let user = ctx.author();

    if is_economy_jailed(ctx).await? { return Ok(()); }

    // ── cooldown check ────────────────────────────────────────────────────────
    let now = chrono::Utc::now().timestamp();
    if let Some(last) = crate::db::get_arbeit_cooldown(&ctx.data().db, guild_id, user.id).await {
        let elapsed = now - last;
        if elapsed < ARBEIT_COOLDOWN_SECS {
            let remaining = ARBEIT_COOLDOWN_SECS - elapsed;
            let mins = remaining / 60;
            let secs = remaining % 60;
            ctx.send(
                poise::CreateReply::default().embed(
                    CreateEmbed::new()
                        .description(
                            lang().work_exhausted
                                .replace("{mins}", &mins.to_string())
                                .replace("{secs}", &secs.to_string())
                        )
                        .color(0xED4245u32),
                ),
            ).await?;
            return Ok(());
        }
    }

    // ── determine tier from level ─────────────────────────────────────────────
    let total_xp = crate::db::get_xp(&ctx.data().db, guild_id, user.id).await;
    let level = crate::xp::level_from_xp(total_xp);
    let tier_idx = match level {
        0..=4   => 0,
        5..=9   => 1,
        10..=19 => 2,
        _       => 3,
    };
    let tier = &TIER_CONFIGS[tier_idx];
    let (tier_name, tier_jobs) = lang().work_tier(tier_idx);

    // ── roll coins and pick job (rng dropped before any await) ───────────────
    let (label, coins, response) = {
        let mut rng = rand::thread_rng();
        let coins: i64 = rng.gen_range(tier.min_coins..=tier.max_coins);
        let (job_name, template) = tier_jobs[rng.gen_range(0..tier_jobs.len())];
        let text = template.replace("{}", &coins.to_string());
        (job_name, coins, text)
    };

    // ── XP: apply booster if active ───────────────────────────────────────────
    let has_booster = crate::db::has_active_shop_item(&ctx.data().db, guild_id, user.id, "xp_booster").await;
    let xp_gain = if has_booster { tier.xp_reward * 2 } else { tier.xp_reward };

    let old_xp  = total_xp;
    let new_xp  = crate::db::add_xp(&ctx.data().db, guild_id, user.id, xp_gain).await;
    let old_lvl = crate::xp::level_from_xp(old_xp);
    let new_lvl = crate::xp::level_from_xp(new_xp);

    // ── apply coins + feed 10% to bank + update cooldown ─────────────────────
    let new_balance = crate::db::add_coins(&ctx.data().db, guild_id, user.id, coins).await;
    crate::db::add_to_bank(&ctx.data().db, guild_id, coins / 10).await;
    crate::db::set_arbeit_cooldown(&ctx.data().db, guild_id, user.id, now).await;

    let tier_label = format!("{} (Level {})", tier_name, level);

    let mut embed = CreateEmbed::new()
        .author(serenity::CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
        .title(format!("💼 {}: {}", label, tier_label))
        .description(response)
        .color(0x57F287u32)
        .field(
            lang().work_result_field,
            format!("+{} Coins → Kontostand: **{} Coins**", coins, new_balance),
            true,
        )
        .field(
            lang().work_xp_field,
            format!("+{} XP{}", xp_gain, if has_booster { " 🚀" } else { "" }),
            true,
        )
        .footer(CreateEmbedFooter::new(lang().work_footer));

    if new_lvl > old_lvl && old_lvl < 50 {
        embed = embed.field(
            lang().work_levelup_field,
            lang().work_levelup_desc
                .replace("{old}", &old_lvl.to_string())
                .replace("{new}", &new_lvl.to_string())
                .replace("{coins}", &(new_lvl * 100).to_string()),
            false,
        );
    }

    let ready_at = now + ARBEIT_COOLDOWN_SECS;
    let remind_btn = CreateButton::new(format!("remind_arbeit_{}_{}", user.id, ready_at))
        .label(lang().work_remind_btn)
        .style(serenity::ButtonStyle::Secondary);

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .components(vec![CreateActionRow::Buttons(vec![remind_btn])]),
    ).await?;

    // ── level-up: grant coins and announce ────────────────────────────────────
    if new_lvl > old_lvl && old_lvl < 50 {
        let reward = (new_lvl * 100) as i64;
        crate::db::add_coins(&ctx.data().db, guild_id, user.id, reward).await;
        crate::db::set_credited_level(&ctx.data().db, guild_id, user.id, new_lvl).await;
        let bot_ch = crate::db::get_bot_channel(&ctx.data().db, guild_id).await;
        if let Some(ch) = bot_ch {
            let _ = ch.send_message(
                ctx.serenity_context(),
                crate::commands::levels::level_up_embed(user.id, new_lvl),
            ).await;
        }
    }

    crate::events::send_bot_log(ctx.serenity_context(), ctx.data(), guild_id,
        serenity::CreateEmbed::new()
            .title(format!("💼 Arbeit: {}", label))
            .color(0x57F287u32)
            .field("Nutzer", format!("<@{}>", user.id), true)
            .field(lang().work_result_field, format!("+{} Coins", coins), true)
            .field("Kontostand", format!("{} Coins", new_balance), true)
            .timestamp(serenity::Timestamp::now()),
    ).await;

    Ok(())
}

/// Bestiehl einen anderen Nutzer - Erwischungsrisiko steigt am Tag
#[poise::command(slash_command, guild_only)]
pub async fn klauen(
    ctx: Context<'_>,
    #[description = "Wen willst du bestehlen?"] opfer: serenity::User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let dieb = ctx.author();

    if is_economy_jailed(ctx).await? { return Ok(()); }

    if opfer.bot {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .description(lang().steal_no_bot)
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }
    if opfer.id == dieb.id {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .description(lang().steal_no_self)
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }

    // ── cooldown ──────────────────────────────────────────────────────────────
    let now = chrono::Utc::now().timestamp();
    if let Some(last) = crate::db::get_klauen_cooldown(&ctx.data().db, guild_id, dieb.id).await {
        let elapsed = now - last;
        if elapsed < KLAUEN_COOLDOWN_SECS {
            let remaining = KLAUEN_COOLDOWN_SECS - elapsed;
            let mins = remaining / 60;
            let secs = remaining % 60;
            ctx.send(poise::CreateReply::default().embed(
                CreateEmbed::new()
                    .description(
                        lang().steal_cooldown
                            .replace("{mins}", &mins.to_string())
                            .replace("{secs}", &secs.to_string())
                    )
                    .color(0xED4245u32),
            )).await?;
            return Ok(());
        }
    }

    // ── diebstahlschutz: auto-fail if victim has the protection item ──────────
    if crate::db::has_active_shop_item(&ctx.data().db, guild_id, opfer.id, "diebstahlschutz").await {
        crate::db::consume_shop_item(&ctx.data().db, guild_id, opfer.id, "diebstahlschutz").await;
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(dieb.tag()).icon_url(dieb.face()))
                .title(lang().steal_protection_title)
                .description(
                    lang().steal_protection_desc
                        .replace("{victim}", &opfer.id.to_string())
                )
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }

    // ── time-based catch probability ──────────────────────────────────────────
    let hour = chrono::Local::now().hour();
    let catch_pct: u32 = if hour >= 22 || hour < 6 {
        40
    } else if (6..10).contains(&hour) || (18..22).contains(&hour) {
        45
    } else {
        60
    };

    let (caught, stolen, fine) = {
        let mut rng = rand::thread_rng();
        let caught = rng.gen_range(0..100) < catch_pct;
        let stolen: i64 = rng.gen_range(0..=15);
        let fine: i64 = rng.gen_range(0..=20);
        (caught, stolen, fine)
    };

    crate::db::set_klauen_cooldown(&ctx.data().db, guild_id, dieb.id, now).await;

    let opfer_mention = format!("<@{}>", opfer.id);

    if caught {
        let new_balance = crate::db::add_coins(&ctx.data().db, guild_id, dieb.id, -fine).await;
        crate::db::add_to_bank(&ctx.data().db, guild_id, fine).await;
        let templates = lang().steal_caught_templates;
        let template = { let mut rng = rand::thread_rng(); templates[rng.gen_range(0..templates.len())] };
        let text = template
            .replace("{target}", &opfer_mention)
            .replace("{coins}", &fine.to_string());

        let time_label = if hour >= 22 || hour < 6 { lang().steal_daytime_night }
            else if (6..10).contains(&hour) || (18..22).contains(&hour) { lang().steal_daytime_morning }
            else { lang().steal_daytime_day };

        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(dieb.tag()).icon_url(dieb.face()))
                .title(lang().steal_caught_title)
                .description(text)
                .color(0xED4245u32)
                .field(lang().steal_balance_field, format!("**{} Coins**", new_balance), true)
                .field(lang().steal_daytime_field, time_label, true),
        )).await?;
        crate::events::send_bot_log(ctx.serenity_context(), ctx.data(), guild_id,
            serenity::CreateEmbed::new()
                .title("🚔 Klauen: Erwischt")
                .color(0xED4245u32)
                .field("Dieb", format!("<@{}>", dieb.id), true)
                .field("Opfer", format!("<@{}>", opfer.id), true)
                .field("Strafe", format!("-{} Coins", fine), true)
                .timestamp(serenity::Timestamp::now()),
        ).await;
    } else {
        let opfer_balance = crate::db::get_coins(&ctx.data().db, guild_id, opfer.id).await;
        let actual_stolen = stolen.min(opfer_balance).max(0);
        crate::db::add_coins(&ctx.data().db, guild_id, opfer.id, -actual_stolen).await;
        let new_balance = crate::db::add_coins(&ctx.data().db, guild_id, dieb.id, actual_stolen).await;

        let templates = lang().steal_success_templates;
        let template = { let mut rng = rand::thread_rng(); templates[rng.gen_range(0..templates.len())] };
        let text = template
            .replace("{target}", &opfer_mention)
            .replace("{coins}", &actual_stolen.to_string());

        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(dieb.tag()).icon_url(dieb.face()))
                .title(lang().steal_success_title)
                .description(text)
                .color(0x57F287u32)
                .field(lang().steal_balance_field, format!("**{} Coins**", new_balance), true),
        )).await?;
        crate::events::send_bot_log(ctx.serenity_context(), ctx.data(), guild_id,
            serenity::CreateEmbed::new()
                .title("💸 Klauen: Erfolgreich")
                .color(0x57F287u32)
                .field("Dieb", format!("<@{}>", dieb.id), true)
                .field("Opfer", format!("<@{}>", opfer.id), true)
                .field("Gestohlen", format!("+{} Coins", actual_stolen), true)
                .timestamp(serenity::Timestamp::now()),
        ).await;
    }

    Ok(())
}

// ── jail check helper ─────────────────────────────────────────────────────────

async fn is_economy_jailed(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap();
    let now = chrono::Utc::now().timestamp();
    if let Some(until) = crate::db::get_jail_until(&ctx.data().db, guild_id, ctx.author().id).await {
        if until > now {
            let remaining = until - now;
            let hours = remaining / 3600;
            let mins = (remaining % 3600) / 60;
            ctx.send(poise::CreateReply::default().embed(
                CreateEmbed::new()
                    .title(lang().jail_title)
                    .description(
                        lang().jail_desc
                            .replace("{hours}", &hours.to_string())
                            .replace("{mins}", &mins.to_string())
                    )
                    .color(0xED4245u32),
            )).await?;
            return Ok(true);
        }
    }
    Ok(false)
}

// ── /banküberfall ─────────────────────────────────────────────────────────────

/// Beraube die Bank - einmal täglich, hohes Risiko, alles oder nichts
#[poise::command(slash_command, guild_only, rename = "bankueberfall")]
pub async fn bankueberfall(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let user = ctx.author();
    let now = chrono::Utc::now().timestamp();

    if is_economy_jailed(ctx).await? { return Ok(()); }

    // ── cooldown: once per 24h ────────────────────────────────────────────────
    if let Some(last) = crate::db::get_bankraub_cooldown(&ctx.data().db, guild_id, user.id).await {
        let elapsed = now - last;
        if elapsed < 86400 {
            let remaining = 86400 - elapsed;
            let hours = remaining / 3600;
            let mins = (remaining % 3600) / 60;
            ctx.send(poise::CreateReply::default().embed(
                CreateEmbed::new()
                    .description(
                        lang().bank_cooldown
                            .replace("{hours}", &hours.to_string())
                            .replace("{mins}", &mins.to_string())
                    )
                    .color(0xED4245u32),
            )).await?;
            return Ok(());
        }
    }

    let bank_balance = crate::db::get_bank(&ctx.data().db, guild_id).await;

    if bank_balance == 0 {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .title(lang().bank_empty_title)
                .description(lang().bank_empty_desc)
                .color(0xFEE75Cu32),
        )).await?;
        return Ok(());
    }

    // ── time-based catch probability ──────────────────────────────────────────
    let hour = chrono::Local::now().hour();
    let catch_pct: u32 = if hour >= 22 || hour < 6 { 55 }
        else if (6..10).contains(&hour) || (18..22).contains(&hour) { 65 }
        else { 75 };

    let (caught, jail_hours) = {
        let mut rng = rand::thread_rng();
        let caught = rng.gen_range(0..100) < catch_pct;
        let jail_hours: i64 = rng.gen_range(8..=12);
        (caught, jail_hours)
    };

    crate::db::set_bankraub_cooldown(&ctx.data().db, guild_id, user.id, now).await;

    let time_label = if hour >= 22 || hour < 6 { lang().bank_daytime_night }
        else if (6..10).contains(&hour) || (18..22).contains(&hour) { lang().bank_daytime_morning }
        else { lang().bank_daytime_day };

    if caught {
        let jail_until = now + jail_hours * 3600;
        crate::db::set_jail_until(&ctx.data().db, guild_id, user.id, jail_until).await;

        let wallet_before = crate::db::get_coins(&ctx.data().db, guild_id, user.id).await;
        let fine = wallet_before / 5;
        let wallet_after = crate::db::add_coins(&ctx.data().db, guild_id, user.id, -fine).await;

        let templates = lang().bank_caught_templates;
        let template = { let mut rng = rand::thread_rng(); templates[rng.gen_range(0..templates.len())] };
        let text = template.replace("{hours}", &jail_hours.to_string());

        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
                .title(lang().bank_caught_title)
                .description(text)
                .color(0xED4245u32)
                .field(lang().bank_fine_field,        format!("-{} Coins (20% des Kontos)", fine), true)
                .field(lang().bank_balance_field,     format!("{} Coins", wallet_after), true)
                .field(lang().bank_release_field,     format!("<t:{}:R>", jail_until), true),
        )).await?;
        crate::events::send_bot_log(ctx.serenity_context(), ctx.data(), guild_id,
            serenity::CreateEmbed::new()
                .title("🚔 Bankraub: Erwischt")
                .color(0xED4245u32)
                .field("Nutzer",     format!("<@{}>", user.id), true)
                .field(lang().bank_jail_hours_field, format!("{} Stunden", jail_hours), true)
                .field("Strafe",     format!("-{} Coins", fine), true)
                .field(lang().bank_release_field, format!("<t:{}:R>", jail_until), false)
                .timestamp(serenity::Timestamp::now()),
        ).await;
    } else {
        let stolen = crate::db::drain_bank(&ctx.data().db, guild_id).await;
        let new_balance = crate::db::add_coins(&ctx.data().db, guild_id, user.id, stolen).await;

        let templates = lang().bank_success_templates;
        let template = { let mut rng = rand::thread_rng(); templates[rng.gen_range(0..templates.len())] };
        let text = template.replace("{coins}", &stolen.to_string());

        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
                .title(lang().bank_success_title)
                .description(text)
                .color(0x57F287u32)
                .field(lang().bank_loot_field,        format!("**{} Coins**", stolen), true)
                .field(lang().bank_new_balance_field, format!("**{} Coins**", new_balance), true)
                .field(lang().bank_daytime_label,     time_label, true),
        )).await?;
        crate::events::send_bot_log(ctx.serenity_context(), ctx.data(), guild_id,
            serenity::CreateEmbed::new()
                .title("💰 Bankraub: Erfolgreich")
                .color(0x57F287u32)
                .field("Nutzer", format!("<@{}>", user.id), true)
                .field(lang().bank_loot_field,        format!("{} Coins", stolen), true)
                .field(lang().bank_new_balance_field, format!("{} Coins", new_balance), true)
                .timestamp(serenity::Timestamp::now()),
        ).await;
    }

    Ok(())
}

/// Dein Kontostand oder der eines anderen Nutzers
#[poise::command(slash_command, guild_only)]
pub async fn coins(
    ctx: Context<'_>,
    #[description = "Nutzer (Standard: du selbst)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let target = user.as_ref().unwrap_or_else(|| ctx.author());
    let guild_id = ctx.guild_id().unwrap();

    if target.bot {
        ctx.send(
            poise::CreateReply::default()
                .embed(info(lang().coins_bot_invalid, "Bots haben kein Konto.")),
        ).await?;
        return Ok(());
    }

    let balance = crate::db::get_coins(&ctx.data().db, guild_id, target.id).await;
    let invites = crate::db::get_invites(&ctx.data().db, guild_id, target.id).await;

    let embed = CreateEmbed::new()
        .author(serenity::CreateEmbedAuthor::new(target.tag()).icon_url(target.face()))
        .color(0xF1C40Fu32)
        .field(lang().coins_balance_field, format!("**{} Coins**", balance), true)
        .field(lang().coins_invites_field, format!("**{}**", invites), true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Top 10 Nutzer nach Coins
#[poise::command(slash_command, guild_only)]
pub async fn coins_leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let entries = crate::db::get_coins_leaderboard(&ctx.data().db, guild_id, 10).await;

    if entries.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .embed(info("Keine Daten", lang().coins_no_data)),
        ).await?;
        return Ok(());
    }

    let medals = ["🥇", "🥈", "🥉"];
    let mut lines = Vec::new();
    for (i, (user_id, coins)) in entries.iter().enumerate() {
        let prefix = medals.get(i).copied().unwrap_or("🔹");
        lines.push(format!("{} **#{}** <@{}> -**{}** Coins", prefix, i + 1, user_id, coins));
    }

    let guild_icon = ctx.guild().and_then(|g| g.icon_url()).unwrap_or_default();
    let guild_name = ctx.guild().map(|g| g.name.clone()).unwrap_or_default();

    let embed = CreateEmbed::new()
        .title(lang().coins_lb_title)
        .description(lines.join("\n"))
        .color(0xF1C40Fu32)
        .thumbnail(guild_icon)
        .footer(CreateEmbedFooter::new(lang().coins_lb_footer.replace("{guild}", &guild_name)));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Coins an einen anderen Nutzer überweisen
#[poise::command(slash_command, guild_only, rename = "ueberweisung")]
pub async fn ueberweisung(
    ctx: Context<'_>,
    #[description = "Empfänger"] empfaenger: serenity::User,
    #[description = "Betrag (min. 1)"] betrag: i64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let sender   = ctx.author();

    if empfaenger.bot {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .description(lang().transfer_bot_invalid)
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }

    if empfaenger.id == sender.id {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .description(lang().transfer_self_invalid)
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }

    if betrag < 1 {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .description(lang().transfer_min_amount)
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }

    let balance = crate::db::get_coins(&ctx.data().db, guild_id, sender.id).await;
    if balance < betrag {
        ctx.send(poise::CreateReply::default().embed(
            CreateEmbed::new()
                .description(
                    lang().transfer_not_enough
                        .replace("{have}", &balance.to_string())
                        .replace("{need}", &betrag.to_string())
                )
                .color(0xED4245u32),
        )).await?;
        return Ok(());
    }

    crate::db::add_coins(&ctx.data().db, guild_id, sender.id, -betrag).await;
    let new_receiver = crate::db::add_coins(&ctx.data().db, guild_id, empfaenger.id, betrag).await;
    let new_sender   = crate::db::get_coins(&ctx.data().db, guild_id, sender.id).await;

    ctx.send(poise::CreateReply::default().embed(
        CreateEmbed::new()
            .author(serenity::CreateEmbedAuthor::new(sender.tag()).icon_url(sender.face()))
            .title(lang().transfer_success_title)
            .description(
                lang().transfer_success_desc
                    .replace("{amount}", &betrag.to_string())
                    .replace("{recipient}", &empfaenger.id.to_string())
            )
            .color(0x57F287u32)
            .field(lang().transfer_sender_balance_field, format!("**{} Coins**", new_sender), true)
            .field(
                lang().transfer_recipient_balance_field.replace("{name}", &empfaenger.name),
                format!("**{} Coins**", new_receiver),
                true,
            ),
    )).await?;

    crate::events::send_bot_log(ctx.serenity_context(), ctx.data(), guild_id,
        serenity::CreateEmbed::new()
            .title("💸 Überweisung")
            .color(0x57F287u32)
            .field("Von",    format!("<@{}>", sender.id),     true)
            .field("An",     format!("<@{}>", empfaenger.id), true)
            .field("Betrag", format!("{} Coins", betrag),     true)
            .timestamp(serenity::Timestamp::now()),
    ).await;

    Ok(())
}
