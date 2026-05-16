mod de;
mod en;

use std::sync::OnceLock;

static LANG: OnceLock<&'static Strings> = OnceLock::new();

pub fn lang() -> &'static Strings {
    LANG.get_or_init(|| {
        match std::env::var("BOT_LANG").as_deref().unwrap_or("de") {
            "en" => &en::EN,
            _    => &de::DE,
        }
    })
}

// ── String table ─────────────────────────────────────────────────────────────

pub struct Strings {
    // ── Economy / Work ───────────────────────────────────────────────────────
    pub work_tier_names: [&'static str; 4],
    pub work_tier_beginner_jobs: &'static [(&'static str, &'static str)],
    pub work_tier_skilled_jobs:  &'static [(&'static str, &'static str)],
    pub work_tier_expert_jobs:   &'static [(&'static str, &'static str)],
    pub work_tier_elite_jobs:    &'static [(&'static str, &'static str)],
    /// Placeholders: {mins} {secs}
    pub work_exhausted: &'static str,
    pub work_result_field: &'static str,
    pub work_xp_field: &'static str,
    pub work_levelup_field: &'static str,
    /// Placeholders: {old} {new} {coins}
    pub work_levelup_desc: &'static str,
    pub work_footer: &'static str,
    pub work_remind_btn: &'static str,

    // ── Economy / Steal ──────────────────────────────────────────────────────
    pub steal_no_bot: &'static str,
    pub steal_no_self: &'static str,
    /// Placeholders: {mins} {secs}
    pub steal_cooldown: &'static str,
    pub steal_protection_title: &'static str,
    /// Placeholders: {victim}
    pub steal_protection_desc: &'static str,
    pub steal_caught_title: &'static str,
    pub steal_success_title: &'static str,
    pub steal_daytime_night: &'static str,
    pub steal_daytime_morning: &'static str,
    pub steal_daytime_day: &'static str,
    /// Placeholders per entry: {target} {coins}
    pub steal_success_templates: &'static [&'static str],
    pub steal_caught_templates:  &'static [&'static str],
    pub steal_balance_field: &'static str,
    pub steal_daytime_field: &'static str,

    // ── Economy / Bank Robbery ───────────────────────────────────────────────
    pub bank_empty_title: &'static str,
    pub bank_empty_desc: &'static str,
    /// Placeholders: {hours} {mins}
    pub bank_cooldown: &'static str,
    pub bank_caught_title: &'static str,
    pub bank_success_title: &'static str,
    pub bank_daytime_night: &'static str,
    pub bank_daytime_morning: &'static str,
    pub bank_daytime_day: &'static str,
    /// Placeholders per entry: {coins}
    pub bank_success_templates: &'static [&'static str],
    /// Placeholders per entry: {hours}
    pub bank_caught_templates: &'static [&'static str],
    /// Placeholders: {fine}
    pub bank_fine_field: &'static str,
    pub bank_balance_field: &'static str,
    pub bank_release_field: &'static str,
    /// Placeholders: {coins}
    pub bank_loot_field: &'static str,
    pub bank_new_balance_field: &'static str,
    /// Placeholders: {hours}
    pub bank_jail_hours_field: &'static str,
    pub bank_daytime_label: &'static str,

    // ── Economy / Jail ───────────────────────────────────────────────────────
    pub jail_title: &'static str,
    /// Placeholders: {hours} {mins}
    pub jail_desc: &'static str,

    // ── Economy / Coins command ──────────────────────────────────────────────
    pub coins_bot_invalid: &'static str,
    pub coins_balance_field: &'static str,
    pub coins_invites_field: &'static str,
    pub coins_no_data: &'static str,
    pub coins_lb_title: &'static str,
    /// Placeholders: {guild}
    pub coins_lb_footer: &'static str,

    // ── Economy / Transfer ───────────────────────────────────────────────────
    pub transfer_bot_invalid: &'static str,
    pub transfer_self_invalid: &'static str,
    pub transfer_min_amount: &'static str,
    /// Placeholders: {have} {need}
    pub transfer_not_enough: &'static str,
    pub transfer_success_title: &'static str,
    /// Placeholders: {amount} {recipient}
    pub transfer_success_desc: &'static str,
    pub transfer_sender_balance_field: &'static str,
    /// Placeholders: {name}
    pub transfer_recipient_balance_field: &'static str,

    // ── Fishing ──────────────────────────────────────────────────────────────
    pub fish_cooldown_title: &'static str,
    /// Placeholders: {mins} {secs}
    pub fish_cooldown_desc: &'static str,
    pub fish_trash_title: &'static str,
    /// Placeholders: {emoji} {name}
    pub fish_trash_desc: &'static str,
    pub fish_legendary_title: &'static str,
    /// Placeholders: {emoji} {name} {price}
    pub fish_legendary_desc: &'static str,
    pub fish_normal_title: &'static str,
    /// Placeholders: {emoji} {name} {price}
    pub fish_normal_desc: &'static str,
    /// Placeholders: {emoji} {name}
    pub fish_footer: &'static str,
    pub fish_remind_btn: &'static str,
    pub fish_inv_empty_title: &'static str,
    pub fish_inv_empty_desc: &'static str,
    pub fish_inv_title: &'static str,
    /// Placeholders: {page} {total} {count}
    pub fish_inv_page_header: &'static str,
    /// Placeholders: {price}
    pub fish_inv_sell_btn: &'static str,
    pub fish_inv_prev_btn: &'static str,
    pub fish_inv_next_btn: &'static str,
    /// Placeholders: {emoji} {name} {price}
    pub fish_inv_sold_line: &'static str,
    pub fish_inv_empty_now: &'static str,
    pub fish_inv_footer: &'static str,
    pub fish_sell_all_empty_title: &'static str,
    pub fish_sell_all_empty_desc: &'static str,
    pub fish_sell_all_title: &'static str,
    pub fish_sell_all_total_field: &'static str,
    /// Placeholders: {count}
    pub fish_sell_all_more: &'static str,
    pub fish_market_title: &'static str,
    pub fish_market_footer: &'static str,
    pub rod_shop_title: &'static str,
    pub rod_shop_footer: &'static str,
    pub rod_shop_status_active: &'static str,
    pub rod_shop_status_free: &'static str,
    pub rod_shop_status_buyable: &'static str,
    pub rod_price_free: &'static str,
    /// Placeholders: {price}
    pub rod_price_coins: &'static str,
    pub rod_already_owned_title: &'static str,
    /// Placeholders: {name}
    pub rod_already_owned_desc: &'static str,
    pub rod_no_downgrade_title: &'static str,
    pub rod_no_downgrade_desc: &'static str,
    pub rod_not_enough_title: &'static str,
    /// Placeholders: {price} {name} {balance}
    pub rod_not_enough_desc: &'static str,
    pub rod_bought_title: &'static str,
    /// Placeholders: {emoji} {name} {price} {desc}
    pub rod_bought_desc: &'static str,
    // Fish display names (keyed by fish id)
    pub fish_name_muell: &'static str,
    pub fish_name_hering: &'static str,
    pub fish_name_forelle: &'static str,
    pub fish_name_barsch: &'static str,
    pub fish_name_hecht: &'static str,
    pub fish_name_goldfisch: &'static str,
    pub fish_name_quantenbarsch: &'static str,
    // Rod display names and descriptions (keyed by rod id)
    pub rod_name_grundangel: &'static str,
    pub rod_name_profiangel: &'static str,
    pub rod_name_meeresangel: &'static str,
    pub rod_name_quantenangel: &'static str,
    pub rod_desc_grundangel: &'static str,
    pub rod_desc_profiangel: &'static str,
    pub rod_desc_meeresangel: &'static str,
    pub rod_desc_quantenangel: &'static str,

    // ── Shop ─────────────────────────────────────────────────────────────────
    pub shop_title: &'static str,
    /// Placeholders: {balance}
    pub shop_footer: &'static str,
    /// Placeholders: {n}
    pub shop_owned_qty: &'static str,
    pub shop_not_enough_label: &'static str,
    /// Placeholders: {have} {need}
    pub shop_not_enough_desc: &'static str,
    /// Placeholders: {emoji} {name}
    pub shop_bought_title: &'static str,
    pub shop_paid_field: &'static str,
    pub shop_new_balance_field: &'static str,
    pub shop_xp_booster_name: &'static str,
    pub shop_xp_booster_desc: &'static str,
    pub shop_angelkoder_name: &'static str,
    pub shop_angelkoder_desc: &'static str,
    pub shop_diebstahlschutz_name: &'static str,
    pub shop_diebstahlschutz_desc: &'static str,
    pub shop_doppelgehalt_name: &'static str,
    pub shop_doppelgehalt_desc: &'static str,
    pub shop_lotto_rabatt_name: &'static str,
    pub shop_lotto_rabatt_desc: &'static str,

    // ── Prestige ─────────────────────────────────────────────────────────────
    pub prestige_not_ready_label: &'static str,
    /// Placeholders: {level}
    pub prestige_not_ready_desc: &'static str,
    pub prestige_title: &'static str,
    /// Placeholders: {user} {prestige} {stars}
    pub prestige_desc: &'static str,
    pub prestige_rank_field: &'static str,
    pub prestige_xp_reset_field: &'static str,
    pub prestige_xp_reset_value: &'static str,

    // ── Daily Salary ─────────────────────────────────────────────────────────
    pub salary_title: &'static str,
    /// Placeholders: {count} {total}
    pub salary_desc: &'static str,
    pub salary_footer: &'static str,

    // ── Loot Drop ────────────────────────────────────────────────────────────
    pub loot_tier_common: &'static str,
    pub loot_tier_rare: &'static str,
    pub loot_tier_legendary: &'static str,
    /// Placeholders: {fish} {coins}
    pub loot_desc: &'static str,
    /// Appended when bonus_xp > 0. Placeholders: {xp}
    pub loot_desc_xp: &'static str,
    pub loot_claim_btn: &'static str,
    pub loot_footer: &'static str,
    pub loot_claimed_title: &'static str,
    /// Placeholders: {user} {fish} {coins}
    pub loot_claimed_desc: &'static str,
    /// Appended when bonus_xp > 0. Placeholders: {xp}
    pub loot_claimed_desc_xp: &'static str,
    pub loot_already_claimed: &'static str,
    pub loot_no_guild: &'static str,

    // ── Casino ───────────────────────────────────────────────────────────────
    /// Placeholders: {channel}
    pub casino_wrong_channel: &'static str,
    pub casino_invalid_bet_title: &'static str,
    /// Placeholders: {min} {max}
    pub casino_invalid_bet_desc: &'static str,
    pub casino_not_enough_title: &'static str,
    /// Placeholders: {have} {need}
    pub casino_not_enough_desc: &'static str,
    pub casino_daily_limit_title: &'static str,
    /// Placeholders: {lost} {limit}
    pub casino_daily_limit_desc: &'static str,
    pub casino_consolation_note: &'static str,
    pub casino_streak_note: &'static str,
    pub casino_slots_two_match: &'static str,
    pub casino_slots_no_match: &'static str,
    pub casino_bj_prompt: &'static str,
    pub casino_bj_double_auto: &'static str,
    pub casino_bj_more: &'static str,
    pub casino_bj_stand: &'static str,
    pub casino_bj_dealer_bust: &'static str,
    pub casino_bj_win: &'static str,
    pub casino_bj_push: &'static str,
    pub casino_bj_loss: &'static str,
    pub casino_dice_title: &'static str,
    pub casino_coinflip_title: &'static str,
    pub casino_coinflip_heads: &'static str,
    pub casino_coinflip_tails: &'static str,
    pub casino_roulette_title: &'static str,
    pub casino_roulette_invalid_title: &'static str,
    pub casino_roulette_invalid_desc: &'static str,
    pub casino_hol_title: &'static str,
    pub casino_hol_prompt: &'static str,
    pub casino_hol_higher_btn: &'static str,
    pub casino_hol_lower_btn: &'static str,
    pub casino_hol_cash_btn: &'static str,
    pub casino_hol_cash_title: &'static str,
    pub casino_hol_wrong_title: &'static str,
    /// Placeholders: {card} {bet}
    pub casino_hol_wrong_desc: &'static str,
    pub casino_hol_max_title: &'static str,
    /// Placeholders: {card}
    pub casino_hol_correct_fmt: &'static str,
    /// Placeholders: {card}
    pub casino_hol_max_desc: &'static str,
    /// Placeholders: {n} {total} {have}
    pub casino_lotto_not_enough: &'static str,
    pub casino_lotto_bought_title: &'static str,
    /// Placeholders: {n} {total} {tickets} {jackpot}
    pub casino_lotto_bought_desc: &'static str,
    pub casino_lotto_footer: &'static str,
    pub casino_lotto_draw_title: &'static str,
    pub casino_lotto_winning_prefix: &'static str,
    pub casino_lotto_no_jackpot: &'static str,
    /// Placeholders: {mentions} {share}
    pub casino_lotto_jackpot_winner: &'static str,
    /// Placeholders: {mentions}
    pub casino_lotto_5hits: &'static str,
    pub casino_lotto_4hits: &'static str,
    pub casino_lotto_3hits: &'static str,
    pub casino_lotto_no_winners: &'static str,
    /// Placeholders: {name}
    pub casino_stats_title: &'static str,
    pub casino_stats_wagered: &'static str,
    pub casino_stats_won: &'static str,
    pub casino_stats_lost: &'static str,
    pub casino_stats_net: &'static str,
    pub casino_stats_biggest: &'static str,
    pub casino_stats_games: &'static str,
    pub casino_stats_win_streak: &'static str,
    pub casino_stats_lose_streak: &'static str,
    pub casino_rangliste_title: &'static str,
    pub casino_rangliste_empty: &'static str,
    pub casino_setup_title: &'static str,
    /// Placeholders: {channel}
    pub casino_setup_set: &'static str,
    pub casino_setup_everywhere: &'static str,
    pub casino_tresor_title: &'static str,
    /// Placeholders: {balance}
    pub casino_tresor_desc: &'static str,
    pub casino_limit_title: &'static str,
    pub casino_limit_none: &'static str,
    /// Placeholders: {limit}
    pub casino_limit_set: &'static str,
    pub casino_jackpot_title: &'static str,
    /// Placeholders: {added} {new}
    pub casino_jackpot_desc: &'static str,

    // ── Levels ───────────────────────────────────────────────────────────────
    pub level_no_bot: &'static str,
    pub level_rank_field: &'static str,
    pub level_level_field: &'static str,
    pub level_xp_field: &'static str,
    pub level_progress_field: &'static str,
    /// Placeholders: {remaining} {next}
    pub level_footer: &'static str,
    pub level_footer_max: &'static str,
    pub leaderboard_no_data: &'static str,
    pub leaderboard_title: &'static str,
    /// Placeholders: {guild}
    pub leaderboard_footer: &'static str,
    /// Placeholders: {count}
    pub scan_start: &'static str,
    /// Placeholders: {messages} {users}
    pub scan_done: &'static str,
    /// Placeholders: {user} {level} {coins}
    pub level_up_msg: &'static str,
    /// Placeholders: {xp}
    pub level_up_footer_next: &'static str,
    pub level_up_footer_max: &'static str,
    /// Placeholders: {user}
    pub reset_xp_removed: &'static str,
    pub reset_xp_none: &'static str,
    pub migrate_title: &'static str,
    /// Placeholders: {users} {coins}
    pub migrate_desc: &'static str,

    // ── Main / channel guard ─────────────────────────────────────────────────
    /// Placeholders: {channel}
    pub wrong_channel: &'static str,
}

