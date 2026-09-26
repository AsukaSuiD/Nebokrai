//! Формирование ответа World → Login со списком персонажей (`0x1FF02`).
//!
//! Источник: `CRsPlayer::OpenPlayerBase` в точной паре `Nworldserver.exe` +
//! `WorldServer.pdb` (идентификаторы — `server/rust/src/manifest/_worldserver_export_manifest.toml`).
//! Wire-контракт VERIFIED по машинным инструкциям владельца и обеих веток
//! `OpenPlayerBaseInDB`/`OpenPlayerBaseInMem`: байт успеха, account и
//! 16-битный счётчик (повторный 16-битный ноль в пустой ветке), строка —
//! индекс short, id dword, C-строка имени, байты level/occupation/sex/country/head,
//! 11 dword ID экипировки и 11 byte уровней в порядке слотов HELM..FAIRY,
//! region dword, статус удаления char. Статус: restore-кандидат → -1; время
//! только из списка deletion (ветка БД — с fallback `GetPlayerDeletionDate`),
//! нулевое → -1; иначе `deletion_days - (int)(difftime/86400)` 8-битной
//! арифметикой с clamp к 0. Адреса разбора — `docs/reconstruction/realm-services.md`.

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::characters::player::{
    CPlayer, PlayerBaseWireSnapshot, PlayerCodecError, PlayerPropertyCoefficients,
};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::persistence::rsplayer::{PlayerBaseDatabaseRow, RsPlayerOwner};
use crate::persistence::rssetup::WorldTdsClient;
use nebokrai_shared::resources::GlobeSetupSnapshot;

pub const PLAYER_BASE_RESPONSE: i32 = 0x0001_FF02;

/// Доступ обработчика списка к снимкам игрока из карты и очереди сохранения.
/// Контекст организации принадлежит владельцу игры; Realm знает лишь его тип.
pub trait WorldPlayerBaseGameView: WorldGameView {
    type OrganizingContext;

