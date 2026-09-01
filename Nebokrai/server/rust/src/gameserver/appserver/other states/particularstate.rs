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
//!
//! Состояниями владеет `CanonicalStateStorage`; `CGame` только доставляет
//! точные `0xBFE03/0xBFE04`. Координатный и object-identity overload-ы `Begin`
//! пока не достигнуты и сохранены ниже как `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::shape::CShape;
use crate::gameserver::appserver::states::state::default_client_state_time;
use crate::nets::netserver::message::CMessage;

pub(crate) const PARTICULAR_STATE_ID: u32 = 0x186a5;
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
