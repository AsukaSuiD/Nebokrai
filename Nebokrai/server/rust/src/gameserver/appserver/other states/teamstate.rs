//! Состояние набора в отряд `CTeamState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/other states/teamstate.cpp`. Материализован достигнутый через
//! client `0x8FF08` lifecycle: имя/пароль, state ID `100006`, бессрочное
//! client-time и additional-data с password bit: begin до session сообщает
//! одного лидера, а полный снимок динамически берёт размер канонической team.
//! Общий полиморфный список `CState` заменён каноническим типизированным
//! хранилищем игрока. Владелец состояния строит пакеты начала, завершения и
//! изменения числа участников; AI раз в пять секунд двумя отдельными чтениями
//! часов проверяет, остался ли игрок лидером найденной team-session.
//! Координатные overload-ы `Begin` и восстановление из старого хранилища пока
//! не достигнуты и сохранены в RAW ниже.

use crate::gameserver::appserver::states::state::default_client_state_time;
use crate::nets::netserver::message::CMessage;

pub(crate) const TEAM_STATE_ID: i32 = 0x0001_86a6;
const TEAM_STATE_CHECK_INTERVAL_MS: u32 = 5_000;
const TEAM_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const TEAM_STATE_END_MESSAGE: i32 = 0x000b_fe04;
const TEAM_STATE_UPDATE_MESSAGE: i32 = 0x000b_fe05;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTeamState {
    team_name: Vec<u8>,
    team_password: Vec<u8>,
    last_check_timestamp_ms: u32,
}

impl CTeamState {
    pub(crate) fn new(team_name: Vec<u8>, team_password: Vec<u8>) -> Self {
        Self {
            team_name,
            team_password,
            last_check_timestamp_ms: 0,
        }
    }

    pub(crate) const fn state_id(&self) -> i32 {
        TEAM_STATE_ID
    }

    /// Базовый `CState::GetClientStateTime` для этого бессрочного state.
    pub(crate) const fn client_state_time(&self) -> i32 {
        default_client_state_time()
    }

    /// До создания team session исходный owner сообщает самого лидера как
    /// единственного участника; bit 16 отмечает непустой пароль.
    pub(crate) fn initial_additional_data(&self) -> u32 {
        (u32::from(!self.team_password.is_empty()) << 16) | 1
    }

    pub(crate) fn additional_data(&self, teammates: usize) -> u32 {
        (u32::from(!self.team_password.is_empty()) << 16)
            | u32::try_from(teammates).unwrap_or(u32::MAX)
    }

    pub(crate) fn team_name(&self) -> &[u8] {
        &self.team_name
    }

    pub(crate) fn team_password(&self) -> &[u8] {
        &self.team_password
    }

    pub(crate) const fn check_due(&self, sampled_at_ms: u32) -> bool {
        self.last_check_timestamp_ms
            .wrapping_add(TEAM_STATE_CHECK_INTERVAL_MS)
            <= sampled_at_ms
    }

    pub(crate) const fn record_check(&mut self, sampled_at_ms: u32) {
        self.last_check_timestamp_ms = sampled_at_ms;
    }

    pub(crate) const fn ends_for_team(
        player_id: i32,
        team_id: i32,
        team_leader_id: Option<i32>,
    ) -> bool {
        team_id != 0 && matches!(team_leader_id, Some(leader_id) if leader_id != player_id)
    }
}

pub(crate) fn team_state_begin_message(player_id: i32, state: &CTeamState) -> CMessage {
    let mut message = CMessage::new(TEAM_STATE_BEGIN_MESSAGE);
    message.add_long(400);
    message.add_long(player_id);
    message.add_long(state.state_id());
    message.add_long(state.client_state_time());
    message.add_ulong(state.initial_additional_data());
    message.base_mut().add(state.team_name());
    message.add_byte(0);
    message
}

pub(crate) fn team_state_end_message(player_id: i32) -> CMessage {
    let mut message = CMessage::new(TEAM_STATE_END_MESSAGE);
    message.add_long(400);
    message.add_long(player_id);
    message.add_long(TEAM_STATE_ID);
    message
}

pub(crate) fn team_state_update_message(
    player_id: i32,
    state: &CTeamState,
    teammate_count: usize,
) -> CMessage {
    let mut message = CMessage::new(TEAM_STATE_UPDATE_MESSAGE);
    message.add_long(player_id);
    message.add_long(player_id);
    message.add_long(state.state_id());
    message.add_ulong(state.additional_data(teammate_count));
    message
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp

// ============================================================================
// FUNCTION: CTeamState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:59
// RVA: 0x001BF800
// ADDRESS: 005bf800
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:77
// RVA: 0x001BF8D0
// ADDRESS: 005bf8d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:143
// RVA: 0x001BFA50
// ADDRESS: 005bfa50
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::CTeamState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:34
// RVA: 0x001BFC60
// ADDRESS: 005bfc60
// PROTOTYPE: undefined __thiscall CTeamState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::~CTeamState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:40
// RVA: 0x001BFCA0
// ADDRESS: 005bfca0
// PROTOTYPE: void __thiscall ~CTeamState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\teamstate.cpp:152
// RVA: 0x001BFF20
// ADDRESS: 005bff20
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
