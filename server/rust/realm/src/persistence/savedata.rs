//! Data/handle-типы save-batch одного `DoSaveData` (`tagDBData` владельца
//! `CGame`). Источник контракта — та же точная пара, что у
//! [`crate::app::world_server`] (`.exe/Nworldserver.exe` +
//! `.exe/WorldServer.pdb`; канонические идентификаторы сборки —
//! `server/rust/src/manifest/_worldserver_export_manifest.toml`).
//!
//! Доказательства: docs/reconstruction/realm-services.md#world-процесс-и-lifecycle
//!
//! Accumulator, session facade и отделённый batch owner остаются чистыми
//! данными: generation-ветки (`app/world_db_data_collect`) и DB I/O
//! save worker-а (`persistence/saveworker`) наполняют и разбирают их,
//! поэтому поля публичны.
//! Frozen-вход worker-а `WorldSaveThreadJob` живёт в
//! [`crate::persistence::savedb`] вместе с цитируемым им lifecycle-типом.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use nebokrai_shared::network::ClientSendQueue;

use crate::characters::player::CPlayer;
use crate::organizations::dbcountry::CountrySaveSnapshot;
use crate::organizations::faction::CFaction;
use crate::organizations::rsenemyfactions::EnemyFactionSaveSnapshot;
use crate::organizations::union::CUnion;
use crate::regions::rsregion::RegionSaveSnapshot;

#[derive(Clone, Copy, Debug)]
pub struct DeletionPlayerSnapshot {
    pub player_id: u32,
    pub deletion_time: i32,
}

/// Действующая часть исходного `CGame::tagDBData`.
///
/// Scalar ID получают safe нулевой baseline до обязательного constructor-load
/// `CRsSetup`; Rust-layout не является копией 32-битного MSVC ABI.
/// `DoSaveData` не имеет
/// Village/City War snapshot-полей или save-фаз, поэтому они здесь не
/// резервируются по одному лишь имени пустых DB-адаптеров. Constructor
/// `tagDBData` создавал пустыми все действующие владельцы списков и
/// отображений; `VecDeque` и `BTreeMap` сохраняют их порядок без служебного
/// sentinel/allocator хранилища MSVC.
pub struct WorldDbData {
    pub player_id: u32,
    pub leave_word_id: i32,
    pub creation_players: VecDeque<Box<CPlayer>>,
    pub restore_players: VecDeque<u32>,
    pub deletion_players: VecDeque<DeletionPlayerSnapshot>,
    pub players: BTreeMap<u32, Box<CPlayer>>,
    pub save_factions: VecDeque<Box<CFaction>>,
    pub faction_goods_war_counts: BTreeMap<i32, i32>,
    pub save_unions: VecDeque<Box<CUnion>>,
    pub delete_factions: VecDeque<i32>,
    pub delete_unions: VecDeque<i32>,
    pub enemy_factions: VecDeque<Option<EnemyFactionSaveSnapshot>>,
    pub regions: VecDeque<Option<RegionSaveSnapshot>>,
    pub countries: VecDeque<Option<CountrySaveSnapshot>>,
}

impl WorldDbData {
    pub const fn new() -> Self {
        Self {
            player_id: 0,
            leave_word_id: 0,
            creation_players: VecDeque::new(),
            restore_players: VecDeque::new(),
            deletion_players: VecDeque::new(),
            players: BTreeMap::new(),
            save_factions: VecDeque::new(),
            faction_goods_war_counts: BTreeMap::new(),
            save_unions: VecDeque::new(),
            delete_factions: VecDeque::new(),
            delete_unions: VecDeque::new(),
            enemy_factions: VecDeque::new(),
            regions: VecDeque::new(),
            countries: VecDeque::new(),
        }
    }
}

/// Эксклюзивный доступ `DoSaveData` к одному сформированному `tagDBData`.
///
/// Trigger сначала заканчивает generation под save-барьером, затем передаёт
/// весь batch отдельному owner-у. DB I/O поэтому не удерживает ни game-owner,
/// ни mutex нового accumulator-а, куда MainLoop складывает следующие данные.
pub struct WorldDbDataSaveSession<'game> {
    pub data: &'game mut WorldDbData,
}

/// Полностью отделённый batch одного фонового `SaveThreadFunc`.
///
/// После handoff MainLoop сразу получает новый пустой `WorldDbData`; старые
/// списки и Login sender остаются живы до terminal исхода именно этого save.
pub struct WorldSaveDataOwner {
    pub data: WorldDbData,
    pub login_sender: Option<Arc<ClientSendQueue>>,
}

