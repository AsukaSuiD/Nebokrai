//! Владелец состояния преображения `CHBYState` GameServer.
//!
//! Контракт подтверждён точной парой `gameserver.exe + GameServer.pdb` и
//! исходным owner-ом `appserver/other states/chbystate.cpp`. Реализация хранит
//! byte-exact 120-байтовый `tagCHBYState`, время, пять временных навыков и
//! сохранённые hotkey 12..23. Системный wrapping tick передаётся caller-ом;
//! Rust-владение заменяет raw `CState*`. Сохранённый ниже псевдокод служит
//! локальным provenance для реализованного owner-а и не входит в runtime.
//! Доступ к little-endian полям делегирован общему legacy codec поверх `bytes`.
//! `Serialize` записывает остаток обратно в keeptime без перезапуска clock;
//! клиентский снимок этого не делает. `AI` (0x005daaa0) завершает состояние
//! только после абсолютного wrapping deadline, а не на его границе.
//! Payload принадлежит общей арене CMoveShape; decode_at читает одну
//! фабрично подтверждённую запись и сохраняет её точный offset, без byte-scan.
//! restart_change_body_state переносит object Begin0x005DB000 после смерти
//! и Begin(NULL, holder, 1)0x005DADB0 в обычном StartAllStates. Первый требует
//! CPlayer RTTI, второй native делает unchecked player-cast; безопасный
//! адаптер явно отказывает non-player, не изображая определённый native результат.
//! Base Begin сохраняет timestamp, затем visual/очистка emotion/mode, Update0
//! и base visual tail, append пяти ID в m_vskill без очистки, hotkeys.
//! Только двухаргументный Begin при !online сохраняет/обнуляет hotkeys12..23.
//! Первые пять накопленных ID задают hotkeys12..16; нулевые ID не шлют пакет.
//! Visual0x005DA390 добавляет навыки последовательно через общий AddSkill,
//! затем читает реальный GetSkill и его virtual+70/+74/+78. ID0 занимает
//! шесть ULONG0; missing skill/properties не создаёт placeholder. Клиентское
//! время0x005DA030 читает часы1/2/3 раза; Unserialize выставляет online=true.
//! Чистый visual-снимок не копирует накопленный m_vskill и не владеет состоянием.
//! Текущий runtime-install создаёт уже начатое состояние через from_factory,
//! поэтому его список заранее содержит пять ID; decode оставляет список пустым.

use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
    resolve_state_move_shape, change_body_client_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const CHANGE_BODY_STATE_ID: u32 = 0x37;
pub(crate) const CHANGE_BODY_SKILL_TYPE: u32 = 55;
const CHANGE_BODY_PARAMETER_BYTES: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangeBodyState {
    pub(crate) has_changed_region: bool,
    pub(crate) visual_effect: u16,
    pub(crate) level: u32,
    pub(crate) keep_time_ms: u32,
    pub(crate) mode: u32,
    pub(crate) change_region: bool,
    pub(crate) restore_online: bool,
    pub(crate) continue_after_death: bool,
    pub(crate) online: bool,
    pub(crate) maximum_hp: u32,
    pub(crate) maximum_mp: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) defense: u32,
    pub(crate) element_resistance: u32,
    pub(crate) cch: u16,
    pub(crate) blast_attack: u16,
    pub(crate) blast_element_attack: u16,
    pub(crate) skills: [(u16, u16); 5],
    begun_skill_ids: Vec<u16>,
    pub(crate) old_hotkeys: [u32; 12],
    pub(crate) started_ms: u32,
    serialized_offset: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangeBodyMutation {
    pub(crate) removed: Option<ChangeBodyState>,
    pub(crate) added: Option<ChangeBodyState>,
    pub(crate) legacy_return: u32,
}

