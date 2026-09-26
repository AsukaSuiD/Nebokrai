//! Данные и скалярные правила городского war-региона `CServerCityRegion`
//! исторического GameServer, перенесённые в Zone `regions/` волной Z-M-X
//! (семья регионов country+nation+city + гейты). Исходный владелец —
//! `appserver/servercityregion.h/.cpp`. Переходный агрегат `CServerCityRegion`
//! остаётся в старом пакете: хранит hub `CServerWarRegion`, карту concrete
//! gates `CCityGate` и делегирует этому агрегату чистый state
//! (defence-return, guard sets, last-attacker колонки) и все скалярные
//! операции без изменения сигнатур; decode-контексты над owner-ом хранилищ,
//! wire-stream readers с `RegionDecodeInputBlock` и evidence-блок остаются у
//! старого пакета.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Достигнутые фазовые callbacks `0x001CF730`,
//! `0x001CFA00..0x001CFD70`, `0x001D09F0`, ownership `0x001CED70/0x001CEF40`,
//! victory `0x001CF1A0`, spatial `0x001CEE80/0x001CEF60`, virtual
//! security/guard attackability `0x001CF0E0/0x001CF050`, gate runtime
//! `0x001CAAA0/0x001CF370..0x001CF640`, clear `0x001CF970`, guard refresh
//! `0x001CF7C0` и direct timeout-forwarding `0x001CFEB0` имеют статус
//! `IMPLEMENTED`; layout gate-полей, decoder/factory returns, фазовый call
//! order, child-ID `+0x158` и base-region registration `VERIFIED_DISASSEMBLY`
//! (наследие шапки старого владельца, без повышения). Оставшаяся поверхность
//! исходного файла сохраняет унаследованный статус `UNKNOWN` у старого
//! пакета; этот модуль переносит только достигнутую семью.
//!
//! `BTreeMap/BTreeSet/Vec` сохраняют STL order. Обязательный defence-return
//! block хранится с нейтральным zero-default до decode без недостижимой
//! Option-границы. Spatial override сохраняет два state-read, defender-only
//! return setup, fallback в базовый/country owner у вызывающего aggregate и
//! игнорирование random bool. `GetSecurity` сначала возвращает `SAFE=2` для
//! mass-state `2` (этот pre-query gate выполняет вызывающий aggregate до
//! cell lookup, сохраняя исходный порядок без лишнего cell read); при найденной
//! клетке scalar-решение действующего `city-state` и marker-а — функция этого
//! модуля. Guard refresh возвращает ordered monster/spawn snapshot.
//! Timeout агрегирует владельцев symbols по faction ID и выбирает первый
//! достаточный ID в map-order; если победителя нет, сохраняется действующий
//! owner faction/union. `OnWinSymbol` внутри `OnFactionVictory` у этой сборки
//! указывает на точный no-op `0x004A8750`, фиктивный callback не создаётся.

use std::collections::{BTreeMap, BTreeSet};

use nebokrai_shared::protocol::LegacyReader;

use super::build::BuildClientPublication;
use super::citygate::CityGateHurtOwnerUpdate;
use super::region::{
    RegionCellAccessBlock, RegionRandomContext, RegionReturnPoint, RegionSecurity,
};
use super::serverregion::returnsetup::ServerReturnSetupBlock;

