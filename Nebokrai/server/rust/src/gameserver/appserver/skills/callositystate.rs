//! Каноническое состояние `CCallosityState` и общий владелец пары закалок.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/callositystate.cpp` и `callositystate2.cpp`. Конкретное
//! второе состояние реализовано в `callositystate2.rs`; enum семейства задаёт
//! конкретный вариант каждого экземпляра общей арены. Обычное наложение
//! заменяет первый найденный экземпляр, загруженные дубли не схлопываются.
//! Vtable первой/второй закалки `0x006607d4/0x006603bc`, слот +0x0c,
//! направляет AI на `0x005d60b0`: строгий абсолютный wrapping deadline.
//! Общий End `0x005fd420` отправляет `0xBFE04` до RemoveState, который
//! вызывает virtual UpdateProperty (`+0x9c`); player использует `0x004593e0`.
//! Exact vtable обеих закалок направляет `GetRemainedTime` на `0x005D5F30`;
//! additional-data остаётся базовым нулём.
//! Общая exact-пара `Serialize/Unserialize` `0x005F1050/0x005F48E0` сохраняет
//! `ID + remaining time + WORD blast factor`; этот формат разделяется с
//! `CAgilityState2` и восстанавливается на player-login до пересчёта свойств.
//! Коэффициент `CCH` применяется только при общем `UpdateProperty`; каждый
//! такой проход повторно публикует начальный визуальный эффект, как
//! `OnUpdateProperties`.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; RTTI-ограничения
//! формул игрока не запрещают жизненный цикл региональных держателей.
//! После visual владелец перечитывается; общий virtual UpdateProperty
//! вызывается только при фактическом удалении этой записи.
//! Прямой End, замена и AI используют один exact-key хвост без чтения часов.

//! Restart воспроизводит только Begin(NULL, holder) (0x005F4830/0x005F10F0):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Visual принадлежит экземпляру общей арены: BeginVisualEffect(1) →
//! concrete Update(0) → базовый visual-хвост; только getter пакета читает часы.

//! Unserialize 0x005F48E0 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.

//! OnUpdateProperties 0x005F10B0 обеих закалок: GetSufferer → существующий
//! visual Update(0) → type400 → WORD-сложение CCH в живом tagProperty.
//! Отсутствующий/ended visual не запрещает формулу; часы читает только его getter.

use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
    update_player_state_properties,
};

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::shape::ShapeIdentity;

use super::callosity::CALLOSITY_SKILL_ID;
use super::callosity2::CALLOSITY_2_SKILL_ID;
use super::callositystate2::CallosityState2;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::states::state::{default_additional_data, timed_client_state_time};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const CALLOSITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const CALLOSITY_STATE_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityState {
    blast_factor: u16,
    started_at_ms: u32,
    time_to_keep: i32,
}

impl CallosityState {
    pub(crate) const fn new(blast_factor: u16, started_at_ms: u32, time_to_keep: i32) -> Self {
        Self {
            blast_factor,
            started_at_ms,
            time_to_keep,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        CALLOSITY_SKILL_ID
    }

    pub(crate) const fn blast_factor(self) -> u16 {
        self.blast_factor
    }

    pub(crate) const fn time_to_keep(self) -> i32 {
        self.time_to_keep
    }



    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.time_to_keep as u32,
            now_milliseconds,
        ) as i32
    }

    pub(crate) const fn additional_data(self) -> u32 {
        default_additional_data()
    }

    pub(crate) const fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.cch = properties.cch.wrapping_add(self.blast_factor);
        properties
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallosityFamilyState {
    Callosity(CallosityState),
    Callosity2(CallosityState2),
}

impl CallosityFamilyState {
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        let (started, keep) = match self {
            Self::Callosity(state) => (state.started_at_ms, state.time_to_keep),
            Self::Callosity2(state) => (state.started_at_ms(), state.time_to_keep()),
        };
        started.wrapping_add(keep as u32) < now_ms
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let keep_time_ms = reader.read_i32()?;
        let blast_factor = reader.read_u16()?;
        match skill_id {
            CALLOSITY_SKILL_ID => Ok(Self::Callosity(CallosityState::new(
                blast_factor,
                now_ms,
                keep_time_ms,
            ))),
            CALLOSITY_2_SKILL_ID => Ok(Self::Callosity2(CallosityState2::new(
                blast_factor,
                now_ms,
                keep_time_ms,
            ))),
            _ => Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            }),
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Callosity(state) => state.skill_id(),
            Self::Callosity2(state) => state.skill_id(),
        }
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        match self {
            Self::Callosity(state) => state.client_state_time(now_milliseconds),
            Self::Callosity2(state) => state.client_state_time(now_milliseconds),
        }
    }

    pub(crate) const fn additional_data(self) -> u32 {
        match self {
            Self::Callosity(state) => state.additional_data(),
            Self::Callosity2(state) => state.additional_data(),
        }
    }



    pub(crate) fn encoded(self, now_ms: u32) -> [u8; CALLOSITY_STATE_BYTES] {
        let mut bytes = [0; CALLOSITY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id().to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_state_time(|| now_ms).to_le_bytes());
        let blast_factor = match self {
            Self::Callosity(state) => state.blast_factor(),
            Self::Callosity2(state) => state.blast_factor(),
        };
        bytes[8..].copy_from_slice(&blast_factor.to_le_bytes());
        bytes
    }

    pub(crate) fn encoded_for_install(self) -> [u8; CALLOSITY_STATE_BYTES] {
        let started_at_ms = match self {
            Self::Callosity(state) => state.started_at_ms,
            Self::Callosity2(state) => state.started_at_ms(),
        };
        self.encoded(started_at_ms)
    }

    pub(crate) const fn apply_to_player(
        self,
        properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        match self {
            Self::Callosity(state) => state.apply_to_player(properties),
            Self::Callosity2(state) => state.apply_to_player(properties),
        }
    }
}

pub(crate) fn end_player_callosity_state(game: &mut CGame, player_id: i32) -> bool {
    let Some((region_id, holder, key)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_region_id(), player.shape().identity(),
            player.move_shape().applied_state_key::<CallosityFamilyState>()?))
    }) else { return false };
    end_callosity_state_key(game, region_id, holder, key)
}

pub(crate) fn end_callosity_state_key(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<CallosityFamilyState>(key, CALLOSITY_STATE_BYTES))
        .is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}

pub(crate) fn update_callosity_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_applied_state_sufferer(game, region_id, holder, key).is_none() {
        return false;
    }
    let _ = update_property_state_visual::<CallosityFamilyState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now) as u32,
    );
    update_player_state_properties::<CallosityFamilyState>(game, region_id, holder, key, |state, player| {
        player.update_state_combat_properties(|properties| state.apply_to_player(properties));
    })
}

pub(crate) fn restart_callosity_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).copied()
        else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_long(state.client_state_time(now));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    true
}

pub(crate) fn update_callosity_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_callosity_state_key(game, region_id, holder, key)
}

pub(crate) fn send_callosity_state_begin(
    game: &mut CGame,
    player_id: i32,
    state: CallosityFamilyState,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(CALLOSITY_STATE_BEGIN_MESSAGE);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    message.add_long(state.client_state_time(game_tick_milliseconds));
    message.add_ulong(state.additional_data());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp

// ============================================================================
// FUNCTION: CCallosityState::CCallosityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:25
// RVA: 0x001F4600
// ADDRESS: 005f4600
// PROTOTYPE: undefined __thiscall CCallosityState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
