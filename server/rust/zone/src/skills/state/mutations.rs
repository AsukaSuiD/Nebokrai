//! Мутирующие операции арены применённых состояний `CMoveShape` из
//! GameServer.exe/GameServer.pdb (пара gameserver.exe SHA-256
//! 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ GameServer.pdb RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2, совпадают).
//! Перенесены из hub `appserver/moveshape.rs` (волна Z-M2b): тики item-часов
//! undead/extended, mut-проекция и append-семья automatic restore (хвост
//! RestoreHpMp 0x00445643..0x00445901), take/begin boss blue fury, replace/take
//! boss blue quake, remove/take/restore защитных щитов, replace/take spiderweb
//! и истёкший battle fairy attribute, а также общий RAW-слой записей
//! Serialize-cache: append/remove/insert со счётчиком, splice замены и
//! span-сдвиги. Порядок записей и машинные якоря Save/RemoveState
//! (0x004D10F0, 0x004CDAB0/0x004CDB20) — прежние свидетельства hub,
//! зафиксированные в шапках `storage` и `serialization`; читающие половины
//! этих же семейств — соседний `accessors`. Тела буквальные: методы
//! переходного CMoveShape преобразованы в свободные функции над
//! заимствованным `CanonicalStateStorage`; region владельца для `mark_begun`
//! передаётся значением `i32`, боевые свойства игрока — zone-проекцией
//! `combat::PlayerCombatProperties`. Сопоставление ordinal/span при удалении
//! (`remove_applied_state_data_inner`) хранится здесь в единственном
//! экземпляре: оставшиеся у hub вызовы (cure-экземпляр, destructor-хвост
//! ClearAllStates) используют его через делегации прежних сигнатур, без
//! второй копии особых ветвей Swordship/EnergyHolding.

use nebokrai_shared::protocol::LegacyWriter;

use crate::combat::PlayerCombatProperties;
use crate::effects::{
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreState, BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES,
    BOSS_BLUE_FURY_STATE_BYTES, BOSS_BLUE_QUAKE_STATE_BYTES, BattleFairyAttributeState,
    BossBlueFuryState, BossBlueQuakeState, ChangeBodyState, DefenseShieldState, ExtendedState,
    ExtendedStateKind, LIFE_SHIELD_STATE_BYTES, MACHINE_SHIELD_STATE_BYTES, MANA_SHIELD_STATE_BYTES,
    PROMOTION_STATE_BYTES, RideState, SPIDER_WEB_STATE_BYTES, SpiderWebState, UndeadState,
};
use crate::skills::statefactory::{known_state_record_offsets, known_state_record_spans};

use super::accessors::{defense_shield, defense_shield_key, defense_shield_keys};
use super::serialization::{read_u32, write_u32};
use super::storage::{
    AppliedState, CanonicalStateStorage, LegacyStateCodec, StateBatch, StateData, StateKey,
};

/// Хвост RestoreHpMp (0x00445643..0x00445901): четыре новых экземпляра
/// после общего End-обхода. Begin(null, holder) не читает часы; старые
/// состояния здесь повторно не удаляются и UpdateProperty не вызывается.
pub fn append_automatic_hp_mp_states(
    storage: &mut CanonicalStateStorage,
    properties: PlayerCombatProperties,
    region_id: i32,
) {
    for state in AutomaticRestoreState::restored(properties.into()) {
        append_automatic_restore_state(storage, state, region_id);
    }
}

/// Общий NULL-user Begin свежего restore: без clock и пакета, visual loop=1.
pub fn append_automatic_restore_state(
    storage: &mut CanonicalStateStorage,
    state: AutomaticRestoreState,
    region_id: i32,
) -> StateKey {
    append_serialized_state_record(&mut storage.ex_states, &state.encoded_for_install());
    let offset = storage.ex_states.len() - AUTOMATIC_RESTORE_STATE_BYTES;
    let key = storage.state_entries.append(state);
    storage.state_entries.set_serialized_span(key, (offset, AUTOMATIC_RESTORE_STATE_BYTES));
    storage.state_entries.mark_begun(key, region_id);
    storage.state_entries.begin_visual(key, 1);
    key
}

pub fn automatic_restore_state_mut(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
) -> Option<&mut AutomaticRestoreState> {
    AutomaticRestoreState::as_data_mut(storage.state_entries.get_mut(key)?)
}

pub fn take_boss_blue_fury_state(storage: &mut CanonicalStateStorage) -> Option<BossBlueFuryState> {
    let state = storage.state_entries.take_first::<BossBlueFuryState>()?;
    remove_serialized_state_record(storage, state.skill_id(), BOSS_BLUE_FURY_STATE_BYTES);
    Some(state)
}

