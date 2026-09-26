//! Статус корпуса: MIXED (startup, gameplay lifecycle и manager round-trips
//! реализованы; ниже сохранены ещё не нужные runtime-цепочке constructors,
//! singleton plumbing, split-helper и compiler funclets).
//! Декомпилятор: Ghidra 12.1.2
//!
//! Данные, top-ten wire decoder и скалярные state/правила перенесены в Zone
//! `regions/servergodsbattleregion` (волна Z-M-Xe, семья war-регионов
//! war+godsbattle+village); здесь hub-обёртки `CGodsBattleMgr` с
//! configuration `CGodsBattleConf` и `CServerGodsBattleRegion` поверх hub
//! `CServerWarRegion` с прежними сигнатурами: SZL/XYD owner-а конфигурации,
//! startup decode-семейство над owner-ом хранилищ и NPC spawn hub-вызовы,
//! re-export семейства для старого пакета (включая
//! `GodsBattleTopTenDecodeError` серверной стены сообщений) и evidence-блок.
//!
//! Startup manager snapshot сохраняет подтверждённый wire, section-local clear,
//! намеренное append-поведение faction rules и обе внутренние audit-записи.
//! Region-set хранит ordered unique ID. Для selector `0x0E` concrete region
//! читает только парный `CServerRegion` prefix: World создаёт RT_GODSBATTLE как
//! обычный `CWorldRegion`, тогда как унаследованный Game decoder пытается читать
//! отсутствующий war-tail из свободной capacity `CMessage`. Эта исходная
//! несовместимость отдельно зафиксирована в ADR-0008.
//! Безразмерный pointer и 256-байтный временный C-string buffer заменены
//! bounded slice/cursor и owned bytes; обрыв возвращает typed error после уже
//! завершённого prefix-а вместо неназначаемого legacy UB. Top-ten SZL exchange
//! хранит единственный overwrite-able requester, exact World request и
//! terminal-marker decoder; client publication выполняет dispatcher-owner.
//! XYD round-trip использует configuration-owned slots, ordered region set и
//! faction player/NPC sets. NPC guard/counter lifecycle и GodsBattle-contend
//! исполняет `CGame`; точный script case `11130` является живым caller-ом, а
//! byte-owned имя contender сохраняет исходную GBK.
//! Monster-death tail проверяет AI `0x17`, повторно разрешает killer identity
//! в owning region и только затем изменяет manager kill-counter.
//! Проверка first-contender намеренно сравнивает normal `m_lFactionID` с
//! сохранённым GodsBattle faction: это несовпадение подтверждено RVA
//! `0x000A9270`, а не исправлено по более позднему C++-донору.
//! CancelContendByPlayerID удаляет все записи игрока, сохраняя порядок
//! остальных; затем безусловно сбрасывает флаг захвата и время.
//! Только NULL player возвращает false.
//! `CGodsBattleMgr::OnEnterContend` аналогично сравнивает индекс NPC-set
//! `0..2` с player faction `5/6`: для допустимого игрока ветвь `SZLGS7`
//! недостижима, поэтому после гибели всех стражей собственный символ тоже
//! начинает contend. Это подтверждённое различие представлений не нормализуется.
//! Faction-specific die-back lookup сохраняет ordered first-match, обязательный
//! registered-region gate и direction `-1`; concrete caller делегирует miss
//! подтверждённому `CServerRegion::GetReturnPoint`. Дизассемблирование RVA
//! `0x000A6230` подтверждает `AL=1` только после найденной записи и audit-call
//! (`0x004A6340`), `AL=0` для обоих miss (`0x004A6375`); missing-текст EXE
//! форматирует во временный buffer, но наружу не публикует.
//! SZL owner дополнительно материализует inclusive tier lookup и обе
//! victim-tier gain/loss формулы; death/team/client ordering остаётся у `CGame`.
//! `OnChangeFaction` сохраняет отдельный обязательный region-set transition:
//! отсутствие live NPC или записи конфигурации не отменяет уже подтверждённую
//! смену стороны. Monster refresh и `OnNpcUpdate` выполняются только после неё;
//! для найденного NPC update всегда сбрасывает kill-counter, а config/world
//! publication остаётся условной на совпавшее имя, как в RVA `0x000AA8A0`.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.h

pub(crate) use nebokrai_zone::regions::servergodsbattleregion::*;

use crate::setup::godsbattleconf::{
    CGodsBattleConf, GodsBattleDecodeError, GodsBattleFactionXydUpdate, GodsBattleSzlCalculation,
};
use std::collections::BTreeSet;

