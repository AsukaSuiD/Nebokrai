//! Сетевой адаптер входных shape-команд GameServer `0x8F901..0x8F905`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/shapemessage.cpp`; доказательная база —
//! `docs/protocol/game-actions.md`. Decode, типы команд и правила применения
//! принадлежат Zone `movement`; этот файл связывает сообщение с контекстом
//! игрока/региона и подключает ещё не перенесённые живые реестры прежнего
//! `CGame` к владельцу Zone через `ShapeMovementGame`.

use crate::gameserver::appserver::ai::baseai::AiShapeAction;
use crate::gameserver::appserver::serverregion::RegionMembershipBlock;
use crate::gameserver::appserver::shape::{ShapeCoordinateBlock, ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{CGame, GameClockContext, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use nebokrai_zone::movement::{
    ShapeMovementError, ShapeMovementGame, apply_shape_move_command, parse_shape_move_command,
    send_shape_move_cannot_move,
};
use tracing::trace;

pub(crate) type GameShapeMessageError = ShapeMovementError;

/// Прежняя граница соседних владельцев (AI-очередь назначений `game`).
pub(crate) fn send_player_cannot_move(
    game: &CGame,
    player_id: i32,
) -> Result<(), GameShapeMessageError> {
    send_shape_move_cannot_move(game, player_id)
}

pub(crate) fn dispatch_game_shape_message<Runtime: GameClockContext>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameShapeMessageError>> {
    let message_type = message.message_type() as u32;
    let command = match parse_shape_move_command(message_type, message.unread_bytes()) {
        Ok(Some(command)) => command,
        Ok(None) => return None,
        Err(error) => return Some(Err(error)),
    };
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let (Some(player_id), Some(region_id)) = (player_id, region_id) else {
        trace!(message_type, ?player_id, ?region_id, "shape-команда пропущена: нет контекста");
        return Some(Ok(()));
    };
    let mut now_milliseconds = || runtime.now_milliseconds();
    Some(apply_shape_move_command(
        game,
        player_id,
        region_id,
        command,
        &mut now_milliseconds,
    ))
}

impl ShapeMovementGame for CGame {
    fn player_shape_identity(&self, player_id: i32) -> Option<ShapeIdentity> {
        self.find_player(player_id)
            .map(|player| player.shape().identity())
    }

    fn player_tile_coordinates(
        &self,
        player_id: i32,
    ) -> Option<Result<(i32, i32), ShapeCoordinateBlock>> {
        let player = self.find_player(player_id)?;
        Some(
            player
                .shape()
                .get_tile_x()
                .and_then(|x| player.shape().get_tile_y().map(|y| (x, y))),
        )
    }

    fn player_is_dead(&self, player_id: i32) -> bool {
        self.find_player(player_id)
            .is_some_and(|player| player.is_dead())
    }

    fn player_contend_state(&self, player_id: i32) -> bool {
        self.find_player(player_id)
            .is_some_and(|player| player.contend_state())
    }

    fn player_active_action(&self, player_id: i32) -> Option<AiShapeAction> {
        self.find_player(player_id)
            .and_then(|player| player.player_ai().current_active_action())
    }

    fn player_current_skill_id(&self, player_id: i32) -> Option<u32> {
        self.find_player(player_id)
            .and_then(|player| player.current_skill_id())
    }

    fn player_script_state_present(&self, player_id: i32, state_id: i32) -> bool {
        self.find_player(player_id)
            .is_some_and(|player| player.script_move_state_count(state_id) != 0)
    }

    fn player_emotion_repeated(&self, emotion_id: i32) -> bool {
        self.emotion_repeated(emotion_id)
    }

    fn player_ai_has_resolved_target(&self, player_id: i32, region_id: i32) -> bool {
        self.player_ai_has_resolved_target(player_id, region_id)
    }

    fn apply_player_client_direction(
        &mut self,
        player_id: i32,
        direction: u8,
    ) -> Option<ShapeIdentity> {
        self.find_player_mut(player_id).map(|player| {
            player.apply_client_direction(direction);
            player.shape().identity()
        })
    }

    fn clear_player_emotion(&mut self, player_id: i32) {
        self.find_player_mut(player_id)
            .expect("shape-команда сохраняет игрока в пределах синхронного dispatch")
            .clear_emotion_state();
    }

    fn perform_player_emotion(
        &mut self,
        player_id: i32,
        emotion_id: i32,
        repeated: bool,
        now_ms: u32,
        ai_available: bool,
        ai_has_target: bool,
    ) -> bool {
        self.find_player_mut(player_id)
            .expect("shape-команда сохраняет игрока в пределах синхронного dispatch")
            .perform_emotion_state(emotion_id, repeated, now_ms, ai_available, ai_has_target)
    }

    fn queue_player_ai_destination(&mut self, player_id: i32, direction: i32, is_run: bool) -> bool {
        self.queue_player_ai_destination(player_id, direction, is_run)
    }

    fn allow_client_change_position(&self) -> bool {
        self.allow_client_change_position()
    }

    fn quest_move_rotation(&self) -> u8 {
        self.quest_move_rotation()
    }

    fn find_shape_in_region(&self, region_id: i32, identity: ShapeIdentity) -> Option<ShapeView> {
        self.find_shape_in_region(region_id, identity)
    }

    fn relocate_region_shape(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<Result<(), RegionMembershipBlock>> {
        self.relocate_region_shape(region_id, identity, tile_x, tile_y)
    }

    fn cancel_player_contend_in_region(&mut self, region_id: i32, player_id: i32) -> bool {
        self.cancel_player_contend_in_region(region_id, player_id)
    }

    fn serialize_player_shape_snapshot(
        &mut self,
        region_id: i32,
        shape: ShapeView,
        now_milliseconds: &mut dyn FnMut() -> u32,
    ) -> Option<(ShapeIdentity, Vec<u8>)> {
        self.serialize_player_shape_snapshot(region_id, shape, now_milliseconds)
    }

    fn serialize_owned_shape_snapshot(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        now_milliseconds: &mut dyn FnMut() -> u32,
    ) -> Option<(ShapeIdentity, Vec<u8>)> {
        self.serialize_owned_shape_snapshot(region_id, identity, now_milliseconds)
    }

    fn send_to_player(&self, player_id: i32, message: &CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_player_shape_around(
        &mut self,
        player_id: i32,
        excluded_player_id: Option<i32>,
        message: &CMessage,
    ) {
        let _ = self.send_player_shape_around(player_id, excluded_player_id, message);
    }

    fn send_shape_position_around(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        message: &CMessage,
    ) {
        let _ = self.send_shape_position_around(region_id, tile_x, tile_y, message);
    }

    fn send_colored_player_notice(
        &mut self,
        player_id: i32,
        first_color: u32,
        second_color: u32,
        string_id: &[u8],
    ) {
        let _ = colored_player_notice_message(
            first_color,
            second_color,
            self.get_string_by_id(string_id),
        )
        .send_to_player(self.net_server(), player_id);
    }
}
