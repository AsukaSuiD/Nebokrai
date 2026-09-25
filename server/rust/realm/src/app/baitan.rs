//! BaiTan-реестр мирового game: очередь заявок, маршруты игроков и счётчики IP.
//!
//! Состав перенесён из `worldserver/worldserver/game.rs`. Исходные методы
//! `CGame` и их четыре `std::map` (+0x56c..+0x594 в образце) подтверждены
//! машинным кодом `Nworldserver.exe` и pubs `WorldServer.pdb` (RSDS
//! 289F1FB3-96A0-4FF4-8B5D-1FD17B50B751 совпадает с образцом, sha256
//! f3ac454d…):
//! `?AddItemToBaiTanRequestList@CGame@@QAEXKJ@Z` 1:0000de30,
//! `?DelItemFromBaiTanList@CGame@@QAEXJ@Z` 1:000119f0,
//! `?AddItemToBaiTanList@CGame@@QAEXJK@Z` 1:00013140,
//! `?DoneBaiTanList@CGame@@QAEXXZ` 1:000131f0.
//!
//! Requests — `ip → player_id` без перезаписи (insert пары, как у исходной
//! `std::map`); refcount растёт `wrapping` при повторном ip (`add [node],1`)
//! и падает на единицу с удалением записи при достижении нуля; routes —
//! `player_id → game server index` тоже без перезаписи. Вычисление маршрута
//! и отправка результата drain-а — игровые сервисы владельца: `&mut self`
//! реестра и `&CGame` по правилам заимствования недоступны одному вызову
//! одновременно, поэтому готовый индекс маршрута приходит параметром.

use std::collections::BTreeMap;

use crate::app::world_message::SendMessageError;

/// Итог регистрации одной bai-tan записи: вставки player→ip и player→route
/// фиксируются отдельно, refcount возвращается после инкремента.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldBaiTanRegistration {
    pub player_id: i32,
    pub ip: u32,
    pub player_ip_inserted: bool,
    pub ip_refcount: i32,
    pub game_server_index: i32,
    pub player_route_inserted: bool,
}

/// Итог удаления из bai-tan списка: найденный ip и остаток refcount различимы,
/// как и фактические удаления записей player→ip и player→route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldBaiTanRemoval {
    pub player_id: i32,
    pub mapped_ip: Option<u32>,
    pub remaining_ip_refcount: Option<i32>,
    pub player_ip_removed: bool,
    pub player_route_removed: bool,
}

/// Одна завершённая заявка drain-а: регистрация и отдельно вычисленный
/// текущий маршрут ответа с исходом отправки `0x0008_040D`.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldBaiTanCompletion {
    pub requested_ip: u32,
    pub player_id: i32,
    pub registration: WorldBaiTanRegistration,
    pub route_game_server_index: i32,
    pub delivery: Result<i32, SendMessageError>,
}

/// Итог drain-а очереди заявок: завершённые записи в порядке итерации map и
/// число заявок, очищенных после обхода.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldDoneBaiTanListReport {
    pub completions: Vec<WorldBaiTanCompletion>,
    pub cleared_requests: usize,
}

/// Четыре map bai-tan семьи исходного `CGame`: очередь заявок по ip,
/// маршруты и адреса игроков, счётчики повторных ip.
#[derive(Default)]
pub struct WorldBaiTanLists {
    requests: BTreeMap<u32, i32>,
    routes: BTreeMap<i32, i32>,
    ip_refcounts: BTreeMap<u32, i32>,
    player_ips: BTreeMap<i32, u32>,
}

impl WorldBaiTanLists {
    pub fn new() -> Self {
        Self::default()
    }

    /// Заявка по ip ставится один раз; `false` — этот ip уже ожидает drain.
    pub fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool {
        if let std::collections::btree_map::Entry::Vacant(entry) = self.requests.entry(ip) {
            entry.insert(player_id);
            true
        } else {
            false
        }
    }

    /// Регистрация ip и маршрута игрока: player→ip и player→route без
    /// перезаписи, refcount растёт `wrapping` при повторном ip. Индекс
    /// маршрута вычисляет владелец игры до вызова — чтение
    /// player/region/game-server не затрагивает реестр.
    pub fn add_item_to_bai_tan_list(
        &mut self,
        player_id: i32,
        ip: u32,
        game_server_index: i32,
    ) -> WorldBaiTanRegistration {
        let player_ip_inserted = if let std::collections::btree_map::Entry::Vacant(entry) =
            self.player_ips.entry(player_id)
        {
            entry.insert(ip);
            true
        } else {
            false
        };

        let ip_refcount = match self.ip_refcounts.get_mut(&ip) {
            Some(refcount) => {
                *refcount = refcount.wrapping_add(1);
                *refcount
            }
            None => {
                self.ip_refcounts.insert(ip, 1);
                1
            }
        };

        let player_route_inserted = if let std::collections::btree_map::Entry::Vacant(entry) =
            self.routes.entry(player_id)
        {
            entry.insert(game_server_index);
            true
        } else {
            false
        };

        WorldBaiTanRegistration {
            player_id,
            ip,
            player_ip_inserted,
            ip_refcount,
            game_server_index,
            player_route_inserted,
        }
    }

    /// Удаление записи: декремент refcount по найденному ip выполняется до
    /// снятия player→ip, затем снимается маршрут — порядок исходного
    /// `DelItemFromBaiTanList`.
    pub fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval {
        let mapped_ip = self.player_ips.get(&player_id).copied();
        let remaining_ip_refcount = mapped_ip.and_then(|ip| self.del_item_to_bai_tan_ip_list(ip));
        let player_ip_removed = self.player_ips.remove(&player_id).is_some();
        let player_route_removed = self.routes.remove(&player_id).is_some();
        WorldBaiTanRemoval {
            player_id,
            mapped_ip,
            remaining_ip_refcount,
            player_ip_removed,
            player_route_removed,
        }
    }

    /// Снимок очереди заявок в порядке итерации map — форма collect перед
    /// drain-ом у владельца списка.
    pub fn request_entries(&self) -> Vec<(u32, i32)> {
        self.requests
            .iter()
            .map(|(&ip, &player_id)| (ip, player_id))
            .collect()
    }

    /// Очистка очереди после drain-а; возвращает число висевших заявок.
    pub fn clear_requests(&mut self) -> usize {
        let cleared_requests = self.requests.len();
        self.requests.clear();
        cleared_requests
    }

    /// Записанный при регистрации маршрут игрока.
    pub fn bai_tan_game_server_index(&self, player_id: i32) -> Option<i32> {
        self.routes.get(&player_id).copied()
    }

    /// Текущий счётчик повторных регистраций одного ip.
    pub fn bai_tan_ip_refcount(&self, ip: u32) -> Option<i32> {
        self.ip_refcounts.get(&ip).copied()
    }

    /// Декремент refcount ip: значение больше единицы уменьшается на месте,
    /// иначе запись удаляется (исходный helper по `Nworldserver.exe`
    /// 0x40d830: `add -1` либо erase узла).
    fn del_item_to_bai_tan_ip_list(&mut self, ip: u32) -> Option<i32> {
        let refcount = self.ip_refcounts.get_mut(&ip)?;
        if 1 < *refcount {
            *refcount -= 1;
            return Some(*refcount);
        }
        self.ip_refcounts.remove(&ip);
        None
    }
}
