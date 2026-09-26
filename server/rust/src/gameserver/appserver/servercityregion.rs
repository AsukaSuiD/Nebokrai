//! GameServer-владелец городского war-region `CServerCityRegion`.
//!
//! Скалярный state, данные, context-контракты и скалярные правила перенесены
//! в Zone `regions/servercityregion` (волна Z-M-X, семья регионов
//! country+nation+city + гейты); региональное гейтовое тело — opcode
//! константы, проекция client state, правила операций и общий footprint scan —
//! перенесено в Zone `regions/citygate`. Здесь hub-обёртка
//! `CServerCityRegion` поверх `CServerWarRegion` и карты concrete `CCityGate`
//! с прежними сигнатурами, decode-контексты над owner-ом хранилищ,
//! wire-stream readers c `RegionDecodeInputBlock`, re-export семейства для
//! старого пакета и evidence-блок.
//!
//! Фазовые callbacks RVA `0x001CF730`, `0x001CFA00..0x001CFD70`,
//! `0x001D09F0`, ownership `0x001CED70/0x001CEF40`, victory `0x001CF1A0`,
//! spatial `SetEnterPosXY/GetReturnPoint` `0x001CEE80/0x001CEF60`, virtual
//! security/guard attackability `0x001CF0E0/0x001CF050`,
//! decoder `0x001D10B0`, `AddCityGate` `0x001D0F00`, gate runtime
//! `0x001CAAA0/0x001CF370..0x001CF640`, clear `0x001CF970`, guard refresh
//! `0x001CF7C0` и direct timeout-forwarding `0x001CFEB0` имеют статус
//! `IMPLEMENTED`; gate IDs, message layout, decoder/factory returns и фазовый
//! call order, child-ID `+0x158` и base-region registration также
//! `VERIFIED_DISASSEMBLY`. Исходники
//! `servercityregion.h/.cpp`, точная пара GameServer. PDB подтверждает
//! наследование `CServerWarRegion`, ordered
//! `m_CityGates +0x274`, guard set/list `+0x2A0/+0x2AC` и defender faction
//! `+0x2C0`; city-gate hurt callback записывает last attacker type/ID в
//! `+0x2B8/+0x2BC`.
//!
//! `BTreeMap/BTreeSet/Vec` сохраняют STL order. Decoder принимает byte-exact
//! World snapshot: `0x20` defence, signed count, `0x2C` gate scalars и две
//! C-строки в старых `char[256]`; missing NUL/overflow остаются локальными
//! safe-блоками. Обязательный defence-return block хранится с нейтральным
//! zero-default до decode без недостижимой Option-границы. Exact EXE подтвердил
//! normal `true`, layout gate-полей и
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
//! принадлежащего city-owner-у прохода ворот. Localized war-log исполняет
//! `CGame`, а context сохраняет только contender-state и публикацию ворот.
//! City-guard AI `10/11` использует concrete guard virtual: в fight-state
//! owning faction/union защищены, остальные player targets разрешены.
//! Прямой `OnWinSymbol` внутри `OnFactionVictory` у этой сборки указывает на
//! точный no-op `0x004A8750`; Rust не сохраняет для него фиктивный callback.
//! Timeout возвращает victory/log snapshot: `CGame` шлёт `0x60138` после
//! ownership mutation и до сохранённого `GS0223/GS0224` war-log sink.
//! Остальная поверхность файла ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).

pub(crate) use nebokrai_zone::regions::servercityregion::*;

use std::collections::{BTreeMap, BTreeSet};

use super::build::{BuildBlockUpdate, BuildClientPublication, BuildClientUpdate};
use super::citygate::{
    CCityGate, CityGateHurtOwnerUpdate, CityGateInit, GATE_OP_CLOSE, GATE_OP_DIED, GATE_OP_OPEN,
    GATE_OP_REFRESH, GateOperationUpdate, city_gate_operation_update, footprint_is_clear,
    gate_client_state,
};
use super::country::countryparam::CCountryParam;
use nebokrai_shared::protocol::LegacyReader;
use super::monster::CMonster;
use super::npc::CNpc;
use super::organizingsystem::attackcitysys::CAttackCitySys;
use super::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionRandomPosition, RegionReturnPoint,
    RegionSecurity,
};
use super::serverregion::{
    CServerRegion, ServerRegionDecodeEffectsContext, ServerRegionDecodeError,
    ServerRegionMonsterContext, ServerRegionMonsterEffectsContext,
    ServerRegionMonsterSpawnEffectsContext, ServerRegionNpcContext,
    ServerRegionNpcSpawnEffectsContext, ServerReturnPlayer,
};
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::serverwarregion::{
    CServerWarRegion, ContendState, RegionDecodeInputBlock, WarContendContext,
    WarRegionClearContext, WarRegionDecodeError, read_region_array,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CityGateState {
    pub(crate) gate: CCityGate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CityRegionDecodeError<BaseError> {
    War(WarRegionDecodeError<BaseError>),
    Input(RegionDecodeInputBlock),
}

pub(crate) trait CityRegionDecodeContext: ServerRegionDecodeEffectsContext {}

impl<Context: ServerRegionDecodeEffectsContext + ?Sized> CityRegionDecodeContext for Context {}

struct CityGuardDecodeContext<'a, Context> {
    context: &'a mut Context,
    guard_monsters: &'a mut BTreeSet<i32>,
    guard_indices: &'a mut Vec<i32>,
}

impl<Context: RegionRandomContext> RegionRandomContext for CityGuardDecodeContext<'_, Context> {
    fn random_below(&mut self, bound: i32) -> i32 {
        self.context.random_below(bound)
    }
}