impl ChangeBodyState {
    pub(crate) fn from_factory(
        level: u32,
        factory: &CSkillFactory,
        started_ms: u32,
    ) -> Option<Self> {
        let properties =
            factory.query_skill_base_properties(CHANGE_BODY_SKILL_TYPE, level as i32)?;
        let get = |usage| Some(properties.query_property(usage));
        let skills = [
            (get(60_011)? as u16, get(60_012)? as u16),
            (get(60_021)? as u16, get(60_022)? as u16),
            (get(60_031)? as u16, get(60_032)? as u16),
            (get(60_041)? as u16, get(60_042)? as u16),
            (get(60_051)? as u16, get(60_052)? as u16),
        ];
        Some(Self {
            has_changed_region: true,
            visual_effect: get(60_001)? as u16,
            level,
            keep_time_ms: get(10_002)?,
            mode: get(60_000)?,
            change_region: get(60_002)? != 0,
            restore_online: get(60_003)? != 0,
            continue_after_death: get(60_004)? != 0,
            online: false,
            maximum_hp: get(118)?,
            maximum_mp: get(119)?,
            minimum_attack: get(116)?,
            maximum_attack: get(117)?,
            defense: get(109)?,
            element_resistance: get(112)?,
            cch: get(108)? as u16,
            blast_attack: get(125)? as u16,
            blast_element_attack: get(126)? as u16,
            skills,
            begun_skill_ids: skills.into_iter().map(|(id, _)| id).collect(),
            old_hotkeys: [0; 12],
            started_ms,
            serialized_offset: None,
        })
    }

    pub(crate) fn decode_at(payload: &[u8], offset: usize, started_ms: u32) -> Option<Self> {
        offset.checked_add(4 + CHANGE_BODY_PARAMETER_BYTES)
            .filter(|end| *end <= payload.len())?;
        if read_u32(payload, offset) != Some(CHANGE_BODY_STATE_ID) {
            return None;
        }
        let base = offset + 4;
        let Some(level) = read_u32(payload, base + 4) else {
            return None;
        };
        if level == 0
            || payload[base] > 1
            || payload[base + 16] > 1
            || payload[base + 17] > 1
            || payload[base + 18] > 1
            || payload[base + 19] > 1
        {
            return None;
        }
        let mut skills = [(0, 0); 5];
        for (index, skill) in skills.iter_mut().enumerate() {
            *skill = (
                read_u16(payload, base + 50 + index * 4).unwrap_or_default(),
                read_u16(payload, base + 52 + index * 4).unwrap_or_default(),
            );
        }
        let mut old_hotkeys = [0; 12];
        for (index, hotkey) in old_hotkeys.iter_mut().enumerate() {
            *hotkey = read_u32(payload, base + 72 + index * 4).unwrap_or_default();
        }
        Some(Self {
            has_changed_region: payload[base] != 0,
            visual_effect: read_u16(payload, base + 2).unwrap_or_default(),
            level,
            keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
            mode: read_u32(payload, base + 12).unwrap_or_default(),
            change_region: payload[base + 16] != 0,
            restore_online: payload[base + 17] != 0,
            continue_after_death: payload[base + 18] != 0,
            online: true,
            maximum_hp: read_u32(payload, base + 20).unwrap_or_default(),
            maximum_mp: read_u32(payload, base + 24).unwrap_or_default(),
            minimum_attack: read_u32(payload, base + 28).unwrap_or_default(),
            maximum_attack: read_u32(payload, base + 32).unwrap_or_default(),
            defense: read_u32(payload, base + 36).unwrap_or_default(),
            element_resistance: read_u32(payload, base + 40).unwrap_or_default(),
            cch: read_u16(payload, base + 44).unwrap_or_default(),
            blast_attack: read_u16(payload, base + 46).unwrap_or_default(),
            blast_element_attack: read_u16(payload, base + 48).unwrap_or_default(),
            skills,
            begun_skill_ids: Vec::new(),
            old_hotkeys,
            started_ms,
            serialized_offset: Some(offset),
        })
    }

    pub(crate) fn remove_serialized(&self, payload: &mut Vec<u8>) {
        let Some(offset) = self.serialized_offset else {
            return;
        };
        if offset + 4 + CHANGE_BODY_PARAMETER_BYTES > payload.len() {
            return;
        }
        payload.drain(offset..offset + 4 + CHANGE_BODY_PARAMETER_BYTES);
        if let Some(count) = read_u32(payload, 0) {
            write_u32(payload, 0, count.saturating_sub(1));
        }
    }

