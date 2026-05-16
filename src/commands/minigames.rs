use poise::serenity_prelude as serenity;
use serenity::{
    CreateActionRow, CreateButton, CreateEmbed, ButtonStyle,
};
use rand::Rng;

use crate::{Context, Error};
use crate::commands::moderation::{err, info, ok};

// ── trivia ────────────────────────────────────────────────────────────────────

static TRIVIA: &[(&str, &[&str], usize)] = &[
    ("Wie viele Beine hat eine Spinne?", &["6", "8", "10", "4"], 1),
    ("Was ist die Hauptstadt von Deutschland?", &["München", "Hamburg", "Berlin", "Köln"], 2),
    ("Wie viele Planeten hat unser Sonnensystem?", &["7", "8", "9", "10"], 1),
    ("Aus welchem Land stammt Pizza?", &["Frankreich", "Spanien", "Griechenland", "Italien"], 3),
    ("Welches Element hat das Kürzel 'O'?", &["Gold", "Ozon", "Sauerstoff", "Osmium"], 2),
    ("Wie viele Meter hat ein Kilometer?", &["100", "1000", "10000", "500"], 1),
    ("Welcher Planet ist der größte in unserem Sonnensystem?", &["Saturn", "Erde", "Mars", "Jupiter"], 3),
    ("In welchem Jahr fiel die Berliner Mauer?", &["1987", "1989", "1991", "1993"], 1),
    ("Wie viele Tage hat ein Schaltjahr?", &["365", "366", "364", "367"], 1),
    ("Was ist H2O?", &["Salz", "Zucker", "Wasser", "Essig"], 2),
    ("Welcher Kontinent ist am größten?", &["Amerika", "Afrika", "Australien", "Asien"], 3),
    ("Wie viele Minuten hat eine Stunde?", &["30", "100", "60", "45"], 2),
    ("Was ist die schnellste Katze der Welt?", &["Löwe", "Tiger", "Gepard", "Leopard"], 2),
    ("Welches Tier ist das größte Landsäugetier?", &["Nilpferd", "Nashorn", "Giraffe", "Elefant"], 3),
    ("Wie viele Stunden hat ein Tag?", &["12", "24", "48", "36"], 1),
    ("Was ist die chemische Formel für Kochsalz?", &["KCl", "HCl", "NaCl", "CaCl2"], 2),
    ("Welche Farbe entsteht, wenn man Blau und Gelb mischt?", &["Lila", "Orange", "Grün", "Braun"], 2),
    ("Wie viele Seiten hat ein Würfel?", &["4", "8", "6", "12"], 2),
    ("Was ist die Hauptstadt von Japan?", &["Osaka", "Kyoto", "Peking", "Tokio"], 3),
    ("Welcher Ozean ist der größte?", &["Atlantik", "Arktis", "Pazifik", "Indik"], 2),
];