use super::serverregion::{
    CServerRegion, ServerRegionDecodeError, ServerRegionNpcContext, ServerRegionNpcSetup,
    ServerRegionNpcSpawnBlock, ServerRegionNpcSpawnOutcome,
};
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::serverwarregion::{
    CServerWarRegion, WarRegionDecodeContext, WarRegionDecodeError,
};

// Точные GBK payload из GameServer .rdata VA `0x00651870` и `0x00651850`.
const REVISE_MONEY_CONFIGURATION_ERROR: &[u8] =
    b"\xC9\xF1\xD6\xAE\xC1\xA6\xD0\xDE\xD5\xFD\xD6\xB5\xC5\xE4\xD6\xC3\xB4\xED\xCE\xF3\xA3\xA1";
const EMPTY_DIE_BACK_CONFIGURATION: &[u8] =
    b"\xA1\xBE\xD6\xEE\xC9\xF1\xD6\xAE\xD5\xBD\xA1\xBF\xCB\xC0\xCD\xF6\xBB\xD8\xB3\xC7\xB5\xC4\xC5\xE4\xD6\xC3\xCE\xAA\xBF\xD5";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGodsBattleMgr {
    configuration: CGodsBattleConf,
    inner: GodsBattleMgrState,
}

impl CGodsBattleMgr {
    pub(crate) const fn configuration(&self) -> &CGodsBattleConf {
        &self.configuration
    }

    pub(crate) fn add_region_set(&mut self, region_id: i32) -> bool {
        self.inner.add_region_set(region_id)
    }

    /// Exact `CGodsBattleMgr::IsGodsBattleRegion`: ordered set membership без
    /// дополнительной проверки concrete region owner.
    pub(crate) fn is_gods_battle_region(&self, region_id: i32) -> bool {
        self.inner.is_gods_battle_region(region_id)
    }

    pub(crate) fn region_ids(&self) -> Vec<i32> {
        self.inner.region_ids()
    }

    pub(crate) const fn pending_top_ten_player_id(&self) -> i32 {
        self.inner.pending_top_ten_player_id()
    }

    /// Exact post-send assignment `GetTopTenSZL`: concurrent request
    /// перезаписывает единственный legacy requester без sequence ID.
    pub(crate) const fn record_top_ten_request(&mut self, player_id: i32) {
        self.inner.record_top_ten_request(player_id);
    }

    pub(crate) fn faction_for_country(&self, country: u8) -> Option<i32> {
        self.configuration
            .faction_for_country(country)
            .map(|faction| faction as i32)
    }

    pub(crate) fn set_xyd(
        &mut self,
        faction_a: u32,
        faction_b: u32,
    ) -> [GodsBattleFactionXydUpdate; 2] {
        [
            self.configuration.set_faction_xyd(1, faction_a),
            self.configuration.set_faction_xyd(2, faction_b),
        ]
    }

    /// Exact `CGodsBattleMgr::GetFactionXYD`: допустимы manager slots 0..2;
    /// нулевой slot не загружается конфигурацией и сохраняет legacy zero.
    pub(crate) fn get_faction_xyd(&self, faction_index: i32) -> u32 {
        let (faction_a, faction_b) = self.configuration.faction_xyd();
        match faction_index {
            0 => 0,
            1 => faction_a,
            2 => faction_b,
            _ => 0,
        }
    }

    pub(crate) fn calculate_szl_gain(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
    ) -> GodsBattleSzlCalculation {
        self.configuration
            .calculate_szl_gain(killer_level, killer_szl, victim_level, victim_szl)
    }

    pub(crate) fn calculate_szl_loss(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
    ) -> GodsBattleSzlCalculation {
        self.configuration
            .calculate_szl_loss(killer_level, killer_szl, victim_level, victim_szl)
    }

    pub(crate) fn szl_level(&self, szl: u32) -> Option<u32> {
        self.configuration.szl_level(szl)
    }

    pub(crate) fn npc_configuration(
        &self,
        name: &[u8],
    ) -> Option<&crate::setup::godsbattleconf::GodsBattleFactionNpcName> {
        self.configuration.npc_by_name(name)
    }

    pub(crate) fn npc_name_by_monster(&self, original_name: &[u8]) -> Option<&[u8]> {
        self.configuration.npc_name_by_monster(original_name)
    }

    pub(crate) fn npc_monster_count(&self, name: &[u8]) -> u32 {
        self.configuration
            .npc_by_name(name)
            .map(|npc| {
                npc.monsters
                    .split(|byte| *byte == b',')
                    .filter(|token| !token.is_empty())
                    .count() as u32
            })
            .unwrap_or(0)
    }