    pub(crate) fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + CHANGE_BODY_PARAMETER_BYTES))
    }

    pub(crate) fn write_serialized(&mut self, payload: &mut [u8], offset: usize) {
        let base = offset + 4;
        write_u32(payload, offset, CHANGE_BODY_STATE_ID);
        payload[base] = u8::from(self.has_changed_region);
        write_u16(payload, base + 2, self.visual_effect);
        write_u32(payload, base + 4, self.level);
        write_u32(payload, base + 8, self.keep_time_ms);
        write_u32(payload, base + 12, self.mode);
        payload[base + 16] = u8::from(self.change_region);
        payload[base + 17] = u8::from(self.restore_online);
        payload[base + 18] = u8::from(self.continue_after_death);
        payload[base + 19] = u8::from(self.online);
        write_u32(payload, base + 20, self.maximum_hp);
        write_u32(payload, base + 24, self.maximum_mp);
        write_u32(payload, base + 28, self.minimum_attack);
        write_u32(payload, base + 32, self.maximum_attack);
        write_u32(payload, base + 36, self.defense);
        write_u32(payload, base + 40, self.element_resistance);
        write_u16(payload, base + 44, self.cch);
        write_u16(payload, base + 46, self.blast_attack);
        write_u16(payload, base + 48, self.blast_element_attack);
        for (index, (skill_id, level)) in self.skills.iter().copied().enumerate() {
            write_u16(payload, base + 50 + index * 4, skill_id);
            write_u16(payload, base + 52 + index * 4, level);
        }
        for (index, hotkey) in self.old_hotkeys.iter().copied().enumerate() {
            write_u32(payload, base + 72 + index * 4, hotkey);
        }
        self.serialized_offset = Some(offset);
    }


    pub(crate) fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        self.client_state_time(|| now_ms)
    }

    pub(crate) fn client_state_time(&self, mut now: impl FnMut() -> u32) -> u32 {
        change_body_client_time(self.started_ms, self.keep_time_ms, &mut now)
    }

    pub(crate) fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn commit_saved_time(&mut self, now_ms: u32) {
        self.keep_time_ms = self.remaining_time_ms(now_ms);
    }

    pub(crate) fn on_change_region(&mut self) -> bool {
        if !self.has_changed_region {
            self.has_changed_region = true;
            return false;
        }
        !self.change_region
    }

    pub(crate) fn on_player_lost(&mut self) -> bool {
        self.has_changed_region = false;
        self.online = true;
        !self.restore_online
    }

    pub(crate) fn update_serialized_runtime(&self, payload: &mut [u8], now_ms: u32) {
        let Some(offset) = self.serialized_offset else {
            return;
        };
        let base = offset + 4;
        if base + CHANGE_BODY_PARAMETER_BYTES > payload.len() {
            return;
        }
        payload[base] = u8::from(self.has_changed_region);
        write_u32(payload, base + 8, self.remaining_time_ms(now_ms));
        payload[base + 19] = u8::from(self.online);
    }

    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}

pub(crate) fn restart_change_body_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    after_death: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if holder.object_type != 400 { return false }
    let Some(mode) = game.find_player(holder.id)
        .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key))
        .map(|state| state.mode)
    else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if !begin_applied_state_visual(game, region_id, holder, key, 1) { return true }
    let Some(player) = game.find_player_mut(holder.id) else { return true };
    player.clear_emotion_state();
    let (head, face, _) = player.appearance_and_mode();
    player.restore_appearance_and_mode(head, face, mode);

    let snapshot = game.find_player(holder.id)
        .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key))
        .map(ChangeBodyBeginVisualSnapshot::from);
    if let Some(snapshot) = snapshot {
        let _ = send_change_body_begin_visual(game, region_id, holder, snapshot, Some(key), now);
    }
    let _ = update_applied_state_visual_base(game, region_id, holder, key);

    let Some(player) = game.find_player_mut(holder.id) else { return true };
    let Some(state) = player.move_shape_mut().applied_state_mut::<ChangeBodyState>(key)
    else { return true };
    state.begun_skill_ids.extend(state.skills.iter().map(|(id, _)| *id));
    let save_hotkeys = after_death && !state.online;
    if save_hotkeys {
        for index in 0..12 {
            let old = player.hotkey((index + 12) as u8).unwrap_or_default();
            let Some(state) = player.move_shape_mut().applied_state_mut::<ChangeBodyState>(key)
            else { return true };
            state.old_hotkeys[index] = old;
            let _ = player.set_hotkey((index + 12) as u8, 0);
        }
    }
    for index in 0..5 {
        let Some(skill_id) = game.find_player(holder.id)
            .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key))
            .and_then(|state| state.begun_skill_ids.get(index)).copied()
        else { return true };
        if skill_id != 0 {
            game.set_script_player_hotkey(holder.id, (index + 12) as u8, u32::from(skill_id) | 0x8000_0000);
        }
    }
    if game.find_player(holder.id)
        .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key)).is_some()
    {
        game.set_script_player_hotkey(holder.id, 17, 0x8000_031f);
    }
    true
}

