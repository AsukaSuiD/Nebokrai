//! Скалярная база `CMonster`: script/tame/pet колонки и их правила. Исходный
//! владелец — `appserver/monster.h/.cpp`; сверка по точной паре `gameserver.exe`
//! + `GameServer.pdb`. Переходный агрегат `CMonster` остаётся в старом пакете,
//! хранит те же колонки и делегирует сюда их поведение без изменения сигнатур;
//! нематериальные accessor-ы полей (`master_info()`, `hit_points()` и им
//! подобные) остаются у переходного владельца.
//!
//! Строка script копируется до первого NUL без завершающего нуля; `MasterInfo`
//! занимает десять DWORD и совпадает с `zone::combat::masterinfo`. Guard-ы
//! tamable-проверки и отказа перезаписи живой master-связи переходный владелец
//! назначает координатору `skills/monstertaming.rs`, здесь хранится только сама
//! запись. Известное расхождение: родной `SetPetLevel` отсекает запись уровня
//! 10+, а переходный владелец намеренно сохраняет прежнее единое поведение
//! setter без отсечения — вопрос отложен для pet-поведения.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#npc-и-базовые-фигуры

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::resources::MonsterProperties;

use super::moveshape::{is_died, MoveShapeState};
use crate::app::game_message::CMessage;
use crate::combat::MasterInfo;

/// Exact `CMonster::DoesCreatureBeenTamed` (RVA `0x000E6460`): одного
/// внутреннего знака недостаточно, требуется живой identity хозяина-игрока.
pub const fn has_player_pet_master(tamed: bool, master_info: MasterInfo) -> bool {
    tamed && master_info.master_type == 400 && master_info.master_id != 0
}

/// Запись знака приручения (поле `m_bHasBeenTamed`, `+0x224`). Guard-ы
/// родного `SetTamedSign` (RVA `0x000E64A0`) остаются у координатора.
pub const fn set_tamed(tamed: &mut bool, next: bool) {
    *tamed = next;
}

/// Тело inline `IsTamable`, наблюдаемое в машинном коде `SetTamedSign`
/// (RVA `0x000E64A0`): свойство tamable и неисчерпанный счётчик попыток.
pub const fn is_tamable(tame_attempt_count: u32, property: &MonsterProperties) -> bool {
    property.tamable == 1 && tame_attempt_count < property.maximum_tame_attempt_count
}

/// Exact `CMonster::IncreaseTameAttemptCount` (RVA `0x000E6490`): DWORD
/// наращивается в исходном порядке taming до проверки признака.
pub const fn increase_tame_attempt_count(tame_attempt_count: &mut u32) {
    *tame_attempt_count = tame_attempt_count.wrapping_add(1);
}

/// Принадлежность питомца конкретному игроку: тип хозяина `400` и его id.
pub const fn is_owned_pet(master_info: MasterInfo, player_id: i32) -> bool {
    master_info.master_type == 400 && master_info.master_id == player_id
}

/// Запись пары pet-progress колонок, родной парой `SetPetLevel`
/// (RVA `0x000E6D00`) и `SetPetExperience` (`0x000E6D20`).
pub const fn set_pet_progress(
    pet_level: &mut u32,
    pet_experience: &mut u32,
    level: u32,
    experience: u32,
) {
    *pet_level = level;
    *pet_experience = experience;
}

/// Чтение пары pet-progress колонок, родной парой `GetPetLevel`
/// (RVA `0x000E6CE0`) и `GetPetExperience` (`0x000E6CF0`).
pub const fn pet_progress(pet_level: u32, pet_experience: u32) -> (u32, u32) {
    (pet_level, pet_experience)
}

/// Запись бит-хранилища десяти float-множителей питомца; родной
/// `AdjustPetProperties` перезаписывает массив множителей целиком.
pub fn adjust_pet_factors(factors: &mut [u32; 10], next: [f32; 10]) {
    *factors = next.map(f32::to_bits);
}

/// Refresh-блок монстра: знак обновления, признак и дистанция лидера и
/// индекс refresh-таблицы записываются одним вызовом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonsterRefreshData {
    pub sign: u16,
    pub leader_sign: u16,
    pub leader_distance: u16,
    pub refresh_index: i32,
}

/// Запись refresh-блока целиком, в исходном порядке колонок.
pub const fn set_refresh_data(
    sign: &mut u16,
    leader_sign: &mut u16,
    leader_distance: &mut u16,
    refresh_index: &mut i32,
    data: MonsterRefreshData,
) {
    *sign = data.sign;
    *leader_sign = data.leader_sign;
    *leader_distance = data.leader_distance;
    *refresh_index = data.refresh_index;
}

/// Точный tail `CMonster::SetScriptFile` (RVA `0x00038C70`): strcpy-семантика
/// прекращает копию на первом NUL, а колонка хранит байты имени файла без
/// завершающего нуля.
pub fn set_script_file(script_file: &mut Vec<u8>, value: &[u8]) {
    let prefix_len = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    script_file.clear();
    script_file.extend_from_slice(&value[..prefix_len]);
}

/// Политика отображаемого имени монстра: назначенное spawn-ом имя объекта
/// имеет приоритет, пустое возвращается на исходное имя property-записи.
pub fn display_name<'a>(registered_name: &'a [u8], original_name: &'a [u8]) -> &'a [u8] {
    if registered_name.is_empty() {
        original_name
    } else {
        registered_name
    }
}