// ── Helper methods ────────────────────────────────────────────────────────────

impl Strings {
    pub fn work_tier(&self, idx: usize) -> (&'static str, &'static [(&'static str, &'static str)]) {
        match idx {
            0 => (self.work_tier_names[0], self.work_tier_beginner_jobs),
            1 => (self.work_tier_names[1], self.work_tier_skilled_jobs),
            2 => (self.work_tier_names[2], self.work_tier_expert_jobs),
            _ => (self.work_tier_names[3], self.work_tier_elite_jobs),
        }
    }

    pub fn fish_display_name(&self, id: &str) -> &'static str {
        match id {
            "muell"         => self.fish_name_muell,
            "hering"        => self.fish_name_hering,
            "forelle"       => self.fish_name_forelle,
            "barsch"        => self.fish_name_barsch,
            "hecht"         => self.fish_name_hecht,
            "goldfisch"     => self.fish_name_goldfisch,
            "quantenbarsch" => self.fish_name_quantenbarsch,
            _               => "",
        }
    }

    pub fn rod_display_name(&self, id: &str) -> &'static str {
        match id {
            "grundangel"   => self.rod_name_grundangel,
            "profiangel"   => self.rod_name_profiangel,
            "meeresangel"  => self.rod_name_meeresangel,
            "quantenangel" => self.rod_name_quantenangel,
            _              => "",
        }
    }

    pub fn rod_display_desc(&self, id: &str) -> &'static str {
        match id {
            "grundangel"   => self.rod_desc_grundangel,
            "profiangel"   => self.rod_desc_profiangel,
            "meeresangel"  => self.rod_desc_meeresangel,
            "quantenangel" => self.rod_desc_quantenangel,
            _              => "",
        }
    }

    pub fn shop_item_name(&self, id: &str) -> &'static str {
        match id {
            "xp_booster"      => self.shop_xp_booster_name,
            "angelkoder"      => self.shop_angelkoder_name,
            "diebstahlschutz" => self.shop_diebstahlschutz_name,
            "doppelgehalt"    => self.shop_doppelgehalt_name,
            "lotto_rabatt"    => self.shop_lotto_rabatt_name,
            _                 => "",
        }
    }

    pub fn shop_item_desc(&self, id: &str) -> &'static str {
        match id {
            "xp_booster"      => self.shop_xp_booster_desc,
            "angelkoder"      => self.shop_angelkoder_desc,
            "diebstahlschutz" => self.shop_diebstahlschutz_desc,
            "doppelgehalt"    => self.shop_doppelgehalt_desc,
            "lotto_rabatt"    => self.shop_lotto_rabatt_desc,
            _                 => "",
        }
    }
}