pub fn begin_boss_blue_fury_state(storage: &mut CanonicalStateStorage, state: BossBlueFuryState) {
    append_serialized_state_record(&mut storage.ex_states, &state.encoded_for_install());
    storage.state_entries.append(state);
}

pub fn replace_boss_blue_quake_state(
    storage: &mut CanonicalStateStorage,
    state: BossBlueQuakeState,
) -> Option<BossBlueQuakeState> {
    remove_serialized_state_record(storage, state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
    append_serialized_state_record(&mut storage.ex_states, &state.encoded_for_install());
    {
        let previous = storage.state_entries.take_first::<BossBlueQuakeState>();
        storage.state_entries.append(state);
        previous
    }
}

pub fn take_boss_blue_quake_state(storage: &mut CanonicalStateStorage) -> Option<BossBlueQuakeState> {
    let state = storage.state_entries.take_first::<BossBlueQuakeState>()?;
    remove_serialized_state_record(storage, state.skill_id(), BOSS_BLUE_QUAKE_STATE_BYTES);
    Some(state)
}

pub fn remove_defense_shield(
    storage: &mut CanonicalStateStorage,
    skill_id: u32,
) -> Option<DefenseShieldState> {
    remove_defense_shield_key(storage, defense_shield_key(storage, skill_id)?)
}

pub fn remove_defense_shield_key(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
) -> Option<DefenseShieldState> {
    let state_id = defense_shield(storage, key)?.skill_id();
    let occurrence = defense_shield_keys(storage).into_iter()
        .filter(|candidate| {
            defense_shield(storage, *candidate).is_some_and(|state| state.skill_id() == state_id)
        })
        .position(|candidate| candidate == key)?;
    let offset = known_state_record_offsets(&storage.ex_states)
        .into_iter()
        .filter(|offset| read_u32(&storage.ex_states, *offset) == Some(state_id))
        .nth(occurrence);
    let state = storage.state_entries.take::<DefenseShieldState>(key)?;
    let bytes = match state {
        DefenseShieldState::Mana(_) => MANA_SHIELD_STATE_BYTES,
        DefenseShieldState::Machine(_) => MACHINE_SHIELD_STATE_BYTES,
        DefenseShieldState::Life(_) => LIFE_SHIELD_STATE_BYTES,
        DefenseShieldState::Promotion(_) => PROMOTION_STATE_BYTES,
    };
    if let Some(offset) = offset {
        remove_serialized_state_record_at(storage, offset, bytes);
    }
    Some(state)
}

fn append_serialized_state_record(ex_states: &mut LegacyStateCodec, record: &[u8]) {
    if ex_states.len() < 4 {
        ex_states.clear();
        LegacyWriter::new(ex_states).write_u32(0);
    }
    let count = read_u32(ex_states, 0).expect("счётчик состояний");
    write_u32(ex_states, 0, count.wrapping_add(1));
    ex_states.extend_from_slice(record);
}

/// Общий push_back уже успешно начатого concrete state. DB-cache здесь
/// технический: record подготовлен owner-ом без вызова игрового Serialize
/// и без повторных часов. Новый span принадлежит тому же поколенческому
/// ключу, поэтому дубли ID удаляются независимо. End/Update остаются caller-у.
pub fn append_applied_state_record<T: AppliedState>(
    storage: &mut CanonicalStateStorage,
    state: T,
    record: &[u8],
) -> StateKey {
    append_serialized_state_record(&mut storage.ex_states, record);
    let span = (storage.ex_states.len() - record.len(), record.len());
    let key = storage.state_entries.append(state);
    storage.state_entries.set_serialized_span(key, span);
    key
}

/// Первый живой слот исходного m_vStates. Предикат задаёт игровой выбор
/// caller-а; метод не копирует payload, не уплотняет и не вызывает End.
/// После callback caller перечитывает эту позицию, если это требует EXE.
pub fn find_state_position(
    storage: &CanonicalStateStorage,
    mut matches: impl FnMut(&StateData) -> bool,
) -> Option<(usize, StateKey)> {
    (0..storage.state_entries.len()).find_map(|index| {
        let key = storage.state_entries.address(index)?;
        let state = storage.state_entries.get(key)?;
        matches(state).then_some((index, key))
    })
}

fn remove_serialized_state_record(
    storage: &mut CanonicalStateStorage,
    state_id: u32,
    amount: usize,
) -> bool {
    let Some(offset) = known_state_record_offsets(&storage.ex_states)
        .into_iter()
        .find(|offset| read_u32(&storage.ex_states, *offset) == Some(state_id))
    else {
        return false;
    };
    remove_serialized_state_record_at(storage, offset, amount)
}

fn remove_serialized_state_record_at(
    storage: &mut CanonicalStateStorage,
    offset: usize,
    amount: usize,
) -> bool {
    let Some(end) = offset.checked_add(amount).filter(|end| *end <= storage.ex_states.len()) else {
        return false;
    };
    storage.ex_states.drain(offset..end);
    let count = read_u32(&storage.ex_states, 0).expect("счётчик состояний");
    write_u32(&mut storage.ex_states, 0, count.saturating_sub(1));
    shift_serialized_state_offsets_after(storage, offset, amount);
    true
}

fn shift_serialized_state_offsets_after(
    storage: &mut CanonicalStateStorage,
    offset: usize,
    amount: usize,
) {
    storage.state_entries.shift_serialized_spans_after_remove(offset, amount);
    storage.state_entries.for_each_mut::<ExtendedState>(|state| state.shift_serialized_offset_after(offset, amount));
    storage.state_entries.for_each_mut::<ChangeBodyState>(|state| state.shift_serialized_offset_after(offset, amount));
    storage.state_entries.for_each_mut::<UndeadState>(|state| state.shift_serialized_offset_after(offset, amount));
    storage.state_entries.for_each_mut::<RideState>(|state| state.shift_serialized_offset_after(offset, amount));
}

pub fn take_defense_shields(
    storage: &mut CanonicalStateStorage,
) -> StateBatch<DefenseShieldState> {
    storage.state_entries.take_batch::<DefenseShieldState>()
}

pub fn restore_defense_shields(
    storage: &mut CanonicalStateStorage,
    states: StateBatch<DefenseShieldState>,
) {
    storage.state_entries.restore_batch(states);
}

/// Регистрация уже начатого состояния в освобождённой caller-ом позиции.
/// Сериализованный cache и все spans сдвигаются один раз для любого owner-а;
/// здесь нет End, игровых часов, Serialize или UpdateProperty.
pub fn insert_replacement_state_record<T: AppliedState>(
    storage: &mut CanonicalStateStorage,
    state: T,
    record: &[u8],
    location: (usize, usize),
) -> Option<StateKey> {
    let (position, offset) = location;
    if position >= storage.state_entries.len() || offset < 4 || offset > storage.ex_states.len() {
        return None;
    }
    let amount = record.len();
    storage.ex_states.splice(offset..offset, record.iter().copied());
    storage.state_entries.shift_serialized_spans_for_insert(offset, amount);
    let count = read_u32(&storage.ex_states, 0).expect("счётчик состояний");
    write_u32(&mut storage.ex_states, 0, count.wrapping_add(1));
    storage.state_entries.for_each_mut::<ExtendedState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
    storage.state_entries.for_each_mut::<ChangeBodyState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
    storage.state_entries.for_each_mut::<UndeadState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
    storage.state_entries.for_each_mut::<RideState>(|known| { known.shift_serialized_offset_for_insert(offset, amount); });
    let _ = storage.state_entries.replace_at(position, state);
    let key = storage.state_entries.address(position)?;
    storage.state_entries.set_serialized_span(key, (offset, amount));
    Some(key)
}

pub fn remove_applied_state_record<T: AppliedState>(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
    amount: usize,
) -> Option<T> {
    T::as_data_ref(storage.state_entries.get(key)?)?;
    remove_applied_state_data(storage, key, amount).and_then(T::from_data)
}

/// Только удаление точного payload и его wire-записи; игровой End с
/// visual, счётчиками и UpdateProperty выполняется владельцем снаружи.
pub fn remove_applied_state_data(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
    amount: usize,
) -> Option<StateData> {
    remove_applied_state_data_inner(storage, key, Some(amount))
}

/// Destructor-only хвост ClearAllStates: wire-размер берётся из того же
/// decoder-а, а не из второго каталога типов или выдуманного базового размера.
pub fn remove_applied_state(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
) -> Option<StateData> {
    remove_applied_state_data_inner(storage, key, None)
}

fn remove_applied_state_data_inner(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
    _amount: Option<usize>,
) -> Option<StateData> {
    let state_id = storage.state_entries.get(key)?.state_id();
    let occurrence = storage.state_entries.entries()
        .filter(|(_, state)| state.state_id() == state_id)
        .position(|(candidate, _)| candidate == key)?;
    let records: Vec<_> = known_state_record_spans(&storage.ex_states).into_iter()
        .filter(|(offset, _)| read_u32(&storage.ex_states, *offset) == Some(state_id))
        .collect();
    let runtime_count = storage.state_entries.entries()
        .filter(|(_, state)| state.state_id() == state_id).count();
    // Известная длина ещё не гарантирует успешную материализацию записи.
    // Как в save, неоднозначный ordinal не разрешает удалять чужие байты.
    let record = (runtime_count == records.len()).then(|| records[occurrence]);
    let span = storage.state_entries.serialized_span(key).or_else(|| match storage.state_entries.get(key)? {
        StateData::Swordship(state) => {
            // Как в save: Replace оставляет runtime-позицию, но переносит
            // DB-запись в хвост. Ordinal повторного ID уже не задаёт экземпляр.
            let encoded = state.encoded();
            known_state_record_spans(&storage.ex_states).into_iter().find(|(offset, size)| {
                storage.ex_states.get(*offset..*offset + *size) == Some(encoded.as_slice())
            })
        }
        StateData::EnergyHolding(state) => {
            // Factory может пропустить неизвестный level, сохранив его
            // wire-запись. Level неизменен; mutable charge для identity
            // непригоден, потому что DB-проекция обновляется при save.
            let level = state.skill_level();
            let same_level = storage.state_entries.entries().filter(|(_, entry)| {
                matches!(entry, StateData::EnergyHolding(entry) if entry.skill_level() == level)
            }).position(|(candidate, _)| candidate == key)?;
            records.into_iter().filter(|(offset, _)| {
                read_u32(&storage.ex_states, offset + 4) == Some(level)
            }).nth(same_level)
        }
        _ => record,
    });
    let position = storage.state_entries.index_of(key)?;
    let state = storage.state_entries.remove_at(position)?;
    if let Some((offset, amount)) = span {
        remove_serialized_state_record_at(storage, offset, amount);
    }
    Some(state)
}

pub fn replace_spider_web_state(
    storage: &mut CanonicalStateStorage,
    state: SpiderWebState,
) -> Option<SpiderWebState> {
    let previous = storage.state_entries.first_key::<SpiderWebState>()
        .and_then(|key| remove_applied_state_record::<SpiderWebState>(storage, key, SPIDER_WEB_STATE_BYTES));
    append_serialized_state_record(&mut storage.ex_states, &state.encoded_for_install());
    storage.state_entries.append(state);
    previous
}

pub fn take_spider_web_state(storage: &mut CanonicalStateStorage) -> Option<SpiderWebState> {
    let key = storage.state_entries.first_key::<SpiderWebState>()?;
    let state = remove_applied_state_record::<SpiderWebState>(storage, key, SPIDER_WEB_STATE_BYTES)?;
    Some(state)
}

pub fn take_expired_battle_fairy_attribute_state(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
    now_ms: u32,
) -> Option<BattleFairyAttributeState> {
    if !BattleFairyAttributeState::as_data_ref(storage.state_entries.get(key)?)?.expired(now_ms) {
        return None;
    }
    remove_applied_state_record::<BattleFairyAttributeState>(storage, key, BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES)
}

pub fn undead_state_tick(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
    mut now_milliseconds: impl FnMut() -> u32,
) -> (bool, Option<(u32, u32)>) {
    let Some(state) = storage.state_entries.get_mut(key).and_then(UndeadState::as_data_mut) else {
        return (false, None);
    };
    if state.keep_time_ms() != 0 && state.expired(now_milliseconds()) {
        return (true, None);
    }
    state.ensure_item_clock_started();
    if state.frequency_ms() != 0 && state.item_index() != 0 && state.item_amount() != 0
        && state.item_due(now_milliseconds())
    {
        state.set_last_item_tick(now_milliseconds());
        return (false, Some((state.item_index(), state.item_amount())));
    }
    (false, None)
}

pub fn extended_state_tick(
    storage: &mut CanonicalStateStorage,
    key: StateKey,
    mut now_milliseconds: impl FnMut() -> u32,
) -> (bool, Option<(u32, u32)>) {
    let Some(state) = storage.state_entries.get_mut(key).and_then(ExtendedState::as_data_mut) else {
        return (false, None);
    };
    if state.keep_time_ms != 0 && state.expired(now_milliseconds()) {
        return (true, None);
    }
    if state.kind == ExtendedStateKind::New {
        if state.last_item_tick_ms == 0 {
            state.last_item_tick_ms = state.started_ms;
        }
        if state.frequency_ms != 0 && state.item_index != 0 && state.item_amount != 0
            && state.item_due(now_milliseconds())
        {
            state.restart_item_clock(now_milliseconds());
            return (false, Some((state.item_index, state.item_amount)));
        }
    }
    (false, None)
}
