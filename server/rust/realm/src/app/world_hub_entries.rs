//! Записи и транспортные контексты таблиц состояния WorldServer (hub-data
//! уровень): materialized-регион, деньги аукциона с точным x87-усечением,
//! отчёты origin-снаряжения и organizing/доставка-контексты обновления
//! faction-информации игрока.
//! Источник контракта — та же точная пара, что у
//! [`crate::app::world_runtime`] (`.exe/Nworldserver.exe` +
//! `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`, RSDS совпадает).
//!
//! Organizing-контексты реализуют realm-трейты [`PlayerOrganizingUpdater`] и
//! [`PlayerFactionInfoContext`]; `CGame` сюда не тянется — маршрут доставки
//! фиксируется полем сессии до извлечения player owner-а из map.
//!
//! Запись присутствия login-игрока перенесена владельцу
//! [`crate::characters::worldplayers`], записи назначения регионов, GameServer
//! и ping-индекса — владельцу [`crate::regions::worldzones`], системная
//! рассылка `tagSysBroadcast` и её AI-отчёты — владельцу
//! [`crate::social::broadcasts`]; прежние пути сохраняются re-export-ами для
//! прежних consumers.

use std::collections::BTreeMap;

use nebokrai_shared::network::ServerCommandHandle;

use crate::app::world_message::CMessage;
use crate::characters::player::{
    PlayerFactionInfoContext, PlayerFactionInfoDelivery, PlayerOrganizingState,
    PlayerOrganizingUpdateError, PlayerOrganizingUpdater, PlayerOriginEquipmentBlock,
    PlayerOriginEquipmentOutcome,
};
use crate::organizations::faction::CFaction;
use crate::organizations::organizingctrl::COrganizingCtrl;

// Записи реестра обслуживающих Zone мира перенесены владельцу `regions`; прежние пути сохранены re-export-ом.
pub use crate::regions::worldzones::{WorldGameServerEntry, WorldRegionAssignment};

// Запись системной broadcast-рассылки и её AI-отчёты перенесены владельцу `social`; прежние пути сохранены re-export-ом.
pub use crate::social::broadcasts::{
    WorldGameAiReport, WorldSystemBroadcast, WorldSystemBroadcastDisposition,
    WorldSystemBroadcastTarget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldAuctionSellerMoney {
    pub fee: i32,
    pub seller_money_after_fee: i32,
}

/// Точно отбрасывает дробную часть произведения signed `long` на один `f32`.
///
/// EXE оставляет произведение в 80-битном x87 до `_ftol2`. Разложение IEEE-754
/// в целую мантиссу и степень сохраняет этот результат без промежуточного
/// округления Rust `f32`; только нештатный overflow/NaN получает определённое
/// насыщение вместо неопределённого C++ float-to-long cast.
pub fn truncate_scaled_legacy_money(amount: i32, factor: f32) -> i32 {
    let bits = factor.to_bits();
    let exponent = (bits >> 23) & 0xFF;
    let fraction = bits & 0x007F_FFFF;
    if exponent == 0xFF {
        if fraction != 0 || amount == 0 {
            return 0;
        }
        return if (amount < 0) ^ (bits >> 31 != 0) {
            i32::MIN
        } else {
            i32::MAX
        };
    }

    let (mantissa, binary_exponent) = if exponent == 0 {
        (u128::from(fraction), -149)
    } else {
        (
            u128::from((1 << 23) | fraction),
            exponent as i32 - 127 - 23,
        )
    };
    let magnitude = u128::from(amount.unsigned_abs()) * mantissa;
    let magnitude = if binary_exponent >= 0 {
        let shift = binary_exponent as u32;
        if shift >= u128::BITS || magnitude > (u128::MAX >> shift) {
            u128::MAX
        } else {
            magnitude << shift
        }
    } else {
        magnitude
            .checked_shr(binary_exponent.unsigned_abs())
            .unwrap_or(0)
    };
    let negative = (amount < 0) ^ (bits >> 31 != 0);
    if negative {
        if magnitude >= 0x8000_0000 {
            i32::MIN
        } else {
            -(magnitude as i32)
        }
    } else {
        magnitude.min(i32::MAX as u128) as i32
    }
}

pub fn truncate_legacy_money(value: f64) -> i32 {
    if value.is_nan() {
        0
    } else if value >= f64::from(i32::MAX) {
        i32::MAX
    } else if value <= f64::from(i32::MIN) {
        i32::MIN
    } else {
        value.trunc() as i32
    }
}

pub use crate::characters::worldplayers::WorldLoginPlayerEntry;

#[derive(Debug)]
pub struct WorldOriginGoodsReport {
    pub entries: Vec<PlayerOriginEquipmentOutcome>,
}

#[derive(Debug)]
pub struct WorldOriginGoodsBlock {
    pub origin_index: usize,
    pub source: PlayerOriginEquipmentBlock,
}

pub struct WorldPlayerOrganizingContext<'a> {
    pub organizing: &'a COrganizingCtrl,
    pub region_types: &'a BTreeMap<i32, Option<u16>>,
}

impl PlayerOrganizingUpdater for WorldPlayerOrganizingContext<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        let mut updater = self.organizing.player_updater(self.region_types);
        updater.set_player_organizing(player_id, organizing)
    }
}

/// Transport-адаптер `CPlayer::UpdateFactionInfo`.
///
/// Маршрут фиксируется до временного извлечения player owner-а из map: исходный
/// lookup выполнялся до вызова send, а повторный поиск через Rust map в этот
/// момент уже не может увидеть заимствованного игрока.
pub struct WorldPlayerFactionInfoContext<'a> {
    pub organizing: WorldPlayerOrganizingContext<'a>,
    pub game_server_id: i32,
    pub sender: Option<ServerCommandHandle>,
}

pub struct WorldFactionPlayerOrganizingContext<'a> {
    pub faction: &'a CFaction,
    pub region_types: &'a BTreeMap<i32, Option<u16>>,
}

impl PlayerOrganizingUpdater for WorldFactionPlayerOrganizingContext<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        self.faction
            .set_player_organizing_projection(player_id, self.region_types, organizing)
    }
}

pub struct WorldDetachedFactionInfoContext<'a> {
    pub organizing: WorldFactionPlayerOrganizingContext<'a>,
    pub game_server_id: i32,
    pub sender: Option<ServerCommandHandle>,
}

impl PlayerOrganizingUpdater for WorldDetachedFactionInfoContext<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        self.organizing
            .set_player_organizing(player_id, organizing)
    }
}

impl PlayerFactionInfoContext for WorldDetachedFactionInfoContext<'_> {
    fn send_player_faction_info(
        &mut self,
        _player_id: i32,
        message: &CMessage,
    ) -> PlayerFactionInfoDelivery {
        PlayerFactionInfoDelivery {
            game_server_id: self.game_server_id,
            result: message.send_to_map_id(self.sender.as_ref(), self.game_server_id),
        }
    }
}

impl PlayerOrganizingUpdater for WorldPlayerFactionInfoContext<'_> {
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        self.organizing
            .set_player_organizing(player_id, organizing)
    }
}

impl PlayerFactionInfoContext for WorldPlayerFactionInfoContext<'_> {
    fn send_player_faction_info(
        &mut self,
        _player_id: i32,
        message: &CMessage,
    ) -> PlayerFactionInfoDelivery {
        PlayerFactionInfoDelivery {
            game_server_id: self.game_server_id,
            result: message.send_to_map_id(self.sender.as_ref(), self.game_server_id),
        }
    }
}