#[derive(Clone, Copy, Debug)]
struct ChangeBodyBeginVisualSnapshot {
    started_ms: u32,
    keep_time_ms: u32,
    mode: u32,
    level: u32,
    visual_effect: u16,
    continue_after_death: bool,
    skills: [(u16, u16); 5],
}

impl From<&ChangeBodyState> for ChangeBodyBeginVisualSnapshot {
    fn from(state: &ChangeBodyState) -> Self {
        Self {
            started_ms: state.started_ms,
            keep_time_ms: state.keep_time_ms,
            mode: state.mode,
            level: state.level,
            visual_effect: state.visual_effect,
            continue_after_death: state.continue_after_death,
            skills: state.skills,
        }
    }
}

pub(crate) fn send_change_body_state_begin_visual(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &ChangeBodyState,
    now: &mut dyn FnMut() -> u32,
) {
    let _ = send_change_body_begin_visual(game, region_id, holder, state.into(), None, now);
}

fn send_change_body_begin_visual(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    snapshot: ChangeBodyBeginVisualSnapshot,
    key: Option<StateKey>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder).is_none() { return false }
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(CHANGE_BODY_STATE_ID as i32);
    message.add_ulong(change_body_client_time(snapshot.started_ms, snapshot.keep_time_ms, &mut *now));
    message.add_long(0);
    message.add_ulong(snapshot.mode);
    message.add_ulong(snapshot.level);
    message.add_ulong(u32::from(snapshot.visual_effect));
    message.add_byte(u8::from(snapshot.continue_after_death));
    for index in 0..5 {
        let (skill_id, level) = if let Some(key) = key {
            let Some(skills) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ChangeBodyState>(key))
                .map(|state| state.skills)
            else { return false };
            skills[index]
        } else { snapshot.skills[index] };
        if skill_id == 0 {
            for _ in 0..6 { message.add_ulong(0); }
            continue;
        }
        let _ = game.add_move_shape_skill(region_id, holder, u32::from(skill_id), i32::from(level));
        let (skill_id, level) = if let Some(key) = key {
            let Some(skills) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ChangeBodyState>(key))
                .map(|state| state.skills)
            else { return false };
            skills[index]
        } else { (skill_id, level) };
        let skill = game.registered_move_shape_skill(region_id, holder, u32::from(skill_id));
        let properties = game.skill_base_properties(u32::from(skill_id), i32::from(level));
        if let (Some(skill), Some(properties)) = (skill, properties)
            && let Some(parameters) = game.registered_skill_client_parameters(skill)
        {
            message.base_mut().add_short(skill_id as i16);
            message.base_mut().add_short(level as i16);
            message.add_ulong(properties.query_property(10_005));
            for value in parameters { message.add_ulong(value); }
        }
    }
    let _ = game.send_move_shape_around(region_id, holder, &message);
    true
}

fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    LegacyReader::at(source, offset).ok()?.read_u16().ok()
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(destination, offset, value).expect("проверенное поле CHBYState");
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле CHBYState");
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp

// ============================================================================
// FUNCTION: CHBYState::GetRemainedTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:61
// RVA: 0x001DA030
// ADDRESS: 005da030
// PROTOTYPE: ulong __thiscall GetRemainedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:454
// RVA: 0x001DA080
// ADDRESS: 005da080
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CHBYState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:70
// RVA: 0x001DA0F0
// ADDRESS: 005da0f0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:370
// RVA: 0x001DA240
// ADDRESS: 005da240
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCHBYStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:482
// RVA: 0x001DA390
// ADDRESS: 005da390
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::~CHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:57
// RVA: 0x001DAA30
// ADDRESS: 005daa30
// PROTOTYPE: void __thiscall ~CHBYState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:412
// RVA: 0x001DAAA0
// ADDRESS: 005daaa0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:425
// RVA: 0x001DAB80
// ADDRESS: 005dab80
// PROTOTYPE: void __thiscall OnChangeRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::CHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:37
// RVA: 0x001DAC90
// ADDRESS: 005dac90
// PROTOTYPE: undefined __thiscall CHBYState(tagCHBYState * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::CHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:47
// RVA: 0x001DAD20
// ADDRESS: 005dad20
// PROTOTYPE: undefined __thiscall CHBYState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:110
// RVA: 0x001DADB0
// ADDRESS: 005dadb0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:239
// RVA: 0x001DB280
// ADDRESS: 005db280
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:305
// RVA: 0x001DB560
// ADDRESS: 005db560
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