impl<Context: ServerRegionNpcSpawnEffectsContext> ServerRegionNpcSpawnEffectsContext
    for CityGuardDecodeContext<'_, Context>
{
    fn log_npc_position_failure(&mut self, npc_name: &[u8]) {
        self.context.log_npc_position_failure(npc_name);
    }
}

impl<Context: ServerRegionNpcSpawnEffectsContext> ServerRegionNpcContext
    for CityGuardDecodeContext<'_, Context>
{
    fn send_npc_entered_around(&mut self, _npc: &CNpc) {}
}

impl<Context: ServerRegionMonsterSpawnEffectsContext> ServerRegionMonsterSpawnEffectsContext
    for CityGuardDecodeContext<'_, Context>
{
    fn log_monster_variant_failure(&mut self, region_id: i32, refresh_index: i32) {
        self.context
            .log_monster_variant_failure(region_id, refresh_index);
    }

    fn log_monster_position_failure(&mut self, origin_name: &[u8]) {
        self.context.log_monster_position_failure(origin_name);
    }
}

impl<Context: ServerRegionMonsterEffectsContext> ServerRegionMonsterEffectsContext
    for CityGuardDecodeContext<'_, Context>
{
    fn send_monster_entered_around(&mut self, region: &CServerRegion, monster: &CMonster) {
        self.context.send_monster_entered_around(region, monster);
    }
}

impl<Context: ServerRegionMonsterEffectsContext> ServerRegionMonsterContext
    for CityGuardDecodeContext<'_, Context>
{
    fn register_guard_monster(&mut self, monster_id: i32) {
        self.guard_monsters.insert(monster_id);
    }

    fn register_guard_index(&mut self, refresh_index: i32) {
        if !self.guard_indices.contains(&refresh_index) {
            self.guard_indices.push(refresh_index);
        }
    }
}

