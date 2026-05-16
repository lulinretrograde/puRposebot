use poise::serenity_prelude as serenity;
use serenity::CreateEmbed;

use crate::{Context, Error};

pub const FISH_GOAL:   i64 = 3;
pub const STEAL_GOAL:  i64 = 2;
pub const CASINO_GOAL: i64 = 5;
pub const BONUS_COINS: i64 = 1_000;
pub const BONUS_XP:    u64 = 500;

fn progress_bar(done: i64, goal: i64) -> String {
    let pct = (done.min(goal) * 10 / goal) as usize;
    let filled = "█".repeat(pct);
    let empty  = "░".repeat(10 - pct);
    format!("{}{} {}/{}", filled, empty, done.min(goal), goal)
}

// ── /aufgaben ─────────────────────────────────────────────────────────────────

/// Tägliche Aufgaben anzeigen
#[poise::command(slash_command, guild_only)]
pub async fn aufgaben(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap();
    let user_id  = ctx.author().id;

    let m = crate::db::get_daily_missions(&ctx.data().db, guild_id, user_id).await;

    let fish_done  = m.fish_done.min(FISH_GOAL);
    let steal_done = m.steal_done.min(STEAL_GOAL);
    let casino_done= m.casino_done.min(CASINO_GOAL);

    let all_done = fish_done >= FISH_GOAL && steal_done >= STEAL_GOAL && casino_done >= CASINO_GOAL;

    // Auto-claim bonus if all done and not yet claimed
    let bonus_status = if all_done && !m.bonus_claimed {
        let claimed = crate::db::claim_daily_bonus(&ctx.data().db, guild_id, user_id).await;
        if claimed {
            crate::db::add_coins(&ctx.data().db, guild_id, user_id, BONUS_COINS).await;
            crate::db::add_xp(&ctx.data().db, guild_id, user_id, BONUS_XP).await;
            format!("🎉 **Bonus erhalten!** +{} Coins, +{} XP", BONUS_COINS, BONUS_XP)
        } else {
            "✅ Bonus bereits beansprucht".to_string()
        }
    } else if all_done && m.bonus_claimed {
        "✅ Bonus bereits beansprucht".to_string()
    } else {
        "⏳ Schließe alle Aufgaben ab, um den Bonus zu erhalten!".to_string()
    };

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("📋 Tägliche Aufgaben")
                    .description(format!(
                        "**{}** — Abschlussbonus: **{} Coins + {} XP**\n\n\
                        🎣 **Angeln** ({} Fänge)\n{}\n\n\
                        💸 **Stehlen** ({} erfolgreiche Diebstähle)\n{}\n\n\
                        🎰 **Casino** ({} Spiele)\n{}\n\n\
                        {}",
                        today,
                        BONUS_COINS, BONUS_XP,
                        FISH_GOAL,   progress_bar(fish_done,  FISH_GOAL),
                        STEAL_GOAL,  progress_bar(steal_done, STEAL_GOAL),
                        CASINO_GOAL, progress_bar(casino_done,CASINO_GOAL),
                        bonus_status,
                    ))
                    .color(if all_done { 0x57F287u32 } else { 0x5865F2u32 }),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
