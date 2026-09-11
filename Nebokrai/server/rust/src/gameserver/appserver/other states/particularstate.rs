//! Состояние предмета `CParticularState`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/particularstate.cpp`. Один экземпляр соответствует
//! уникальному ненулевому значению `GAP_EXCEPTION_STATE` из packet/equipment.
//! Успешный container add создаёт состояние немедленно, `RestoreHpMp` завершает
//! все экземпляры, а AI после исходной двухсекундной границы проверяет наличие
//! предмета. Нулевая отметка проверки намеренно не продвигается: после первой
//! границы оригинал обходит оба контейнера на каждом вызове AI.
//! Полный клиентский снимок сохраняет каждый экземпляр отдельной state-тройкой
//! с тем же particular attribute в virtual additional-data.
//! Exact `Serialize/Unserialize` по `0x005E23D0/0x00601350` используют общий
//! 8-байтный кодек `ID + additional_data`; загрузка и удаление обновляют тот же
//! ordered `ex_states`, поэтому запись не обрывает следующие состояния.
//!
//! Состояниями владеет `CanonicalStateStorage`; `CGame` только доставляет
//! точные `0xBFE03/0xBFE04`. Координатный и object-identity overload-ы `Begin`
//! пока не достигнуты и сохранены ниже как `UNKNOWN` (исследовательский декомпилят хранится локально).
//! restart_particular_state переносит object Begin 0x004F9710:
//! nonnull sufferer → base Begin(NULL, holder) → visual SetRun(1),
//! Update(0) и base visual tail → checkstamp=0. Поле checkstamp не растёт
//! в native AI и представлено постоянным нулём, а не вторым таймером.
//! Unserialize 0x00601350 часов не читает; Begin-пакет тоже бессрочный.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{
    default_client_state_time, begin_base_applied_state, begin_applied_state_visual,
    update_applied_state_visual_base, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const PARTICULAR_STATE_ID: u32 = 0x186a5;
pub(crate) const PARTICULAR_STATE_BYTES: usize = 8;
const PARTICULAR_STATE_CHECK_INTERVAL_MS: u32 = 2_000;
const PARTICULAR_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const PARTICULAR_STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ParticularState {
    additional_data: u32,
}

impl ParticularState {
    pub(crate) const fn new(additional_data: u32) -> Option<Self> {
        if additional_data == 0 {
            None
        } else {
            Some(Self { additional_data })
        }
    }

    pub(crate) const fn additional_data(self) -> u32 {
        self.additional_data
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_u32()?;
        Ok(Self {
            additional_data: reader.read_u32()?,
        })
    }

    pub(crate) fn encoded(self) -> [u8; PARTICULAR_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(PARTICULAR_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(PARTICULAR_STATE_ID);
        writer.write_u32(self.additional_data);
        bytes
            .try_into()
            .expect("размер particular-state фиксирован")
    }

    pub(crate) const fn state_id(self) -> i32 {
        PARTICULAR_STATE_ID as i32
    }

    pub(crate) const fn client_state_time(self) -> i32 {
        default_client_state_time()
    }

    pub(crate) const fn due(self, now_ms: u32) -> bool {
        PARTICULAR_STATE_CHECK_INTERVAL_MS <= now_ms
    }
}

pub(crate) fn restart_particular_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let message = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<ParticularState>(key)
                .map(|state| particular_state_visual_message(shape.shape(), *state, true)));
        if let Some(message) = message {
            let _ = game.send_move_shape_around(region_id, holder, &message);
        }
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    true
}

pub(crate) fn particular_state_visual_message(
    player: &CShape,
    state: ParticularState,
    begin: bool,
) -> CMessage {
    let identity = player.identity();
    let mut message = CMessage::new(if begin {
        PARTICULAR_STATE_BEGIN_MESSAGE
    } else {
        PARTICULAR_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.state_id());
    if begin {
        message.add_long(state.client_state_time());
        message.add_long(state.additional_data() as i32);
    }
    message
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\particularstate.cpp

// ============================================================================
// FUNCTION: CParticularState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\particularstate.cpp:58
// RVA: 0x000F9540
// ADDRESS: 004f9540
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CParticularState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\particularstate.cpp:78
// RVA: 0x000F9610
// ADDRESS: 004f9610
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
