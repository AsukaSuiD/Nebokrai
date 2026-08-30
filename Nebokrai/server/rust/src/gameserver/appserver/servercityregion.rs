//! Concrete GameServer-владелец городского war-region `CServerCityRegion`.
//!
//! Фазовые callbacks RVA `0x001CF730`, `0x001CFA00..0x001CFD70`,
//! `0x001D09F0`, ownership `0x001CED70/0x001CEF40`, victory `0x001CF1A0`,
//! spatial `SetEnterPosXY/GetReturnPoint` `0x001CEE80/0x001CEF60`, virtual
//! security `0x001CF0E0`,
//! decoder `0x001D10B0`, `AddCityGate` `0x001D0F00`, gate runtime
//! `0x001CAAA0/0x001CF370..0x001CF640`, clear `0x001CF970`, guard refresh
//! `0x001CF7C0` и direct timeout-forwarding `0x001CFEB0` имеют статус
//! `IMPLEMENTED`; gate IDs, message layout, decoder/factory returns и фазовый
//! call order, child-ID `+0x158` и base-region registration также
//! `VERIFIED_DISASSEMBLY`. Исходники
//! `servercityregion.h/.cpp`, точная пара GameServer. PDB подтверждает
//! наследование `CServerWarRegion`, ordered
//! `m_CityGates +0x274`, guard set/list `+0x2A0/+0x2AC` и defender faction
//! `+0x2C0`.
//!
//! `BTreeMap/BTreeSet/Vec` сохраняют STL order. Decoder принимает byte-exact
//! World snapshot: `0x20` defence, signed count, `0x2C` gate scalars и две
//! C-строки в старых `char[256]`; missing NUL/overflow остаются локальными
//! safe-блоками. Exact EXE подтвердил normal `true`, layout gate-полей и
//! pointer-return factory. Gate map хранит logical/runtime IDs, имя и concrete
//! `CCityGate`; safe owner сам выдаёт legacy child-ID и регистрирует тот же
//! gate в base region/area. Initial и action-dependent block сразу меняет
//! owning `CServerRegion`; closing footprint читает ту же карту напрямую, а
//! runtime context оставляет только клиентскую публикацию.
//! Inherited `CServerWarRegion::AI` вызывается реальным `CGame::AI` через City
//! adapter с weekly membership, defender/owner и network/log effects.
//! Rust name/ID queries используют `Option` вместо pointer/sentinel формы и не
//! декодируют исходные name bytes.
//! Spatial override сохраняет два state-read, defender-only return setup,
//! fallback в базовый/country owner, same-region gate, игнорирование random
//! bool и финальный player position effect. Два отброшенных `GetTileY/X`
//! вызова остаются в context именно в исходном порядке: EXE показывает, что
//! это virtual float getters с x87 conversion, а не простые field reads.
//! `GetSecurity` сначала возвращает `SAFE=2` для mass-state `2`; иначе читает
//! packed cell security, сохраняет отдельный второй state-read и только при
//! fight-state `3` плюс war-marker `1` возвращает `CITYWAR=3`. Out-of-bounds
//! остаётся safe, а несогласованный cell storage делегируется typed-границе
//! `CRegion`. Самостоятельные STL/Catch/Unwind/deleting-thunk экспорты сняты
//! общей технической классификацией; доменные callbacks и lifecycle не
//! классифицировались этим sweep.
//! Timeout агрегирует владельцев
//! symbols по faction ID и выбирает первый достаточный ID в map-order; если
//! победителя нет, сохраняется действующий owner faction/union. Guard refresh
//! возвращает ordered monster/spawn snapshot, который `CGame/CMonster`
//! исполняют через канонические region и wire owners. Message, localized
//! `OnClearOtherPlayer` возвращает ordered player snapshot владельцу `CGame`,
//! который применяет точный faction-фильтр и обычную смену региона до
//! принадлежащего city-owner-у прохода ворот. Message, localized war-log и
//! остальные player-transition эффекты остаются точным context-контрактом.
//! Прямой `OnWinSymbol` внутри `OnFactionVictory` у этой сборки указывает на
//! точный no-op `0x004A8750`; Rust не сохраняет для него фиктивный callback.
//! Timeout возвращает victory/log snapshot: `CGame` шлёт `0x60138` после
//! ownership mutation и до сохранённого `GS0223/GS0224` war-log sink.
//! Остальная поверхность файла ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).

