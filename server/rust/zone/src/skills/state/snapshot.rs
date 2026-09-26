//! Клиентский писатель снимка состояний `CMoveShape::AddToByteArray_ForClient`
//! (паблик off 0xCCD30 → RVA 0xCDD30 → VA 0x004CDD30, moveshape.cpp:1779)
//! исторического GameServer (пара gameserver.exe SHA-256
//! 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ GameServer.pdb RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2, совпадают).
//! Перенесён из hub `appserver/moveshape.rs` (волна Z-M3): методы переходного
//! CMoveShape преобразованы в свободные функции над заимствованными `CShape` и
//! `CanonicalStateStorage`; число членов команды и динамические часы передаёт
//! caller. Тела буквальные — сверены прежними порциями вместе с клиентским
//! каталогом записей.
//!
//! Писатель идёт двумя проходами по живой арене `m_vStates`, без DB Serialize
//! и без уплотнения пустых позиций: сначала счёт непустых позиций арены, затем
//! запись ID/time/additional и имени Team в том же порядке позиций. Значения
//! каждой записи — соседний `catalog` (`state_client_record`); машинное
//! свидетельство проходов и getters остаётся в его шапке. Fresh-вариант до
//! установки первого состояния изображает нулевой count и не притворяется
//! общим проходом. Незагруженный opaque owner блокирует снимок целиком: его
//! клиентский контракт не подтверждён, и неизвестные байты не подменяются
//! пустым состоянием.

use nebokrai_shared::protocol::LegacyWriter;

use crate::regions::shape::CShape;

use super::catalog::state_client_record;
use super::storage::CanonicalStateStorage;

/// Материализует точный fresh-object prefix
/// `CMoveShape::AddToByteArray_ForClient`: после `CShape` идут died-byte и
/// нулевой count состояний. Писатель намеренно не изображает общий state
/// serializer и применяется до установки первого состояния.
pub fn encode_fresh_client_snapshot(
    shape: &CShape,
    include_child: bool,
    is_dead: bool,
) -> Option<Vec<u8>> {
    let mut payload = Vec::new();
    shape
        .add_to_byte_array(&mut payload, include_child)
        .then_some(())?;
    let mut writer = LegacyWriter::new(&mut payload);
    writer.write_u8(u8::from(is_dead));
    writer.write_i32(0);
    Some(payload)
}

/// Материализует общий CMoveShape::AddToByteArray_ForClient из живых
/// экземпляров, а не их DB-записей. Незагруженный opaque owner блокирует
/// snapshot: его клиентский контракт пока не подтверждён.
pub fn encode_client_snapshot(
    shape: &CShape,
    state_storage: &CanonicalStateStorage,
    include_child: bool,
    is_dead: bool,
    timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Option<Vec<u8>> {
    encode_client_snapshot_with_team_count(
        shape,
        state_storage,
        include_child,
        is_dead,
        1,
        timed_state_now_milliseconds,
    )
}

/// CMoveShape::AddToByteArray_ForClient (0x004CDD30): два прохода
/// живого m_vStates, без DB Serialize и без уплотнения пустых позиций.
/// Player-owner передаёт канонический размер CTeam для GetAdditionalData.
pub fn encode_client_snapshot_with_team_count(
    shape: &CShape,
    state_storage: &CanonicalStateStorage,
    include_child: bool,
    is_dead: bool,
    team_member_count: usize,
    mut timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Option<Vec<u8>> {
    // Непрозрачный незагруженный owner не выдаётся за пустое состояние.
    if state_storage.ex_states.opaque_count != 0
        || (!state_storage.ex_states.header_was_present
            && !state_storage.ex_states.opaque_tail.is_empty())
    {
        return None;
    }
    let mut total_count = 0i32;
    for index in 0..state_storage.state_entries.len() {
        if let Some(key) = state_storage.state_entries.address(index) {
            state_storage.state_entries.get(key)?;
            total_count = total_count.checked_add(1)?;
        }
    }
    let mut payload = Vec::new();
    shape
        .add_to_byte_array(&mut payload, include_child)
        .then_some(())?;
    let mut writer = LegacyWriter::new(&mut payload);
    writer.write_u8(u8::from(is_dead));
    writer.write_i32(total_count);
    for index in 0..state_storage.state_entries.len() {
        let Some(key) = state_storage.state_entries.address(index) else { continue };
        let state = state_storage.state_entries.get(key)?;
        writer.write_u32(state.state_id());
        let record =
            state_client_record(state, team_member_count, &mut timed_state_now_milliseconds);
        writer.write_i32(record.time);
        writer.write_u32(record.additional);
        if let Some(name) = record.team_name {
            writer.write_c_string(name);
        }
    }
    Some(payload)
}