/// Точная завершающая часть владельца защиты первого нападающего из
/// `CMonster::OnBeenHurted` (RVA `0x000E6EF0`, appserver/monster.cpp:920):
/// чужой удар в окне защиты отклоняется без записи таймера, свой повторяет
/// чтение часов и продлевает окно, вне окна первый нападающий
/// переустанавливается.
pub fn register_attacking_player(
    first_attack_player_id: &mut i32,
    last_attack_timer_ms: &mut u32,
    attacker_player_id: i32,
    protection_ms: u32,
    mut clock: impl FnMut() -> u32,
) -> bool {
    if *first_attack_player_id != 0
        && clock().wrapping_sub(*last_attack_timer_ms) <= protection_ms
    {
        if *first_attack_player_id != attacker_player_id {
            return false;
        }
        *last_attack_timer_ms = clock();
        return true;
    }
    *first_attack_player_id = attacker_player_id;
    *last_attack_timer_ms = clock();
    true
}

/// Скалярная часть допуска Nation-уведомления из `CMonster::OnBeenHurted`:
/// действие `ACT_DIED` и смерть по здоровью отсекаются независимо.
pub const fn can_trigger_nation_damage(action: u16, hit_points: u32) -> bool {
    action != 6 && !is_died(hit_points)
}

/// Заёмная проекция `CMonster` для клиентского снимка монстра: hub старого
/// пакета передаёт свои колонки и вычисленное `maximum_hp` (та формула —
/// `combat/monsterformula`), разрешение имени хозяина остаётся владельцу игре.
pub struct MonsterClientSnapshotParts<'a> {
    pub move_shape: &'a MoveShapeState,
    pub hit_points: u32,
    pub maximum_hp: u32,
    pub tamed: bool,
    pub master_info: MasterInfo,
    pub pet_level: u32,
    pub pet_experience: u32,
    pub property: &'a MonsterProperties,
    pub master_name: &'a [u8],
}

/// Точный fresh-monster `AddToByteArray` tail поверх client-prefix
/// `CMoveShape`: max_hp/hp/kind/figure/sound/picture/name_color/hp_bar и
/// tamed-ветвления 1/2/0. Статусы ветвлений перенесены без повышения.
fn append_monster_client_snapshot_tail(parts: &MonsterClientSnapshotParts, payload: &mut Vec<u8>) {
    let property = parts.property;
    let mut writer = LegacyWriter::new(payload);
    writer.write_u32(parts.maximum_hp);
    writer.write_u32(parts.hit_points);
    writer.write_u8(property.kind as u8);
    writer.write_u8(property.figure as u8);
    writer.write_u16(property.sound_id as u16);
    writer.write_u8(property.picture_level as u8);
    writer.write_u8(property.name_color as u8);
    writer.write_u8(property.hp_bar_color as u8);

    if parts.tamed && parts.master_info.master_type == 400 && parts.master_info.master_id != 0 {
        writer.write_u8(1);
        writer.write_i32(parts.master_info.master_type);
        writer.write_i32(parts.master_info.master_id);
        writer.write_c_string(parts.master_name);
        writer.write_u32(parts.pet_level);
        writer.write_u32(parts.pet_experience);
    } else if property.tamable == 1 && property.maximum_tame_attempt_count == 0 {
        writer.write_u8(2);
        writer.write_i32(parts.master_info.master_type);
        writer.write_i32(parts.master_info.master_id);
        writer.write_c_string(parts.master_name);
    } else {
        writer.write_u8(0);
    }
}

/// Полный fresh-снимок: client-prefix `CMoveShape` плюс хвост монстра.
pub fn encode_fresh_monster_client_snapshot(parts: &MonsterClientSnapshotParts) -> Option<Vec<u8>> {
    let mut payload = crate::skills::state::encode_fresh_client_snapshot(
        parts.move_shape.shape(),
        true,
        parts.hit_points == 0,
    )?;
    append_monster_client_snapshot_tail(parts, &mut payload);
    Some(payload)
}

/// Вариант с timed-state блоком (те же часы, что двигает caller).
pub fn encode_monster_client_snapshot(
    parts: &MonsterClientSnapshotParts,
    timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Option<Vec<u8>> {
    let mut payload = crate::skills::state::encode_client_snapshot(
        parts.move_shape.shape(),
        &parts.move_shape.state_storage,
        true,
        parts.hit_points == 0,
        timed_state_now_milliseconds,
    )?;
    append_monster_client_snapshot_tail(parts, &mut payload);
    Some(payload)
}

fn build_monster_enter_envelope(move_shape: &MoveShapeState, payload: Vec<u8>) -> Option<CMessage> {
    let identity = move_shape.shape().identity();
    let mut message = CMessage::new(0x000b_f502);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.base_mut().add_guid(identity.ex_id);
    message.add_long(i32::try_from(payload.len()).ok()?);
    message.base_mut().add(&payload);
    message.add_byte(0);
    Some(message)
}

/// Exact `CServerRegion::AddMonster` envelope `0xBF502`: идентичность фигуры,
/// длина payload, байт 0. Выбор around-получателей остаётся у owning
/// region/CGame и не дублируется здесь.
pub fn build_fresh_monster_enter_message(parts: &MonsterClientSnapshotParts) -> Option<CMessage> {
    let payload = encode_fresh_monster_client_snapshot(parts)?;
    build_monster_enter_envelope(parts.move_shape, payload)
}

/// Вариант `0xBF502` с timed-state снимком.
pub fn build_monster_enter_message(
    parts: &MonsterClientSnapshotParts,
    timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Option<CMessage> {
    let payload = encode_monster_client_snapshot(parts, timed_state_now_milliseconds)?;
    build_monster_enter_envelope(parts.move_shape, payload)
}