use std::collections::{BTreeMap, BTreeSet};

use super::build::{BuildBlockUpdate, BuildClientUpdate, BuildRuntimeContext};
use super::citygate::{CCityGate, CityGateInit};
use super::country::countryparam::CCountryParam;
use super::legacycodec::LegacyReader;
use super::organizingsystem::attackcitysys::{AttackCityMembershipBlock, CAttackCitySys};
use super::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionRandomPosition, RegionReturnPoint,
    RegionSecurity,
};
use super::serverregion::{
    CServerRegion, ServerRegionDecodeError, ServerReturnPlayer, ServerReturnSetupBlock,
};
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::serverwarregion::{
    CServerWarRegion, ContendState, RegionDecodeInputBlock, WarContendContext, WarRegionContext,
    WarRegionDecodeContext, WarRegionDecodeError, read_region_array,
};

const OC_OPEN: i32 = 0;
const OC_CLOSE: i32 = 1;
const OC_REFRESH: i32 = 2;
const OC_DIED: i32 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CityGateState {
    pub(crate) logical_id: i32,
    pub(crate) city_gate_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) gate: CCityGate,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CityDefenceReturnState {
    pub(crate) region_id: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) does_recall_when_lost: i32,
    pub(crate) move_monster_when_refeash: i32,
    pub(crate) use_return: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CityGateBuild {
    pub(crate) logical_id: i32,
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
pub(crate) enum CityRegionDecodeError<BaseError> {
    War(WarRegionDecodeError<BaseError>),
    Input(RegionDecodeInputBlock),
}

pub(crate) trait CityRegionDecodeContext: WarRegionDecodeContext {}

impl<Context: WarRegionDecodeContext + ?Sized> CityRegionDecodeContext for Context {}

pub(crate) trait CityGateRuntimeContext: BuildRuntimeContext {}

impl<Context: BuildRuntimeContext + ?Sized> CityGateRuntimeContext for Context {}

pub(crate) trait CityReturnPointContext {
    /// Сохраняет первый отброшенный virtual `CShape::GetTileY`.
    fn read_city_player_tile_y(&mut self, player_id: i32) -> i32;

    /// Сохраняет следующий отброшенный virtual `CShape::GetTileX`.
    fn read_city_player_tile_x(&mut self, player_id: i32) -> i32;
}

pub(crate) trait CityEntryContext: CityReturnPointContext + RegionRandomContext {
    /// Выполняет virtual player slot `+0x88` с `(x, y)`.
    fn set_city_player_position(&mut self, player_id: i32, x: i32, y: i32);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CityReturnPointError {
    DefenceSetup,
    Base(ServerReturnSetupBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CityEntryError {
    ReturnPoint(CityReturnPointError),
    Cell(RegionCellAccessBlock),
}

pub(crate) trait CityRegionContext: WarRegionContext + CityGateRuntimeContext {
    /// Пишет localized template в канал `war` с аргументами `(war, region name)`.
    fn write_war_log(&mut self, string_id: &'static str, war_number: i32, region_name: &str);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CityVictoryUpdate {
    pub(crate) war_number: i32,
    pub(crate) region_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) union_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CityWarTimeoutEffect {
    pub(crate) victory: Option<CityVictoryUpdate>,
    pub(crate) log_string_id: &'static str,
    pub(crate) war_number: i32,
    pub(crate) region_name: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CityGuardRefreshTargets {
    pub(crate) monster_ids: Vec<i32>,
    pub(crate) spawn_indices: Vec<i32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerCityRegion {
    pub(crate) war: CServerWarRegion,
    pub(crate) city_gates: BTreeMap<i32, CityGateState>,
    pub(crate) defence_side_return: Option<CityDefenceReturnState>,
    pub(crate) guard_monsters: BTreeSet<i32>,
    pub(crate) guard_indices: Vec<i32>,
    pub(crate) defence_side_faction_id: i32,
}

impl CServerCityRegion {
    pub(crate) fn get_return_point<Context: CityReturnPointContext>(
        &self,
        player: Option<ServerReturnPlayer>,
        country_param: &mut CCountryParam,
        context: &mut Context,
    ) -> Result<RegionReturnPoint, CityReturnPointError> {
        let Some(player) = player else {
            return Ok(self.war.base.region.get_return_point());
        };

        let _ = context.read_city_player_tile_y(player.id);
        let _ = context.read_city_player_tile_x(player.id);
        let first_state = self.war.base.get_city_state();
        let defender_window = first_state == 3 || self.war.base.get_city_state() == 2;
        if defender_window
            && player.faction_id != 0
            && player.faction_id == self.defence_side_faction_id
        {
            let setup = self
                .defence_side_return
                .ok_or(CityReturnPointError::DefenceSetup)?;
            return Ok(RegionReturnPoint {
                region_id: setup.region_id,
                left: setup.left,
                top: setup.top,
                right: setup.right,
                bottom: setup.bottom,
                direction: -1,
            });
        }

        self.war
            .base
            .get_return_point(Some(player), country_param)
            .map_err(CityReturnPointError::Base)
    }

    pub(crate) fn set_enter_pos_xy<Context: CityEntryContext>(
        &self,
        player: ServerReturnPlayer,
        country_param: &mut CCountryParam,
        context: &mut Context,
    ) -> Result<Option<RegionRandomPosition>, CityEntryError> {
        let first_state = self.war.base.get_city_state();
        if first_state != 3 && self.war.base.get_city_state() != 2 {
            return Ok(None);
        }

        let point = self
            .get_return_point(Some(player), country_param, context)
            .map_err(CityEntryError::ReturnPoint)?;
        if point.region_id != self.war.base.id {
            return Ok(None);
        }

        let position = self
            .war
            .base
            .region
            .get_random_pos_in_range(
                point.left,
                point.top,
                point.right.wrapping_sub(point.left),
                point.bottom.wrapping_sub(point.top),
                context,
            )
            .map_err(CityEntryError::Cell)?;
        context.set_city_player_position(player.id, position.x, position.y);
        Ok(Some(position))
    }

    pub(crate) fn decord_from_byte_array<Context: CityRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
    ) -> Result<bool, CityRegionDecodeError<ServerRegionDecodeError>> {
        let _ = self
            .war
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
            .map_err(CityRegionDecodeError::War)?;

        let defence = read_region_array::<0x20>(source, cursor, "m_DefenceSideRS")
            .map_err(CityRegionDecodeError::Input)?;
        self.defence_side_return = Some(decode_defence_return(defence));

        let gate_count = read_city_i32(source, cursor, "m_CityGates count")
            .map_err(CityRegionDecodeError::Input)?;
        for _ in 0..gate_count.max(0) {
            let build =
                read_city_gate_build(source, cursor).map_err(CityRegionDecodeError::Input)?;
            self.add_city_gate(build, area_width, area_height);
        }
        self.defence_side_faction_id = self.war.base.param.owned_faction_id;
        Ok(true)
    }

    pub(crate) fn add_city_gate(
        &mut self,
        build: CityGateBuild,
        area_width: i32,
        area_height: i32,
    ) -> Option<i32> {
        let city_gate_id = self.war.base.take_child_id();
        let gate = CCityGate::from_created(CityGateInit {
            id: city_gate_id,
            graphics_id: build.picture_id,
            region_id: self.war.base.id,
            name: build.name.clone(),
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
            .war
            .base
            .register_stationary_child(gate.shape_view(), area_width, area_height)
            .is_err()
        {
            return None;
        }
        let _legacy_void = self
            .war
            .base
            .apply_build_block(gate.current_block_update());
        self.city_gates.insert(
            build.logical_id,
            CityGateState {
                logical_id: build.logical_id,
                city_gate_id,
                name: build.name,
                gate,
            },
        );
        Some(city_gate_id)
    }

    pub(crate) fn on_contend_time_over<Context: WarContendContext>(
        &mut self,
        contender: &ContendState,
        context: &mut Context,
    ) -> Result<(), Context::MembershipError> {
        self.war.on_contend_time_over(contender, context)
    }

    pub(crate) fn is_owner(&self, faction_id: i32) -> bool {
        faction_id != 0 && faction_id == self.war.base.param.owned_faction_id
    }

    pub(crate) fn is_apply_war_faction(
        &self,
        schedules: &CAttackCitySys,
        faction_id: i32,
    ) -> Result<bool, AttackCityMembershipBlock> {
        schedules.is_already_declar_for_war(self.war.base.war_number, faction_id)
    }

    pub(crate) fn get_security(
        &self,
        x: i32,
        y: i32,
    ) -> Result<RegionSecurity, RegionCellAccessBlock> {
        if self.war.base.get_city_state() == 2 {
            return Ok(RegionSecurity::SAFE);
        }
        let Some(cell) = self.war.base.region.get_cell(x, y)? else {
            return Ok(RegionSecurity::SAFE);
        };
        let security = cell.security();
        if security == RegionSecurity::SAFE {
            return Ok(RegionSecurity::SAFE);
        }
        if self.war.base.get_city_state() == 3 && cell.city_war_marker() == 1 {
            return Ok(RegionSecurity::CITY_WAR);
        }
        Ok(security)
    }

    pub(crate) fn set_owned_city_org(&mut self, faction_id: i32, union_id: i32) {
        self.war.base.set_owned_city_org(faction_id, union_id);
        self.defence_side_faction_id = faction_id;
    }

    pub(crate) fn on_faction_win_one_symbol(&mut self, faction_id: i32, symbol_id: i32) {
        if symbol_id == 0 {
            self.defence_side_faction_id = faction_id;
        }
    }

    pub(crate) fn on_war_declare<Context: CityRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        self.war.on_war_declare(war_number);
        self.defence_side_faction_id = self.war.base.param.owned_faction_id;
        context.write_war_log("GS0221", war_number, &self.war.base.name);
    }

    pub(crate) fn on_war_start<Context: CityRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        if self.war.base.war_number != war_number {
            return;
        }
        self.war.base.on_war_start(war_number);
        context.write_war_log("GS0220", war_number, &self.war.base.name);
    }

    pub(crate) fn on_war_mass<Context: CityRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        if self.war.base.war_number != war_number {
            return;
        }
        self.war.base.on_war_mass(war_number);
        context.write_war_log("GS0222", war_number, &self.war.base.name);
    }

    pub(crate) fn on_war_time_out(&mut self, war_number: i32) -> Option<CityWarTimeoutEffect> {
        if self.war.base.war_number != war_number {
            return None;
        }

        let mut faction_symbols = BTreeMap::<i32, i32>::new();
        for &faction_id in self.war.faction_win_symbol.values() {
            let count = faction_symbols.entry(faction_id).or_default();
            *count = count.wrapping_add(1);
        }

        let winner = faction_symbols
            .iter()
            .find(|(_, count)| self.war.win_victory_symbol_num <= **count)
            .map(|(&faction_id, _)| faction_id);
        let (faction_id, union_id, log_string_id) = winner
            .map(|faction_id| (faction_id, 0, "GS0223"))
            .unwrap_or((
                self.war.base.param.owned_faction_id,
                self.war.base.param.owned_union_id,
                "GS0224",
            ));
        let victory = self.apply_faction_victory(faction_id, union_id);
        Some(CityWarTimeoutEffect {
            victory,
            log_string_id,
            war_number,
            region_name: self.war.base.name.clone(),
        })
    }

    pub(crate) fn on_war_end<Context: CityRegionContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        if self.war.base.war_number != war_number || self.war.base.city_state == 0 {
            return;
        }
        self.war.on_war_end(war_number);
        self.clear_region(context);
        context.write_war_log("GS0225", war_number, &self.war.base.name);
    }

    pub(crate) fn refresh_and_close_gates<Context: CityGateRuntimeContext>(
        &mut self,
        context: &mut Context,
    ) {
        let logical_ids: Vec<_> = self.city_gates.keys().copied().collect();
        for logical_id in logical_ids {
            let _ = self.operator_city_gate(logical_id, OC_REFRESH);
            let _ = self.operator_city_gate(logical_id, OC_CLOSE);
            self.update_city_gate_to_client(logical_id, context);
        }
    }

    pub(crate) fn on_refresh_region(&self, _war_number: i32) -> CityGuardRefreshTargets {
        self.guard_refresh_targets()
    }

    fn apply_faction_victory(
        &mut self,
        faction_id: i32,
        union_id: i32,
    ) -> Option<CityVictoryUpdate> {
        if self.war.base.city_state == 0 {
            return None;
        }
        self.set_owned_city_org(faction_id, union_id);
        Some(CityVictoryUpdate {
            war_number: self.war.base.war_number,
            region_id: self.war.base.id,
            faction_id,
            union_id,
        })
    }

    pub(crate) fn clear_region<Context: CityRegionContext>(&mut self, context: &mut Context) {
        self.war.clear_region(context);
        let logical_ids: Vec<_> = self.city_gates.keys().copied().collect();
        for logical_id in logical_ids {
            let _ = self.operator_city_gate(logical_id, OC_REFRESH);
            self.update_city_gate_to_client(logical_id, context);
        }
    }

    pub(crate) fn operator_city_gate(&mut self, logical_id: i32, operation: i32) -> bool {
        let Some(gate_state) = self.city_gates.get(&logical_id) else {
            return false;
        };

        let pointer_result = match operation {
            OC_OPEN => true,
            OC_CLOSE => city_gate_footprint_is_clear(&self.war.base, &gate_state.gate),
            OC_REFRESH | OC_DIED => true,
            _ => true,
        };

        if pointer_result {
            let gate = &mut self
                .city_gates
                .get_mut(&logical_id)
                .expect("gate найден до неизменяющего map вызова")
                .gate;
            let update = match operation {
                OC_OPEN => apply_gate_action(gate, 7),
                OC_CLOSE => apply_gate_action(gate, 1),
                OC_REFRESH => {
                    gate.refresh_hp();
                    apply_gate_action(gate, 7)
                }
                OC_DIED => apply_gate_action(gate, 6),
                _ => None,
            };
            if let Some(update) = update {
                let _legacy_void = self.war.base.apply_build_block(update);
            }
        }

        // RVA 0x001CF4C0 игнорирует bool pointer-overload: найденный map key
        // возвращает true даже когда OC_Close не прошёл проверку footprint.
        true
    }

    pub(crate) fn update_city_gate_to_client<Context: CityGateRuntimeContext>(
        &self,
        logical_id: i32,
        context: &mut Context,
    ) {
        let Some(gate) = self.city_gates.get(&logical_id).map(|state| &state.gate) else {
            return;
        };
        context.send_build_update(
            self.war.base.id,
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

    pub(crate) fn city_gate_is_close(&self, logical_id: i32) -> bool {
        let Some(gate) = self.city_gates.get(&logical_id) else {
            return false;
        };
        city_gate_footprint_is_clear(&self.war.base, &gate.gate)
    }

    pub(crate) fn get_city_gate_state(&self, logical_id: i32) -> i32 {
        let Some(gate) = self.city_gates.get(&logical_id).map(|state| &state.gate) else {
            return -1;
        };
        match gate.action {
            7 => 0,
            0 | 1 => 1,
            6 => 2,
            _ => -1,
        }
    }

    /// Возвращает runtime child-ID из `tagCityGate`; отсутствующий logical key
    /// выражен `Option`, а не старым sentinel/pointer API.
    pub(crate) fn city_gate_id(&self, logical_id: i32) -> Option<i32> {
        self.city_gates
            .get(&logical_id)
            .map(|gate| gate.city_gate_id)
    }

    /// Возвращает исходные name bytes без недоказанной смены кодировки.
    pub(crate) fn city_gate_name(&self, logical_id: i32) -> Option<&[u8]> {
        self.city_gates
            .get(&logical_id)
            .map(|gate| gate.name.as_slice())
    }

    /// Exact virtual `AddGurdMonster`: повторный ID не меняет набор.
    pub(crate) fn add_gurd_monster(&mut self, monster_id: i32) {
        self.guard_monsters.insert(monster_id);
    }

    /// Exact virtual `AddGuardIndex`: первый порядок регистрации сохраняется,
    /// повторный refresh index не добавляется второй раз.
    pub(crate) fn add_guard_index(&mut self, refresh_index: i32) {
        if !self.guard_indices.contains(&refresh_index) {
            self.guard_indices.push(refresh_index);
        }
    }

    pub(crate) fn guard_refresh_targets(&self) -> CityGuardRefreshTargets {
        CityGuardRefreshTargets {
            monster_ids: self.guard_monsters.iter().copied().collect(),
            spawn_indices: self.guard_indices.clone(),
        }
    }
}

/// Общий PDB-symbol `CServerCityRegion::CityGateIsClose` RVA `0x001CAAA0`:
/// country-region вызывает именно его, поэтому обе region-цепочки используют
/// один доказанный x-major footprint scan без объединения самих владельцев.
pub(crate) fn city_gate_footprint_is_clear(region: &CServerRegion, gate: &CCityGate) -> bool {
    let footprint = gate.footprint();
    let width = i32::from(footprint.width_increment);
    let height = i32::from(footprint.height_increment);
    // VERIFIED_DISASSEMBLY RVA 0x001CAAA0: x86 `sub/add` и loop increment
    // работают по DWORD с wrapping; это определяет поведение точнее, чем
    // потенциальный signed-overflow UB исходного C++. Существенный фрагмент:
    // `sub ebx,edi; add edi,eax; add edi,1; cmp edi,ebp; jle ...`.
    let left = footprint.tile_x.wrapping_sub(width);
    let right = footprint.tile_x.wrapping_add(width);
    let top = footprint.tile_y.wrapping_sub(height);
    let bottom = footprint.tile_y.wrapping_add(height);

    let mut tile_x = left;
    while tile_x <= right {
        let mut tile_y = top;
        while tile_y <= bottom {
            if region
                .block_at(tile_x, tile_y)
                .is_some_and(|cell| cell & 7 == 3)
            {
                return false;
            }
            tile_y = tile_y.wrapping_add(1);
        }
        tile_x = tile_x.wrapping_add(1);
    }
    true
}

fn apply_gate_action(gate: &mut CCityGate, action: u16) -> Option<BuildBlockUpdate> {
    gate.set_action(action)
}

fn decode_defence_return(bytes: [u8; 0x20]) -> CityDefenceReturnState {
    CityDefenceReturnState {
        region_id: city_i32_at(&bytes, 0x00),
        left: city_i32_at(&bytes, 0x04),
        top: city_i32_at(&bytes, 0x08),
        right: city_i32_at(&bytes, 0x0C),
        bottom: city_i32_at(&bytes, 0x10),
        does_recall_when_lost: city_i32_at(&bytes, 0x14),
        move_monster_when_refeash: city_i32_at(&bytes, 0x18),
        use_return: city_i32_at(&bytes, 0x1C),
    }
}

fn read_city_gate_build(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CityGateBuild, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x2C>(source, cursor, "tagBuild scalar block")?;
    let name = read_city_c_string(source, cursor, "tagBuild.strName")?;
    let script = read_city_c_string(source, cursor, "tagBuild.strScript")?;
    Ok(CityGateBuild {
        logical_id: city_i32_at(&bytes, 0x00),
        picture_id: city_i32_at(&bytes, 0x04),
        direction: city_i32_at(&bytes, 0x08),
        // `tagBuild` хранит DWORD, но PDB-virtual `SetAction` принимает `ushort`.
        action: LegacyReader::at(&bytes, 0x0C)
            .and_then(|mut reader| reader.read_u16())
            .expect("фиксированный city block содержит action"),
        max_hp: city_i32_at(&bytes, 0x10),
        defence: city_i32_at(&bytes, 0x14),
        width_increment: city_i32_at(&bytes, 0x18),
        title_x: city_i32_at(&bytes, 0x1C),
        title_y: city_i32_at(&bytes, 0x20),
        height_increment: city_i32_at(&bytes, 0x24),
        element_resistance: city_i32_at(&bytes, 0x28),
        name,
        script,
    })
}

fn read_city_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, RegionDecodeInputBlock> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        RegionDecodeInputBlock::UnexpectedEnd { field, offset: block.offset, needed: 4, available: block.available }
    })?;
    let value = reader.read_i32().map_err(|block| RegionDecodeInputBlock::UnexpectedEnd { field, offset: block.offset, needed: block.needed, available: block.available })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_city_c_string(
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
            // local `char[256]`; эффект повреждения stack неизвестен.
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

fn city_i32_at<const N: usize>(bytes: &[u8; N], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("фиксированный city block содержит поле")
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.h

// ============================================================================
// FUNCTION: CServerCityRegion::CityGateIsClose
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CAAA0
//
// IMPLEMENTED выше и VERIFIED_DISASSEMBLY: x-major footprint scan, low-byte
// width/height, wrapping DWORD arithmetic, пропуск out-of-bounds/null cells и
// отказ при `(cell & 7) == 3`.
//

// ============================================================================
// FUNCTION: CServerCityRegion::tagBuild::~tagBuild
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp
// RVA: 0x001CB140
// ADDRESS: 005cb140
// PROTOTYPE: void __thiscall ~tagBuild(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerCountryRegion::tagWorldFlag::~tagWorldFlag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp
// RVA: 0x001CB190
// ADDRESS: 005cb190
// PROTOTYPE: void __thiscall ~tagWorldFlag(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagContend::tagContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp
// RVA: 0x001CBA70
// ADDRESS: 005cba70
// PROTOTYPE: undefined __thiscall tagContend(tagContend * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:41
// RVA: 0x001CED20
// ADDRESS: 005ced20
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::AI
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:78
// RVA: 0x001CED40
// ADDRESS: 005ced40
// PROTOTYPE: void __thiscall AI(void)
//
// Реальный CGame virtual caller вызывает inherited `CServerWarRegion::AI`
// через typed City adapter; exact one-call override не дублируется wrapper-ом.

// ============================================================================
// FUNCTION: CServerRegion::OnSymbolDestroy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:275
// RVA: 0x001CED50
// ADDRESS: 005ced50
// PROTOTYPE: void __thiscall OnSymbolDestroy(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerCityRegion::SymbolIsAttackAble` входит в canonical
// `ServerRegionOwner` virtual family. Предварительный `GetWarState` не имеет
// side effects, результат virtual безусловно false.

// ============================================================================
// FUNCTION: CServerCityRegion::IsOwner
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:580
// RVA: 0x001CED70
//
// IMPLEMENTED выше: nonzero faction ID должен совпасть с текущим city owner.
//

// ============================================================================
// FUNCTION: CServerCityRegion::OnFactionWinOneSymbol
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CEDA0
//
// IMPLEMENTED выше: symbol-zero defender update.

// ============================================================================
// FUNCTION: CServerCityRegion::IsApplyWarFacsMem
// STATUS: IMPLEMENTED / BLOCKED_MISSING_FACT
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:638
// RVA: 0x001CEDC0
//
// IMPLEMENTED выше: текущий war number делегируется CAttackCitySys вместе с
// faction ID. Недоставленный bIsEveryWeek распространяется как локальная
// ошибка, а не превращается в выбранный агентом membership-результат.
//

// ============================================================================
// FUNCTION: CServerCityRegion::SetEnterPosXY
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:105
// RVA: 0x001CEE80
//
// Реализовано выше: двойной state gate, virtual return-point, same-region
// comparison, random range и player SetPos имеют статус VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CServerCityRegion::SetOwnedCityOrg
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CEF40
//
// IMPLEMENTED выше: ownership и defender faction; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::GetReturnPoint
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:443
// RVA: 0x001CEF60
//
// Реализовано выше: null default, TileY/TileX calls, defender setup и base
// fallback имеют статус VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CServerCityRegion::GuardIsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:485
// RVA: 0x001CF050
// ADDRESS: 005cf050
// PROTOTYPE: bool __thiscall GuardIsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::GetSecurity
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:554
// RVA: 0x001CF0E0
//
// Реализовано выше в общей virtual-family region security; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CServerCityRegion::GetDiedStateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:572
// RVA: 0x001CF170
// ADDRESS: 005cf170
// PROTOTYPE: long __thiscall GetDiedStateTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::OnFactionVictory
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF1A0
//
// IMPLEMENTED выше: state gate, ownership, доказанный no-op и принадлежащий
// `CGame` 0x60138; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OperatorCityGate
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF370
//
// IMPLEMENTED выше: exact operations `0..3`, refresh HP-before-action и
// close-check-before-action; прочие enum values остаются успешным no-op.
//

// ============================================================================
// FUNCTION: CServerCityRegion::OperatorCityGate
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF4C0
//
// IMPLEMENTED выше и VERIFIED_DISASSEMBLY: lookup использует логический map
// key; найденный key возвращает true, игнорируя false pointer-overload.
//

// ============================================================================
// FUNCTION: CServerCityRegion::UpdateCityGateToClient
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF510
//
// IMPLEMENTED выше и VERIFIED_DISASSEMBLY: `0xBF60F` содержит type, runtime
// object ID, action, max HP и HP именно в этом порядке; send-around — context.
//

// ============================================================================
// FUNCTION: CServerCityRegion::CityGateIsClose
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF600
//
// IMPLEMENTED выше: miss даёт false, найденный logical ID делегирует точному
// footprint scan pointer-варианта.
//

// ============================================================================
// FUNCTION: CServerCityRegion::GetCityGateState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF640
//
// IMPLEMENTED выше: action `7 -> 0`, `0/1 -> 1`, `6 -> 2`, прочие и miss
// дают `-1`.
//

// ============================================================================
// FUNCTION: CServerCityRegion::OnClearOtherPlayer
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF730
//
// IMPLEMENTED выше через `CGame`: faction-filtered return и ordered gate
// 2/1/update; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::RefreshGuard
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF7C0
//
// IMPLEMENTED выше: ordered guard-monster и spawn refresh; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::ClearRegion
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CF970
//
// IMPLEMENTED выше: war clear и ordered gate 2/update; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OnRefreshRegion
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CFA00
//
// IMPLEMENTED выше: guard refresh delegation; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OnWarStart
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CFA10
//
// IMPLEMENTED выше: war-ID gate, Fight state и GS0220 log; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OnWarDeclare
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CFB30
//
// IMPLEMENTED выше: war declare, defender snapshot и GS0221 log; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OnWarMass
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CFC50
//
// IMPLEMENTED выше: war-ID gate, Mass state и GS0222 log; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OnWarEnd
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001CFD70
//
// IMPLEMENTED выше: war/state gates, clear и GS0225 log; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::OnContendTimeOver
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:591
// RVA: 0x001CFEB0
//
// IMPLEMENTED выше: value-copy thunk не добавляет семантики и прямо вызывает
// общий `CServerWarRegion::OnContendTimeOver`.

// ============================================================================
// FUNCTION: CServerCityRegion::AddGurdMonster
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:133
// RVA: 0x001D0360
//
// Реализовано выше: ordered set сохраняет уникальность monster ID;
// технические STL tree lookup/insert детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::AddGuardIndex
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:139
// RVA: 0x001D0650
//
// Реализовано выше: линейная unique-проверка и append сохраняют исходный
// registration order; технические STL list детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::~CServerCityRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:36
// RVA: 0x001D0910
// ADDRESS: 005d0910
// PROTOTYPE: void __thiscall ~CServerCityRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::DelGurdMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.h:80
// RVA: 0x001D09D0
// ADDRESS: 005d09d0
// PROTOTYPE: void __thiscall DelGurdMonster(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::OnWarTimeOut
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001D09F0
//
// IMPLEMENTED выше: map-order winner selection, CGame victory send и
// GS0223/GS0224 log; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerCityRegion::CServerCityRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:30
// RVA: 0x001D0E00
// ADDRESS: 005d0e00
// PROTOTYPE: undefined __thiscall CServerCityRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerCityRegion::AddCityGate
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:85
// RVA: 0x001D0F00
//
// IMPLEMENTED выше: child-ID инкрементируется до factory type `0x4B0`;
// успешный gate получает exact properties/state/title/action/script и заменяет
// запись map.

// ============================================================================
// FUNCTION: CServerCityRegion::DecordFromByteArray
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servercityregion.cpp:47
// RVA: 0x001D10B0
//
// IMPLEMENTED выше: после War decoder читаются defence `0x20`, signed gate
// count и ordered builds; defender faction назначается последним, normal return
// равен true.

// COMPONENT_VARIANT_END: GameServer
