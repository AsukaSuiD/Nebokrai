//! Клиентские кадры quest-lifecycle и World-запросы переоформления заданий
//! исторического `CGame`: `0xBFF2C/2D/2E/2F` (add/complete/remove/position),
//! `0xBF728/729/72A` (enabled/time begin/clear) и World-запросы `0x6013B/C`
//! для offline игрока. Исходный владелец `appserver/game.cpp`; сверка по
//! точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! Локальный допуск `CPlayer` (`quests/progress`, `quests/availability`),
//! выбор момента отправки и transport остаются у caller-а; здесь — только
//! имя opcode и буквальная запись кадра (short + legacy C-string /
//! short + три long / byte / long, как у исходных writer-ов).
//! Доказательства:
//! docs/reconstruction/gameserver-npc-and-regions.md#квестовые-записи-и-wire-снимки-cplayer

use nebokrai_shared::resources::QuestEntry;

use crate::app::game_message::CMessage;
use crate::trade::session::append_legacy_c_string;

use super::client::append_client_quest_record;

const PLAYER_QUEST_ADD_MESSAGE: i32 = 0x000b_ff2c;
const PLAYER_QUEST_COMPLETE_MESSAGE: i32 = 0x000b_ff2d;
const PLAYER_QUEST_REMOVE_MESSAGE: i32 = 0x000b_ff2e;
const PLAYER_QUEST_POSITION_MESSAGE: i32 = 0x000b_ff2f;
const PLAYER_QUEST_ENABLED_MESSAGE: i32 = 0x000b_f728;
const PLAYER_QUEST_TIME_BEGIN_MESSAGE: i32 = 0x000b_f729;
const PLAYER_QUEST_TIME_CLEAR_MESSAGE: i32 = 0x000b_f72a;
const WORLD_QUEST_ADD_REQUEST: i32 = 0x0006_013b;
const WORLD_QUEST_REMOVE_REQUEST: i32 = 0x0006_013c;

/// Кадр `0xBFF2C` добавления задания: клиентская запись тем же набором и
/// порядком полей, что снимок входа (`quests/client`).
pub fn player_quest_add_frame(quest_id: u16, quest: &QuestEntry) -> CMessage {
    let mut message = CMessage::new(PLAYER_QUEST_ADD_MESSAGE);
    let mut record = Vec::new();
    append_client_quest_record(&mut record, quest_id, quest);
    message.base_mut().add(&record);
    message
}

/// Кадр `0xBFF2D` завершения задания: u16 ID и legacy C-string имени.
pub fn player_quest_complete_frame(quest_id: u16, quest_name: &[u8]) -> CMessage {
    let mut message = CMessage::new(PLAYER_QUEST_COMPLETE_MESSAGE);
    message.base_mut().add_short(quest_id as i16);
    append_legacy_c_string(message.base_mut(), quest_name);
    message
}

/// Кадр `0xBFF2E` удаления задания: u16 ID и legacy C-string имени.
pub fn player_quest_remove_frame(quest_id: u16, quest_name: &[u8]) -> CMessage {
    let mut message = CMessage::new(PLAYER_QUEST_REMOVE_MESSAGE);
    message.base_mut().add_short(quest_id as i16);
    append_legacy_c_string(message.base_mut(), quest_name);
    message
}

/// Кадр `0xBFF2F` позиции задания: u16 ID, затем long region/tile-x/tile-y.
pub fn player_quest_position_frame(
    quest_id: u16,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(PLAYER_QUEST_POSITION_MESSAGE);
    message.base_mut().add_short(quest_id as i16);
    message.add_long(region_id);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

/// Кадр `0xBF728` доступности заданий: один byte флага.
pub fn player_quest_enabled_frame(enabled: bool) -> CMessage {
    let mut message = CMessage::new(PLAYER_QUEST_ENABLED_MESSAGE);
    message.add_byte(u8::from(enabled));
    message
}

/// Кадр `0xBF729` начала отсчёта задания: long лимита.
pub fn player_quest_time_begin_frame(time_limit: i32) -> CMessage {
    let mut message = CMessage::new(PLAYER_QUEST_TIME_BEGIN_MESSAGE);
    message.add_long(time_limit);
    message
}

/// Кадр `0xBF72A` очистки отсчёта задания: пустое тело.
pub fn player_quest_time_clear_frame() -> CMessage {
    CMessage::new(PLAYER_QUEST_TIME_CLEAR_MESSAGE)
}

/// World-запрос `0x6013B` добавления задания offline игроку: long player,
/// u16 ID.
pub fn world_quest_add_request_frame(player_id: i32, quest_id: u16) -> CMessage {
    let mut request = CMessage::new(WORLD_QUEST_ADD_REQUEST);
    request.add_long(player_id);
    request.base_mut().add_short(quest_id as i16);
    request
}

/// World-запрос `0x6013C` удаления задания offline игроку: long player,
/// u16 ID.
pub fn world_quest_remove_request_frame(player_id: i32, quest_id: u16) -> CMessage {
    let mut request = CMessage::new(WORLD_QUEST_REMOVE_REQUEST);
    request.add_long(player_id);
    request.base_mut().add_short(quest_id as i16);
    request
}
