//! GameServer-владелец общей country-region поверхности `ServerCountryRegion`.
//!
//! Скалярный state (symbol ownership, area-maps, guard sets, стороны, фазовые
//! флаги), данные, context-контракты и скалярные правила перенесены в Zone
//! `regions/servercountryregion` (волна Z-M-X, семья регионов
//! country+nation+city + гейты); региональное гейтовое тело — opcode
//! константы, проекция client state, правила операций и общий footprint scan —
//! перенесено в Zone `regions/citygate`. Здесь hub-обёртка
//! `CServerCountryRegion` поверх `CServerRegion`: contender-список и карты
//! concrete gates/flags остаются здесь (element `ContendState` и gate/flag
//! типы — hub), scalar-решения делегируются Zone-агрегату `inner`, прежние
//! сигнатуры не менялись; wire-stream readers c `RegionDecodeInputBlock`,
//! re-export семейства для старого пакета и evidence-блок остаются здесь.
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
//! меняет только HP до обязательного `0xBF60F`. Gate/flag block применяет
//! собственный base-region, поэтому startup decoder не требует process-side
//! build owner-а. `ClearRegion` возвращает ordered build publications и
//! canonical `CGame` отправляет их после возврата owner-а. Guard spawn targets
//! сохраняют исходный defend/attack camp; общий `CGame` adapter применяет
//! созданные monster ID/index и fresh-entry effects после возврата owner-а.
//! Area lookup сохраняет
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
//! out-of-bounds дают `SAFE=2`, true возвращает packed base security;
//! несогласованный cell storage остаётся отдельной typed-границей.
//!
//! Writer `CountryWarSys::update_apply_war` записывает defend/attack до
//! `UpdateContendPlayer`, а phase chain материализует declare/prepare/war
//! callbacks и clear. Victory callback `OnFlagDestroy` RVA `0x001CAD50`
//! независимо от исходного war-byte оставляет его false при совпавшем region
//! ID; переданный country long не читает. Constructor RVA `0x001CE5D0` не
//! инициализирует эти два `long` и три phase bool; Rust безопасно задаёт
//! обязательные нейтральные `0/false` до доказанных writer/callback-ов, поэтому
//! достигнутые reader-ы не вводят недостижимую Option/error-границу.
//! Точный EXE подтвердил исходную странность `OnPrepareBegin/End`: оба проверяют
//! `_state_prepare`, но меняют `_state_declare`; безопасный исходный gate закрыт.
//! `CancelContendByPlayer` содержит отдельный
//! исходный дефект: non-null player немедленно получает `false`, а null-ветка
//! читает absolute `0x8` и вызывает метод с null; safe Rust не придумывает ей
//! результат. Отдельные тела STL collection internals, `Catch/Unwind` и
//! deleting-destructor thunk сняты как единая доказанная техническая группа;
//! их наблюдаемые call-site contracts сохранены, доменные constructors,
//! destructors и оставшиеся callbacks не классифицировались этим sweep.

pub(crate) use nebokrai_zone::regions::servercountryregion::*;

use std::collections::BTreeMap;

