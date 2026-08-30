//! GameServer-владелец общей country-region поверхности `ServerCountryRegion`.
//!
//! Полный subtype decoder RVA `0x001CD3F0`, gate runtime
//! `0x001CAC80/0x001CADD0/0x001CB1E0..0x001CB310`, `RefreshGates`
//! `0x001CB750`, `RefreshFlags` `0x001CB880`, `ClearRegion` `0x001CBA50`,
//! phase callbacks `0x001CABC0..0x001CAC60`, spatial
//! `SetEnterPosXY/GetReturnPoint` `0x001CA9F0/0x001CE010`,
//! virtual security `0x001CAD70`,
//! guard ownership/refresh
//! `0x001CB370/0x001CC960..0x001CCA30`, `GetCamp` `0x001CE540`, attackability
//! `0x001CE830..0x001CE970`, contender damage/time
//! `0x001CA980/0x001CAFF0/0x001CB0D0`, enter/list/symbol
//! `0x001CB710/0x001CCAC0/0x001CCB50/0x001CE190/0x001CE2B0`, AI/victory
//! `0x001CE380/0x001CEA10` и `IsPlayerContendSymbol` `0x001D24E0` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `servercountryregion.h/.cpp`. Gate layout, camp `0/1`, map-key, overload
//! return, flag layout, area selection и refresh/clear order имеют статус
//! `VERIFIED_DISASSEMBLY`.
//!
//! В отличие от city-owner, обе country gate-карты индексируются runtime
//! child-ID, а `tagGate.field_28` decoder читает, но не применяет. Общий
//! base-region owner выдаёт exact child-ID и регистрирует тот же gate/flag в
//! child/area membership. `BTreeMap`
//! сохраняет порядок `std::map`. Flags являются обычными `CBuild` type `0x44C`:
//! wire `field_24` не применяется, initial action остаётся `0`, а refresh
//! меняет только HP до обязательного `0xBF60F`. Area lookup сохраняет
//! `random(map.size())` и mutating `operator[]`; base fallback теперь замкнут
//! через `CServerRegion` и `CCountryParam`. `SetEnterPosXY` сохраняет fight-only
//! gate, same-region comparison, игнорирование random bool и player SetPos;
//! coordinate RNG и внешний player-effect остаются context-границами. Factory
//! остаётся явной границей; guard mutation, send-around, spawn index refresh и
//! массовый kick после clear исполняют canonical owners `CGame/CMonster`. Guard
//! sets используют `BTreeSet` с порядком defend monsters, attack monsters,
//! defend indices, attack indices. `GetCamp` сравнивает defend первым;
//! attackability разрешает
//! только player type `400`, активную войну и объект противоположного camp.
//! `Vec` сохраняет list-order contender-ов, `BTreeMap` — symbol ownership;
//! проверка player/symbol рассматривает только первую запись player-а, а
//! cancel-by-symbol удаляет все совпадения в порядке списка. AI собирает все
//! завершённые снимки до callbacks и сохраняет wrapping `DWORD`/signed math.
//! Реальный caller находится в `CGame::AI`: adapter передаёт live base-region,
//! общий monster/weather prefix, canonical players, state/notice wire и
//! virtual country-symbol callback.
//! `INT_MIN / -1` и недоказанные invalid x87 conversions остаются локальными
//! typed-границами.
//! `GetSecurity` до cell lookup проверяет исходный war-byte: false и
//! out-of-bounds дают `SAFE=2`, true возвращает packed base security, а ещё не
//! записанный constructor-byte и несогласованный cell storage остаются
//! раздельными typed-границами.
//!
//! Writer `CountryWarSys::update_apply_war` записывает defend/attack до
//! `UpdateContendPlayer`, а phase chain материализует declare/prepare/war
//! callbacks и clear. Victory callback `OnFlagDestroy` RVA `0x001CAD50`
//! независимо от исходного war-byte оставляет его false при совпавшем region
//! ID; переданный country long не читает. Constructor RVA `0x001CE5D0` не
//! инициализирует эти два `long` и три phase bool; Rust хранит их как `Option`
//! до доказанного writer-а.
//! Точный EXE подтвердил исходную странность `OnPrepareBegin/End`: оба проверяют
//! `_state_prepare`, но меняют `_state_declare`, поэтому неизвестный prepare-byte
//! остаётся локальной typed-границей. `CancelContendByPlayer` содержит отдельный
//! исходный дефект: non-null player немедленно получает `false`, а null-ветка
//! читает absolute `0x8` и вызывает метод с null; safe Rust не придумывает ей
//! результат. Отдельные тела STL collection internals, `Catch/Unwind` и
//! deleting-destructor thunk сняты как единая доказанная техническая группа;
//! их наблюдаемые call-site contracts сохранены, доменные constructors,
//! destructors и оставшиеся callbacks не классифицировались этим sweep.

use std::collections::{BTreeMap, BTreeSet};

use super::build::{BuildClientUpdate, BuildInit, BuildRuntimeContext, CBuild};
use super::citygate::{CCityGate, CityGateInit};
use super::country::countryparam::CCountryParam;
use super::legacycodec::LegacyReader;
use super::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionRandomPosition, RegionReturnPoint,
    RegionSecurity,
};
use super::servercityregion::{CityGateRuntimeContext, city_gate_footprint_is_clear};
use super::serverregion::{
    CServerRegion, ServerRegionDecodeContext, ServerRegionDecodeError,
    ServerRegionMonsterContext, ServerRegionMonsterRectBlock, ServerReturnPlayer,
    ServerReturnSetupBlock,
};
use super::serverwarregion::{
    ContendArithmeticBlock, ContendState, RegionDecodeInputBlock, read_region_array,
};
use crate::gameserver::gameserver::game::GameClockContext;

const WC_DEFEND: i32 = 0;
const WC_ATTACK: i32 = 1;

