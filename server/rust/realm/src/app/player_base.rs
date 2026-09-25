//! Формирование ответа World → Login со списком персонажей (`0x1FF02`).
//!
//! Источник: `CRsPlayer::OpenPlayerBase` в `Nworldserver.exe` (SHA-256
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`)
//! и парный `WorldServer.pdb` (RSDS `289F1FB3-96A0-4FF4-8B5D-1FD17B50B751`, age 1).
//! По инструкциям VA `0x0050F7E0..0x0050F845` подтверждены байт успеха,
//! account, 16-битный счётчик и повторный 16-битный ноль перед возвратом
//! в ветке без строк. Прежний Rust записывал там 32-битный ноль; исправлено
//! в вызывающем обработчике.
//! Порядок остальных полей строки и формула оставшихся дней пока PARTIAL:
//! ниже сохранена прежняя Rust-реализация без изменения алгоритма.

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::characters::player::PlayerBaseWireSnapshot;
use crate::persistence::rsplayer::PlayerBaseDatabaseRow;

pub const PLAYER_BASE_RESPONSE: i32 = 0x0001_FF02;

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