use super::build::{
    BuildBlockUpdate, BuildClientPublication, BuildClientUpdate, BuildInit, CBuild,
};
use super::citygate::{
    CCityGate, CityGateInit, GATE_OP_CLOSE, GATE_OP_REFRESH, GateOperationUpdate,
    country_gate_operation_update, gate_client_state,
};
use super::country::countryparam::CCountryParam;
use nebokrai_shared::protocol::LegacyReader;
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionRandomPosition, RegionReturnPoint,
    RegionSecurity,
};
use super::servercityregion::city_gate_footprint_is_clear;
use super::serverregion::{
    CServerRegion, ServerRegionDecodeContext, ServerRegionDecodeError,
    ServerRegionMonsterRectBlock, ServerReturnPlayer,
};
use super::serverwarregion::{
    ContendArithmeticBlock, ContendState, RegionDecodeInputBlock, read_region_array,
};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryRegionDecodeError<BaseError> {
    Base(BaseError),
    Input(RegionDecodeInputBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryRegionAiError {
    Base(ServerRegionMonsterRectBlock),
    Arithmetic(ContendArithmeticBlock),
}

pub(crate) trait CountryRegionDecodeContext: ServerRegionDecodeContext {}

impl<Context: ServerRegionDecodeContext + ?Sized> CountryRegionDecodeContext for Context {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryDamageError {
    X87(CountryDamageArithmeticBlock),
    Percentage(ContendArithmeticBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CServerCountryRegion {
    pub(crate) base: CServerRegion,
    pub(crate) contenders: Vec<ContendState>,
    pub(crate) defend_gates: BTreeMap<i32, CCityGate>,
    pub(crate) attack_gates: BTreeMap<i32, CCityGate>,
    pub(crate) defend_flags: BTreeMap<i32, CBuild>,
    pub(crate) attack_flags: BTreeMap<i32, CBuild>,
    inner: ServerCountryRegionState,
}

impl Default for CServerCountryRegion {
    fn default() -> Self {
        Self {
            base: CServerRegion::default(),
            contenders: Vec::new(),
            defend_gates: BTreeMap::new(),
            attack_gates: BTreeMap::new(),
            defend_flags: BTreeMap::new(),
            attack_flags: BTreeMap::new(),
            inner: ServerCountryRegionState::default(),
        }
    }
}

impl CServerCountryRegion {
    pub(crate) fn decord_from_byte_array<Context: CountryRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, CountryRegionDecodeError<ServerRegionDecodeError>>
    {
        let _ = self
            .base
            .decord_from_byte_array(
                source,
                cursor,
                include_child,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
            )
            .map_err(CountryRegionDecodeError::Base)?;
        self.decode_gate_section(source, cursor, COUNTRY_CAMP_DEFEND, area_width, area_height)?;
        self.decode_gate_section(source, cursor, COUNTRY_CAMP_ATTACK, area_width, area_height)?;
        self.decode_flag_section(source, cursor, COUNTRY_CAMP_DEFEND, area_width, area_height)?;
        self.decode_flag_section(source, cursor, COUNTRY_CAMP_ATTACK, area_width, area_height)?;
        self.decode_area_section(source, cursor, COUNTRY_CAMP_DEFEND)?;
        self.decode_area_section(source, cursor, COUNTRY_CAMP_ATTACK)?;
        Ok(true)
    }

    pub(crate) fn operator_city_gate(
        &mut self,
        city_gate_id: i32,
        operation: i32,
        camp: i32,
    ) -> bool {
        let Some(gate) = self.gates(camp).and_then(|gates| gates.get(&city_gate_id)) else {
            return false;
        };
        if operation == GATE_OP_CLOSE && !city_gate_footprint_is_clear(&self.base, gate) {
            return false;
        }
        let gate = self
            .gates_mut(camp)
            .and_then(|gates| gates.get_mut(&city_gate_id))
            .expect("gate найден до неизменяющего map вызова");
        let update = apply_gate_operation(gate, country_gate_operation_update(operation));
        if let Some(update) = update {
            let _legacy_void = self.base.apply_build_block(update);
        }
        true
    }

    pub(crate) fn get_city_gate_state(&self, city_gate_id: i32, camp: i32) -> i32 {
        let Some(gate) = self.gates(camp).and_then(|gates| gates.get(&city_gate_id)) else {
            return -1;
        };
        gate_client_state(gate.action())
    }

    pub(crate) fn refresh_gates(&mut self) -> Vec<BuildClientPublication> {
        let region_id = self.base.id;
        let mut updates = Vec::new();
        let defend_ids: Vec<_> = self.defend_gates.keys().copied().collect();
        for gate_id in defend_ids {
            let update = apply_gate_operation(
                self.defend_gates
                    .get_mut(&gate_id)
                    .expect("snapshot построен из defend gate map"),
                country_gate_operation_update(GATE_OP_REFRESH),
            );
            if let Some(update) = update {
                let _legacy_void = self.base.apply_build_block(update);
            }
            updates.push(city_gate_publication(
                region_id,
                self.defend_gates
                    .get(&gate_id)
                    .expect("defend gate не удаляется во время refresh"),
            ));
        }
        let attack_ids: Vec<_> = self.attack_gates.keys().copied().collect();
        for gate_id in attack_ids {
            let update = apply_gate_operation(
                self.attack_gates
                    .get_mut(&gate_id)
                    .expect("snapshot построен из attack gate map"),
                country_gate_operation_update(GATE_OP_REFRESH),
            );
            if let Some(update) = update {
                let _legacy_void = self.base.apply_build_block(update);
            }
            updates.push(city_gate_publication(
                region_id,
                self.attack_gates
                    .get(&gate_id)
                    .expect("attack gate не удаляется во время refresh"),
            ));
        }
        updates
    }

    /// Собственная refresh-часть country `ClearRegion`; следующий
    /// `KickOutAllPlayerToReturnPoint` исполняет владеющий картой игроков
    /// `CGame`, чтобы смена региона не уходила во внешний callback.
    pub(crate) fn refresh_for_clear(&mut self) -> CountryClearRefreshEffects {
        let mut build_updates = self.refresh_gates();
        build_updates.extend(self.refresh_flags());
        CountryClearRefreshEffects {
            guard_targets: self.inner.guard_refresh_targets(),
            build_updates,
        }
    }

    pub(crate) fn refresh_flags(&mut self) -> Vec<BuildClientPublication> {
        let region_id = self.base.id;
        let mut updates = Vec::new();
        for flag in self.defend_flags.values_mut() {
            updates.push(refresh_country_flag_object(region_id, flag));
        }
        for flag in self.attack_flags.values_mut() {
            updates.push(refresh_country_flag_object(region_id, flag));
        }
        updates
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

        // Нулевые стороны заданы safe constructor-ом до первого
        // `CountryWarSys::update_apply_war`; exact ctor оставлял здесь UB.
        match self
            .inner
            .war_return_point(self.base.id, player.country, context)
        {
            Some(point) => Ok(point),
            None => Ok(self.base.region.get_return_point()),
        }
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
        let Some(area) = self.inner.war_entry_area(camp, context) else {
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
        if !self.inner.war_active() {
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
        self.inner.is_win_symbol(country, symbol_id)
    }

    pub(crate) fn cancel_contend_by_player(
        &mut self,
        player: Option<&CountryContendPlayer>,
    ) -> Result<bool, CountryNullPlayerCancelBlock> {
        decide_cancel_contend_by_player(player.is_some())
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
        self.inner.capture_symbol(contender.id, i32::from(country));
        context.on_country_win_one_symbol(i32::from(country), contender.id);
        context.send_country_symbol_captured_region_notice(country, &contender.name);
        context.send_country_symbol_captured_top_info(country, &self.base.name, &contender.name);
    }

    /// Writer-side region projection: defend записывается раньше attack.
    pub(crate) fn set_country_sides(&mut self, defend_country: i32, attack_country: i32) {
        self.inner.set_country_sides(defend_country, attack_country);
    }

    /// Country vtable `0x65D464`, slot `+0x104`, указывает на единственный
    /// `ret` по `0x485540`: после записи сторон эта разновидность региона не
    /// фильтрует contender-ов, в отличие от city/village war owners.
    pub(crate) const fn update_contend_player(&mut self) {
        self.inner.update_contend_player();
    }

    pub(crate) const fn country_side_bytes(&self) -> (u8, u8) {
        self.inner.country_side_bytes()
    }

    pub(crate) fn on_declare_begin(&mut self, region_id: i32) {
        self.inner.on_declare_begin(region_id, self.base.id);
    }

    pub(crate) fn on_declare_end(&mut self, region_id: i32) {
        self.inner.on_declare_end(region_id, self.base.id);
    }

    pub(crate) fn on_prepare_begin(&mut self, region_id: i32) {
        self.inner.on_prepare_begin(region_id, self.base.id);
    }

    pub(crate) fn on_prepare_end(&mut self, region_id: i32) {
        self.inner.on_prepare_end(region_id, self.base.id);
    }

    pub(crate) fn on_war_start(&mut self, region_id: i32) {
        self.inner.on_war_start(region_id, self.base.id);
    }

    pub(crate) fn on_war_timeout(&mut self, region_id: i32) {
        self.inner.on_war_timeout(region_id, self.base.id);
    }

    pub(crate) fn on_war_end(&mut self, region_id: i32) {
        // VERIFIED_DISASSEMBLY: vtable `+0x150` совпадает с `OnTimeOut`
        // `+0x14C` и указывает на одно RVA `0x001CAC60`.
        self.inner.on_war_end(region_id, self.base.id);
    }

    pub(crate) fn on_flag_destroy(&mut self, region_id: i32, _country: i32) {
        self.inner.on_flag_destroy(region_id, self.base.id);
    }

    pub(crate) fn add_gurd_monster(&mut self, monster_id: i32, camp: i32) {
        self.inner.add_gurd_monster(monster_id, camp);
    }

    pub(crate) fn del_gurd_monster(&mut self, monster_id: i32, camp: i32) {
        self.inner.del_gurd_monster(monster_id, camp);
    }

    pub(crate) fn add_guard_index(&mut self, spawn_index: i32, camp: i32) {
        self.inner.add_guard_index(spawn_index, camp);
    }

    pub(crate) fn guard_refresh_targets(&self) -> CountryGuardRefreshTargets {
        self.inner.guard_refresh_targets()
    }

    pub(crate) fn get_camp<Context: CountryCampContext>(
        &self,
        player_id: i32,
        context: &mut Context,
    ) -> i32 {
        self.inner.get_camp(player_id, context)
    }

    pub(crate) fn gate_is_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        context: &mut Context,
    ) -> bool {
        self.is_collection_attack_able(target, attacker, CountryTargetKind::Gate, context)
    }

    pub(crate) fn flag_is_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        context: &mut Context,
    ) -> bool {
        self.is_collection_attack_able(target, attacker, CountryTargetKind::Flag, context)
    }

    pub(crate) fn guard_is_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        context: &mut Context,
    ) -> bool {
        self.is_collection_attack_able(target, attacker, CountryTargetKind::Guard, context)
    }

    fn is_collection_attack_able<Context: CountryCampContext>(
        &self,
        target: Option<CountryMoveShape>,
        attacker: Option<CountryMoveShape>,
        kind: CountryTargetKind,
        context: &mut Context,
    ) -> bool {
        let (Some(target), Some(attacker)) = (target, attacker) else {
            return false;
        };
        if attacker.object_type != 400 {
            return false;
        }
        // Safe constructor задаёт закрытую фазу до первого OnStart; exact ctor
        // оставлял byte неинициализированным.
        if !self.inner.war_active() {
            return false;
        }

        let camp = self.get_camp(attacker.id, context);
        match camp {
            COUNTRY_CAMP_DEFEND => self.target_collection_contains(COUNTRY_CAMP_ATTACK, kind, target.id),
            COUNTRY_CAMP_ATTACK => self.target_collection_contains(COUNTRY_CAMP_DEFEND, kind, target.id),
            _ => false,
        }
    }

    fn target_collection_contains(&self, camp: i32, kind: CountryTargetKind, id: i32) -> bool {
        match (camp, kind) {
            (COUNTRY_CAMP_DEFEND, CountryTargetKind::Gate) => self.defend_gates.contains_key(&id),
            (COUNTRY_CAMP_ATTACK, CountryTargetKind::Gate) => self.attack_gates.contains_key(&id),
            (COUNTRY_CAMP_DEFEND, CountryTargetKind::Flag) => self.defend_flags.contains_key(&id),
            (COUNTRY_CAMP_ATTACK, CountryTargetKind::Flag) => self.attack_flags.contains_key(&id),
            (_, CountryTargetKind::Guard) => self.inner.guard_collection_contains(camp, id),
            _ => false,
        }
    }

    fn decode_gate_section(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        camp: i32,
        area_width: i32,
        area_height: i32,
    ) -> Result<(), CountryRegionDecodeError<ServerRegionDecodeError>> {
        let count = read_country_i32(source, cursor, country_gate_count_field(camp))
            .map_err(CountryRegionDecodeError::Input)?;
        for _ in 0..count.max(0) {
            let build =
                read_country_gate_build(source, cursor).map_err(CountryRegionDecodeError::Input)?;
            self.add_country_gate(camp, build, area_width, area_height);
        }
        Ok(())
    }

    fn add_country_gate(
        &mut self,
        camp: i32,
        build: CountryGateBuild,
        area_width: i32,
        area_height: i32,
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
        if self
            .base
            .register_stationary_child(gate.shape_view(), area_width, area_height)
            .is_err()
        {
            return None;
        }
        let _legacy_void = self.base.apply_build_block(gate.current_block_update());
        // VERIFIED_DISASSEMBLY: country map key — `CCityGate::m_lID +8`, не
        // `tagGate.field_00`/logical ID, который использует city-owner.
        self.gates_mut(camp)
            .expect("decoder передаёт только доказанный camp")
            .insert(city_gate_id, gate);
        Some(city_gate_id)
    }

    fn decode_flag_section(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        camp: i32,
        area_width: i32,
        area_height: i32,
    ) -> Result<(), CountryRegionDecodeError<ServerRegionDecodeError>> {
        let count = read_country_i32(source, cursor, country_flag_count_field(camp))
            .map_err(CountryRegionDecodeError::Input)?;
        for _ in 0..count.max(0) {
            let build =
                read_country_flag_build(source, cursor).map_err(CountryRegionDecodeError::Input)?;
            self.add_country_flag(camp, build, area_width, area_height);
        }
        Ok(())
    }

    fn add_country_flag(
        &mut self,
        camp: i32,
        build: CountryFlagBuild,
        area_width: i32,
        area_height: i32,
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
        if self
            .base
            .register_stationary_child(flag.shape_view(), area_width, area_height)
            .is_err()
        {
            return None;
        }
        let _legacy_void = self.base.apply_build_block(flag.current_block_update());
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
            self.inner.insert_decoded_area(camp, area);
        }
        Ok(())
    }

    fn gates(&self, camp: i32) -> Option<&BTreeMap<i32, CCityGate>> {
        match camp {
            COUNTRY_CAMP_DEFEND => Some(&self.defend_gates),
            COUNTRY_CAMP_ATTACK => Some(&self.attack_gates),
            _ => None,
        }
    }

    fn gates_mut(&mut self, camp: i32) -> Option<&mut BTreeMap<i32, CCityGate>> {
        match camp {
            COUNTRY_CAMP_DEFEND => Some(&mut self.defend_gates),
            COUNTRY_CAMP_ATTACK => Some(&mut self.attack_gates),
            _ => None,
        }
    }

    fn flags_mut(&mut self, camp: i32) -> Option<&mut BTreeMap<i32, CBuild>> {
        match camp {
            COUNTRY_CAMP_DEFEND => Some(&mut self.defend_flags),
            COUNTRY_CAMP_ATTACK => Some(&mut self.attack_flags),
            _ => None,
        }
    }
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

fn apply_gate_operation(
    gate: &mut CCityGate,
    rule: GateOperationUpdate,
) -> Option<BuildBlockUpdate> {
    if rule.refresh_hp {
        gate.refresh_hp();
    }
    rule.next_action.and_then(|action| gate.set_action(action))
}

fn city_gate_publication(region_id: i32, gate: &CCityGate) -> BuildClientPublication {
    BuildClientPublication {
        region_id,
        build_id: gate.id(),
        update: BuildClientUpdate {
            object_type: gate.object_type(),
            object_id: gate.id() as u32,
            action: gate.action(),
            max_hp: gate.max_hp(),
            hp: gate.hp(),
        },
    }
}

fn refresh_country_flag_object(
    region_id: i32,
    flag: &mut CBuild,
) -> BuildClientPublication {
    flag.refresh_hp();
    BuildClientPublication {
        region_id,
        build_id: flag.id(),
        update: BuildClientUpdate {
            object_type: flag.object_type(),
            object_id: flag.id() as u32,
            action: flag.action(),
            max_hp: flag.max_hp(),
            hp: flag.hp(),
        },
    }
}

fn read_country_gate_build(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CountryGateBuild, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x2C>(source, cursor, "tagGate scalar block")?;
    let name = read_country_c_string(source, cursor, "tagGate.strName")?;
    let script = read_country_c_string(source, cursor, "tagGate.strScript")?;
    Ok(country_gate_build_from_block(&bytes, name, script))
}

fn read_country_flag_build(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CountryFlagBuild, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x28>(source, cursor, "tagFlag scalar block")?;
    let name = read_country_c_string(source, cursor, "tagFlag.strName")?;
    let script = read_country_c_string(source, cursor, "tagFlag.strScript")?;
    Ok(country_flag_build_from_block(&bytes, name, script))
}

fn read_country_area(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CountryAreaState, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x14>(source, cursor, "tagArea scalar block")?;
    Ok(country_area_state_from_block(&bytes))
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