/// Trivia-Frage beantworten und Coins gewinnen
#[poise::command(slash_command, guild_only)]
pub async fn trivia(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();
    let user_id  = ctx.author().id;

    let idx = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..TRIVIA.len())
    };
    let (question, answers, correct_idx) = TRIVIA[idx];

    let labels = ["A", "B", "C", "D"];
    let buttons: Vec<CreateButton> = answers.iter().enumerate().map(|(i, ans)| {
        CreateButton::new(format!("trivia_{}", i))
            .label(format!("{}) {}", labels[i], ans))
            .style(ButtonStyle::Primary)
    }).collect();

    let components = vec![CreateActionRow::Buttons(buttons)];

    let reply = ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🧠 Trivia")
                    .description(question)
                    .color(0x5865F2u32)
                    .footer(serenity::CreateEmbedFooter::new("Du hast 20 Sekunden!")),
            )
            .components(components),
    )
    .await?;

    let msg = reply.message().await?;

    let interaction = msg
        .await_component_interaction(ctx.serenity_context())
        .author_id(user_id)
        .timeout(std::time::Duration::from_secs(20))
        .await;

    match interaction {
        Some(i) => {
            let chosen: usize = i.data.custom_id.trim_start_matches("trivia_").parse().unwrap_or(99);
            let correct = chosen == correct_idx;
            let prize: i64 = if correct { 250 } else { 0 };

            if correct {
                crate::db::add_coins(&ctx.data().db, guild_id, user_id, prize).await;
            }

            let desc = if correct {
                format!("✅ Richtig! **{}** ist korrekt!\n\n**+{} Coins**", answers[correct_idx], prize)
            } else {
                format!("❌ Falsch! Die richtige Antwort war **{}**.", answers[correct_idx])
            };

            let _ = i.create_response(
                ctx.http(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(
                            CreateEmbed::new()
                                .title("🧠 Trivia")
                                .description(format!("**{}**\n\n{}", question, desc))
                                .color(if correct { 0x57F287u32 } else { 0xED4245u32 }),
                        )
                        .components(vec![]),
                ),
            ).await;
        }
        None => {
            let _ = reply.edit(
                ctx,
                poise::CreateReply::default()
                    .embed(CreateEmbed::new()
                        .title("🧠 Trivia — Zeit abgelaufen!")
                        .description(format!("Die richtige Antwort war: **{}**", answers[correct_idx]))
                        .color(0xFEE75Cu32))
                    .components(vec![]),
            ).await;
        }
    }

    Ok(())
}

// ── tipprennen ────────────────────────────────────────────────────────────────

static RACE_TEXTS: &[&str] = &[
    "Der schnelle braune Fuchs springt über den faulen Hund",
    "Alle Menschen sind frei und gleich an Würde und Rechten geboren",
    "Im Herbst fallen die bunten Blätter von den Bäumen",
    "Der Morgen hat Gold im Mund sagt ein altes Sprichwort",
    "Wer rastet der rostet ist eine bekannte deutsche Redewendung",
    "Die Sonne scheint hell über den grünen Wiesen im Sommer",
    "Heute ist ein wunderschöner Tag für einen langen Spaziergang",
    "Übung macht den Meister besonders beim Tippen mit zehn Fingern",
];

/// Tipprennen: Tippe den Text als Erster ab und gewinne Coins
#[poise::command(slash_command, guild_only, rename = "tipprennen")]
pub async fn tipprennen(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let text = {
        let mut rng = rand::thread_rng();
        RACE_TEXTS[rng.gen_range(0..RACE_TEXTS.len())]
    };

    ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("⌨️ Tipprennen!")
                    .description(format!("Tippe exakt diesen Text (30 Sekunden):\n\n```{}```", text))
                    .color(0xFEE75Cu32),
            ),
    )
    .await?;

    let channel_id = ctx.channel_id();
    let start = std::time::Instant::now();

    let response = channel_id
        .await_reply(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(30))
        .filter(move |m| {
            let content = m.content.trim().to_lowercase();
            let target  = text.trim().to_lowercase();
            !m.author.bot && content == target
        })
        .await;

    match response {
        Some(msg) => {
            let elapsed = start.elapsed().as_secs_f64();
            let guild_id = ctx.guild_id().unwrap();
            let prize: i64 = if elapsed < 10.0 { 500 } else if elapsed < 20.0 { 300 } else { 150 };
            crate::db::add_coins(&ctx.data().db, guild_id, msg.author.id, prize).await;

            ctx.send(
                poise::CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("⌨️ Tipprennen — Gewonnen!")
                            .description(format!(
                                "<@{}> hat gewonnen in **{:.2}s**!\n\n**+{} Coins**",
                                msg.author.id, elapsed, prize
                            ))
                            .color(0x57F287u32),
                    ),
            )
            .await?;
        }
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("⌨️ Tipprennen — Zeit abgelaufen!")
                            .description("Niemand hat den Text rechtzeitig abgetippt.")
                            .color(0xED4245u32),
                    ),
            )
            .await?;
        }
    }

    Ok(())
}