    pub(crate) fn npc_killed_monster_count(&self, name: &[u8]) -> u32 {
        self.inner.npc_killed_monster_count(name)
    }

    pub(crate) fn reset_npc_killed_monster_count(&mut self, name: &[u8]) {
        self.inner.reset_npc_killed_monster_count(name);
    }

    pub(crate) fn increment_npc_killed_monster_count(&mut self, name: &[u8]) -> Option<u32> {
        self.inner.increment_npc_killed_monster_count(name)
    }

    pub(crate) fn update_npc_faction(
        &mut self,
        name: &[u8],
        faction: i32,
    ) -> Option<crate::setup::godsbattleconf::GodsBattleNpcFactionUpdate> {
        self.configuration.set_npc_faction(name, faction)
    }

    pub(crate) fn return_point(
        &self,
        region_id: i32,
        faction: i32,
    ) -> Option<crate::gameserver::appserver::region::RegionReturnPoint> {
        if !self.inner.is_gods_battle_region(region_id) {
            return None;
        }
        let position = self
            .configuration
            .die_back_position(region_id, faction as u32)?;
        Some(crate::gameserver::appserver::region::RegionReturnPoint {
            region_id: position.destination_region,
            left: position.left,
            top: position.top,
            right: position.right,
            bottom: position.bottom,
            direction: -1,
        })
    }

    pub(crate) fn decode_top_ten(
        &self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<Vec<GodsBattleTopTenEntry>, GodsBattleTopTenDecodeError> {
        decode_gods_battle_top_ten(source, cursor)
    }

    /// Воспроизводит `CGodsBattleMgr::DecordFromByteArray`, включая оба
    /// внутренних audit side effect-а в исходных позициях.
    pub(crate) fn decord_from_byte_array<AddLogText, PutStringToFile>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        add_log_text: &mut AddLogText,
        put_string_to_file: &mut PutStringToFile,
    ) -> Result<(), GodsBattleDecodeError>
    where
        AddLogText: FnMut(&[u8]),
        PutStringToFile: FnMut(&str, &[u8]),
    {
        self.configuration.decord_from_byte_array(
            source,
            cursor,
            || add_log_text(REVISE_MONEY_CONFIGURATION_ERROR),
            || put_string_to_file("godsbattleLog", EMPTY_DIE_BACK_CONFIGURATION),
        )
    }
}

/// Startup-часть concrete GodsBattle region. Constructor подтверждает
/// наследование `CServerWarRegion`; player faction membership уже связан с
/// Add/Remove tail, NPC/contend gameplay коллекции сохраняют owned defaults.
#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerGodsBattleRegion {
    pub(crate) war: CServerWarRegion,
    inner: GodsBattleRegionState,
}