    fn creation_player_count_in_cdkey(&self, account: &[u8]) -> u8;
    fn creation_player_ids_by_cdkey(&self, account: &[u8]) -> Vec<u32>;
    fn clone_map_player_for_base(
        &mut self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing: &Self::OrganizingContext,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError>;
    fn clone_saving_player_for_base(
        &self,
        player_id: u32,
        registry: &GoodsBasePropertiesRegistry,
        organizing: &Self::OrganizingContext,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<Option<Box<CPlayer>>, PlayerCodecError>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerBaseOutcome {
    pub account: Vec<u8>,
    pub succeeded: bool,
    pub declared_count: Option<u8>,
    pub emitted_rows: i32,
    pub response_type: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerBaseWireRow {
    player_id: u32,
    name: Vec<u8>,
    level: u8,
    occupation: u8,
    sex: u8,
    country: u8,
    head: u8,
    equipment_ids: [u32; 11],
    equipment_levels: [u8; 11],
    region_id: i32,
    deletion_status: i8,
}

impl PlayerBaseWireRow {
    pub fn from_database(row: PlayerBaseDatabaseRow, deletion_status: i8) -> Self {
        Self {
            player_id: row.id,
            name: row.name,
            level: row.level,
            occupation: row.occupation,
            sex: row.sex,
            country: row.country,
            head: row.head,
            equipment_ids: row.equipment_ids,
            equipment_levels: row.equipment_levels,
            region_id: row.region_id,
            deletion_status,
        }
    }

    pub fn from_snapshot(
        player_id: u32,
        snapshot: PlayerBaseWireSnapshot,
        deletion_status: i8,
    ) -> Self {
        Self {
            player_id,
            name: snapshot.name,
            level: snapshot.level,
            occupation: snapshot.occupation,
            sex: snapshot.sex,
            country: snapshot.country,
            head: snapshot.head,
            equipment_ids: snapshot.equipment_ids,
            equipment_levels: snapshot.equipment_levels,
            region_id: snapshot.region_id,
            deletion_status,
        }
    }

    pub fn append_to(self, row_index: i32, response: &mut CMessage) {
        response.base_mut().add_short(row_index as i16);
        response.base_mut().add_ulong(self.player_id);
        response.base_mut().add(c_string_prefix(&self.name));
        response.base_mut().add_char(0);
        response.base_mut().add_byte(self.level);
        response.base_mut().add_byte(self.occupation);
        response.base_mut().add_byte(self.sex);
        response.base_mut().add_byte(self.country);
        response.base_mut().add_byte(self.head);
        for equipment_id in self.equipment_ids {
            response.base_mut().add_ulong(equipment_id);
        }
        for equipment_level in self.equipment_levels {
            response.base_mut().add_byte(equipment_level);
        }
        response.base_mut().add_long(self.region_id);
        response.base_mut().add_char(self.deletion_status);
    }
}

fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

pub fn remaining_deletion_days(deletion_days: u32, deletion_time: i32) -> i8 {
    let now = chrono::Local::now().timestamp();
    let elapsed_seconds = now - i64::from(deletion_time);
    let elapsed_days = (elapsed_seconds as f64 / 86_400.0) as i32;
    let remaining = (deletion_days as u8 as i8).wrapping_sub(elapsed_days as u8 as i8);
    remaining.max(0)
}

pub fn send_player_base(
    game: &dyn WorldGameView,
    account: Vec<u8>,
    succeeded: bool,
    declared_count: Option<u8>,
    emitted_rows: i32,
    response: CMessage,
) -> WorldPlayerBaseOutcome {
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldPlayerBaseOutcome {
        account,
        succeeded,
        declared_count,
        emitted_rows,
        response_type: PLAYER_BASE_RESPONSE,
        wire,
        delivery,
    }
}

pub fn send_player_base_failure(
    game: &dyn WorldGameView,
    account: Vec<u8>,
    declared_count: Option<u8>,
) -> WorldPlayerBaseOutcome {
    let mut response = CMessage::new(PLAYER_BASE_RESPONSE);
    response.base_mut().add_char(0);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    send_player_base(game, account, false, declared_count, 0, response)
}

/// Ветка `0x4FB01`: строки БД, живых и создаваемых игроков обрабатываются
/// в прежнем порядке. Параметр БД обобщённый, поскольку `RsPlayerOwner`
/// нельзя использовать как dyn-трейт.
pub async fn on_player_base<G, D>(
    game: &mut G,
    organizing: &G::OrganizingContext,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    globe_setup: &GlobeSetupSnapshot,
    rs_player: &mut D,
    mut player_database: Option<&mut WorldTdsClient>,
    mut request: CMessage,
) -> WorldPlayerBaseOutcome
where
    G: WorldPlayerBaseGameView,
    D: RsPlayerOwner<CPlayer>,
{
    let account = request
        .base_mut()
        .get_str_bytes(0x14)
        .unwrap_or_default();
    let Some(database_count) = rs_player
        .get_player_count_in_db_by_cdkey(&account, player_database.as_deref_mut())
        .await
    else {
        while let Some(notice) = rs_player.pop_notice() {
            eprintln!(
                "WorldServer: получение количества персонажей завершилось ошибкой: {:?}: {}",
                notice.operation, notice.error
            );
        }
        return send_player_base_failure(game, account, None);
    };

    let creation_count = game.creation_player_count_in_cdkey(&account);
    let declared_count = database_count.wrapping_add(creation_count);
    let mut response = CMessage::new(PLAYER_BASE_RESPONSE);
    response.base_mut().add_char(1);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    response.base_mut().add_word(u16::from(declared_count));
    if declared_count == 0 {
        response.base_mut().add_char(1);
        response.base_mut().add(&account);
        response.base_mut().add_char(0);
        // CRsPlayer::OpenPlayerBase, VA 0x0050F816: Add(short), два байта.
        response.base_mut().add_word(0);
        return send_player_base(game, account, true, Some(0), 0, response);
    }

    let database_rows = match rs_player
        .open_player_base_in_db(&account, player_database.as_deref_mut())
        .await
    {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("WorldServer: базовые данные персонажей не прочитаны: {error:?}");
            while let Some(notice) = rs_player.pop_notice() {
                eprintln!(
                    "WorldServer: ошибка чтения базовых данных персонажей: {:?}: {}",
                    notice.operation, notice.error
                );
            }
            return send_player_base_failure(game, account, Some(declared_count));
        }
    };

    let mut row_index = 0_i32;
    for database_row in database_rows {
        let player_id = database_row.id;
        let deletion_status = if game.is_restore_player_exist(player_id) {
            -1
        } else {
            let mut deletion_time = game.deletion_player_time(player_id);
            if deletion_time == 0 {
                deletion_time = rs_player
                    .get_player_deletion_date(player_id, player_database.as_deref_mut())
                    .await;
            }
            if deletion_time == 0 {
                -1
            } else {
                remaining_deletion_days(globe_setup.deletion_days(), deletion_time)
            }
        };

        let runtime = match game.clone_map_player_for_base(
            player_id,
            registry,
            organizing,
            coefficients,
        ) {
            Ok(Some(player)) => match player.player_base_wire_snapshot() {
                Ok(snapshot) => Some(snapshot),
                Err(_) => {
                    return send_player_base_failure(game, account, Some(declared_count));
                }
            },
            Ok(None) => match game.clone_saving_player_for_base(
                player_id,
                registry,
                organizing,
                coefficients,
            ) {
                Ok(Some(player)) => match player.player_base_wire_snapshot() {
                    Ok(snapshot) => Some(snapshot),
                    Err(_) => {
                        return send_player_base_failure(game, account, Some(declared_count));
                    }
                },
                Ok(None) => None,
                Err(_) => {
                    return send_player_base_failure(game, account, Some(declared_count));
                }
            },
            Err(_) => return send_player_base_failure(game, account, Some(declared_count)),
        };
        let row = runtime.map_or_else(
            || PlayerBaseWireRow::from_database(database_row, deletion_status),
            |snapshot| {
                PlayerBaseWireRow::from_snapshot(player_id, snapshot, deletion_status)
            },
        );
        row.append_to(row_index, &mut response);
        row_index = row_index.wrapping_add(1);
    }

    let creation_player_ids = game.creation_player_ids_by_cdkey(&account);
    for player_id in creation_player_ids {
        let snapshot = match game.clone_map_player_for_base(
            player_id,
            registry,
            organizing,
            coefficients,
        ) {
            Ok(Some(player)) => match player.player_base_wire_snapshot() {
                Ok(snapshot) if snapshot.id != 0 => snapshot,
                Ok(_) => continue,
                Err(_) => {
                    return send_player_base_failure(game, account, Some(declared_count));
                }
            },
            Ok(None) => continue,
            Err(_) => return send_player_base_failure(game, account, Some(declared_count)),
        };
        let deletion_time = game.deletion_player_time(player_id);
        let deletion_status = if deletion_time == 0 {
            -1
        } else {
            remaining_deletion_days(globe_setup.deletion_days(), deletion_time)
        };
        PlayerBaseWireRow::from_snapshot(player_id, snapshot, deletion_status)
            .append_to(row_index, &mut response);
        row_index = row_index.wrapping_add(1);
    }

    send_player_base(
        game,
        account,
        true,
        Some(declared_count),
        row_index,
        response,
    )
}
