//! Статус корпуса: MIXED (startup, gameplay lifecycle и manager round-trips
//! реализованы; ниже сохранены ещё не нужные runtime-цепочке constructors,
//! singleton plumbing, split-helper и compiler funclets).
//! Декомпилятор: Ghidra 12.1.2
//! Сырой C++ ниже после typed owner-а является комментарием, а не
//! Rust-реализацией.
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

use crate::setup::godsbattleconf::{
    CGodsBattleConf, GodsBattleDecodeError, GodsBattleFactionXydUpdate, GodsBattleSzlCalculation,
};
use std::collections::BTreeSet;
use thiserror::Error;

use super::serverregion::{
    CServerRegion, ServerRegionDecodeError, ServerRegionNpcContext, ServerRegionNpcSetup,
    ServerRegionNpcSpawnBlock, ServerRegionNpcSpawnOutcome,
};
use super::legacycodec::LegacyReader;
use super::skills::skillfactory::CSkillFactory;
use crate::setup::monsterlist::MonsterRegistry;
use super::serverwarregion::{CServerWarRegion, WarRegionDecodeContext, WarRegionDecodeError};

// Точные GBK payload из GameServer .rdata VA `0x00651870` и `0x00651850`.
const REVISE_MONEY_CONFIGURATION_ERROR: &[u8] =
    b"\xC9\xF1\xD6\xAE\xC1\xA6\xD0\xDE\xD5\xFD\xD6\xB5\xC5\xE4\xD6\xC3\xB4\xED\xCE\xF3\xA3\xA1";