// ── counting channel setup ────────────────────────────────────────────────────

/// Zählkanal einrichten (Nutzer zählen nacheinander hoch)
#[poise::command(slash_command, required_permissions = "MANAGE_GUILD", guild_only, rename = "zaehlen-setup")]
pub async fn zaehlen_setup(
    ctx: Context<'_>,
    #[description = "Kanal für das Zählspiel (leer = deaktivieren)"] kanal: Option<serenity::GuildChannel>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap();
    let ch       = kanal.as_ref().map(|c| c.id);

    crate::db::set_counting_channel(&ctx.data().db, guild_id, ch).await;

    let msg = match &kanal {
        Some(c) => format!("✅ Zählkanal auf <#{}> gesetzt.", c.id),
        None    => "✅ Zählkanal deaktiviert.".to_string(),
    };

    ctx.send(poise::CreateReply::default().embed(ok("Zählkanal", &msg)).ephemeral(true)).await?;
    Ok(())
}

// ── connect 4 ─────────────────────────────────────────────────────────────────

const C4_COLS: usize = 7;
const C4_ROWS: usize = 6;

fn c4_board_display(board: &[[u8; C4_COLS]; C4_ROWS]) -> String {
    let col_nums = "1️⃣2️⃣3️⃣4️⃣5️⃣6️⃣7️⃣";
    let rows: Vec<String> = board.iter().map(|row| {
        row.iter().map(|&cell| match cell {
            1 => "🔴",
            2 => "🔵",
            _ => "⚫",
        }).collect::<Vec<_>>().join("")
    }).collect();
    format!("{}\n{}", rows.join("\n"), col_nums)
}

fn c4_drop(board: &mut [[u8; C4_COLS]; C4_ROWS], col: usize, player: u8) -> Option<usize> {
    for row in (0..C4_ROWS).rev() {
        if board[row][col] == 0 {
            board[row][col] = player;
            return Some(row);
        }
    }
    None
}

fn c4_check_win(board: &[[u8; C4_COLS]; C4_ROWS], player: u8) -> bool {
    // Horizontal
    for r in 0..C4_ROWS {
        for c in 0..C4_COLS-3 {
            if (0..4).all(|i| board[r][c+i] == player) { return true; }
        }
    }
    // Vertical
    for r in 0..C4_ROWS-3 {
        for c in 0..C4_COLS {
            if (0..4).all(|i| board[r+i][c] == player) { return true; }
        }
    }
    // Diagonal ↘
    for r in 0..C4_ROWS-3 {
        for c in 0..C4_COLS-3 {
            if (0..4).all(|i| board[r+i][c+i] == player) { return true; }
        }
    }
    // Diagonal ↙
    for r in 0..C4_ROWS-3 {
        for c in 3..C4_COLS {
            if (0..4).all(|i| board[r+i][c-i] == player) { return true; }
        }
    }
    false
}

fn c4_buttons(disabled: bool) -> Vec<CreateActionRow> {
    let cols: Vec<CreateButton> = (0..C4_COLS).map(|i| {
        CreateButton::new(format!("c4_{}", i))
            .label((i+1).to_string())
            .style(ButtonStyle::Secondary)
            .disabled(disabled)
    }).collect();
    vec![CreateActionRow::Buttons(cols)]
}

