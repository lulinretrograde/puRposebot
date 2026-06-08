# puRposebot

A feature-rich Discord bot written in Rust with economy, fishing, casino, leveling, moderation, and anti-nuke systems. Originally built for a German-speaking server, but fully localizable via a single environment variable.

## Features

| System | Commands |
|---|---|
| **Economy** | `/arbeit` (work), `/klauen` (steal), `/bankueberfall` (bank heist), `/coins`, `/ueberweisung` (transfer) |
| **Fishing** | `/angeln` (fish), `/inventar` (inventory), `/fischmarkt` (market prices), `/angelshop` (rod shop) |
| **Casino** | `/slots`, `/blackjack`, `/wuerfeln` (dice), `/muenzwurf` (coin flip), `/roulette`, `/kartenspiel` (higher/lower), `/lotto` |
| **Shop** | `/laden` (shop), `/kaufen` (buy), `/prestige` |
| **Levels** | `/level`, `/leaderboard`, XP from messages and voice |
| **Moderation** | `/ban`, `/kick`, `/mute`, `/warn`, `/jail`, `/unjail`, tickets, bug reports |
| **Anti-Nuke** | Automatic protection against mass bans, kicks, channel deletions, and raids |
| **Giveaways** | `/giveaway` - button-based with winner selection |
| **Automation** | Daily salary, loot drops every 30 minutes, lotto draw at midnight UTC |

## Deployment

### Requirements

- Rust (stable, 2021 edition)
- SQLite
- A Discord bot token with the following intents: `GUILDS`, `GUILD_MEMBERS`, `GUILD_MESSAGES`, `GUILD_VOICE_STATES`, `MESSAGE_CONTENT`

### Quick Start

```bash
git clone <repo>
cd fuckasskackbot

# Create .env
echo 'DISCORD_TOKEN=your_token_here' > .env
echo 'BOT_LANG=en' >> .env   # or 'de' for German (default)

cargo build --release
./target/release/fuckasskackbot
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DISCORD_TOKEN` | (required) | Your Discord bot token |
| `BOT_LANG` | `de` | Bot language: `de` (German) or `en` (English) |
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

### Running with PM2

```bash
pm2 start ./target/release/fuckasskackbot --name fuckasskackbot
pm2 save
pm2 startup
```

After rebuilding, restart with:
```bash
pm2 restart fuckasskackbot
```

### First-Time Setup

After the bot is running, use `/setup` in your server to configure:
- Bot channel (where commands are allowed)
- Log channel (for audit events)
- Mod log channel
- Welcome channel and message
- Role assignments

## Changing the Language

Set `BOT_LANG=en` in your environment before starting the bot. All user-facing responses, button labels, embed titles, and flavor text will switch to English.

```bash
# .env file
BOT_LANG=en
```

**Note:** Slash command *names* (like `/arbeit`, `/klauen`, `/wuerfeln`) and option choice labels are compiled into the binary and registered with Discord at startup. These are currently German regardless of `BOT_LANG`. If you want fully English command names, rename the commands in `src/commands/` and update the `rename = "..."` attributes, then re-register with `/resync` or by restarting the bot.

## Adding a New Language

The entire string table lives in `src/lang/`. Adding a new language takes three steps:

### 1. Create `src/lang/xx.rs`

Copy `src/lang/en.rs` and translate every field value. The struct fields and keys must stay identical, only the string values change.

```rust
// src/lang/xx.rs
use super::Strings;

pub static XX: Strings = Strings {
    wrong_channel: "❌ This command is only in <#{channel}>.",
    work_exhausted: "⏳ Tired! **{mins}m {secs}s** left.",
    // ... all other fields
};
```

Placeholders like `{mins}`, `{user}`, `{coins}` are replaced at runtime, keep them exactly as-is.

### 2. Register the language in `src/lang/mod.rs`

Add your module and a match arm:

```rust
mod xx;  // add this line at the top

pub fn lang() -> &'static Strings {
    LANG.get_or_init(|| {
        match std::env::var("BOT_LANG").as_deref().unwrap_or("de") {
            "en" => &en::EN,
            "xx" => &xx::XX,   // add this line
            _    => &de::DE,
        }
    })
}
```

### 3. Set `BOT_LANG=xx` and rebuild

```bash
BOT_LANG=xx cargo build --release
pm2 restart fuckasskackbot
```

## Project Structure

```
src/
  lang/
    mod.rs      ← Strings struct definition, language loader
    de.rs       ← German strings (default)
    en.rs       ← English strings
  commands/
    economy.rs  ← /arbeit, /klauen, /bankueberfall, /coins, /ueberweisung
    fishing.rs  ← /angeln, /inventar, /fischmarkt, /angelshop, /rute-kaufen
    casino.rs   ← /slots, /blackjack, /wuerfeln, /muenzwurf, /roulette, /kartenspiel, /lotto
    shop.rs     ← /laden, /kaufen, /prestige + loot drop + daily salary
    levels.rs   ← /level, /leaderboard, /scan-xp, /reset-xp
    moderation.rs
    antinuke.rs
    giveaway.rs
    setup.rs
    welcome.rs
    utility.rs
  antinuke.rs   ← Anti-nuke enforcement logic
  events.rs     ← Discord event handlers (XP, voice, logging, tickets)
  db.rs         ← SQLite queries via sqlx
  xp.rs         ← XP/level math
  config.rs     ← Shared state types
  main.rs       ← Bot setup, command registration, channel guard
```

## Database

The bot uses SQLite (`fuckasskackbot.db` in the working directory). The schema is created automatically on first startup via `sqlx` migrations embedded in `db.rs`. No manual setup needed.

## Building

```bash
cargo build --release          # production binary
cargo build                    # debug binary (faster compile, slower runtime)
```

The binary is at `target/release/fuckasskackbot`.