const OC_OPEN: i32 = 0;
const OC_CLOSE: i32 = 1;
const OC_REFRESH: i32 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryGateBuild {
    pub(crate) picture_id: i32,
    pub(crate) direction: i32,
    pub(crate) action: u16,
    pub(crate) max_hp: i32,
    pub(crate) defence: i32,
    pub(crate) width_increment: i32,
    pub(crate) title_x: i32,
    pub(crate) title_y: i32,
    pub(crate) height_increment: i32,
    pub(crate) element_resistance: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) script: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryFlagBuild {
    pub(crate) picture_id: i32,
    pub(crate) direction: i32,
    pub(crate) max_hp: i32,
    pub(crate) defence: i32,
    pub(crate) width_increment: i32,
    pub(crate) title_x: i32,
    pub(crate) title_y: i32,
    pub(crate) height_increment: i32,
    pub(crate) element_resistance: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) script: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryAreaState {
    pub(crate) id: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryBattleStateBlock {
    DefendCountry,
    AttackCountry,
    WarActive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryRegionDecodeError<BaseError> {
    Base(BaseError),
    Input(RegionDecodeInputBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountrySecurityError {
    BattleState(CountryBattleStateBlock),
    Cell(RegionCellAccessBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryReturnPointError {
    BattleState(CountryBattleStateBlock),
    Base(ServerReturnSetupBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryEntryError {
    ReturnPoint(CountryReturnPointError),
    Cell(RegionCellAccessBlock),
}

pub(crate) trait CountryRegionDecodeContext: ServerRegionDecodeContext + BuildRuntimeContext {}

impl<Context: ServerRegionDecodeContext + BuildRuntimeContext + ?Sized> CountryRegionDecodeContext
    for Context
{
}

pub(crate) trait CountryRegionRuntimeContext:
    CityGateRuntimeContext + ServerRegionMonsterContext + GameClockContext
{}

impl<T> CountryRegionRuntimeContext for T where
    T: CityGateRuntimeContext + ServerRegionMonsterContext + GameClockContext
{
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryGuardRefreshTargets {
    pub(crate) monster_ids: Vec<i32>,
    pub(crate) spawn_indices: Vec<i32>,
}

pub(crate) trait CountryReturnPointContext {
    /// Вызывает исходный `random(map.size())`; реакция на нулевой count остаётся
    /// у достигнутого RNG-owner-а, а returned DWORD используется map key.
    fn random_country_area_key(&mut self, area_count: u32) -> i32;
}

pub(crate) trait CountryEntryContext:
    CountryReturnPointContext + RegionRandomContext
{
    /// Выполняет virtual player slot `+0x88` с `(x, y)`.
    fn set_country_player_position(&mut self, player_id: i32, x: i32, y: i32);
}

pub(crate) trait CountryCampContext {
    /// Возвращает `CPlayer::m_btCountry` только для существующего player ID.
    fn country_player_country(&mut self, player_id: i32) -> Option<u8>;
}

pub(crate) trait CountryContendEntryContext {
    /// Возвращает младшие 32 бита монотонного миллисекундного счётчика.
    fn now_millis(&mut self) -> u32;

    /// Шлёт player-у `0xBFF29` с одним signed значением времени.
    fn send_contend_time(&mut self, player_id: i32, time: i32);

    /// Меняет contend-state у уже известного non-null player pointer.
    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Шлёт player-localized `GS0228/GS0229` с исходным красным цветом.
    fn notify_player(&mut self, player_id: i32, string_id: &'static str);
}

pub(crate) trait CountryContendContext: CountryContendEntryContext {
    /// Выполняет исходный `CServerRegion::AI` до contender-tick.
    fn run_base_region_ai(
        &mut self,
        region: &mut CServerRegion,
    ) -> Result<(), ServerRegionMonsterRectBlock>;

    /// Имитирует lookup non-null player-а в глобальном `s_mapPlayer`.
    fn find_global_player(&mut self, player_id: i32) -> Option<CountryContendPlayer>;

    /// Меняет contend-state найденного global player-а.
    fn set_global_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Concrete virtual slot `+0x10C`: `(country, symbol_id)`.
    fn on_country_win_one_symbol(&mut self, country: i32, symbol_id: i32);

    /// Шлёт region `GS0226(country-name, symbol-name)`.
    fn send_country_symbol_captured_region_notice(&mut self, country: u8, symbol_name: &str);

    /// Шлёт top-info `GS0227(country-name, region-name, symbol-name)` маршрутом
    /// `(-1, 0, 1, 1)`.
    fn send_country_symbol_captured_top_info(
        &mut self,
        country: u8,
        region_name: &str,
        symbol_name: &str,
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryRegionAiError {
    Base(ServerRegionMonsterRectBlock),
    Arithmetic(ContendArithmeticBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryContendPlayer {
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) country: u8,
    pub(crate) shape_type: i32,
    pub(crate) is_dead: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryDamagePlayer {
    pub(crate) player_id: i32,
    pub(crate) max_hp: u32,
}

/// BLOCKED_MISSING_FACT: для NaN/inf/out-of-range x87 `fistp i32` точная
/// реакция процесса не доказана; safe Rust не назначает ей saturating cast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryDamageArithmeticBlock {
    pub(crate) max_time: i32,
    pub(crate) damage: i32,
    pub(crate) max_hp: u32,
    pub(crate) dec_time_param_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryDamageError {
    X87(CountryDamageArithmeticBlock),
    Percentage(ContendArithmeticBlock),
}

/// BLOCKED_MISSING_FACT: RVA `0x001CCAC0` в null-ветке читает absolute
/// address `0x00000008`, а затем вызывает `CPlayer::SetContendState` с null.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryNullPlayerCancelBlock;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryMoveShape {
    pub(crate) object_type: i32,
    pub(crate) id: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerCountryRegion {
    pub(crate) base: CServerRegion,
    pub(crate) contenders: Vec<ContendState>,
    pub(crate) symbol_hold: BTreeMap<i32, i32>,
    pub(crate) defend_gates: BTreeMap<i32, CCityGate>,
    pub(crate) attack_gates: BTreeMap<i32, CCityGate>,
    pub(crate) defend_flags: BTreeMap<i32, CBuild>,
    pub(crate) attack_flags: BTreeMap<i32, CBuild>,
    pub(crate) defend_areas: BTreeMap<i32, CountryAreaState>,
    pub(crate) attack_areas: BTreeMap<i32, CountryAreaState>,
    pub(crate) defend_guards: BTreeSet<i32>,
    pub(crate) defend_guard_indices: BTreeSet<i32>,
    pub(crate) attack_guards: BTreeSet<i32>,
    pub(crate) attack_guard_indices: BTreeSet<i32>,
    pub(crate) defend_country: Option<i32>,
    pub(crate) attack_country: Option<i32>,
    pub(crate) declare_active: Option<bool>,
    pub(crate) prepare_active: Option<bool>,
    pub(crate) war_active: Option<bool>,
}

impl CServerCountryRegion {
    pub(crate) fn decord_from_byte_array<Context: CountryRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        context: &mut Context,
    ) -> Result<bool, CountryRegionDecodeError<ServerRegionDecodeError<Context::RuntimeError>>>
    {
        let _ = self
            .base
            .decord_from_byte_array(source, cursor, include_child, context)
            .map_err(CountryRegionDecodeError::Base)?;
        self.decode_gate_section(source, cursor, WC_DEFEND, context)?;
        self.decode_gate_section(source, cursor, WC_ATTACK, context)?;
        self.decode_flag_section(source, cursor, WC_DEFEND, context)?;
        self.decode_flag_section(source, cursor, WC_ATTACK, context)?;
        self.decode_area_section(source, cursor, WC_DEFEND)?;
        self.decode_area_section(source, cursor, WC_ATTACK)?;
        Ok(true)
    }

    pub(crate) fn operator_city_gate<Context: CityGateRuntimeContext>(
        &mut self,
        city_gate_id: i32,
        operation: i32,
        camp: i32,
        context: &mut Context,
    ) -> bool {
        let region_id = self.base.id;
        let Some(gate) = self.gates(camp).and_then(|gates| gates.get(&city_gate_id)) else {
            return false;
        };
        if operation == OC_CLOSE && !city_gate_footprint_is_clear(region_id, gate, context) {
            return false;
        }
        let gate = self
            .gates_mut(camp)
            .and_then(|gates| gates.get_mut(&city_gate_id))
            .expect("gate найден до неизменяющего map вызова");
        operate_city_gate(gate, operation, context);
        true
    }

    pub(crate) fn get_city_gate_state(&self, city_gate_id: i32, camp: i32) -> i32 {
        let Some(gate) = self.gates(camp).and_then(|gates| gates.get(&city_gate_id)) else {
            return -1;
        };
        match gate.action {
            7 => 0,
            0 | 1 => 1,
            6 => 2,
            _ => -1,
        }
    }

    pub(crate) fn update_city_gate_to_client<Context: CityGateRuntimeContext>(
        &self,
        city_gate_id: i32,
        camp: i32,
        context: &mut Context,
    ) {
        let Some(gate) = self.gates(camp).and_then(|gates| gates.get(&city_gate_id)) else {
            return;
        };
        update_city_gate_object(self.base.id, gate, context);
    }

    pub(crate) fn refresh_gates<Context: CityGateRuntimeContext>(&mut self, context: &mut Context) {
        let region_id = self.base.id;
        for gate in self.defend_gates.values_mut() {
            refresh_city_gate_object(region_id, gate, context);
        }
        for gate in self.attack_gates.values_mut() {
            refresh_city_gate_object(region_id, gate, context);
        }
    }

    /// Собственная refresh-часть country `ClearRegion`; следующий
    /// `KickOutAllPlayerToReturnPoint` исполняет владеющий картой игроков
    /// `CGame`, чтобы смена региона не уходила во внешний callback.
    pub(crate) fn refresh_for_clear<Context: CityGateRuntimeContext>(
        &mut self,
        context: &mut Context,
    ) -> CountryGuardRefreshTargets {
        self.refresh_gates(context);
        self.refresh_flags(context);
        self.guard_refresh_targets()
    }

    pub(crate) fn refresh_flags<Context: BuildRuntimeContext>(&mut self, context: &mut Context) {
        let region_id = self.base.id;
        for flag in self.defend_flags.values_mut() {
            refresh_country_flag_object(region_id, flag, context);
        }
        for flag in self.attack_flags.values_mut() {
            refresh_country_flag_object(region_id, flag, context);
        }
    }

    pub(crate) fn get_return_point<Context: CountryReturnPointContext>(
        &mut self,
        player: Option<ServerReturnPlayer>,
        country_param: &mut CCountryParam,
        context: &mut Context,
    ) -> Result<RegionReturnPoint, CountryReturnPointError> {
        let Some(player) = player else {
            return Ok(self.base.region.get_return_point());
        };
        if self.base.city_state != 3 {
            return self
                .base
                .get_return_point(Some(player), country_param)
                .map_err(CountryReturnPointError::Base);
        }

        // BLOCKED_MISSING_FACT: constructor RVA 0x001CE5D0 не инициализирует
        // primitive `_defend_country/_attack_country`; до доказанного вызова
        // `CountryWarSys::update_apply_war` Safe Rust не назначает им нули.
        let defend_country = self
            .defend_country
            .ok_or(CountryReturnPointError::BattleState(
                CountryBattleStateBlock::DefendCountry,
            ))?;
        let areas = if i32::from(player.country) == defend_country {
            &mut self.defend_areas
        } else {
            let attack_country =
                self.attack_country
                    .ok_or(CountryReturnPointError::BattleState(
                        CountryBattleStateBlock::AttackCountry,
                    ))?;
            if i32::from(player.country) != attack_country {
                return Ok(self.base.region.get_return_point());
            }
            &mut self.attack_areas
        };
        let area_key = context.random_country_area_key(areas.len() as u32);
        // Original `std::map::operator[]` вставлял zeroed/default `tagArea`,
        // если RNG key отсутствовал среди загруженных IDs.
        let area = areas.entry(area_key).or_default();
        Ok(RegionReturnPoint {
            region_id: self.base.id,
            left: area.left,
            top: area.top,
            right: area.right,
            bottom: area.bottom,
            direction: -1,
        })
    }

    pub(crate) fn set_enter_pos_xy<Context: CountryEntryContext>(
        &mut self,
        player: ServerReturnPlayer,
        country_param: &mut CCountryParam,
        context: &mut Context,
    ) -> Result<Option<RegionRandomPosition>, CountryEntryError> {
        if self.base.get_city_state() != 3 {
            return Ok(None);
        }

        let point = self
            .get_return_point(Some(player), country_param, context)
            .map_err(CountryEntryError::ReturnPoint)?;
        if point.region_id != self.base.id {
            return Ok(None);
        }

        let position = self
            .base
            .region
            .get_random_pos_in_range(
                point.left,
                point.top,
                point.right.wrapping_sub(point.left),
                point.bottom.wrapping_sub(point.top),
                context,
            )
            .map_err(CountryEntryError::Cell)?;
        context.set_country_player_position(player.id, position.x, position.y);
        Ok(Some(position))
    }

    /// Entry path клиентского `0x9050B`: camp выбирает ordered area-map,
    /// `random(size)` является порядковым индексом iterator-а, а не map key.
    /// Пустая карта и недостигнутый индекс сохраняют исходный silent no-op.
    pub(crate) fn country_war_entry_position<Context: RegionRandomContext>(
        &self,
        camp: i32,
        context: &mut Context,
    ) -> Result<Option<RegionRandomPosition>, RegionCellAccessBlock> {
        let areas = match camp {
            WC_DEFEND => &self.defend_areas,
            WC_ATTACK => &self.attack_areas,
            _ => return Ok(None),
        };
        let index = context.random_below(areas.len() as i32);
        let Some(area) = usize::try_from(index)
            .ok()
            .and_then(|index| areas.values().nth(index))
        else {
            return Ok(None);
        };
        self.base
            .region
            .get_random_pos_in_range(
                area.left,
                area.top,
                area.right.wrapping_sub(area.left),
                area.bottom.wrapping_sub(area.top),
                context,
            )
            .map(Some)
    }

    pub(crate) fn get_security(
        &self,
        x: i32,
        y: i32,
    ) -> Result<RegionSecurity, CountrySecurityError> {
        let war_active = self.war_active.ok_or(CountrySecurityError::BattleState(
            CountryBattleStateBlock::WarActive,
        ))?;
        if !war_active {
            return Ok(RegionSecurity::SAFE);
        }
        self.base
            .region
            .get_security(x, y)
            .map_err(CountrySecurityError::Cell)
    }

    pub(crate) fn update_contend_time<Context: CountryContendContext>(
        &self,
        player_id: i32,
        time: i32,
        context: &mut Context,
    ) {
        context.send_contend_time(player_id, time);
    }

    pub(crate) fn on_player_damage<Context: CountryContendContext>(
        &mut self,
        player: Option<CountryDamagePlayer>,
        damage: i32,
        dec_time_param: f32,
        context: &mut Context,
    ) -> Result<(), CountryDamageError> {
        let Some(player) = player else {
            return Ok(());
        };
        if damage <= 0 {
            return Ok(());
        }
        let Some(index) = self
            .contenders
            .iter()
            .position(|contender| contender.player_id == player.player_id)
        else {
            return Ok(());
        };
        let decrement = legacy_country_damage_decrement(
            self.contenders[index].max_time,
            damage,
            player.max_hp,
            dec_time_param,
        )
        .map_err(CountryDamageError::X87)?;
        let contender = &mut self.contenders[index];
        contender.current_time = contender.current_time.wrapping_sub(decrement).max(0);
        let percentage = country_contend_percentage(contender.current_time, contender.max_time)
            .map_err(CountryDamageError::Percentage)?;
        let player_id = contender.player_id;
        context.send_contend_time(player_id, percentage);
        Ok(())
    }

    pub(crate) fn dec_contend_time<Context: CountryContendContext>(
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
        let percentage = country_contend_percentage(contender.current_time, contender.max_time)?;
        context.send_contend_time(player_id, percentage);
        Ok(())
    }

    pub(crate) fn is_win_symbol(&self, country: i32, symbol_id: i32) -> bool {
        self.symbol_hold.get(&symbol_id) == Some(&country)
    }

    pub(crate) fn cancel_contend_by_player(
        &mut self,
        player: Option<&CountryContendPlayer>,
    ) -> Result<bool, CountryNullPlayerCancelBlock> {
        // VERIFIED_DISASSEMBLY RVA 0x001CCAC0: условие оригинала инвертировано;
        // любой реальный player немедленно получает `false` без side effects.
        if player.is_some() {
            return Ok(false);
        }
        Err(CountryNullPlayerCancelBlock)
    }

    pub(crate) fn on_enter_contend<Context: CountryContendEntryContext>(
        &mut self,
        player: Option<&CountryContendPlayer>,
        symbol_id: i32,
        symbol_name: &str,
        max_time: i32,
        context: &mut Context,
    ) -> Result<(), CountryNullPlayerCancelBlock> {
        let Some(player) = player else {
            return Ok(());
        };
        if player.shape_type == 6 || player.is_dead {
            return Ok(());
        }
        if is_player_contend_symbol(&self.contenders, player.player_id, symbol_id) {
            return Ok(());
        }
        if self.is_win_symbol(i32::from(player.country), symbol_id) {
            context.notify_player(player.player_id, "GS0228");
            return Ok(());
        }

        let _ = self.cancel_contend_by_player(Some(player))?;
        self.add_contend(Some(player), symbol_id, symbol_name, max_time, context);
        context.notify_player(player.player_id, "GS0229");
        Ok(())
    }

    pub(crate) fn add_contend<Context: CountryContendEntryContext>(
        &mut self,
        player: Option<&CountryContendPlayer>,
        symbol_id: i32,
        symbol_name: &str,
        max_time: i32,
        context: &mut Context,
    ) {
        let Some(player) = player else {
            return;
        };
        self.contenders.push(ContendState {
            id: symbol_id,
            name: symbol_name.to_owned(),
            player_id: player.player_id,
            // PDB называет поле country-tag `lCountry`, но RVA `0x001CE190`
            // записывает именно `CPlayer::m_lFactionID +0xB28`.
            faction_id: player.faction_id,
            current_time: 0,
            max_time,
            start_time_ms: context.now_millis(),
        });
        context.set_known_player_contend_state(player.player_id, true);
        context.send_contend_time(player.player_id, 0);
    }

    pub(crate) fn cancel_contend_by_symbol<Context: CountryContendContext>(
        &mut self,
        symbol_id: i32,
        context: &mut Context,
    ) {
        let mut index = 0;
        while index < self.contenders.len() {
            if self.contenders[index].id != symbol_id {
                index += 1;
                continue;
            }
            let player_id = self.contenders[index].player_id;
            context.send_contend_time(player_id, 0);
            if context.find_global_player(player_id).is_some() {
                context.set_global_player_contend_state(player_id, false);
            }
            self.contenders.remove(index);
        }
    }

    pub(crate) fn ai<Context: CountryContendContext>(
        &mut self,
        context: &mut Context,
    ) -> Result<(), CountryRegionAiError> {
        context
            .run_base_region_ai(&mut self.base)
            .map_err(CountryRegionAiError::Base)?;
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
                let percentage =
                    country_contend_percentage(contender.current_time, contender.max_time)
                        .map_err(CountryRegionAiError::Arithmetic)?;
                context.send_contend_time(contender.player_id, percentage);
            }
        }
        for contender in completed {
            self.on_contend_time_over(&contender, context);
        }
        Ok(())
    }

    pub(crate) fn on_contend_time_over<Context: CountryContendContext>(
        &mut self,
        contender: &ContendState,
        context: &mut Context,
    ) {
        let Some(player) = context.find_global_player(contender.player_id) else {
            return;
        };
        let country = player.country;
        self.cancel_contend_by_symbol(contender.id, context);
        self.symbol_hold.insert(contender.id, i32::from(country));
        context.on_country_win_one_symbol(i32::from(country), contender.id);
        context.send_country_symbol_captured_region_notice(country, &contender.name);
        context.send_country_symbol_captured_top_info(country, &self.base.name, &contender.name);
    }

    /// Writer-side region projection: defend записывается раньше attack.
    pub(crate) fn set_country_sides(&mut self, defend_country: i32, attack_country: i32) {
        self.defend_country = Some(defend_country);
        self.attack_country = Some(attack_country);
    }

    /// Country vtable `0x65D464`, slot `+0x104`, указывает на единственный
    /// `ret` по `0x485540`: после записи сторон эта разновидность региона не
    /// фильтрует contender-ов, в отличие от city/village war owners.
    pub(crate) const fn update_contend_player(&mut self) {}

    pub(crate) fn country_side_bytes(&self) -> Result<(u8, u8), CountryBattleStateBlock> {
        let defend_country = self
            .defend_country
            .ok_or(CountryBattleStateBlock::DefendCountry)?;
        let attack_country = self
            .attack_country
            .ok_or(CountryBattleStateBlock::AttackCountry)?;
        Ok((defend_country as u8, attack_country as u8))
    }

    pub(crate) fn on_declare_begin(&mut self, region_id: i32) {
        if self.base.id == region_id {
            self.declare_active = Some(true);
        }
    }

    pub(crate) fn on_declare_end(&mut self, region_id: i32) {
        if self.base.id == region_id {
            self.declare_active = Some(false);
        }
    }

    pub(crate) fn on_prepare_begin(&mut self, region_id: i32) {
        if self.base.id != region_id {
            return;
        }
        match self.prepare_active {
            Some(false) => self.declare_active = Some(true),
            Some(true) => {}
            None if self.declare_active != Some(true) => self.declare_active = None,
            None => {}
        }
    }

    pub(crate) fn on_prepare_end(&mut self, region_id: i32) {
        if self.base.id != region_id {
            return;
        }
        match self.prepare_active {
            Some(true) => self.declare_active = Some(false),
            Some(false) => {}
            None if self.declare_active != Some(false) => self.declare_active = None,
            None => {}
        }
    }

    pub(crate) fn on_war_start(&mut self, region_id: i32) {
        if self.base.id == region_id {
            self.war_active = Some(true);
        }
    }

    pub(crate) fn on_war_timeout(&mut self, region_id: i32) {
        if self.base.id == region_id {
            self.war_active = Some(false);
        }
    }

    pub(crate) fn on_war_end(&mut self, region_id: i32) {
        // VERIFIED_DISASSEMBLY: vtable `+0x150` совпадает с `OnTimeOut`
        // `+0x14C` и указывает на одно RVA `0x001CAC60`.
        self.on_war_timeout(region_id);
    }

    pub(crate) fn on_flag_destroy(&mut self, region_id: i32, _country: i32) {
        if self.base.id == region_id {
            self.war_active = Some(false);
        }
    }

    pub(crate) fn add_gurd_monster(&mut self, monster_id: i32, camp: i32) {
        if let Some(guards) = self.guards_mut(camp) {
            guards.insert(monster_id);
        }
    }

    pub(crate) fn del_gurd_monster(&mut self, monster_id: i32, camp: i32) {
        if let Some(guards) = self.guards_mut(camp) {
            guards.remove(&monster_id);
        }
    }

    pub(crate) fn add_guard_index(&mut self, spawn_index: i32, camp: i32) {
        if let Some(indices) = self.guard_indices_mut(camp) {
            indices.insert(spawn_index);
        }
    }

    pub(crate) fn guard_refresh_targets(&self) -> CountryGuardRefreshTargets {
        CountryGuardRefreshTargets {
            monster_ids: self
                .defend_guards
                .iter()
                .chain(&self.attack_guards)
                .copied()
                .collect(),
            spawn_indices: self
                .defend_guard_indices
                .iter()
                .chain(&self.attack_guard_indices)
                .copied()
                .collect(),
        }
    }

    pub(crate) fn get_camp<Context: CountryCampContext>(
        &self,
        player_id: i32,
        context: &mut Context,
    ) -> Result<i32, CountryBattleStateBlock> {
        if player_id == 0 {
            return Ok(-1);
        }
        let Some(country) = context.country_player_country(player_id) else {
            return Ok(-1);
        };
        let defend_country = self
            .defend_country
            .ok_or(CountryBattleStateBlock::DefendCountry)?;
        if i32::from(country) == defend_country {
            return Ok(WC_DEFEND);
        }
        let attack_country = self
            .attack_country
            .ok_or(CountryBattleStateBlock::AttackCountry)?;
        Ok(if i32::from(country) == attack_country {
            WC_ATTACK
        } else {
            -1
        })
    }

    pub(crate) fn gate_is_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        context: &mut Context,
    ) -> Result<bool, CountryBattleStateBlock> {
        self.is_collection_attack_able(target, attacker, CountryTargetKind::Gate, context)
    }

    pub(crate) fn flag_is_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        context: &mut Context,
    ) -> Result<bool, CountryBattleStateBlock> {
        self.is_collection_attack_able(target, attacker, CountryTargetKind::Flag, context)
    }

    pub(crate) fn guard_is_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        context: &mut Context,
    ) -> Result<bool, CountryBattleStateBlock> {
        self.is_collection_attack_able(target, attacker, CountryTargetKind::Guard, context)
    }

    fn is_collection_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        kind: CountryTargetKind,
        context: &mut Context,
    ) -> Result<bool, CountryBattleStateBlock> {
        let (Some(target), Some(attacker)) = (target, attacker) else {
            return Ok(false);
        };
        if attacker.object_type != 400 {
            return Ok(false);
        }
        // BLOCKED_MISSING_FACT: constructor RVA 0x001CE5D0 оставляет
        // `_state_war` без значения до первого доказанного `OnStart/OnTimeOut`.
        // Проверенные до этого false-ветви поле не читают.
        let war_active = self.war_active.ok_or(CountryBattleStateBlock::WarActive)?;
        if !war_active {
            return Ok(false);
        }

        let camp = self.get_camp(attacker.id, context)?;
        Ok(match camp {
            WC_DEFEND => self.target_collection_contains(WC_ATTACK, kind, target.id),
            WC_ATTACK => self.target_collection_contains(WC_DEFEND, kind, target.id),
            _ => false,
        })
    }

    fn target_collection_contains(&self, camp: i32, kind: CountryTargetKind, id: i32) -> bool {
        match (camp, kind) {
            (WC_DEFEND, CountryTargetKind::Gate) => self.defend_gates.contains_key(&id),
            (WC_ATTACK, CountryTargetKind::Gate) => self.attack_gates.contains_key(&id),
            (WC_DEFEND, CountryTargetKind::Flag) => self.defend_flags.contains_key(&id),
            (WC_ATTACK, CountryTargetKind::Flag) => self.attack_flags.contains_key(&id),
            (WC_DEFEND, CountryTargetKind::Guard) => self.defend_guards.contains(&id),
            (WC_ATTACK, CountryTargetKind::Guard) => self.attack_guards.contains(&id),
            _ => false,
        }
    }

    fn guards_mut(&mut self, camp: i32) -> Option<&mut BTreeSet<i32>> {
        match camp {
            WC_DEFEND => Some(&mut self.defend_guards),
            WC_ATTACK => Some(&mut self.attack_guards),
            _ => None,
        }
    }

    fn guard_indices_mut(&mut self, camp: i32) -> Option<&mut BTreeSet<i32>> {
        match camp {
            WC_DEFEND => Some(&mut self.defend_guard_indices),
            WC_ATTACK => Some(&mut self.attack_guard_indices),
            _ => None,
        }
    }

    fn decode_gate_section<Context: CountryRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        camp: i32,
        context: &mut Context,
    ) -> Result<(), CountryRegionDecodeError<ServerRegionDecodeError<Context::RuntimeError>>> {
        let count = read_country_i32(source, cursor, country_gate_count_field(camp))
            .map_err(CountryRegionDecodeError::Input)?;
        for _ in 0..count.max(0) {
            let build =
                read_country_gate_build(source, cursor).map_err(CountryRegionDecodeError::Input)?;
            self.add_country_gate(camp, build, context);
        }
        Ok(())
    }

    fn add_country_gate<Context: CountryRegionDecodeContext>(
        &mut self,
        camp: i32,
        build: CountryGateBuild,
        context: &mut Context,
    ) -> Option<i32> {
        let city_gate_id = self.base.take_child_id();
        let gate = CCityGate::from_created(CityGateInit {
            id: city_gate_id,
            graphics_id: build.picture_id,
            region_id: self.base.id,
            name: build.name,
            direction: build.direction,
            action: build.action,
            max_hp: build.max_hp,
            defence: build.defence,
            width_increment: build.width_increment,
            tile_x: build.title_x,
            tile_y: build.title_y,
            height_increment: build.height_increment,
            element_resistance: build.element_resistance,
            script: build.script,
        });
        let (area_width, area_height) = context.area_dimensions();
        if self
            .base
            .register_stationary_child(gate.shape_view(), area_width, area_height)
            .is_err()
        {
            return None;
        }
        context.apply_build_block(gate.current_block_update());
        // VERIFIED_DISASSEMBLY: country map key — `CCityGate::m_lID +8`, не
        // `tagGate.field_00`/logical ID, который использует city-owner.
        self.gates_mut(camp)
            .expect("decoder передаёт только доказанный camp")
            .insert(city_gate_id, gate);
        Some(city_gate_id)
    }

    fn decode_flag_section<Context: CountryRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        camp: i32,
        context: &mut Context,
    ) -> Result<(), CountryRegionDecodeError<ServerRegionDecodeError<Context::RuntimeError>>> {
        let count = read_country_i32(source, cursor, country_flag_count_field(camp))
            .map_err(CountryRegionDecodeError::Input)?;
        for _ in 0..count.max(0) {
            let build =
                read_country_flag_build(source, cursor).map_err(CountryRegionDecodeError::Input)?;
            self.add_country_flag(camp, build, context);
        }
        Ok(())
    }

    fn add_country_flag<Context: CountryRegionDecodeContext>(
        &mut self,
        camp: i32,
        build: CountryFlagBuild,
        context: &mut Context,
    ) -> Option<i32> {
        let flag_id = self.base.take_child_id();
        let flag = CBuild::from_created(BuildInit {
            id: flag_id,
            graphics_id: build.picture_id,
            region_id: self.base.id,
            name: build.name,
            direction: build.direction,
            max_hp: build.max_hp,
            defence: build.defence,
            width_increment: build.width_increment,
            tile_x: build.title_x,
            tile_y: build.title_y,
            height_increment: build.height_increment,
            element_resistance: build.element_resistance,
            script: build.script,
        });
        let (area_width, area_height) = context.area_dimensions();
        if self
            .base
            .register_stationary_child(flag.shape_view(), area_width, area_height)
            .is_err()
        {
            return None;
        }
        context.apply_build_block(flag.current_block_update());
        self.flags_mut(camp)
            .expect("decoder передаёт только доказанный camp")
            .insert(flag_id, flag);
        Some(flag_id)
    }

    fn decode_area_section<BaseError>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        camp: i32,
    ) -> Result<(), CountryRegionDecodeError<BaseError>> {
        let count = read_country_i32(source, cursor, country_area_count_field(camp))
            .map_err(CountryRegionDecodeError::Input)?;
        for _ in 0..count.max(0) {
            let area =
                read_country_area(source, cursor).map_err(CountryRegionDecodeError::Input)?;
            self.areas_mut(camp)
                .expect("decoder передаёт только доказанный camp")
                .insert(area.id, area);
        }
        Ok(())
    }

    fn gates(&self, camp: i32) -> Option<&BTreeMap<i32, CCityGate>> {
        match camp {
            WC_DEFEND => Some(&self.defend_gates),
            WC_ATTACK => Some(&self.attack_gates),
            _ => None,
        }
    }

    fn gates_mut(&mut self, camp: i32) -> Option<&mut BTreeMap<i32, CCityGate>> {
        match camp {
            WC_DEFEND => Some(&mut self.defend_gates),
            WC_ATTACK => Some(&mut self.attack_gates),
            _ => None,
        }
    }

    fn flags_mut(&mut self, camp: i32) -> Option<&mut BTreeMap<i32, CBuild>> {
        match camp {
            WC_DEFEND => Some(&mut self.defend_flags),
            WC_ATTACK => Some(&mut self.attack_flags),
            _ => None,
        }
    }

    fn areas_mut(&mut self, camp: i32) -> Option<&mut BTreeMap<i32, CountryAreaState>> {
        match camp {
            WC_DEFEND => Some(&mut self.defend_areas),
            WC_ATTACK => Some(&mut self.attack_areas),
            _ => None,
        }
    }
}

fn legacy_country_damage_decrement(
    max_time: i32,
    damage: i32,
    max_hp: u32,
    dec_time_param: f32,
) -> Result<i32, CountryDamageArithmeticBlock> {
    let block = || CountryDamageArithmeticBlock {
        max_time,
        damage,
        max_hp,
        dec_time_param_bits: dec_time_param.to_bits(),
    };

    // VERIFIED_DISASSEMBLY RVA 0x001CAFF0: damage и unsigned max HP сначала
    // становятся f32; отношение с global factor сохраняется как f32, затем
    // `fild max_time`, умножение и `fistp i32` идут с truncation RC.
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

fn country_contend_percentage(
    current_time: i32,
    max_time: i32,
) -> Result<i32, ContendArithmeticBlock> {
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

#[derive(Clone, Copy)]
enum CountryTargetKind {
    Gate,
    Flag,
    Guard,
}

fn operate_city_gate<Context: CityGateRuntimeContext>(
    gate: &mut CCityGate,
    operation: i32,
    context: &mut Context,
) {
    match operation {
        OC_OPEN => apply_gate_action(gate, 7, context),
        OC_CLOSE => apply_gate_action(gate, 1, context),
        OC_REFRESH => {
            gate.refresh_hp();
            apply_gate_action(gate, 7, context);
        }
        // Country pointer-overload, в отличие от city-owner, для `OC_Died`
        // и неизвестных operation успешно ничего не делает.
        _ => {}
    }
}

fn refresh_city_gate_object<Context: CityGateRuntimeContext>(
    region_id: i32,
    gate: &mut CCityGate,
    context: &mut Context,
) {
    gate.refresh_hp();
    apply_gate_action(gate, 7, context);
    update_city_gate_object(region_id, gate, context);
}

fn apply_gate_action<Context: CityGateRuntimeContext>(
    gate: &mut CCityGate,
    action: u16,
    context: &mut Context,
) {
    if let Some(update) = gate.set_action(action) {
        context.apply_build_block(update);
    }
}

fn update_city_gate_object<Context: CityGateRuntimeContext>(
    region_id: i32,
    gate: &CCityGate,
    context: &mut Context,
) {
    context.send_build_update(
        region_id,
        gate.id,
        BuildClientUpdate {
            object_type: gate.object_type,
            object_id: gate.id as u32,
            action: gate.action,
            max_hp: gate.max_hp,
            hp: gate.hp,
        },
    );
}

fn refresh_country_flag_object<Context: BuildRuntimeContext>(
    region_id: i32,
    flag: &mut CBuild,
    context: &mut Context,
) {
    flag.refresh_hp();
    context.send_build_update(
        region_id,
        flag.id,
        BuildClientUpdate {
            object_type: flag.object_type,
            object_id: flag.id as u32,
            action: flag.action,
            max_hp: flag.max_hp(),
            hp: flag.hp(),
        },
    );
}

fn read_country_gate_build(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CountryGateBuild, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x2C>(source, cursor, "tagGate scalar block")?;
    let name = read_country_c_string(source, cursor, "tagGate.strName")?;
    let script = read_country_c_string(source, cursor, "tagGate.strScript")?;
    // VERIFIED_DISASSEMBLY RVA 0x001CD3F0: country `tagGate` начинается с
    // picture ID; city `tagBuild` имеет дополнительный logical ID в field_00.
    // Последний DWORD `field_28` остаётся сознательно прочитанной частью
    // `bytes`, но исходная функция не переносит его в созданный gate.
    Ok(CountryGateBuild {
        picture_id: country_i32_at(&bytes, 0x00),
        direction: country_i32_at(&bytes, 0x04),
        action: LegacyReader::at(&bytes, 0x08)
            .and_then(|mut reader| reader.read_u16())
            .expect("фиксированный country block содержит action"),
        max_hp: country_i32_at(&bytes, 0x0C),
        defence: country_i32_at(&bytes, 0x10),
        width_increment: country_i32_at(&bytes, 0x14),
        title_x: country_i32_at(&bytes, 0x18),
        title_y: country_i32_at(&bytes, 0x1C),
        height_increment: country_i32_at(&bytes, 0x20),
        element_resistance: country_i32_at(&bytes, 0x24),
        name,
        script,
    })
}

fn read_country_flag_build(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CountryFlagBuild, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x28>(source, cursor, "tagFlag scalar block")?;
    let name = read_country_c_string(source, cursor, "tagFlag.strName")?;
    let script = read_country_c_string(source, cursor, "tagFlag.strScript")?;
    // VERIFIED_DISASSEMBLY RVA 0x001CD3F0: field_00 — graphics ID;
    // field_04 — direction; fields_08..20 — шесть CBuild properties/position.
    // field_24 входит в wire, но не переносится в factory-объект.
    Ok(CountryFlagBuild {
        picture_id: country_i32_at(&bytes, 0x00),
        direction: country_i32_at(&bytes, 0x04),
        max_hp: country_i32_at(&bytes, 0x08),
        defence: country_i32_at(&bytes, 0x0C),
        width_increment: country_i32_at(&bytes, 0x10),
        title_x: country_i32_at(&bytes, 0x14),
        title_y: country_i32_at(&bytes, 0x18),
        height_increment: country_i32_at(&bytes, 0x1C),
        element_resistance: country_i32_at(&bytes, 0x20),
        name,
        script,
    })
}

fn read_country_area(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CountryAreaState, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x14>(source, cursor, "tagArea scalar block")?;
    Ok(CountryAreaState {
        id: country_i32_at(&bytes, 0x00),
        left: country_i32_at(&bytes, 0x04),
        top: country_i32_at(&bytes, 0x08),
        right: country_i32_at(&bytes, 0x0C),
        bottom: country_i32_at(&bytes, 0x10),
    })
}

fn read_country_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, RegionDecodeInputBlock> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| RegionDecodeInputBlock::UnexpectedEnd { field, offset: block.offset, needed: 4, available: block.available })?;
    let value = reader.read_i32().map_err(|block| RegionDecodeInputBlock::UnexpectedEnd { field, offset: block.offset, needed: block.needed, available: block.available })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_country_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, RegionDecodeInputBlock> {
    const LEGACY_CAPACITY: usize = 0x100;

    let mut value = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = LegacyReader::at(source, offset).map_err(|block| RegionDecodeInputBlock::UnexpectedEnd { field, offset: block.offset, needed: 1, available: block.available })?;
        let byte = reader.read_u8().map_err(|block| RegionDecodeInputBlock::UnexpectedEnd { field, offset: block.offset, needed: block.needed, available: block.available })?;
        *cursor = reader.position();
        if value.len() == LEGACY_CAPACITY {
            // BLOCKED_MISSING_FACT: byte уже потреблён перед первой записью за
            // legacy `char[256]`; эффект повреждения stack неизвестен.
            return Err(RegionDecodeInputBlock::LegacyStringOverflow {
                field,
                first_out_of_bounds_offset: offset,
            });
        }
        if byte == 0 {
            return Ok(value);
        }
        value.push(byte);
    }
}

fn country_i32_at<const N: usize>(bytes: &[u8; N], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("фиксированный country gate block содержит поле")
}

fn country_gate_count_field(camp: i32) -> &'static str {
    match camp {
        WC_DEFEND => "m_DefendGates count",
        WC_ATTACK => "m_AttackGates count",
        _ => "invalid country gate camp count",
    }
}

fn country_flag_count_field(camp: i32) -> &'static str {
    match camp {
        WC_DEFEND => "m_DefendFlags count",
        WC_ATTACK => "m_AttackFlags count",
        _ => "invalid country flag camp count",
    }
}

fn country_area_count_field(camp: i32) -> &'static str {
    match camp {
        WC_DEFEND => "m_DefendArea count",
        WC_ATTACK => "m_AttackArea count",
        _ => "invalid country area camp count",
    }
}

pub(crate) fn is_player_contend_symbol(
    contenders: &[ContendState],
    player_id: i32,
    symbol_id: i32,
) -> bool {
    contenders
        .iter()
        .find(|contender| contender.player_id == player_id)
        .is_some_and(|contender| contender.id == symbol_id)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp

// ============================================================================
// FUNCTION: _FactionNpcName::~_FactionNpcName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x000A5EA0
// ADDRESS: 004a5ea0
// PROTOTYPE: void __thiscall ~_FactionNpcName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: _FactionNpcName::_FactionNpcName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x000A6390
// ADDRESS: 004a6390
// PROTOTYPE: undefined __thiscall _FactionNpcName(_FactionNpcName * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerCountryRegion::UpdateContentTime
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:885
// RVA: 0x001CA980
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::SetEnterPosXY
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:156
// RVA: 0x001CA9F0
//
// Реализовано выше: fight gate, virtual return-point, same-region comparison,
// random range и player SetPos имеют статус VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::OnDeclareBegin
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:566
// RVA: 0x001CABC0
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::OnDeclareEnd
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:580
// RVA: 0x001CABE0
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::OnPrepareBegin
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:593
// RVA: 0x001CAC00
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::OnPrepareEnd
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:606
// RVA: 0x001CAC20
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::OnStart
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:619
// RVA: 0x001CAC40
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::OnTimeOut
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:634
// RVA: 0x001CAC60
//
// Реализовано выше в связной country phase chain.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::UpdateCityGateToClient
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CAC80
//
// Реализовано выше: pointer-overload, structured 0xBF60F и send-around; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::OnFlagDestroy
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:1124
// RVA: 0x001CAD50
//
// Реализовано выше в связной country victory chain.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::GetSecurity
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:1131
// RVA: 0x001CAD70
//
// Реализовано выше в общей virtual-family region security; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::OperatorCityGate
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CADD0
//
// Реализовано выше: pointer-overload: open/close/refresh и успешный no-op остальных операций; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::OnPlayerDamage
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:794
// RVA: 0x001CAFF0
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::DecContendTime
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:820
// RVA: 0x001CB0D0
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::OperatorCityGate
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CB1E0
//
// Реализовано выше: runtime-ID/camp lookup возвращает точный bool pointer-overload; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::GetCityGateState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CB250
//
// Реализовано выше: runtime-ID/camp lookup и action -> state; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::UpdateCityGateToClient
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CB310
//
// Реализовано выше: runtime-ID/camp lookup и pointer update; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::RefreshGuard
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:423
// RVA: 0x001CB370
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::IsWinSymbol
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:1004
// RVA: 0x001CB710
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::RefreshGates
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CB750
//
// Реализовано выше: defend затем attack, map-order, HP/action/update; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::RefreshFlags
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CB880
//
// Реализовано выше: defend затем attack, map-order, HP=max и 0xBF60F; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::ClearRegion
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CBA50
//
// Реализовано выше: gates, flags, guard, затем base kick-out; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::AddGurdMonster
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:387
// RVA: 0x001CC960
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::DelGurdMonster
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:397
// RVA: 0x001CC9F0
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::AddGuardIndex
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:405
// RVA: 0x001CCA30
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::CancelContendByPlayer
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:842
// RVA: 0x001CCAC0
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::OnEnterContend
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:938
// RVA: 0x001CCB50
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::~ServerCountryRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:21
// RVA: 0x001CD150
// ADDRESS: 005cd150
// PROTOTYPE: void __thiscall ~ServerCountryRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerCountryRegion::DecordFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CD3F0
//
// Реализовано выше: base, gates, flags и areas в точном wire-order; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::GetReturnPoint
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp
// RVA: 0x001CE010
//
// Реализовано выше: fight-state country area либо base fallback; VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: ServerCountryRegion::AddContend
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:757
// RVA: 0x001CE190
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::CancelContendBySymbol
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:863
// RVA: 0x001CE2B0
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::AI
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:1014
// RVA: 0x001CE380
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::GetCamp
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:1058
// RVA: 0x001CE540
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::ServerCountryRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:17
// RVA: 0x001CE5D0
// ADDRESS: 005ce5d0
// PROTOTYPE: undefined __thiscall ServerCountryRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerCountryRegion::GateIsAttackAble
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:487
// RVA: 0x001CE830
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::FlagIsAttackAble
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:513
// RVA: 0x001CE8D0
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::GuardIsAttackAble
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:539
// RVA: 0x001CE970
//
// Реализовано выше в связной country battle-side цепочке.
//
// ============================================================================
// FUNCTION: ServerCountryRegion::OnContendTimeOver
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:892
// RVA: 0x001CEA10
//
// Реализовано выше в связной country contender chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: ServerCountryRegion::IsPlayerContendSymbol
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercountryregion.cpp:743
// RVA: 0x001D24E0
//
// IMPLEMENTED выше: ищется первая запись player-а; результат сравнивает только
// её symbol ID, list miss возвращает false.

// COMPONENT_VARIANT_END: GameServer
