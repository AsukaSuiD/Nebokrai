//! Паутина CSpiderWebState (0x199), переходный адаптер Game: запрет движения
//! и боя, снимаемый истечением или Cure, но не защитным действием. Источник:
//! gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/skills/spiderwebstate.cpp. Данные и 8-байтная запись перенесены
//! в Zone `effects/blind.rs` (конструкторы VA 0x005EA6E0/0x005EA750,
//! vtable 0x0065FBCC).
//!
//! Vtable наследует AI/End от CBlindState, а OnAction пуст. End адресует
//! конкретный ключ общей арены и снимает по одному запрету движения и боя.
//! Объектный primary использует общий Blind Begin: timestamp при U,
//! BFE03 и move/fight-lock предшествуют публикации нового экземпляра.
//! StartAllStates восстанавливает блокировки после загрузки.
//! RAW координатной и типизированной перегрузок Begin сохранён отдельно
//! от объектного пути.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) use nebokrai_zone::effects::{SPIDER_WEB_STATE_BYTES, SpiderWebState};

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
