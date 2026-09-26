//! Клиентская запись задания из `CPlayer` Game
//! (`server/gameserver/appserver/player.cpp`).
//! Точная пара `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`
//! (идентификаторы — docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки).
//! Снимок входа `AddQuestDataByteArray_ForClient` VA 0x0043E229–0x0043E339
//! и уведомление добавления `CPlayer::AddQuest` VA 0x004453F4–0x004454FE
//! (`0xBFF2C`) записывают одинаковый набор полей в одном порядке: u16 ID,
//! пять u32 (old, type, level, difficulty, track), три C-строки
//! (short description, name, description), u8 display и четыре i32
//! (region, x, y, effect); последний i32 читается со смещения +0x74
//! после +0x78/+0x7C/+0x80.

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::resources::QuestEntry;

/// Добавляет запись одного задания в клиентский буфер. Фильтр списка
/// и выбор момента отправки остаются у владельца-потребителя.
pub fn append_client_quest_record(payload: &mut Vec<u8>, quest_id: u16, quest: &QuestEntry) {
    let mut writer = LegacyWriter::new(payload);
    writer.write_u16(quest_id);
    writer.write_u32(quest.old);
    writer.write_u32(quest.quest_type);
    writer.write_u32(quest.level);
    writer.write_u32(quest.difficulty);
    writer.write_u32(quest.track);
    writer.write_c_string(&quest.short_description);
    writer.write_c_string(&quest.name);
    writer.write_c_string(&quest.description);
    writer.write_u8(u8::from(quest.display));
    writer.write_i32(quest.region_id);
    writer.write_i32(quest.tile_x);
    writer.write_i32(quest.tile_y);
    writer.write_i32(quest.effect_id);
}
