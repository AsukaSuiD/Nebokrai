//! Паутина CSpiderWebState (0x199): запрет движения и боя, снимаемый
//! истечением или Cure, но не защитным действием. Источник: gameserver.exe
//! + GameServer.pdb, исходный владелец appserver/skills/spiderwebstate.cpp.
//!
//! Vtable наследует AI/End от CBlindState, а OnAction
//! пуст. Строгий wrapping deadline действует при нулевом сроке; End адресует
//! конкретный ключ общей арены и снимает по одному запрету движения и боя.
//! Объектный primary использует общий Blind Begin: timestamp при U,
//! BFE03 и move/fight-lock предшествуют публикации нового экземпляра.
//! DB-запись ID/remaining занимает 8 байт; StartAllStates восстанавливает
//! блокировки. GetRemainedTime сохраняет отдельное второе чтение
//! часов при положительном остатке. RAW координатной и типизированной
//! перегрузок Begin (0x005EA7D0/0x005EA8B0) сохранён отдельно от объектного пути.

use super::spiderweb::SPIDER_WEB_SKILL_ID;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const SPIDER_WEB_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderWebState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl SpiderWebState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != SPIDER_WEB_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }

    pub(crate) const fn skill_id(self) -> u32 {
        SPIDER_WEB_SKILL_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn encoded_for_install(self) -> [u8; SPIDER_WEB_STATE_BYTES] {
        let mut bytes = [0; SPIDER_WEB_STATE_BYTES];
        bytes[..4].copy_from_slice(&SPIDER_WEB_SKILL_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

impl super::blindstate::BlindStatePayload for SpiderWebState {
    fn blind_state_id(&self) -> u32 { SPIDER_WEB_SKILL_ID }
    fn begin_at(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_time(now) as u32 }
    fn install_record(&self) -> [u8; SPIDER_WEB_STATE_BYTES] { self.encoded_for_install() }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_spider_web_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SpiderWebState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn finish_player_spider_web_state_on_defense(
    game: &mut CGame,
    player_id: i32,
    _now_ms: u32,
) -> bool {
    let context = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.move_shape().applied_state_key::<SpiderWebState>()?))
    });
    let Some((region_id, identity, key)) = context else { return false };
    super::blindstate::end_blind_state(game, region_id, identity, key)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp

// ============================================================================
// FUNCTION: CSpiderWebState::CSpiderWebState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:24
// RVA: 0x001EA750
// ADDRESS: 005ea750
// PROTOTYPE: undefined __thiscall CSpiderWebState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::~CSpiderWebState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:33
// RVA: 0x001EA7C0
// ADDRESS: 005ea7c0
// PROTOTYPE: void __thiscall ~CSpiderWebState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:61
// RVA: 0x001EA7D0
// ADDRESS: 005ea7d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:81
// RVA: 0x001EA8B0
// ADDRESS: 005ea8b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