/// Скалярный state city war-региона без hub-хранилищ: defence-return setup,
/// guard monster/spawn sets и last-attacker колонки `+0x2B8/+0x2BC`
/// (PDB offsets наследуются из шапки старого владельца). Defender faction
/// `+0x2C0` остаётся колонкой hub-aggregate: её читает и переписывает
/// runtime `CGame` напрямую, поэтому state получает её параметром.
/// Поля открыты для раздельных borrow-ов decode-контекста старого пакета,
/// как в остальных data-типах Zone.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ServerCityRegionState {
    pub defence_side_return: CityDefenceReturnState,
    pub guard_monsters: BTreeSet<i32>,
    pub guard_indices: Vec<i32>,
    pub last_gate_attacker_type: i32,
    pub last_gate_attacker_id: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CityDefenceReturnState {
    pub region_id: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub does_recall_when_lost: i32,
    pub move_monster_when_refeash: i32,
    pub use_return: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CityGateBuild {
    pub logical_id: i32,
    pub picture_id: i32,
    pub direction: i32,
    pub action: u16,
    pub max_hp: i32,
    pub defence: i32,
    pub width_increment: i32,
    pub title_x: i32,
    pub title_y: i32,
    pub height_increment: i32,
    pub element_resistance: i32,
    pub name: Vec<u8>,
    pub script: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityReturnPointError {
    Base(ServerReturnSetupBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityEntryError {
    ReturnPoint(CityReturnPointError),
    Cell(RegionCellAccessBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CityVictoryUpdate {
    pub war_number: i32,
    pub region_id: i32,
    pub faction_id: i32,
    pub union_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CityWarLogEffect {
    pub string_id: &'static str,
    pub war_number: i32,
    pub region_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CityWarEndEffect {
    pub log: CityWarLogEffect,
    pub build_updates: Vec<BuildClientPublication>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CityWarTimeoutEffect {
    pub victory: Option<CityVictoryUpdate>,
    pub log_string_id: &'static str,
    pub war_number: i32,
    pub region_name: String,
}

/// Исход агрегации symbols по faction ID в map-order: победитель получает
/// `GS0223` без union, иначе сохраняется действующий owner (`GS0224`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CityWarWinnerDecision {
    pub faction_id: i32,
    pub union_id: i32,
    pub log_string_id: &'static str,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CityGuardRefreshTargets {
    pub monster_ids: Vec<i32>,
    pub spawn_indices: Vec<i32>,
}

pub trait CityReturnPointContext {
    /// Сохраняет первый отброшенный virtual `CShape::GetTileY`.
    fn read_city_player_tile_y(&mut self, player_id: i32) -> i32;

    /// Сохраняет следующий отброшенный virtual `CShape::GetTileX`.
    fn read_city_player_tile_x(&mut self, player_id: i32) -> i32;
}

pub trait CityEntryContext: CityReturnPointContext + RegionRandomContext {
    /// Выполняет virtual player slot `+0x88` с `(x, y)`.
    fn set_city_player_position(&mut self, player_id: i32, x: i32, y: i32);
}

/// Exact `IsOwner` RVA `0x001CED70`: нулевой или чужой faction не владеет.
pub const fn city_is_owner(owned_faction_id: i32, faction_id: i32) -> bool {
    faction_id != 0 && faction_id == owned_faction_id
}

/// Scalar-решение `GetSecurity` после cell query: out-of-bounds и исходный
/// `SAFE` возвращают `SAFE`; fight-state `3` с marker `1` переключает
/// `CITYWAR=3`, иначе возвращается packed base security клетки. Второй
/// state-read передаётся отдельным параметром, как и в оригинале.
// Не const fn: сравнение `RegionSecurity` вызывает derived `PartialEq::eq`,
// который не является const (E0015 в константных функциях).
pub fn city_security_rule(
    cell_security: Option<RegionSecurity>,
    city_war_marker: u8,
    fight_state: i32,
) -> RegionSecurity {
    let security = match cell_security {
        Some(security) => security,
        None => return RegionSecurity::SAFE,
    };
    if security == RegionSecurity::SAFE {
        return RegionSecurity::SAFE;
    }
    if fight_state == 3 && city_war_marker == 1 {
        return RegionSecurity::CITY_WAR;
    }
    security
}

/// Exact virtual `GuardIsAttackAble` `0x001CF050` зависит только от
/// скалярных колонок: вне active city-war state базовый результат остаётся
/// true. Во время state `3` player своей owning faction либо owning union не
/// может быть второй стороной атаки городского стража; нулевые owner IDs не
/// создают защитного совпадения.
pub const fn city_guard_is_attackable(
    city_state: i32,
    target_type: i32,
    target_faction_id: i32,
    target_union_id: i32,
    owned_faction_id: i32,
    owned_union_id: i32,
) -> bool {
    if city_state != 3 || target_type != 400 {
        return true;
    }
    if owned_faction_id != 0 && target_faction_id == owned_faction_id {
        return false;
    }
    owned_union_id == 0 || target_union_id != owned_union_id
}

/// Агрегация `OnWarTimeOut`: считает владельцев symbols по faction ID,
/// выбирает первый достаточный ID в map-order; без победителя сохраняется
/// действующий owner faction/union с log `GS0224`.
pub fn decide_city_war_timeout(
    faction_win_symbol: &BTreeMap<i32, i32>,
    win_victory_symbol_num: i32,
    owned_faction_id: i32,
    owned_union_id: i32,
) -> CityWarWinnerDecision {
    let mut faction_symbols = BTreeMap::<i32, i32>::new();
    for &faction_id in faction_win_symbol.values() {
        let count = faction_symbols.entry(faction_id).or_default();
        *count = count.wrapping_add(1);
    }

    let winner = faction_symbols
        .iter()
        .find(|(_, count)| win_victory_symbol_num <= **count)
        .map(|(&faction_id, _)| faction_id);
    let (faction_id, union_id, log_string_id) = winner
        .map(|faction_id| (faction_id, 0, "GS0223"))
        .unwrap_or((owned_faction_id, owned_union_id, "GS0224"));
    CityWarWinnerDecision {
        faction_id,
        union_id,
        log_string_id,
    }
}

/// Byte-exact `m_DefenceSideRS` block `0x20`: семь signed little-endian
/// DWORD в исходном порядке.
pub fn decode_defence_return(bytes: [u8; 0x20]) -> CityDefenceReturnState {
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

/// Byte-exact city `tagBuild` scalar block `0x2C`: logical ID в `field_00`;
/// action хранится `DWORD`, но PDB-virtual `SetAction` принимает `ushort`.
/// Name/script приходят потоковыми C-строками у вызывающего aggregate.
pub fn city_gate_build_from_block(
    bytes: &[u8; 0x2C],
    name: Vec<u8>,
    script: Vec<u8>,
) -> CityGateBuild {
    CityGateBuild {
        logical_id: city_i32_at(bytes, 0x00),
        picture_id: city_i32_at(bytes, 0x04),
        direction: city_i32_at(bytes, 0x08),
        // `tagBuild` хранит DWORD, но PDB-virtual `SetAction` принимает `ushort`.
        action: LegacyReader::at(bytes, 0x0C)
            .and_then(|mut reader| reader.read_u16())
            .expect("фиксированный city block содержит action"),
        max_hp: city_i32_at(bytes, 0x10),
        defence: city_i32_at(bytes, 0x14),
        width_increment: city_i32_at(bytes, 0x18),
        title_x: city_i32_at(bytes, 0x1C),
        title_y: city_i32_at(bytes, 0x20),
        height_increment: city_i32_at(bytes, 0x24),
        element_resistance: city_i32_at(bytes, 0x28),
        name,
        script,
    }
}

impl ServerCityRegionState {
    /// Defender-only return setup: два исходных state-read и hub-колонка
    /// defender faction передаются параметрами; defender window `3 || 2` с
    /// совпадающим non-zero faction возвращает defence-return rect с
    /// direction `-1`.
    pub fn defence_return_point(
        &self,
        first_state: i32,
        second_state: i32,
        player_faction_id: i32,
        defence_side_faction_id: i32,
    ) -> Option<RegionReturnPoint> {
        let defender_window = first_state == 3 || second_state == 2;
        if !(defender_window
            && player_faction_id != 0
            && player_faction_id == defence_side_faction_id)
        {
            return None;
        }
        let setup = self.defence_side_return;
        Some(RegionReturnPoint {
            region_id: setup.region_id,
            left: setup.left,
            top: setup.top,
            right: setup.right,
            bottom: setup.bottom,
            direction: -1,
        })
    }

    pub fn decode_defence_return_finished(&mut self, state: CityDefenceReturnState) {
        self.defence_side_return = state;
    }

    /// Exact virtual `AddGurdMonster`: повторный ID не меняет набор.
    pub fn add_gurd_monster(&mut self, monster_id: i32) {
        self.guard_monsters.insert(monster_id);
    }

    /// Exact virtual `AddGuardIndex`: первый порядок регистрации сохраняется,
    /// повторный refresh index не добавляется второй раз.
    pub fn add_guard_index(&mut self, refresh_index: i32) {
        if !self.guard_indices.contains(&refresh_index) {
            self.guard_indices.push(refresh_index);
        }
    }

    pub fn guard_refresh_targets(&self) -> CityGuardRefreshTargets {
        CityGuardRefreshTargets {
            monster_ids: self.guard_monsters.iter().copied().collect(),
            spawn_indices: self.guard_indices.clone(),
        }
    }

    /// Применяет exact `CCityGate::OnBeenHurted` owner-effect: совпадающий
    /// region ID обновляет last-attacker колонки, чужой игнорируется.
    pub fn apply_gate_hurt_owner_update(
        &mut self,
        owner_region_id: i32,
        update: CityGateHurtOwnerUpdate,
    ) -> bool {
        if owner_region_id != update.region_id {
            return false;
        }
        self.last_gate_attacker_type = update.attacker_type;
        self.last_gate_attacker_id = update.attacker_id;
        true
    }
}

fn city_i32_at<const N: usize>(bytes: &[u8; N], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("фиксированный city block содержит поле")
}