/// 4 gewinnt gegen einen anderen Nutzer spielen
#[poise::command(slash_command, guild_only, rename = "viergewinnt")]
pub async fn viergewinnt(
    ctx: Context<'_>,
    #[description = "Gegner"] gegner: serenity::User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();

    if gegner.bot {
        ctx.send(poise::CreateReply::default().embed(err("Kein Bot-Spiel", "Bots können nicht spielen.")).ephemeral(true)).await?;
        return Ok(());
    }
    if gegner.id == ctx.author().id {
        ctx.send(poise::CreateReply::default().embed(err("Nein", "Du kannst nicht gegen dich selbst spielen.")).ephemeral(true)).await?;
        return Ok(());
    }

    let players = [ctx.author().id, gegner.id];
    let mut board = [[0u8; C4_COLS]; C4_ROWS];
    let mut current = 0usize;

    let reply = ctx.send(
        poise::CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("🔴🔵 4 Gewinnt")
                    .description(format!("{}\n\n<@{}> ist dran (🔴)", c4_board_display(&board), players[current]))
                    .color(0x5865F2u32),
            )
            .components(c4_buttons(false)),
    )
    .await?;

    let msg = reply.message().await?;

    loop {
        let Some(interaction) = msg
            .await_component_interaction(ctx.serenity_context())
            .author_id(players[current])
            .timeout(std::time::Duration::from_secs(60))
            .await
        else {
            let _ = reply.edit(ctx, poise::CreateReply::default()
                .embed(CreateEmbed::new()
                    .title("🔴🔵 4 Gewinnt — Abgebrochen")
                    .description(format!("<@{}> hat nicht rechtzeitig gespielt.", players[current]))
                    .color(0xFEE75Cu32))
                .components(c4_buttons(true))
            ).await;
            break;
        };

        let col: usize = interaction.data.custom_id.trim_start_matches("c4_").parse().unwrap_or(99);
        if col >= C4_COLS { continue; }

        let player_num = (current as u8) + 1;

        if c4_drop(&mut board, col, player_num).is_none() {
            // Column full, skip (interaction still needs response)
            let _ = interaction.create_response(ctx.http(), serenity::CreateInteractionResponse::Acknowledge).await;
            continue;
        }

        let emoji = if current == 0 { "🔴" } else { "🔵" };

        if c4_check_win(&board, player_num) {
            let prize: i64 = 500;
            crate::db::add_coins(&ctx.data().db, guild_id, players[current], prize).await;
            let _ = interaction.create_response(
                ctx.http(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(CreateEmbed::new()
                            .title("🔴🔵 4 Gewinnt")
                            .description(format!("{}\n\n{} <@{}> gewinnt! **+{} Coins**", c4_board_display(&board), emoji, players[current], prize))
                            .color(0x57F287u32))
                        .components(vec![])
                )
            ).await;
            break;
        }

        // Check draw
        let full = board[0].iter().all(|&c| c != 0);
        if full {
            let _ = interaction.create_response(
                ctx.http(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(CreateEmbed::new()
                            .title("🔴🔵 4 Gewinnt — Unentschieden!")
                            .description(c4_board_display(&board))
                            .color(0xFEE75Cu32))
                        .components(vec![])
                )
            ).await;
            break;
        }

        current = 1 - current;
        let next_emoji = if current == 0 { "🔴" } else { "🔵" };

        let _ = interaction.create_response(
            ctx.http(),
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(CreateEmbed::new()
                        .title("🔴🔵 4 Gewinnt")
                        .description(format!("{}\n\n{} <@{}> ist dran", c4_board_display(&board), next_emoji, players[current]))
                        .color(0x5865F2u32))
                    .components(c4_buttons(false))
            )
        ).await;
    }

    Ok(())
}

// ── tictactoe ─────────────────────────────────────────────────────────────────

fn ttt_display(board: &[u8; 9]) -> String {
    let cells: Vec<&str> = board.iter().map(|&c| match c {
        1 => "❌",
        2 => "⭕",
        _ => "⬜",
    }).collect();
    format!("{}{}{}\n{}{}{}\n{}{}{}",
        cells[0], cells[1], cells[2],
        cells[3], cells[4], cells[5],
        cells[6], cells[7], cells[8],
    )
}

