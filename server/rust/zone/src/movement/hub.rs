//! Hub-швы правил движения ещё не перенесённых владельцев.
//!
//! По форме dispatcher-facades `ai/monsterai.rs`: trait перечисляет только
//! узкие маршруты к живым реестрам прежнего `CGame` (игрок, пространственный
//! реестр региона, regional aggregate, net-owner). Сами правила применения
//! клиентских команд — `crate::movement::apply`; когда соответствующий
//! владелец переезжает в Zone, его шов сужается, а не разрастается.

use crate::ai::AiShapeAction;
use crate::app::game_message::CMessage;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::membership::RegionMembershipBlock;
use crate::regions::shape::{ShapeCoordinateBlock, ShapeView};

/// Переходный фасад прежнего владельца `CGame` для правил движения: узкие
/// чтения живого игрока, пространственный реестр и рассылка кадров. Имена
/// членов сохраняют исходные операции старого владельца.
pub trait ShapeMovementGame {
    /// Идентичность формы живого игрока; `None` — игрок не зарегистрирован.
    fn player_shape_identity(&self, player_id: i32) -> Option<ShapeIdentity>;

    /// Текущие tile-координаты живого игрока; блок координат сохраняется
    /// как факт владельца формы, а не отказ команды.
    fn player_tile_coordinates(
        &self,
        player_id: i32,
    ) -> Option<Result<(i32, i32), ShapeCoordinateBlock>>;

    fn player_is_dead(&self, player_id: i32) -> bool;

    fn player_contend_state(&self, player_id: i32) -> bool;

    /// Текущее active action FIFO `CPlayerAI` живого игрока.
    fn player_active_action(&self, player_id: i32) -> Option<AiShapeAction>;

    fn player_current_skill_id(&self, player_id: i32) -> Option<u32>;

    /// Признак наличия сценарного move-state с заданным ID.
    fn player_script_state_present(&self, player_id: i32, state_id: i32) -> bool;

    /// `CEmotionMgr` прежнего владельца: реакция на повторную эмоцию.
    fn player_emotion_repeated(&self, emotion_id: i32) -> bool;

    fn player_ai_has_resolved_target(&self, player_id: i32, region_id: i32) -> bool;

    /// Применяет клиентское направление и возвращает идентичность формы.
    fn apply_player_client_direction(
        &mut self,
        player_id: i32,
        direction: u8,
    ) -> Option<ShapeIdentity>;

    /// Сбрасывает состояние эмоции; игрок гарантированно жив на момент вызова
    /// (синхронный dispatch той же команды), отказ — дефект инварианта hub.
    fn clear_player_emotion(&mut self, player_id: i32);

    /// Достигнутая `PerformEmotion` мутация состояния: возвращает признак
    /// разрешённой круговой публикации.
    fn perform_player_emotion(
        &mut self,
        player_id: i32,
        emotion_id: i32,
        repeated: bool,
        now_ms: u32,
        ai_available: bool,
        ai_has_target: bool,
    ) -> bool;

    /// Постановка назначения клиента в принадлежащий игроку `CPlayerAI`;
    /// `false` — очередь отклонила цель и владелец не зарегистрирован.
    fn queue_player_ai_destination(&mut self, player_id: i32, direction: i32, is_run: bool) -> bool;

    /// `CGlobeSetup::bAllowClientChangePos` процесса.
    fn allow_client_change_position(&self) -> bool;

    /// Серверная rotation для сверки с клиентским значением `0x8F903`.
    fn quest_move_rotation(&self) -> u8;

    /// Пространственный реестр живого региона.
    fn find_shape_in_region(&self, region_id: i32, identity: ShapeIdentity) -> Option<ShapeView>;

    /// Виртуальный `SetTileXY` достигнутых владельцев форм региона.
    fn relocate_region_shape(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<Result<(), RegionMembershipBlock>>;

    /// Virtual `CancelContendByPlayerID` военного региона.
    fn cancel_player_contend_in_region(&mut self, region_id: i32, player_id: i32) -> bool;

    /// Клиентский снимок живого игрока его serializer-владельцем.
    fn serialize_player_shape_snapshot(
        &mut self,
        region_id: i32,
        shape: ShapeView,
        now_milliseconds: &mut dyn FnMut() -> u32,
    ) -> Option<(ShapeIdentity, Vec<u8>)>;

    /// Клиентский снимок фигуры его региональным owner-ом.
    fn serialize_owned_shape_snapshot(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        now_milliseconds: &mut dyn FnMut() -> u32,
    ) -> Option<(ShapeIdentity, Vec<u8>)>;

    /// Адресная отправка одного кадра игроку силами net-owner-а.
    fn send_to_player(&self, player_id: i32, message: &CMessage);

    /// Круговая рассылка вокруг живого игрока.
    fn send_player_shape_around(
        &mut self,
        player_id: i32,
        excluded_player_id: Option<i32>,
        message: &CMessage,
    );

    /// Круговая рассылка вокруг прежней позиции, до перемещения.
    fn send_shape_position_around(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        message: &CMessage,
    );

    /// Цветное системное уведомление строки setup-таблицы одному игроку.
    fn send_colored_player_notice(
        &mut self,
        player_id: i32,
        first_color: u32,
        second_color: u32,
        string_id: &[u8],
    );
}
