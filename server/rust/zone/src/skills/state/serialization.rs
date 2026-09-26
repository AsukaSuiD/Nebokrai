//! DB Save/Load-кодек применённых состояний `CMoveShape` из GameServer.exe/GameServer.pdb
//! (пара gameserver.exe SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ GameServer.pdb RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2, совпадают).
//! Contract: docs/gameplay/attributes-and-states.md.
//! Перенесён из hub `appserver/moveshape.rs`: Save/Load-семья (`serialize_ex_states_for_save`,
//! `serialized_ex_states`, общий обход записей, `replace_ex_states`,
//! `clear_persisted_runtime_state`) вместе с примитивами кодека. Арена записей —
//! соседний `storage`; decode одиночной записи — `skills::statefactory`
//! (нативный `CStateFactory::Unserialize`, RVA `0x001D7D00`).
//! Машинный якорь Save-прохода — сохранённый RAW `CMoveShape::AddExStatesToByteArray`
//! (0x004D10F0, moveshape.cpp:1814): уплотнение позиций арены в начале mutable
//! GameSave соответствует этому проходу и UpdateAbnormality (0x004CFD00).
//! Порядок записей Serialize-cache, над которым работает этот кодек, задают
//! сохранённые RAW RemoveState (0x004CDAB0, 0x004CDB20): append/remove/insert
//! записей и владение этим порядком — соседний `mutations` (сохранённые
//! свидетельства также в шапке `storage`).
//!
//! Нормализация при переносе (тела буквальные): методы переходного CMoveShape
//! преобразованы в свободные функции над заимствованными `LegacyStateCodec` и
//! `AppliedStateEntries`; identity владельца передаётся значением `ShapeIdentity`,
//! часы — clock-замыканием caller-а. Уплотнение Save вызывает `compact` арены
//! напрямую: hub `compact_state_slots` остаётся для живого UpdateAbnormality.
//! `clear_persisted_runtime_state` сбрасывает реестр навыков generic-сваркой
//! `SkillIdentityAccess`, codec, арену и скаляры запрета боя в исходном порядке.
//! Бинарные примитивы read/write объявлены `pub`: ими пользуются сохранённые
//! RAW-операции hub moveshape.

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};

use crate::effects::{ChangeBodyState, DefenseShieldState, ExtendedState, UndeadState};
use crate::regions::ShapeIdentity;
use crate::regions::skillregistry::{SkillIdentityAccess, SkillRegistry};
use crate::skills::skillfactory::CSkillFactory;
use crate::skills::statefactory::{
    decode_state_record_into_cache, known_state_record_offsets, known_state_record_spans,
};

use super::storage::{
    AppliedState, AppliedStateEntries, LegacyStateCodec, StateData, StateSerialization,
};