fn ttt_buttons(board: &[u8; 9]) -> Vec<CreateActionRow> {
    let rows: Vec<CreateActionRow> = (0..3).map(|r| {
        let btns: Vec<CreateButton> = (0..3).map(|c| {
            let idx = r * 3 + c;
            let label = match board[idx] {
                1 => "❌".to_string(),
                2 => "⭕".to_string(),
                _ => format!("{}", idx + 1),
            };
            CreateButton::new(format!("ttt_{}", idx))
                .label(label)
                .style(if board[idx] == 0 { ButtonStyle::Secondary } else { ButtonStyle::Primary })
                .disabled(board[idx] != 0)
        }).collect();
        CreateActionRow::Buttons(btns)
    }).collect();
    rows
}

fn ttt_check_win(board: &[u8; 9], player: u8) -> bool {
    let wins = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];
    wins.iter().any(|w| w.iter().all(|&i| board[i] == player))
}

/// TicTacToe gegen einen anderen Nutzer spielen
#[poise::command(slash_command, guild_only)]
pub async fn tictactoe(
    ctx: Context<'_>,
    #[description = "Gegner"] gegner: serenity::User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();

    if gegner.bot || gegner.id == ctx.author().id {
        ctx.send(poise::CreateReply::default().embed(err("Ungültig", "Kein gültiger Gegner.")).ephemeral(true)).await?;
        return Ok(());
    }

    let players = [ctx.author().id, gegner.id];
    let mut board = [0u8; 9];
    let mut current = 0usize;

    let reply = ctx.send(
        poise::CreateReply::default()
            .embed(CreateEmbed::new()
                .title("❌⭕ TicTacToe")
                .description(format!("{}\n\n❌ <@{}> ist dran", ttt_display(&board), players[0]))
                .color(0x5865F2u32))
            .components(ttt_buttons(&board)),
    )
    .await?;

    let msg = reply.message().await?;

    loop {
        let Some(interaction) = msg
            .await_component_interaction(ctx.serenity_context())
            .author_id(players[current])
            .timeout(std::time::Duration::from_secs(60))
            .await
        else {
            let _ = reply.edit(ctx, poise::CreateReply::default()
                .embed(CreateEmbed::new()
                    .title("❌⭕ TicTacToe — Abgebrochen")
                    .description(format!("<@{}> hat nicht rechtzeitig gespielt.", players[current]))
                    .color(0xFEE75Cu32))
                .components(vec![])
            ).await;
            break;
        };

        let idx: usize = interaction.data.custom_id.trim_start_matches("ttt_").parse().unwrap_or(99);
        if idx >= 9 || board[idx] != 0 { continue; }

        let player_num = (current as u8) + 1;
        board[idx] = player_num;

        let emoji = if current == 0 { "❌" } else { "⭕" };

        if ttt_check_win(&board, player_num) {
            let prize: i64 = 300;
            crate::db::add_coins(&ctx.data().db, guild_id, players[current], prize).await;
            let _ = interaction.create_response(ctx.http(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(CreateEmbed::new()
                            .title("❌⭕ TicTacToe")
                            .description(format!("{}\n\n{} <@{}> gewinnt! **+{} Coins**", ttt_display(&board), emoji, players[current], prize))
                            .color(0x57F287u32))
                        .components(vec![])
                )
            ).await;
            break;
        }

        if board.iter().all(|&c| c != 0) {
            let _ = interaction.create_response(ctx.http(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(CreateEmbed::new()
                            .title("❌⭕ TicTacToe — Unentschieden!")
                            .description(ttt_display(&board))
                            .color(0xFEE75Cu32))
                        .components(vec![])
                )
            ).await;
            break;
        }

        current = 1 - current;
        let next_emoji = if current == 0 { "❌" } else { "⭕" };

        let _ = interaction.create_response(ctx.http(),
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(CreateEmbed::new()
                        .title("❌⭕ TicTacToe")
                        .description(format!("{}\n\n{} <@{}> ist dran", ttt_display(&board), next_emoji, players[current]))
                        .color(0x5865F2u32))
                    .components(ttt_buttons(&board))
            )
        ).await;
    }

    Ok(())
}
