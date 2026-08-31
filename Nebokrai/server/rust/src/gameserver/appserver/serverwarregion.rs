//! GameServer-владелец общего war-region `CServerWarRegion`.
//!
//! Contender lifecycle `OnEnterContend` RVA `0x001D26F0`, `DecContendTime`
//! `0x001D25F0`, `CancelContendByPlayerID` `0x001D2C40`, `AddContend`
//! `0x001D2D80`, `AI` `0x001D31A0`, `SetFacWinSymbol` `0x001D33A0`,
//! `CancelContendBySymbolID` `0x001D3460` и `OnContendTimeOver` `0x001D3820`
//! имеет статус `IMPLEMENTED, VERIFIED_DISASSEMBLY`. Decoder
//! `DecordFromByteArray` RVA `0x001D3110`, `UpdateContendPlayer` `0x001D3530`,
//! phase callbacks и clear/reset также `IMPLEMENTED`. Исходники
//! `serverwarregion.h/.cpp`, точная пара GameServer. PDB подтверждает `tagContend`
//! размером `0x34`, ordered `m_listContend +0x24C`, `m_FacWinSymbol +0x258` и
//! три signed counters `+0x264..+0x26C`.
//!
//! `Vec` и `BTreeMap` сохраняют list/map order. Статический scratch-list из AI
//! заменён локальным `Vec`: он по-прежнему собирает все завершения до callbacks
//! и не переносит недоказанную общую гонку MSVC-runtime. `DWORD`-время и signed
//! умножения сохраняют wrapping. Единственный неопределённый x86-край
//! `INT_MIN / -1` не получает придуманной реакции и возвращает локальный
//! `BLOCKED_MISSING_FACT`. Player/message/string-table owners остаются узким
//! context-контрактом; city weekly membership продолжает возвращать `Result`.
//! Base AI также возвращает typed monster-spawn block: при нём weather и
//! contender tail не выполняются.
//! `OnPlayerDamage` сохраняет исходную f32/x87 цепочку
//! `max_time * (damage / max_hp * fDecTimeParam)`, signed clamp и публикацию
//! процента. NaN/inf/out-of-range `fistp` остаются явным локальным блоком.
//! Конструктор `0x001D3740` создаёт base-region, пустые ordered contender/map
//! и нулевые counters; `Default`, `Vec` и `BTreeMap` выражают это буквально.
//! Деструктор `0x001D3060` освобождает map и base-object, что безопасно и без
//! дополнительной семантики выполняют автоматические `Drop` полей.
//! Decoder сначала делегирует сырому `CServerRegion` через узкий context, затем
//! читает три signed little-endian DWORD и обновляет только keys `0..total`:
//! старые map-keys за новым total оригинал не очищает. Безразмерный legacy read
//! за payload остаётся typed `BLOCKED_MISSING_FACT`.

use std::collections::BTreeMap;