const EMPTY_DIE_BACK_CONFIGURATION: &[u8] =
    b"\xA1\xBE\xD6\xEE\xC9\xF1\xD6\xAE\xD5\xBD\xA1\xBF\xCB\xC0\xCD\xF6\xBB\xD8\xB3\xC7\xB5\xC4\xC5\xE4\xD6\xC3\xCE\xAA\xBF\xD5";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGodsBattleMgr {
    configuration: CGodsBattleConf,
    region_set: BTreeSet<i32>,
    killed_monster_count: std::collections::BTreeMap<Vec<u8>, u32>,
    pending_top_ten_player_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleTopTenEntry {
    pub(crate) faction: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) szl: u32,
    pub(crate) level: u32,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum GodsBattleTopTenDecodeError {
    #[error("GodsBattle top-ten обрывается на {offset} в поле {field}")]
    UnexpectedEnd { offset: usize, field: &'static str },
    #[error("GodsBattle top-ten name с {offset} не имеет NUL-терминатора")]
    MissingNameTerminator { offset: usize },
    #[error("GodsBattle top-ten name с {offset} длиной {length} не помещается в 260 байт")]
    NameOutsideLegacyBuffer { offset: usize, length: usize },
}

impl CGodsBattleMgr {
    pub(crate) const fn configuration(&self) -> &CGodsBattleConf {
        &self.configuration
    }

    pub(crate) fn add_region_set(&mut self, region_id: i32) -> bool {
        self.region_set.insert(region_id)
    }

    /// Exact `CGodsBattleMgr::IsGodsBattleRegion`: ordered set membership без
    /// дополнительной проверки concrete region owner.
    pub(crate) fn is_gods_battle_region(&self, region_id: i32) -> bool {
        self.region_set.contains(&region_id)
    }

    pub(crate) fn region_ids(&self) -> Vec<i32> {
        self.region_set.iter().copied().collect()
    }

    pub(crate) const fn pending_top_ten_player_id(&self) -> i32 {
        self.pending_top_ten_player_id
    }

    /// Exact post-send assignment `GetTopTenSZL`: concurrent request
    /// перезаписывает единственный legacy requester без sequence ID.
    pub(crate) const fn record_top_ten_request(&mut self, player_id: i32) {
        self.pending_top_ten_player_id = player_id;
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
        self.killed_monster_count.get(name).copied().unwrap_or(0)
    }

    pub(crate) fn reset_npc_killed_monster_count(&mut self, name: &[u8]) {
        self.killed_monster_count.insert(name.to_vec(), 0);
    }

    pub(crate) fn increment_npc_killed_monster_count(&mut self, name: &[u8]) -> Option<u32> {
        let count = self.killed_monster_count.get_mut(name)?;
        *count = count.wrapping_add(1);
        Some(*count)
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
        if !self.region_set.contains(&region_id) {
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
        let mut entries = Vec::new();
        loop {
            let marker = read_top_ten_i32(source, cursor, "marker")?;
            if marker == 0 {
                return Ok(entries);
            }
            let faction = read_top_ten_i32(source, cursor, "faction")?;
            let name_offset = *cursor;
            let remaining =
                source
                    .get(name_offset..)
                    .ok_or(GodsBattleTopTenDecodeError::UnexpectedEnd {
                        offset: name_offset,
                        field: "name",
                    })?;
            let length = remaining.iter().position(|byte| *byte == 0).ok_or(
                GodsBattleTopTenDecodeError::MissingNameTerminator {
                    offset: name_offset,
                },
            )?;
            if length >= 260 {
                return Err(GodsBattleTopTenDecodeError::NameOutsideLegacyBuffer {
                    offset: name_offset,
                    length,
                });
            }
            let name = remaining[..length].to_vec();
            *cursor = cursor.wrapping_add(length + 1);
            let szl = read_top_ten_i32(source, cursor, "szl")? as u32;
            let level = read_top_ten_i32(source, cursor, "level")? as u32;
            entries.push(GodsBattleTopTenEntry {
                faction,
                name,
                szl,
                level,
            });
        }
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

fn read_top_ten_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GodsBattleTopTenDecodeError> {
    let offset = *cursor;
    let mut reader = LegacyReader::at(source, offset)
        .map_err(|_| GodsBattleTopTenDecodeError::UnexpectedEnd { offset, field })?;
    let value = reader
        .read_i32()
        .map_err(|_| GodsBattleTopTenDecodeError::UnexpectedEnd { offset, field })?;
    *cursor = reader.position();
    Ok(value)
}

/// Startup-часть concrete GodsBattle region. Constructor подтверждает
/// наследование `CServerWarRegion`; player faction membership уже связан с
/// Add/Remove tail, NPC/contend gameplay коллекции сохраняют owned defaults.
#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerGodsBattleRegion {
    pub(crate) war: CServerWarRegion,
    faction_players: [BTreeSet<i32>; 3],
    faction_npcs: [BTreeSet<i32>; 3],
    contenders: Vec<GodsBattleContender>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleContender {
    pub(crate) symbol_id: i32,
    pub(crate) symbol_name: Vec<u8>,
    pub(crate) player_id: i32,
    pub(crate) gods_battle_faction: i32,
    pub(crate) current_time: i32,
    pub(crate) max_time: i32,
    pub(crate) start_time_ms: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GodsBattleContendAdvance {
    pub(crate) progress: Vec<(i32, i32)>,
    pub(crate) completed: Vec<GodsBattleContender>,
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
        let faction_npcs = &mut self.faction_npcs;
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
        let faction_npcs = &mut self.faction_npcs;
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
        let Some(index) = gods_battle_faction_index(faction) else {
            return false;
        };
        faction_npcs[index].insert(npc_id)
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
        let faction_npcs = &mut self.faction_npcs;
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
        match faction {
            5 => self.faction_players[1].insert(player_id),
            6 => self.faction_players[2].insert(player_id),
            _ => false,
        }
    }

    pub(crate) fn remove_faction_player(&mut self, player_id: i32) -> bool {
        self.faction_players
            .iter_mut()
            .fold(false, |removed, players| {
                players.remove(&player_id) || removed
            })
    }

    pub(crate) fn faction_player_ids(&self, faction: i32) -> Option<Vec<i32>> {
        let index = match faction {
            5 => 1,
            6 => 2,
            _ => return None,
        };
        Some(self.faction_players[index].iter().copied().collect())
    }

    pub(crate) fn npc_faction_index(&self, npc_id: i32) -> Option<usize> {
        self.faction_npcs
            .iter()
            .position(|npcs| npcs.contains(&npc_id))
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
        match object_type {
            400 => match player_faction(object_id) {
                Some(5) => 1,
                Some(6) => 2,
                _ => -1,
            },
            500 => self
                .npc_faction_index(object_id)
                .map_or(-1, |index| index as i32),
            _ => -1,
        }
    }

    pub(crate) fn remove_faction_npc(&mut self, npc_id: i32) -> bool {
        self.faction_npcs
            .iter_mut()
            .fold(false, |removed, npcs| npcs.remove(&npc_id) || removed)
    }

    pub(crate) fn change_npc_faction(&mut self, npc_id: i32, faction: i32) -> bool {
        let Some(target) = gods_battle_faction_index(faction) else {
            return false;
        };
        let Some(previous) = self.npc_faction_index(npc_id) else {
            return false;
        };
        if self.faction_npcs[target].contains(&npc_id) {
            return false;
        }
        if self.faction_npcs[previous].remove(&npc_id) {
            self.faction_npcs[target].insert(npc_id);
        }
        true
    }

    pub(crate) fn is_player_contending_symbol(&self, player_id: i32, symbol_id: i32) -> bool {
        self.contenders
            .iter()
            .any(|contender| contender.player_id == player_id && contender.symbol_id == symbol_id)
    }

    /// Список обрабатывается полностью до player-state и `0xBFF29(0)`.
    /// Как у War, эти безусловные действия выполняет runtime после mutation.
    pub(crate) fn remove_contenders_for_player(&mut self, player_id: i32) {
        self.contenders.retain(|contender| contender.player_id != player_id);
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
        let first_for_legacy_faction = self
            .contenders
            .iter()
            .all(|contender| contender.gods_battle_faction != normal_faction);
        self.contenders.push(GodsBattleContender {
            symbol_id,
            symbol_name: symbol_name.to_vec(),
            player_id,
            gods_battle_faction,
            current_time: 0,
            max_time,
            start_time_ms: now_ms,
        });
        first_for_legacy_faction
    }

    pub(crate) fn cancel_contend_by_symbol(
        &mut self,
        symbol_id: i32,
    ) -> Option<GodsBattleContender> {
        let index = self
            .contenders
            .iter()
            .position(|contender| contender.symbol_id == symbol_id)?;
        Some(self.contenders.remove(index))
    }

    pub(crate) fn advance_contenders(&mut self, now_ms: u32) -> GodsBattleContendAdvance {
        let mut advance = GodsBattleContendAdvance::default();
        for contender in &mut self.contenders {
            let elapsed = now_ms.wrapping_sub(contender.start_time_ms);
            let candidate = (contender.current_time as u32).wrapping_add(elapsed);
            if candidate >= contender.max_time as u32 {
                advance.progress.push((contender.player_id, 100));
                advance.completed.push(contender.clone());
            } else if elapsed >= 1000 {
                contender.current_time = contender.current_time.wrapping_add(elapsed as i32);
                contender.start_time_ms = now_ms;
                let percentage = if contender.max_time == 0 {
                    0
                } else {
                    contender.current_time.wrapping_mul(100) / contender.max_time
                };
                advance.progress.push((contender.player_id, percentage));
            }
        }
        advance
    }
}

fn gods_battle_faction_index(faction: i32) -> Option<usize> {
    match faction {
        7 => Some(0),
        5 => Some(1),
        6 => Some(2),
        _ => None,
    }
}

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetFactionXYD
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:932
// RVA: 0x000A5B50
// ADDRESS: 004a5b50
// PROTOTYPE: ulong __thiscall GetFactionXYD(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::IsPlayerContendSymbol
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:438
// RVA: 0x000A5E00
// ADDRESS: 004a5e00
// PROTOTYPE: bool __thiscall IsPlayerContendSymbol(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::IsGodsBattleRegion
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1216
// RVA: 0x000A6200
// ADDRESS: 004a6200
// PROTOTYPE: bool __thiscall IsGodsBattleRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetReturnPoint
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1340
// RVA: 0x000A6230
// ADDRESS: 004a6230
// PROTOTYPE: bool __thiscall GetReturnPoint(long param_1, ulong param_2, long * param_3, long * param_4, long * param_5, long * param_6, long * param_7, long * param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::DelObj
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:183
// RVA: 0x000A6640
// ADDRESS: 004a6640
// PROTOTYPE: void __thiscall DelObj(int param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnEnterContend
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:410
// RVA: 0x000A66E0
// ADDRESS: 004a66e0
// PROTOTYPE: void __thiscall OnEnterContend(CPlayer * param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetNpcNameByMonster
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1220
// RVA: 0x000A67B0
// ADDRESS: 004a67b0
// PROTOTYPE: bool __thiscall GetNpcNameByMonster(CMonster * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetAlreadyDieCount
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1320
// RVA: 0x000A6850
// ADDRESS: 004a6850
// PROTOTYPE: ulong __thiscall GetAlreadyDieCount(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::RemoveObject
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:138
// RVA: 0x000A7270
// ADDRESS: 004a7270
// PROTOTYPE: void __thiscall RemoveObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CServerGodsBattleRegion::~CServerGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:19
// RVA: 0x000A86B0
// ADDRESS: 004a86b0
// PROTOTYPE: void __thiscall ~CServerGodsBattleRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CServerGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:14
// RVA: 0x000A90F0
// ADDRESS: 004a90f0
// PROTOTYPE: undefined __thiscall CServerGodsBattleRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::GetObjFaction
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:312
// RVA: 0x000A91B0
// ADDRESS: 004a91b0
// PROTOTYPE: Fation __thiscall GetObjFaction(int param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AddContend
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:361
// RVA: 0x000A9270
// ADDRESS: 004a9270
// PROTOTYPE: void __thiscall AddContend(CPlayer * param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CancelContendBySymbol
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:568
// RVA: 0x000A9590
// ADDRESS: 004a9590
// PROTOTYPE: void __thiscall CancelContendBySymbol(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AI
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:592
// RVA: 0x000A9660
// ADDRESS: 004a9660
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a9779
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:639
// RVA: 0x000A9779
// ADDRESS: 004a9779
// PROTOTYPE: undefined Catch@004a9779()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::StrSplit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1161
// RVA: 0x000A9C70
// ADDRESS: 004a9c70
// PROTOTYPE: bool __thiscall StrSplit(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, vector<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetMonsterCountByNpcName
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1237
// RVA: 0x000A9DB0
// ADDRESS: 004a9db0
// PROTOTYPE: ulong __thiscall GetMonsterCountByNpcName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnNpcMonsterDie
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1253
// RVA: 0x000A9FC0
// ADDRESS: 004a9fc0
// PROTOTYPE: void __thiscall OnNpcMonsterDie(CMonster * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnEnterContend
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1078
// RVA: 0x000AA5D0
// ADDRESS: 004aa5d0
// PROTOTYPE: bool __thiscall OnEnterContend(CPlayer * param_1, CNpc * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnNpcUpdate
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1186
// RVA: 0x000AA8A0
// ADDRESS: 004aa8a0
// PROTOTYPE: void __thiscall OnNpcUpdate(CNpc * param_1, ulong param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::CGodsBattleMgr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:893
// RVA: 0x000AAAF0
// ADDRESS: 004aaaf0
// PROTOTYPE: undefined __thiscall CGodsBattleMgr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:912
// RVA: 0x000AABF0
// ADDRESS: 004aabf0
// PROTOTYPE: CGodsBattleMgr * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnNpcSetFaction
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:648
// RVA: 0x000AAC60
// ADDRESS: 004aac60
// PROTOTYPE: void __thiscall OnNpcSetFaction(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnMonsterDie
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:698
// RVA: 0x000AB150
// ADDRESS: 004ab150
// PROTOTYPE: void __thiscall OnMonsterDie(CMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::GetReturnPoint
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:715
// RVA: 0x000AB1C0
// ADDRESS: 004ab1c0
// PROTOTYPE: void __thiscall GetReturnPoint(CPlayer * param_1, long * param_2, long * param_3, long * param_4, long * param_5, long * param_6, long * param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::RefreshMonsterForNpc
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1150
// RVA: 0x000AB740
// ADDRESS: 004ab740
// PROTOTYPE: bool __thiscall RefreshMonsterForNpc(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AddObject
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:24
// RVA: 0x000AB7C0
// ADDRESS: 004ab7c0
// PROTOTYPE: void __thiscall AddObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnChangeFaction
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:226
// RVA: 0x000ABC30
// ADDRESS: 004abc30
// PROTOTYPE: bool __thiscall OnChangeFaction(int param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnContendTimeOver
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:476
// RVA: 0x000ABDF0
// ADDRESS: 004abdf0
// PROTOTYPE: void __thiscall OnContendTimeOver(tagContend * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f3739
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// RVA: 0x000F3739
// ADDRESS: 004f3739
// PROTOTYPE: undefined Catch@004f3739()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f38a6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// RVA: 0x000F38A6
// ADDRESS: 004f38a6
// PROTOTYPE: undefined Catch@004f38a6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
