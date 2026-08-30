//! Concrete `CNpc` исторического GameServer.
//!
//! Источник: `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходный
//! owner `server/gameserver/appserver/npc.h/.cpp`. Constructor создаёт
//! `CMoveShape`, назначает type `500`, пустой byte-string script и
//! `show-list = true`. Остальные scalar-поля constructor не записывает: Rust
//! хранит live/born как `Option`, а region spawn назначает их до публикации.
//! `LossHP` всегда возвращает ноль; decoder ничего не читает и возвращает true.
//! `Vec`/`Drop` заменяют SSO/destructor без дополнительного поведения.
//!
//! `Talk` формирует `0xBF801` и выбирает игроков из девяти соседних area, после
//! чего применяет строгий coordinate-filter `abs(dx) < AREA_WIDTH` и
//! `abs(dy) < AREA_HEIGHT`; storage traversal и доставка остаются у `CGame`.
//! `AI` сохраняет wrapping lifetime predicate, around delete-wire и немедленное
//! удаление из region owner-а. Клиентский snapshot совпадает с базовым shape-
//! префиксом и не добавляет NPC-specific полей.

use super::moveshape::{CMoveShape, MoveShapePositionFacts};
use super::shape::{ShapeFigure, ShapeView};
use crate::nets::netserver::message::CMessage;

const NPC_TYPE: i32 = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CNpc {
    move_shape: CMoveShape,
    script_file: Vec<u8>,
    show_list: bool,
    live_time_ms: Option<u32>,
    born_time_ms: Option<u32>,
}

impl CNpc {
    pub(crate) fn with_constructor_defaults() -> Self {
        let mut move_shape = CMoveShape::default();
        move_shape.shape_mut().base_object_mut().set_type(NPC_TYPE);
        Self {
            move_shape,
            script_file: Vec::new(),
            show_list: true,
            live_time_ms: None,
            born_time_ms: None,
        }
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) const fn npc_id(&self) -> i32 {
        self.move_shape.shape().identity().id
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.move_shape.shape().base_object().get_name()
    }

    pub(crate) fn set_script_file(&mut self, script_file: &[u8]) {
        let prefix_len = script_file
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(script_file.len());
        self.script_file.clear();
        self.script_file
            .extend_from_slice(&script_file[..prefix_len]);
    }

    pub(crate) fn script_file(&self) -> &[u8] {
        &self.script_file
    }

    pub(crate) fn shape_view(&self) -> Option<ShapeView> {
        let shape = self.move_shape.shape();
        Some(ShapeView {
            identity: shape.identity(),
            tile_x: shape.get_tile_x().ok()?,
            tile_y: shape.get_tile_y().ok()?,
            pos_x_bits: shape.get_pos_x().to_bits(),
            pos_y_bits: shape.get_pos_y().to_bits(),
            figure: ShapeFigure::default(),
        })
    }

    /// Формирует точный `CNpc::Talk`/script `3301` кадр. Переданное сценарием
    /// имя сохраняется как wire-поле; обычный native caller передаёт `name()`.
    pub(crate) fn build_script_talk_message(&self, name: &[u8], text: &[u8]) -> CMessage {
        let identity = self.move_shape.shape().identity();
        let mut message = CMessage::new(0x000b_f801);
        message.add_long(0);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add(name);
        message.add_byte(0);
        message.base_mut().add(text);
        message.add_byte(0);
        message
    }

    pub(crate) fn encode_client_snapshot(&self, include_child: bool) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.move_shape
            .shape()
            .add_to_byte_array(&mut payload, include_child)
            .then_some(payload)
    }

    pub(crate) fn movement_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        MoveShapePositionFacts {
            current_hit_points: 0,
            figure: ShapeFigure::default(),
            current_area: None,
            area_width,
            area_height,
        }
    }

    pub(crate) const fn set_show_list(&mut self, show_list: bool) {
        self.show_list = show_list;
    }

    pub(crate) const fn show_list(&self) -> bool {
        self.show_list
    }

    pub(crate) const fn set_live_time(&mut self, live_time_ms: u32) {
        self.live_time_ms = Some(live_time_ms);
    }

    pub(crate) const fn live_time(&self) -> Option<u32> {
        self.live_time_ms
    }

    pub(crate) const fn set_born_time(&mut self, born_time_ms: u32) {
        self.born_time_ms = Some(born_time_ms);
    }

    pub(crate) const fn born_time(&self) -> Option<u32> {
        self.born_time_ms
    }

    pub(crate) const fn lifetime_expired(&self, now_ms: u32) -> bool {
        match (self.live_time_ms, self.born_time_ms) {
            (Some(live_time), Some(born_time)) if live_time != 0 => {
                live_time < now_ms.wrapping_sub(born_time)
            }
            _ => false,
        }
    }

    /// Exact virtual `LossHP`: NPC не получает урон через combat-chain.
    pub(crate) const fn loss_hp(&mut self, _amount: i32, _source: Option<&CMoveShape>) -> u16 {
        0
    }

    pub(crate) const fn decord_from_byte_array(
        &mut self,
        _source: &[u8],
        _cursor: &mut usize,
        _include_child: bool,
    ) -> bool {
        true
    }
}