use super::legacycodec::LegacyReader;
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::servercountryregion::is_player_contend_symbol;
use super::serverregion::{
    CServerRegion, ServerRegionDecodeContext, ServerRegionDecodeError, ServerRegionMonsterRectBlock,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ContendState {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) current_time: i32,
    pub(crate) max_time: i32,
    pub(crate) start_time_ms: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ContendPlayerState {
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) union_id: i32,
    pub(crate) country: u8,
    pub(crate) faction_name: String,
    pub(crate) shape_type: i32,
    pub(crate) is_dead: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SymbolCaptureLog<'a> {
    pub(crate) war_number: i32,
    pub(crate) owned_faction_id: i32,
    pub(crate) owned_union_id: i32,
    pub(crate) faction_name: &'a str,
    pub(crate) faction_id: i32,
    pub(crate) player_id: i32,
    pub(crate) symbol_id: i32,
    pub(crate) union_id: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WarRegionOwnership {
    pub(crate) faction_id: i32,
    pub(crate) union_id: i32,
}

/// BLOCKED_MISSING_FACT: exact EXE выполняет signed `idiv`; реакция процесса
/// на единственную пару `INT_MIN / -1` не доказана текущим owner-ом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContendArithmeticBlock {
    pub(crate) current_time: i32,
    pub(crate) max_time: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WarDamagePlayer {
    pub(crate) player_id: i32,
    pub(crate) max_hp: u32,
}

/// BLOCKED_MISSING_FACT: для NaN/inf/out-of-range x87 `fistp i32` точная
/// реакция процесса не доказана; safe Rust не назначает ей saturating cast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WarDamageArithmeticBlock {
    pub(crate) max_time: i32,
    pub(crate) damage: i32,
    pub(crate) max_hp: u32,
    pub(crate) dec_time_param_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WarDamageError {
    X87(WarDamageArithmeticBlock),
    Percentage(ContendArithmeticBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ContendAiError<MembershipError> {
    Base(ServerRegionMonsterRectBlock),
    Arithmetic(ContendArithmeticBlock),
    Membership(MembershipError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionDecodeInputBlock {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    LegacyStringOverflow {
        field: &'static str,
        first_out_of_bounds_offset: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WarRegionDecodeError<BaseError> {
    Base(BaseError),
    Input(RegionDecodeInputBlock),
}

pub(crate) trait WarRegionDecodeContext: ServerRegionDecodeContext {}

impl<Context: ServerRegionDecodeContext + ?Sized> WarRegionDecodeContext for Context {}

pub(crate) trait WarRegionContext {
    type MembershipError;

    /// Возвращает текущую faction найденного player; `None` сохраняет contender.
    fn player_faction_id(&mut self, player_id: i32) -> Option<i32>;

    /// Может остановить обновление на локальной неизвестной schedule-owner-а.
    fn is_apply_war_faction(&mut self, faction_id: i32) -> Result<bool, Self::MembershipError>;

    /// Шлёт player-у `0xBFF29` с одним signed значением времени.
    fn send_contend_time(&mut self, player_id: i32, time: i32);

    /// Меняет contend-state player из глобального `s_mapPlayer`, если он существует.
    fn set_global_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Меняет state только у region-local player, найденного через `FindChildObject`.
    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool);
}

pub(crate) trait WarRegionClearContext {
    /// Узкий exact tail `ClearRegion`: schedule membership и global lookup
    /// здесь не вызываются, нужны только reset wire и region-local state.
    fn send_contend_time(&mut self, player_id: i32, time: i32);
    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool);
}

impl<Context: WarRegionContext + ?Sized> WarRegionClearContext for Context {
    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        WarRegionContext::send_contend_time(self, player_id, time);
    }

    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool) {
        WarRegionContext::set_region_player_contend_state(self, region_id, player_id, state);
    }
}

/// Узкий context входа в захват. Script caller не обязан подменять заглушками
/// AI/victory effects, которые `OnEnterContend` никогда не вызывает.
pub(crate) trait WarContendEntryContext: WarRegionContext {
    /// Возвращает младшие 32 бита монотонного миллисекундного счётчика.
    fn now_millis(&mut self) -> u32;

    /// Вызывает concrete City/Village `IsOwner` для текущего региона.
    fn is_owner(&mut self, faction_id: i32) -> bool;

    fn player_has_good(&mut self, player_id: i32, good_name: &str) -> bool;

    /// Меняет contend-state у уже известного non-null player pointer.
    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Шлёт player-localized строку без форматных аргументов.
    fn notify_player(&mut self, player_id: i32, string_id: &'static str);

    /// Concrete virtual slot `+0x118`: Village сохраняет строку, City — no-op.
    fn register_needed_good(&mut self, good_name: &str);

    /// Шлёт region `GS0246(country-name, faction-name, symbol-name)`.
    fn send_first_faction_contender_notice(
        &mut self,
        country: u8,
        faction_name: &str,
        symbol_name: &str,
    );
}

pub(crate) trait WarContendContext: WarContendEntryContext {
    /// Выполняет исходный `CServerRegion::AI` до contender-tick.
    fn run_base_region_ai(
        &mut self,
        region: &mut CServerRegion,
    ) -> Result<(), ServerRegionMonsterRectBlock>;

    /// Имитирует lookup в глобальном `s_mapPlayer`, возвращая стабильный снимок.
    fn find_global_player(&mut self, player_id: i32) -> Option<ContendPlayerState>;

    fn on_faction_win_one_symbol(&mut self, faction_id: i32, symbol_id: i32);
    /// Выполняет concrete victory callback и возвращает owner-state после него.
    fn on_faction_victory(
        &mut self,
        faction_id: i32,
        union_id: i32,
        current_owner: WarRegionOwnership,
    ) -> WarRegionOwnership;

    /// Шлёт region `0xBF806(-1, 0xFFFF0000, GS0241)`.
    fn send_symbol_captured_region_notice(&mut self);

    /// Формирует `GS0227(faction-name, symbol-name)` и маршрут `(-1, 0, 1, 1)`.
    fn send_symbol_captured_top_info(&mut self, faction_name: &str, symbol_name: &str);

    /// Пишет exact header и `GS0242` в канал `war` с полями в исходном порядке.
    fn write_symbol_capture_logs(&mut self, capture: SymbolCaptureLog<'_>);
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerWarRegion {
    pub(crate) base: CServerRegion,
    pub(crate) contenders: Vec<ContendState>,
    pub(crate) faction_win_symbol: BTreeMap<i32, i32>,
    pub(crate) symbol_total_num: i32,
    pub(crate) win_victory_symbol_num: i32,
    pub(crate) victory_symbol_num: i32,
}

impl CServerWarRegion {
    pub(crate) fn decord_from_byte_array<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError>> {
        self.decord_from_byte_array_with_npc_entry(
            source,
            cursor,
            include_child,
            area_width,
            area_height,
            monster_registry,
            skill_factory,
            context,
            |_, _, _| {},
        )
    }

    pub(crate) fn decord_from_byte_array_with_npc_entry<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
        after_npc_entry: impl FnMut(&mut CServerRegion, i32, &mut Context),
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError>> {
        let _ = self
            .base
            .decord_from_byte_array_with_npc_entry(
                source,
                cursor,
                include_child,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
                after_npc_entry,
            )
            .map_err(WarRegionDecodeError::Base)?;

        self.symbol_total_num = read_region_i32(source, cursor, "m_lSymbolTotalNum")
            .map_err(WarRegionDecodeError::Input)?;
        self.win_victory_symbol_num = read_region_i32(source, cursor, "m_lWinVicSymbolNum")
            .map_err(WarRegionDecodeError::Input)?;
        self.victory_symbol_num = read_region_i32(source, cursor, "m_lVicSymbolNum")
            .map_err(WarRegionDecodeError::Input)?;

        // RVA 0x001D3110 не очищает map: keys за новым total сохраняются.
        let owner_faction_id = self.base.param.owned_faction_id;
        for symbol_id in 0..self.symbol_total_num.max(0) {
            self.faction_win_symbol.insert(symbol_id, owner_faction_id);
        }
        Ok(true)
    }

    pub(crate) fn on_enter_contend<Context: WarContendEntryContext>(
        &mut self,
        player: Option<&ContendPlayerState>,
        symbol_id: i32,
        symbol_name: &str,
        max_time: i32,
        goods: [&str; 4],
        context: &mut Context,
    ) -> Result<(), Context::MembershipError> {
        let Some(player) = player else {
            return Ok(());
        };
        if player.faction_id == 0 || player.shape_type == 6 || player.is_dead {
            return Ok(());
        }
        if self.get_is_faction_win_symbol(player.faction_id, symbol_id) {
            context.notify_player(player.player_id, "GS0243");
            return Ok(());
        }

        // RVA 0x001D26F0: owner-check обязан коротко замкнуть city membership,
        // поскольку тот может быть заблокирован отсутствующим weekly-флагом.
        if !context.is_owner(player.faction_id)
            && !context.is_apply_war_faction(player.faction_id)?
        {
            context.notify_player(player.player_id, "GS0244");
            return Ok(());
        }
        if is_player_contend_symbol(&self.contenders, player.player_id, symbol_id) {
            return Ok(());
        }
        if goods
            .iter()
            .any(|good| !good.is_empty() && !context.player_has_good(player.player_id, good))
        {
            context.notify_player(player.player_id, "GS0245");
            return Ok(());
        }

        self.cancel_contend_by_player_id(Some(player), context);
        self.add_contend(player, symbol_id, symbol_name, max_time, context);
        context.notify_player(player.player_id, "GS0229");
        for good in goods {
            context.register_needed_good(good);
        }
        Ok(())
    }

    pub(crate) fn cancel_contend_by_player_id<Context: WarContendEntryContext>(
        &mut self,
        player: Option<&ContendPlayerState>,
        context: &mut Context,
    ) -> bool {
        let Some(player) = player else {
            return false;
        };
        self.remove_contenders_for_player(player.player_id);
        context.set_known_player_contend_state(player.player_id, false);
        context.send_contend_time(player.player_id, 0);
        true
    }

    /// Удаляет только owned contender-записи. Player-state и `0xBFF29`
    /// остаются у вызывающего runtime-owner-а и применяются после mutation.
    pub(crate) fn remove_contenders_for_player(&mut self, player_id: i32) {
        self.contenders
            .retain(|contender| contender.player_id != player_id);
    }

    pub(crate) fn add_contend<Context: WarContendEntryContext>(
        &mut self,
        player: &ContendPlayerState,
        symbol_id: i32,
        symbol_name: &str,
        max_time: i32,
        context: &mut Context,
    ) {
        let first_of_faction = self
            .contenders
            .iter()
            .all(|contender| contender.faction_id != player.faction_id);
        self.contenders.push(ContendState {
            id: symbol_id,
            name: symbol_name.to_owned(),
            player_id: player.player_id,
            faction_id: player.faction_id,
            current_time: 0,
            max_time,
            start_time_ms: context.now_millis(),
        });
        context.set_known_player_contend_state(player.player_id, true);
        context.send_contend_time(player.player_id, 0);
        if first_of_faction {
            context.send_first_faction_contender_notice(
                player.country,
                &player.faction_name,
                symbol_name,
            );
        }
    }

    pub(crate) fn dec_contend_time<Context: WarRegionContext>(
        &mut self,
        player_id: Option<i32>,
        amount: i32,
        context: &mut Context,
    ) -> Result<(), ContendArithmeticBlock> {
        let Some(player_id) = player_id else {
            return Ok(());
        };
        let Some(contender) = self
            .contenders
            .iter_mut()
            .find(|contender| contender.player_id == player_id)
        else {
            return Ok(());
        };
        contender.current_time = contender.current_time.wrapping_sub(amount).max(0);
        let percentage = contend_percentage(contender.current_time, contender.max_time)?;
        context.send_contend_time(player_id, percentage);
        Ok(())
    }

    pub(crate) fn on_player_damage<Context: WarRegionContext>(
        &mut self,
        player: Option<WarDamagePlayer>,
        damage: i32,
        dec_time_param: f32,
        context: &mut Context,
    ) -> Result<(), WarDamageError> {
        let Some(player) = player else {
            return Ok(());
        };
        if damage <= 0 {
            return Ok(());
        }
        let Some(contender) = self
            .contenders
            .iter_mut()
            .find(|contender| contender.player_id == player.player_id)
        else {
            return Ok(());
        };
        let decrement =
            legacy_war_damage_decrement(contender.max_time, damage, player.max_hp, dec_time_param)
                .map_err(WarDamageError::X87)?;
        contender.current_time = contender.current_time.wrapping_sub(decrement).max(0);
        let percentage = contend_percentage(contender.current_time, contender.max_time)
            .map_err(WarDamageError::Percentage)?;
        context.send_contend_time(contender.player_id, percentage);
        Ok(())
    }

    pub(crate) fn get_is_faction_win_symbol(&self, faction_id: i32, symbol_id: i32) -> bool {
        self.faction_win_symbol.get(&symbol_id) == Some(&faction_id)
    }

    pub(crate) fn cancel_contend_by_symbol_id<Context: WarRegionContext>(
        &mut self,
        symbol_id: i32,
        context: &mut Context,
    ) {
        let Some(index) = self
            .contenders
            .iter()
            .position(|contender| contender.id == symbol_id)
        else {
            return;
        };
        let player_id = self.contenders[index].player_id;
        context.send_contend_time(player_id, 0);
        context.set_global_player_contend_state(player_id, false);
        self.contenders.remove(index);
    }

    pub(crate) fn set_faction_win_symbol<Context: WarContendContext>(
        &mut self,
        faction_id: i32,
        union_id: i32,
        symbol_id: i32,
        context: &mut Context,
    ) {
        self.faction_win_symbol.insert(symbol_id, faction_id);
        context.on_faction_win_one_symbol(faction_id, symbol_id);
        if faction_id == self.base.param.owned_faction_id {
            return;
        }
        let owned_symbols = self
            .faction_win_symbol
            .values()
            .fold(0_i32, |count, &owner| {
                count.wrapping_add(i32::from(owner == faction_id))
            });
        if owned_symbols >= self.win_victory_symbol_num {
            let owner = context.on_faction_victory(
                faction_id,
                union_id,
                WarRegionOwnership {
                    faction_id: self.base.param.owned_faction_id,
                    union_id: self.base.param.owned_union_id,
                },
            );
            self.base.param.owned_faction_id = owner.faction_id;
            self.base.param.owned_union_id = owner.union_id;
        }
    }

    pub(crate) fn ai<Context: WarContendContext>(
        &mut self,
        context: &mut Context,
    ) -> Result<(), ContendAiError<Context::MembershipError>> {
        context
            .run_base_region_ai(&mut self.base)
            .map_err(ContendAiError::Base)?;
        let now_ms = context.now_millis();
        let mut completed = Vec::new();
        for contender in &mut self.contenders {
            let elapsed = now_ms.wrapping_sub(contender.start_time_ms);
            let candidate = (contender.current_time as u32).wrapping_add(elapsed);
            if candidate >= contender.max_time as u32 {
                context.send_contend_time(contender.player_id, 100);
                completed.push(contender.clone());
            } else if elapsed >= 1_000 {
                contender.current_time = contender.current_time.wrapping_add(elapsed as i32);
                contender.start_time_ms = now_ms;
                let percentage = contend_percentage(contender.current_time, contender.max_time)
                    .map_err(ContendAiError::Arithmetic)?;
                context.send_contend_time(contender.player_id, percentage);
            }
        }

        for contender in completed {
            self.on_contend_time_over(&contender, context)
                .map_err(ContendAiError::Membership)?;
        }
        Ok(())
    }

    pub(crate) fn on_contend_time_over<Context: WarContendContext>(
        &mut self,
        contender: &ContendState,
        context: &mut Context,
    ) -> Result<(), Context::MembershipError> {
        let Some(player) = context.find_global_player(contender.player_id) else {
            return Ok(());
        };
        if player.faction_id == 0 {
            return Ok(());
        }
        if !context.is_owner(player.faction_id)
            && !context.is_apply_war_faction(player.faction_id)?
        {
            return Ok(());
        }

        self.cancel_contend_by_symbol_id(contender.id, context);
        self.set_faction_win_symbol(player.faction_id, player.union_id, contender.id, context);
        context.send_symbol_captured_region_notice();
        context.send_symbol_captured_top_info(&player.faction_name, &contender.name);
        context.write_symbol_capture_logs(SymbolCaptureLog {
            war_number: self.base.war_number,
            owned_faction_id: self.base.param.owned_faction_id,
            owned_union_id: self.base.param.owned_union_id,
            faction_name: &player.faction_name,
            faction_id: player.faction_id,
            player_id: contender.player_id,
            symbol_id: contender.id,
            union_id: player.union_id,
        });
        Ok(())
    }

    pub(crate) fn on_war_declare(&mut self, war_number: i32) {
        self.base.on_war_declare(war_number);
        self.reset_symbol_owners();
    }

    pub(crate) fn on_war_end(&mut self, war_number: i32) {
        self.base.on_war_end(war_number);
        self.faction_win_symbol.clear();
    }

    pub(crate) fn reset_war_state(&mut self, war_number: i32, state: i32) {
        self.base.reset_war_state(war_number, state);
        if (1..4).contains(&state) {
            self.reset_symbol_owners();
        }
    }

    pub(crate) fn update_contend_player<Context: WarRegionContext>(
        &mut self,
        context: &mut Context,
    ) -> Result<(), Context::MembershipError> {
        if self.base.city_state != 3 {
            return Ok(());
        }

        let mut index = 0;
        while index < self.contenders.len() {
            let player_id = self.contenders[index].player_id;
            let Some(faction_id) = context.player_faction_id(player_id) else {
                index += 1;
                continue;
            };
            if context.is_apply_war_faction(faction_id)? {
                index += 1;
                continue;
            }

            context.send_contend_time(player_id, 0);
            context.set_global_player_contend_state(player_id, false);
            self.contenders.remove(index);
            return Ok(());
        }
        Ok(())
    }

    pub(crate) fn clear_region<Context: WarRegionClearContext>(&mut self, context: &mut Context) {
        for contender in &self.contenders {
            context.send_contend_time(contender.player_id, 0);
            context.set_region_player_contend_state(self.base.id, contender.player_id, false);
        }
        self.contenders.clear();
    }

    fn reset_symbol_owners(&mut self) {
        self.faction_win_symbol.clear();
        let owner_faction = self.base.param.owned_faction_id;
        for symbol in 0..self.symbol_total_num.max(0) {
            self.faction_win_symbol.insert(symbol, owner_faction);
        }
    }
}

fn contend_percentage(current_time: i32, max_time: i32) -> Result<i32, ContendArithmeticBlock> {
    if max_time == 0 {
        return Ok(0);
    }
    current_time
        .wrapping_mul(100)
        .checked_div(max_time)
        .ok_or(ContendArithmeticBlock {
            current_time,
            max_time,
        })
}

fn legacy_war_damage_decrement(
    max_time: i32,
    damage: i32,
    max_hp: u32,
    dec_time_param: f32,
) -> Result<i32, WarDamageArithmeticBlock> {
    let block = || WarDamageArithmeticBlock {
        max_time,
        damage,
        max_hp,
        dec_time_param_bits: dec_time_param.to_bits(),
    };

    // VERIFIED_DISASSEMBLY RVA 0x001D2510: damage и unsigned MaxHP сначала
    // становятся f32; ratio сохраняется как f32, затем x87 умножает его на
    // signed max_time и выгружает i32 с установленным процессом truncation RC.
    let damage_as_float = damage as f32;
    let max_hp_as_float = max_hp as f32;
    let ratio = ((f64::from(damage_as_float) / f64::from(max_hp_as_float))
        * f64::from(dec_time_param)) as f32;
    let scaled = f64::from(max_time) * f64::from(ratio);
    if !scaled.is_finite() || scaled < f64::from(i32::MIN) || scaled >= 2_147_483_648.0_f64 {
        return Err(block());
    }
    Ok(scaled.trunc() as i32)
}

pub(crate) fn read_region_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], RegionDecodeInputBlock> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: N,
            available: block.available,
        }
    })?;
    let bytes = reader
        .read_bytes(N)
        .map_err(|block| RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn read_region_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, RegionDecodeInputBlock> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader
        .read_i32()
        .map_err(|block| RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}