impl<Context: ServerRegionDecodeEffectsContext> ServerRegionDecodeEffectsContext
    for CityGuardDecodeContext<'_, Context>
{
    fn now_millis(&mut self) -> u32 {
        self.context.now_millis()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CServerCityRegion {
    pub(crate) war: CServerWarRegion,
    pub(crate) city_gates: BTreeMap<i32, CityGateState>,
    /// Defender faction `+0x2C0` остаётся плоской колонкой aggregate: её
    /// читает и переписывает runtime `CGame` напрямую, а derived warfare
    /// манипуляции здесь же; scalar state Zone получает её параметром.
    pub(crate) defence_side_faction_id: i32,
    inner: ServerCityRegionState,
}

impl Default for CServerCityRegion {
    fn default() -> Self {
        Self {
            war: CServerWarRegion::default(),
            city_gates: BTreeMap::new(),
            defence_side_faction_id: 0,
            inner: ServerCityRegionState::default(),
        }
    }
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
        if let Some(point) = self.inner.defence_return_point(
            first_state,
            self.war.base.get_city_state(),
            player.faction_id,
            self.defence_side_faction_id,
        ) {
            return Ok(point);
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
        let mut guard_context = CityGuardDecodeContext {
            context,
            guard_monsters: &mut self.inner.guard_monsters,
            guard_indices: &mut self.inner.guard_indices,
        };
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
                &mut guard_context,
            )
            .map_err(CityRegionDecodeError::War)?;

        let defence = read_region_array::<0x20>(source, cursor, "m_DefenceSideRS")
            .map_err(CityRegionDecodeError::Input)?;
        self.inner
            .decode_defence_return_finished(decode_defence_return(defence));

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
            CityGateState { gate },
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
        city_is_owner(self.war.base.param.owned_faction_id, faction_id)
    }

    pub(crate) fn is_apply_war_faction(
        &self,
        schedules: &CAttackCitySys,
        faction_id: i32,
    ) -> bool {
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
        let cell = self.war.base.region.get_cell(x, y)?;
        Ok(city_security_rule(
            cell.map(|cell| cell.security()),
            cell.map_or(0, |cell| cell.city_war_marker()),
            self.war.base.get_city_state(),
        ))
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

    pub(crate) fn on_war_declare(&mut self, war_number: i32) -> CityWarLogEffect {
        self.war.on_war_declare(war_number);
        self.defence_side_faction_id = self.war.base.param.owned_faction_id;
        CityWarLogEffect {
            string_id: "GS0221",
            war_number,
            region_name: self.war.base.name.clone(),
        }
    }

    pub(crate) fn on_war_start(&mut self, war_number: i32) -> Option<CityWarLogEffect> {
        if self.war.base.war_number != war_number {
            return None;
        }
        self.war.base.on_war_start(war_number);
        Some(CityWarLogEffect {
            string_id: "GS0220",
            war_number,
            region_name: self.war.base.name.clone(),
        })
    }

    pub(crate) fn on_war_mass(&mut self, war_number: i32) -> Option<CityWarLogEffect> {
        if self.war.base.war_number != war_number {
            return None;
        }
        self.war.base.on_war_mass(war_number);
        Some(CityWarLogEffect {
            string_id: "GS0222",
            war_number,
            region_name: self.war.base.name.clone(),
        })
    }

    pub(crate) fn on_war_time_out(&mut self, war_number: i32) -> Option<CityWarTimeoutEffect> {
        if self.war.base.war_number != war_number {
            return None;
        }

        let decision = decide_city_war_timeout(
            &self.war.faction_win_symbol,
            self.war.win_victory_symbol_num,
            self.war.base.param.owned_faction_id,
            self.war.base.param.owned_union_id,
        );
        let victory = self.apply_faction_victory(decision.faction_id, decision.union_id);
        Some(CityWarTimeoutEffect {
            victory,
            log_string_id: decision.log_string_id,
            war_number,
            region_name: self.war.base.name.clone(),
        })
    }

    pub(crate) fn on_war_end<Context>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> Option<CityWarEndEffect>
    where
        Context: WarRegionClearContext,
    {
        if self.war.base.war_number != war_number || self.war.base.city_state == 0 {
            return None;
        }
        self.war.on_war_end(war_number);
        let build_updates = self.clear_region(context);
        Some(CityWarEndEffect {
            log: CityWarLogEffect {
                string_id: "GS0225",
                war_number,
                region_name: self.war.base.name.clone(),
            },
            build_updates,
        })
    }

    pub(crate) fn refresh_and_close_gates(&mut self) -> Vec<BuildClientPublication> {
        let mut updates = Vec::new();
        let logical_ids: Vec<_> = self.city_gates.keys().copied().collect();
        for logical_id in logical_ids {
            let _ = self.operator_city_gate(logical_id, GATE_OP_REFRESH);
            let _ = self.operator_city_gate(logical_id, GATE_OP_CLOSE);
            if let Some(update) = self.city_gate_client_publication(logical_id) {
                updates.push(update);
            }
        }
        updates
    }

    pub(crate) fn on_refresh_region(&self, _war_number: i32) -> CityGuardRefreshTargets {
        self.inner.guard_refresh_targets()
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

    pub(crate) fn clear_region<Context>(
        &mut self,
        context: &mut Context,
    ) -> Vec<BuildClientPublication>
    where
        Context: WarRegionClearContext,
    {
        self.war.clear_region(context);
        let mut updates = Vec::new();
        let logical_ids: Vec<_> = self.city_gates.keys().copied().collect();
        for logical_id in logical_ids {
            let _ = self.operator_city_gate(logical_id, GATE_OP_REFRESH);
            if let Some(update) = self.city_gate_client_publication(logical_id) {
                updates.push(update);
            }
        }
        updates
    }

    pub(crate) fn operator_city_gate(&mut self, logical_id: i32, operation: i32) -> bool {
        let Some(gate_state) = self.city_gates.get(&logical_id) else {
            return false;
        };

        let pointer_result = match operation {
            GATE_OP_OPEN => true,
            GATE_OP_CLOSE => city_gate_footprint_is_clear(&self.war.base, &gate_state.gate),
            GATE_OP_REFRESH | GATE_OP_DIED => true,
            _ => true,
        };

        if pointer_result {
            let gate = &mut self
                .city_gates
                .get_mut(&logical_id)
                .expect("gate найден до неизменяющего map вызова")
                .gate;
            let update = apply_gate_operation(gate, city_gate_operation_update(operation));
            if let Some(update) = update {
                let _legacy_void = self.war.base.apply_build_block(update);
            }
        }

        // RVA 0x001CF4C0 игнорирует bool pointer-overload: найденный map key
        // возвращает true даже когда OC_Close не прошёл проверку footprint.
        true
    }

    pub(crate) fn city_gate_client_publication(
        &self,
        logical_id: i32,
    ) -> Option<BuildClientPublication> {
        let gate = &self.city_gates.get(&logical_id)?.gate;
        Some(BuildClientPublication {
            region_id: self.war.base.id,
            build_id: gate.id(),
            update: BuildClientUpdate {
                object_type: gate.object_type(),
                object_id: gate.id() as u32,
                action: gate.action(),
                max_hp: gate.max_hp(),
                hp: gate.hp(),
            },
        })
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
        gate_client_state(gate.action())
    }

    /// Возвращает runtime child-ID из `tagCityGate`; отсутствующий logical key
    /// выражен `Option`, а не старым sentinel/pointer API.
    pub(crate) fn city_gate_id(&self, logical_id: i32) -> Option<i32> {
        self.city_gates
            .get(&logical_id)
            .map(|state| state.gate.id())
    }

    /// Возвращает исходные name bytes без недоказанной смены кодировки.
    pub(crate) fn city_gate_name(&self, logical_id: i32) -> Option<&[u8]> {
        self.city_gates
            .get(&logical_id)
            .map(|state| state.gate.name())
    }

    /// Exact virtual `AddGurdMonster`: повторный ID не меняет набор.
    pub(crate) fn add_gurd_monster(&mut self, monster_id: i32) {
        self.inner.add_gurd_monster(monster_id);
    }

    /// Exact virtual `AddGuardIndex`: первый порядок регистрации сохраняется,
    /// повторный refresh index не добавляется второй раз.
    pub(crate) fn add_guard_index(&mut self, refresh_index: i32) {
        self.inner.add_guard_index(refresh_index);
    }

    /// Exact `CServerCityRegion::GuardIsAttackAble`: scalar-правило state `3`
    /// и owning faction/union находится в Zone; базовая атака вне active
    /// city-war state разрешена.
    pub(crate) fn guard_is_attackable(
        &self,
        target_type: i32,
        target_faction_id: i32,
        target_union_id: i32,
    ) -> bool {
        city_guard_is_attackable(
            self.war.base.get_city_state(),
            target_type,
            target_faction_id,
            target_union_id,
            self.war.base.owned_city_faction(),
            self.war.base.owned_city_union(),
        )
    }

    pub(crate) fn apply_gate_hurt_owner_update(
        &mut self,
        update: CityGateHurtOwnerUpdate,
    ) -> bool {
        self.inner
            .apply_gate_hurt_owner_update(self.war.base.id, update)
    }

    /// Находит concrete gate по runtime child-ID, с которым объект
    /// зарегистрирован в base region, и сразу применяет точный owner-effect
    /// `CCityGate::OnBeenHurted`. Logical ID таблицы `tagCityGate` здесь не
    /// участвует: combat lookup исходно приходит по identity объекта.
    pub(crate) fn city_gate_on_been_hurted(
        &mut self,
        city_gate_id: i32,
        attacker_type: i32,
        attacker_id: i32,
    ) -> bool {
        let update = self
            .city_gates
            .values()
            .find(|state| state.gate.id() == city_gate_id)
            .map(|state| state.gate.on_been_hurted(attacker_type, attacker_id));
        update.is_some_and(|update| self.apply_gate_hurt_owner_update(update))
    }

    pub(crate) fn guard_refresh_targets(&self) -> CityGuardRefreshTargets {
        self.inner.guard_refresh_targets()
    }
}

/// Общий PDB-symbol `CServerCityRegion::CityGateIsClose` RVA `0x001CAAA0`:
/// country-region вызывает именно его; сам x-major scan живёт в Zone
/// `regions/citygate`, а эта facade связывает owning region и concrete gate.
pub(crate) fn city_gate_footprint_is_clear(region: &CServerRegion, gate: &CCityGate) -> bool {
    footprint_is_clear(&gate.footprint(), |tile_x, tile_y| {
        region.block_at(tile_x, tile_y)
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

fn read_city_gate_build(
    source: &[u8],
    cursor: &mut usize,
) -> Result<CityGateBuild, RegionDecodeInputBlock> {
    let bytes = read_region_array::<0x2C>(source, cursor, "tagBuild scalar block")?;
    let name = read_city_c_string(source, cursor, "tagBuild.strName")?;
    let script = read_city_c_string(source, cursor, "tagBuild.strScript")?;
    Ok(city_gate_build_from_block(&bytes, name, script))
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