/// Save-проход с уплотнением позиций арены: единственный mutable GameSave
/// дописывает opaque tail и возвращает runtime-порядок только при полном
/// сопоставлении записей с DB-кодеком.
pub fn serialize_ex_states_for_save(
    ex_states: &mut LegacyStateCodec,
    state_entries: &mut AppliedStateEntries,
    now_ms: u32,
    timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Vec<u8> {
    let _ = state_entries.compact();
    let payload = serialize_state_records_with_entries(
        ex_states.to_vec(),
        StateSerialization::Save(state_entries),
        now_ms,
        timed_state_now_milliseconds,
        true,
    );
    ex_states.with_opaque_tail(payload)
}

/// ReadOnly-проекция текущего cache без уплотнения и без commit времени.
pub fn serialized_ex_states(
    ex_states: &LegacyStateCodec,
    state_entries: &AppliedStateEntries,
    now_ms: u32,
    timed_state_now_milliseconds: impl FnMut() -> u32,
) -> Vec<u8> {
    ex_states.with_opaque_tail(
        serialize_state_records(ex_states, state_entries, now_ms, timed_state_now_milliseconds, false),
    )
}

fn serialize_state_records(
    ex_states: &LegacyStateCodec,
    state_entries: &AppliedStateEntries,
    now_ms: u32,
    timed_state_now_milliseconds: impl FnMut() -> u32,
    canonical_order: bool,
) -> Vec<u8> {
    serialize_state_records_with_entries(
        ex_states.to_vec(),
        StateSerialization::ReadOnly(state_entries),
        now_ms,
        timed_state_now_milliseconds,
        canonical_order,
    )
}

fn serialize_state_records_with_entries(
    mut payload: Vec<u8>,
    mut states: StateSerialization<'_>,
    now_ms: u32,
    mut timed_state_now_milliseconds: impl FnMut() -> u32,
    canonical_order: bool,
) -> Vec<u8> {
    let spans = known_state_record_spans(&payload);
    let declared_count = read_u32(&payload, 0).map(|count| count as usize);
    let parsed_end = spans.last().map_or(4, |(offset, amount)| offset + amount);
    let mut complete = declared_count == Some(spans.len()) && parsed_end == payload.len();
    let mut used = vec![false; spans.len()];
    let mut ordered_records = Vec::with_capacity(spans.len());

    for index in 0..states.entries().len() {
        let Some(key) = states.entries().address(index) else { continue };
        let Some(state) = states.entries().get(key) else {
            complete = false;
            continue;
        };
        let state_id = state.state_id();
        let exact_span = states.entries().serialized_span(key).map(Some).or_else(|| match state {
            StateData::ChangeBody(state) => Some(state.serialized_span()),
            StateData::Extended(state) => Some(state.serialized_span()),
            StateData::Undead(state) => Some(state.serialized_span()),
            StateData::Ride(state) => Some(state.serialized_span()),
            _ => None,
        });
        let record_index = if let Some(span) = exact_span {
            span.and_then(|span| spans.iter().enumerate().position(|(index, candidate)| {
                !used[index] && *candidate == span
                    && read_u32(&payload, candidate.0) == Some(state_id)
            }))
        } else if let StateData::Swordship(state) = state {
            // Replace сохраняет runtime-позицию, но DB remove+append может
            // поменять порядок повторных ID. Все поля этой записи известны.
            let record = state.encoded();
            spans.iter().enumerate().position(|(index, (offset, amount))| {
                !used[index] && payload.get(*offset..offset + amount) == Some(record.as_slice())
            })
        } else {
            let runtime_count = states.entries().iter_data()
                .filter(|state| state.state_id() == state_id).count();
            let wire_count = spans.iter()
                .filter(|(offset, _)| read_u32(&payload, *offset) == Some(state_id)).count();
            (runtime_count == wire_count).then(|| {
                spans.iter().enumerate().position(|(index, (offset, _))| {
                    !used[index] && read_u32(&payload, *offset) == Some(state_id)
                })
            }).flatten()
        };
        if let Some(record_index) = record_index {
            used[record_index] = true;
        } else {
            complete = false;
        }

        // Часы вызываются здесь, в едином порядке m_vStates. Отсутствие
        // однозначной DB-пары запрещает запись, но не добавляет type-pass.
        let mut commit_time: Option<(u32, fn(&mut StateData, u32))> = None;
        let encoded = match state {
            StateData::ChangeBody(state) => {
                let remaining = state.client_state_time(&mut timed_state_now_milliseconds);
                if let Some(record_index) = record_index {
                    state.update_serialized_record(
                        &mut payload, spans[record_index].0, remaining,
                    );
                }
                commit_time = Some((remaining, |data, remaining| {
                    if let Some(state) = ChangeBodyState::as_data_mut(data) {
                        state.commit_serialized_time(remaining);
                    }
                }));
                None
            }
            StateData::Extended(state) => {
                let remaining = state.client_state_time(&mut timed_state_now_milliseconds);
                if let Some(record_index) = record_index {
                    state.update_serialized_record(&mut payload, spans[record_index].0, remaining);
                }
                commit_time = Some((remaining, |data, remaining| {
                    if let Some(state) = ExtendedState::as_data_mut(data) {
                        state.commit_serialized_time(remaining);
                    }
                }));
                None
            }
            StateData::Undead(state) => {
                let remaining = state.client_state_time(&mut timed_state_now_milliseconds);
                if let Some(record_index) = record_index {
                    state.update_serialized_record(&mut payload, spans[record_index].0, remaining);
                }
                commit_time = Some((remaining, |data, remaining| {
                    if let Some(state) = UndeadState::as_data_mut(data) {
                        state.commit_serialized_time(remaining);
                    }
                }));
                None
            }
            StateData::LeafCut(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::LeafCut3(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Kerosene(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::PoisonFog(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::MeteorArrow(state) => Some(state.encoded().to_vec()),
            StateData::Script(state) => Some(state.encoded(&mut timed_state_now_milliseconds)),
            StateData::ConsumableRestore(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Blind(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Seal(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Strike(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::KnockOut(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::SpiderWeb(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::GodBless(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Cure(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Weak(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::SoulCollect(state) => Some(state.encoded().to_vec()),
            StateData::SpriteBurn(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::SpiderPoison(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::DaubPoison(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::BossBlueQuake(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::KnightCut(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::BoaLock(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Rush(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Roar(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Pillar(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::RageBreak(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Hearten(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Heal(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Fury(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::TianShenXiaFan(state) => Some(state.encoded().to_vec()),
            StateData::Wangsheng(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::DefenseShield(state) => Some(match state {
                DefenseShieldState::Mana(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                DefenseShieldState::Machine(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                DefenseShieldState::Life(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
                DefenseShieldState::Promotion(state) => state.encoded(&mut timed_state_now_milliseconds).to_vec(),
            }),
            StateData::LeafCut2(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Rush2(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::Agility2(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::BloodLoss(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::EnergyHolding(state) => Some(state.encoded().to_vec()),
            StateData::Callosity(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::BossBlueFury(state) => Some(state.encoded(now_ms).to_vec()),
            StateData::PoisonArrow(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            StateData::BattleFairyAttribute(state) => Some(state.encoded(&mut timed_state_now_milliseconds).to_vec()),
            // Эти неизменяемые записи уже синхронизированы при установке.
            // В частности, не обнуляем сохранённый padding tagWuXingState.
            StateData::PersistentAgility(_) | StateData::TaiJi(_)
            | StateData::EnlargeFullMiss(_) | StateData::EnlargeMaxHp(_)
            | StateData::EnlargeMaxMp(_) | StateData::Origin(_)
            | StateData::Swordship(_) | StateData::WuXing(_)
            | StateData::AutomaticRestore(_) | StateData::Particular(_)
            | StateData::Team(_) | StateData::Ride(_) => None,
        };
        if let Some((remaining, commit)) = commit_time
            && let Some(state) = states.get_mut(key)
        {
            // Serialize сохраняет тот же остаток без повторного getter/clock
            // и без сброса started/item timestamp. ReadOnly не даёт &mut.
            commit(state, remaining);
        }
        if let Some(record_index) = record_index {
            let (offset, amount) = spans[record_index];
            if let Some(record) = encoded {
                if record.len() == amount {
                    payload[offset..offset + amount].copy_from_slice(&record);
                } else {
                    complete = false;
                }
            }
            ordered_records.push(record_index);
        }
    }

    if !canonical_order || !complete || used.iter().any(|used| !used) {
        return payload;
    }
    let mut ordered = Vec::with_capacity(payload.len());
    ordered.extend_from_slice(&payload[..4]);
    for record_index in ordered_records {
        let (offset, amount) = spans[record_index];
        ordered.extend_from_slice(&payload[offset..offset + amount]);
    }
    ordered
}

/// Load-проход: типизированный decode каждой известной записи в новую арену,
/// неизвестный хвост и остаток declared count сохраняются в codec как есть.
pub fn replace_ex_states(
    ex_states: &mut LegacyStateCodec,
    state_entries: &mut AppliedStateEntries,
    state_owner: ShapeIdentity,
    states: Vec<u8>,
    skill_factory: &CSkillFactory,
    now: &mut dyn FnMut() -> u32,
) {
    state_entries.clear();
    let declared_count = read_u32(&states, 0);
    let mut payload = 0u32.to_le_bytes().to_vec();
    let mut cursor = if declared_count.is_some() { 4 } else { 0 };
    let mut decoded_count = 0u32;
    for _ in 0..declared_count.unwrap_or(0) {
        let cache_offset = payload.len();
        let Some((state, consumed)) = decode_state_record_into_cache(
            &states, cursor, &mut payload, state_owner, skill_factory, now,
        ) else { break };
        let cache_size = payload.len() - cache_offset;
        state_entries.append_loaded_data(state, (cache_offset, cache_size));
        cursor += consumed;
        decoded_count += 1;
    }
    write_u32(&mut payload, 0, decoded_count);
    *ex_states = LegacyStateCodec {
        payload,
        opaque_tail: states[cursor..].to_vec(),
        opaque_count: declared_count.unwrap_or(0) - decoded_count,
        header_was_present: declared_count.is_some(),
    };
}

/// Полный сброс persisted runtime-состояния владельца: реестр навыков,
/// codec, арена состояний и скаляры запрета боя в исходном порядке.
pub fn clear_persisted_runtime_state<S: SkillIdentityAccess>(
    skills: &mut SkillRegistry<S>,
    ex_states: &mut LegacyStateCodec,
    state_entries: &mut AppliedStateEntries,
    can_fight_count: &mut i32,
    can_fight: &mut bool,
) {
    skills.clear();
    ex_states.clear();
    state_entries.clear();
    *can_fight_count = 0;
    *can_fight = true;
}

pub fn update_known_state_record(payload: &mut [u8], state_id: u32, record: &[u8]) {
    update_nth_known_state_record(payload, state_id, 0, record);
}

pub fn update_nth_known_state_record(payload: &mut [u8], state_id: u32, occurrence: usize, record: &[u8]) {
    if let Some(offset) = known_state_record_offsets(payload)
        .into_iter()
        .filter(|offset| read_u32(payload, *offset) == Some(state_id))
        .nth(occurrence)
        && let Some(destination) = payload.get_mut(offset..offset + record.len())
    {
        destination.copy_from_slice(record);
    }
}

pub fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    LegacyReader::at(source, offset).ok()?.read_u16().ok()
}

pub fn read_i16(source: &[u8], offset: usize) -> Option<i16> {
    LegacyReader::at(source, offset).ok()?.read_i16().ok()
}

pub fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

pub fn read_i32(source: &[u8], offset: usize) -> Option<i32> {
    LegacyReader::at(source, offset).ok()?.read_i32().ok()
}

pub fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(destination, offset, value).expect("проверенное поле состояния");
}

pub fn write_i16(destination: &mut [u8], offset: usize, value: i16) {
    LegacyWriter::write_i16_at(destination, offset, value).expect("проверенное поле состояния");
}

pub fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле состояния");
}

pub fn write_i32(destination: &mut [u8], offset: usize, value: i32) {
    LegacyWriter::write_i32_at(destination, offset, value).expect("проверенное поле состояния");
}