impl WorldDbDataSaveSession<'_> {
    pub const fn setup_ids(&self) -> (u32, i32) {
        (self.data.player_id, self.data.leave_word_id)
    }

    pub fn creation_players_len(&self) -> usize {
        self.data.creation_players.len()
    }

    pub fn creation_player(&self, index: usize) -> Option<&CPlayer> {
        self.data.creation_players.get(index).map(Box::as_ref)
    }

    pub fn remove_creation_player(&mut self, index: usize) {
        drop(self.data.creation_players.remove(index));
    }

    pub fn restore_players_len(&self) -> usize {
        self.data.restore_players.len()
    }

    pub fn restore_player_id(&self, index: usize) -> Option<u32> {
        self.data.restore_players.get(index).copied()
    }

    pub fn remove_restore_player(&mut self, index: usize) {
        let _ = self.data.restore_players.remove(index);
    }

    pub fn deletion_players_len(&self) -> usize {
        self.data.deletion_players.len()
    }

    pub fn deletion_player(&self, index: usize) -> Option<DeletionPlayerSnapshot> {
        self.data.deletion_players.get(index).copied()
    }

    pub fn remove_deletion_player(&mut self, index: usize) {
        let _ = self.data.deletion_players.remove(index);
    }

    pub fn players(&self) -> &BTreeMap<u32, Box<CPlayer>> {
        &self.data.players
    }

    pub fn remove_player(&mut self, player_id: u32) {
        drop(self.data.players.remove(&player_id));
    }

    pub fn delete_union_ids(&mut self) -> &[i32] {
        self.data.delete_unions.make_contiguous()
    }

    pub fn clear_delete_unions(&mut self) {
        self.data.delete_unions.clear();
    }

    pub fn delete_faction_ids(&mut self) -> &[i32] {
        self.data.delete_factions.make_contiguous()
    }

    pub fn clear_delete_factions(&mut self) {
        self.data.delete_factions.clear();
    }

    pub fn save_factions_len(&self) -> usize {
        self.data.save_factions.len()
    }

    pub fn first_saved_faction_mut(&mut self) -> Option<&mut CFaction> {
        self.data.save_factions.front_mut().map(Box::as_mut)
    }

    pub fn remove_first_saved_faction(&mut self) {
        if let Some(faction) = self.data.save_factions.pop_front() {
            self.data
                .faction_goods_war_counts
                .remove(&faction.faction_id());
        }
    }

    pub fn clear_saved_faction_nodes(&mut self) {
        self.data.save_factions.clear();
        self.data.faction_goods_war_counts.clear();
    }

    pub fn first_saved_faction_goods_war_count(&self) -> Option<i32> {
        let faction_id = self.data.save_factions.front()?.faction_id();
        self.data.faction_goods_war_counts.get(&faction_id).copied()
    }

    pub fn save_unions_len(&self) -> usize {
        self.data.save_unions.len()
    }

    pub fn first_saved_union(&self) -> Option<&CUnion> {
        self.data.save_unions.front().map(Box::as_ref)
    }

    pub fn remove_first_saved_union(&mut self) {
        drop(self.data.save_unions.pop_front());
    }

    pub fn clear_saved_union_nodes(&mut self) {
        self.data.save_unions.clear();
    }

    pub fn regions_len(&self) -> usize {
        self.data.regions.len()
    }

    pub fn region(&self, index: usize) -> Option<Option<RegionSaveSnapshot>> {
        self.data.regions.get(index).copied()
    }

    pub fn destroy_saved_region(&mut self, index: usize) {
        if let Some(region) = self.data.regions.get_mut(index) {
            *region = None;
        }
    }

    pub fn clear_saved_region_nodes(&mut self) {
        self.data.regions.clear();
    }

    pub fn enemy_factions(&mut self) -> &[Option<EnemyFactionSaveSnapshot>] {
        self.data.enemy_factions.make_contiguous()
    }

    pub fn clear_saved_enemy_factions(&mut self) {
        for enemy_faction in &mut self.data.enemy_factions {
            *enemy_faction = None;
        }
        self.data.enemy_factions.clear();
    }

    pub fn countries_len(&self) -> usize {
        self.data.countries.len()
    }

    pub fn first_country(&self) -> Option<Option<&CountrySaveSnapshot>> {
        self.data.countries.front().map(Option::as_ref)
    }

    pub fn remove_first_saved_country(&mut self) {
        drop(self.data.countries.pop_front());
    }
}
