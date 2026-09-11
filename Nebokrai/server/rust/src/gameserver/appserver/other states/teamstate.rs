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
//! Exact `Serialize/Unserialize` по `0x005BFA50/0x005BFF20` сохраняют
//! `ID + team-name C-string + password C-string`; bounded Rust-кодек оставляет
//! допустимый максимум каждого legacy-буфера 255 байт. Координатные overload-ы
//! `Begin` пока не достигнуты и сохранены в RAW ниже.
//! restart_team_recruitment_state переносит object Begin 0x005BF9A0:
//! nonnull sufferer → base Begin(NULL, holder) → visual SetRun(1)/Update(0)
//! → base visual tail → lastcheck=0. Update0x005BFAD0 читает настоящий
//! GetAdditionalData0x005BFDD0: размер живой team либо1, затем password-bit,
//! а не постоянный initial count; non-player также оставляет исходную1.
//! Unserialize0x005BFF20 читает только две строки, без часов.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    default_client_state_time, begin_base_applied_state, begin_applied_state_visual,
    update_applied_state_visual_base, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const TEAM_STATE_ID: i32 = 0x0001_86a6;
const TEAM_STATE_STRING_CAPACITY: usize = 256;
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

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_i32()?;
        let team_name = reader.read_c_string(TEAM_STATE_STRING_CAPACITY)?.to_vec();
        let team_password = reader.read_c_string(TEAM_STATE_STRING_CAPACITY)?.to_vec();
        Ok(Self::new(team_name, team_password))
    }

    pub(crate) fn serialized_size(payload: &[u8], offset: usize) -> Option<usize> {
        let mut reader = LegacyReader::at(payload, offset).ok()?;
        let _state_id = reader.read_i32().ok()?;
        let _team_name = reader.read_c_string(TEAM_STATE_STRING_CAPACITY).ok()?;
        let _team_password = reader.read_c_string(TEAM_STATE_STRING_CAPACITY).ok()?;
        reader.position().checked_sub(offset)
    }

    pub(crate) fn encoded_for_install(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(6 + self.team_name.len() + self.team_password.len());
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(TEAM_STATE_ID);
        writer.write_c_string(&self.team_name);
        writer.write_c_string(&self.team_password);
        bytes
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

pub(crate) fn restart_team_recruitment_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CTeamState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let message = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<CTeamState>(key))
            .map(|state| {
                let teammates = if holder.object_type == 400 {
                    game.find_player(holder.id)
                        .map(|player| game.team_state_member_count(player.team_id()))
                        .unwrap_or(1)
                } else { 1 };
                let mut message = CMessage::new(TEAM_STATE_BEGIN_MESSAGE);
                message.add_long(holder.object_type);
                message.add_long(holder.id);
                message.add_long(state.state_id());
                message.add_long(state.client_state_time());
                message.add_ulong(state.additional_data(teammates));
                message.base_mut().add(state.team_name());
                message.add_byte(0);
                message
            });
        if let Some(message) = message {
            let _ = game.send_move_shape_around(region_id, holder, &message);
        }
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<CTeamState>(key))
    {
        state.last_check_timestamp_ms = 0;
    }
    true
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


// COMPONENT_VARIANT_END: GameServer
