//! Данные, top-ten wire decoder и скалярные правила GodsBattle владельцев
//! (`CGodsBattleMgr`, `CServerGodsBattleRegion`). Исходный владелец —
//! `appserver/servergodsbattleregion.h/.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb` (идентификаторы сборки —
//! `server/rust/src/manifest/_gameserver_export_manifest.toml`; статусы
//! унаследованы от шапки старого владельца, без повышения). Переходные
//! агрегаты остаются в старом пакете: manager хранит configuration
//! `CGodsBattleConf` (setup владелец старого пакета), а region — hub
//! `CServerWarRegion`; этому модулю делегируются top-ten entry/error/decoder,
//! скалярный state manager-а (ordered region set, knock standings,
//! единственный top-ten requester) и скалярный state региона (faction
//! player/NPC membership и ordered contender-list) со всеми их операциями.
//! NPC guard/counter lifecycle и GodsBattle-contend исполняет `CGame`; точный
//! script case `11130` является живым caller-ом, а byte-owned имя contender
//! сохраняет исходную GBK.
//!
//! Region-set хранит ordered unique ID. Top-ten SZL exchange хранит
//! единственный overwrite-able requester и terminal-marker decoder: безразмерный
//! pointer и 256-байтный временный C-string buffer заменены bounded
//! slice/cursor и owned bytes; обрыв возвращает typed error после уже
//! завершённого prefix-а вместо неназначаемого legacy UB. Knock standings
//! повторно разрешают killer identity в owning region до изменения
//! kill-counter manager-ом. Проверка first-contender намеренно сравнивает
//! normal `m_lFactionID` с сохранённым GodsBattle faction: это несовпадение
//! подтверждено RVA `0x000A9270`, а не исправлено по более позднему
//! C++-донору. `CancelContendByPlayerID` удаляет все записи игрока, сохраняя
//! порядок остальных; только NULL player возвращает false.
//! `CGodsBattleMgr::OnEnterContend` сравнивает индекс NPC-set `0..2` с player
//! faction `5/6`: для допустимого игрока ветвь `SZLGS7` недостижима, поэтому
//! после гибели всех стражей собственный символ тоже начинает contend. Это
//! подтверждённое различие представлений не нормализуется.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use nebokrai_shared::protocol::LegacyReader;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GodsBattleTopTenEntry {
    pub faction: i32,
    pub name: Vec<u8>,
    pub szl: u32,
    pub level: u32,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum GodsBattleTopTenDecodeError {
    #[error("GodsBattle top-ten обрывается на {offset} в поле {field}")]
    UnexpectedEnd { offset: usize, field: &'static str },
    #[error("GodsBattle top-ten name с {offset} не имеет NUL-терминатора")]
    MissingNameTerminator { offset: usize },
    #[error("GodsBattle top-ten name с {offset} длиной {length} не помещается в 260 байт")]
    NameOutsideLegacyBuffer { offset: usize, length: usize },
}

/// Exact `CGodsBattleMgr` terminal-marker decoder: читает пары
/// marker/faction, C-строку имени в пределах legacy `char[260]` и два signed
/// DWORD; нулевой marker завершает список, а missing NUL и переполнение
/// остаются typed-границами.
pub fn decode_gods_battle_top_ten(
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

/// Скалярный state GodsBattle manager-а без configuration: ordered region
/// set, knock standings по byte-имени NPC и единственный overwrite-able
/// top-ten requester.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GodsBattleMgrState {
    region_set: BTreeSet<i32>,
    killed_monster_count: BTreeMap<Vec<u8>, u32>,
    pending_top_ten_player_id: i32,
}

impl GodsBattleMgrState {
    pub fn add_region_set(&mut self, region_id: i32) -> bool {
        self.region_set.insert(region_id)
    }

    /// Exact `CGodsBattleMgr::IsGodsBattleRegion`: ordered set membership без
    /// дополнительной проверки concrete region owner.
    pub fn is_gods_battle_region(&self, region_id: i32) -> bool {
        self.region_set.contains(&region_id)
    }

    pub fn region_ids(&self) -> Vec<i32> {
        self.region_set.iter().copied().collect()
    }

    pub const fn pending_top_ten_player_id(&self) -> i32 {
        self.pending_top_ten_player_id
    }

    /// Exact post-send assignment `GetTopTenSZL`: concurrent request
    /// перезаписывает единственный legacy requester без sequence ID.
    pub const fn record_top_ten_request(&mut self, player_id: i32) {
        self.pending_top_ten_player_id = player_id;
    }

    pub fn npc_killed_monster_count(&self, name: &[u8]) -> u32 {
        self.killed_monster_count.get(name).copied().unwrap_or(0)
    }

    pub fn reset_npc_killed_monster_count(&mut self, name: &[u8]) {
        self.killed_monster_count.insert(name.to_vec(), 0);
    }

    pub fn increment_npc_killed_monster_count(&mut self, name: &[u8]) -> Option<u32> {
        let count = self.killed_monster_count.get_mut(name)?;
        *count = count.wrapping_add(1);
        Some(*count)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GodsBattleContender {
    pub symbol_id: i32,
    pub symbol_name: Vec<u8>,
    pub player_id: i32,
    pub gods_battle_faction: i32,
    pub current_time: i32,
    pub max_time: i32,
    pub start_time_ms: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GodsBattleContendAdvance {
    pub progress: Vec<(i32, i32)>,
    pub completed: Vec<GodsBattleContender>,
}

/// Скалярный state concrete GodsBattle region: faction player/NPC membership
/// `[neutral, faction-5, faction-6]` и ordered contender-list. Поля открыты
/// для раздельных borrow-ов decode-контекста старого пакета.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GodsBattleRegionState {
    pub faction_players: [BTreeSet<i32>; 3],
    pub faction_npcs: [BTreeSet<i32>; 3],
    contenders: Vec<GodsBattleContender>,
}

impl GodsBattleRegionState {
    pub fn add_faction_npc_membership(
        faction_npcs: &mut [BTreeSet<i32>; 3],
        npc_id: i32,
        faction: i32,
    ) -> bool {
        let Some(index) = gods_battle_faction_index(faction) else {
            return false;
        };
        faction_npcs[index].insert(npc_id)
    }

    pub fn add_faction_player(&mut self, player_id: i32, faction: i32) -> bool {
        match faction {
            5 => self.faction_players[1].insert(player_id),
            6 => self.faction_players[2].insert(player_id),
            _ => false,
        }
    }

    pub fn remove_faction_player(&mut self, player_id: i32) -> bool {
        self.faction_players
            .iter_mut()
            .fold(false, |removed, players| {
                players.remove(&player_id) || removed
            })
    }

    pub fn faction_player_ids(&self, faction: i32) -> Option<Vec<i32>> {
        let index = match faction {
            5 => 1,
            6 => 2,
            _ => return None,
        };
        Some(self.faction_players[index].iter().copied().collect())
    }

    pub fn npc_faction_index(&self, npc_id: i32) -> Option<usize> {
        self.faction_npcs
            .iter()
            .position(|npcs| npcs.contains(&npc_id))
    }

    /// Exact `CServerGodsBattleRegion::GetObjFaction`: player faction 5/6
    /// преобразуется в raw `1/2`, NPC возвращает индекс одного из трёх sets,
    /// а неизвестный type/object — `-1` (`INVALID_FACTION`).
    pub fn get_obj_faction(
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

    pub fn remove_faction_npc(&mut self, npc_id: i32) -> bool {
        self.faction_npcs
            .iter_mut()
            .fold(false, |removed, npcs| npcs.remove(&npc_id) || removed)
    }

    pub fn change_npc_faction(&mut self, npc_id: i32, faction: i32) -> bool {
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

    pub fn is_player_contending_symbol(&self, player_id: i32, symbol_id: i32) -> bool {
        self.contenders
            .iter()
            .any(|contender| contender.player_id == player_id && contender.symbol_id == symbol_id)
    }

    /// Список обрабатывается полностью до player-state и `0xBFF29(0)`.
    /// Как у War, эти безусловные действия выполняет runtime после mutation.
    pub fn remove_contenders_for_player(&mut self, player_id: i32) {
        self.contenders.retain(|contender| contender.player_id != player_id);
    }

    /// Проверка first-for-faction использует normal `m_lFactionID`, тогда как
    /// contender хранит `lGodsBattleFaciton`; несовпадение подтверждено EXE.
    pub fn add_contender(
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

    pub fn cancel_contend_by_symbol(
        &mut self,
        symbol_id: i32,
    ) -> Option<GodsBattleContender> {
        let index = self
            .contenders
            .iter()
            .position(|contender| contender.symbol_id == symbol_id)?;
        Some(self.contenders.remove(index))
    }

    pub fn advance_contenders(&mut self, now_ms: u32) -> GodsBattleContendAdvance {
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