impl CServerGodsBattleRegion {
    /// Декодирует доказанный World -> Game startup prefix для RT_GODSBATTLE.
    ///
    /// Оригинальный `CWorldRegion::AddToByteArray` заканчивает logical payload
    /// после base snapshot. Поэтому вызов inherited `CServerWarRegion` здесь
    /// воспроизвёл бы чтение за `_Mylast`, а не формат сообщения.
    pub(crate) fn decord_initial_world_base_snapshot_with_npc_entry<
        Context: WarRegionDecodeContext,
    >(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        area_width: i32,
        area_height: i32,
        monster_registry: &MonsterRegistry,
        skill_factory: &CSkillFactory,
        context: &mut Context,
        mut after_npc_entry: impl FnMut(
            &mut CServerRegion,
            &mut [BTreeSet<i32>; 3],
            i32,
            &mut Context,
        ),
    ) -> Result<bool, ServerRegionDecodeError> {
        let faction_npcs = &mut self.inner.faction_npcs;
        self.war.base.decord_from_byte_array_with_npc_entry(
            source,
            cursor,
            include_child,
            area_width,
            area_height,
            monster_registry,
            skill_factory,
            context,
            |region, npc_id, context| {
                after_npc_entry(region, faction_npcs, npc_id, context);
            },
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
        mut after_npc_entry: impl FnMut(
            &mut CServerRegion,
            &mut [BTreeSet<i32>; 3],
            i32,
            &mut Context,
        ),
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError>> {
        let faction_npcs = &mut self.inner.faction_npcs;
        self.war
            .decord_from_byte_array_with_npc_entry(
                source,
                cursor,
                include_child,
                area_width,
                area_height,
                monster_registry,
                skill_factory,
                context,
                |region, npc_id, context| {
                    after_npc_entry(region, faction_npcs, npc_id, context);
                },
            )
    }

    pub(crate) fn add_faction_npc_membership(
        faction_npcs: &mut [BTreeSet<i32>; 3],
        npc_id: i32,
        faction: i32,
    ) -> bool {
        GodsBattleRegionState::add_faction_npc_membership(faction_npcs, npc_id, faction)
    }

    /// Создаёт NPC через базовый регион и выполняет подтверждённое завершение
    /// `AddObject` до круговой публикации и до следующего создания объекта.
    pub(crate) fn add_npc_with_clock_and_entry<Context: ServerRegionNpcContext>(
        &mut self,
        setup: &ServerRegionNpcSetup,
        remember_setup: bool,
        send_around: bool,
        area_width: i32,
        area_height: i32,
        context: &mut Context,
        now_ms: impl FnMut(&mut Context) -> u32,
        mut after_npc_entry: impl FnMut(
            &mut CServerRegion,
            &mut [BTreeSet<i32>; 3],
            i32,
            &mut Context,
        ),
    ) -> Result<ServerRegionNpcSpawnOutcome, ServerRegionNpcSpawnBlock> {
        let faction_npcs = &mut self.inner.faction_npcs;
        self.war.base.add_npc_with_clock_and_entry(
            setup,
            remember_setup,
            send_around,
            area_width,
            area_height,
            context,
            now_ms,
            |region, npc_id, context| {
                after_npc_entry(region, faction_npcs, npc_id, context);
            },
            |npc, context| context.send_npc_entered_around(npc),
        )
    }

    pub(crate) fn add_faction_player(&mut self, player_id: i32, faction: i32) -> bool {
        self.inner.add_faction_player(player_id, faction)
    }

    pub(crate) fn remove_faction_player(&mut self, player_id: i32) -> bool {
        self.inner.remove_faction_player(player_id)
    }

    pub(crate) fn faction_player_ids(&self, faction: i32) -> Option<Vec<i32>> {
        self.inner.faction_player_ids(faction)
    }

    pub(crate) fn npc_faction_index(&self, npc_id: i32) -> Option<usize> {
        self.inner.npc_faction_index(npc_id)
    }

    /// Exact `CServerGodsBattleRegion::GetObjFaction`: player faction 5/6
    /// преобразуется в raw `1/2`, NPC возвращает индекс одного из трёх sets,
    /// а неизвестный type/object — `-1` (`INVALID_FACTION`).
    pub(crate) fn get_obj_faction(
        &self,
        object_type: i32,
        object_id: i32,
        player_faction: impl FnOnce(i32) -> Option<i32>,
    ) -> i32 {
        self.inner
            .get_obj_faction(object_type, object_id, player_faction)
    }

    pub(crate) fn remove_faction_npc(&mut self, npc_id: i32) -> bool {
        self.inner.remove_faction_npc(npc_id)
    }

    pub(crate) fn change_npc_faction(&mut self, npc_id: i32, faction: i32) -> bool {
        self.inner.change_npc_faction(npc_id, faction)
    }

    pub(crate) fn is_player_contending_symbol(&self, player_id: i32, symbol_id: i32) -> bool {
        self.inner.is_player_contending_symbol(player_id, symbol_id)
    }

    /// Список обрабатывается полностью до player-state и `0xBFF29(0)`.
    /// Как у War, эти безусловные действия выполняет runtime после mutation.
    pub(crate) fn remove_contenders_for_player(&mut self, player_id: i32) {
        self.inner.remove_contenders_for_player(player_id);
    }

    /// Проверка first-for-faction использует normal `m_lFactionID`, тогда как
    /// contender хранит `lGodsBattleFaciton`; несовпадение подтверждено EXE.
    pub(crate) fn add_contender(
        &mut self,
        player_id: i32,
        normal_faction: i32,
        gods_battle_faction: i32,
        symbol_id: i32,
        symbol_name: &[u8],
        max_time: i32,
        now_ms: u32,
    ) -> bool {
        self.inner.add_contender(
            player_id,
            normal_faction,
            gods_battle_faction,
            symbol_id,
            symbol_name,
            max_time,
            now_ms,
        )
    }

    pub(crate) fn cancel_contend_by_symbol(
        &mut self,
        symbol_id: i32,
    ) -> Option<GodsBattleContender> {
        self.inner.cancel_contend_by_symbol(symbol_id)
    }

    pub(crate) fn advance_contenders(&mut self, now_ms: u32) -> GodsBattleContendAdvance {
        self.inner.advance_contenders(now_ms)
    }
}

// COMPONENT_VARIANT_END: GameServer
