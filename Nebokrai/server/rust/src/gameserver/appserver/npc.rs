//! Достигнутая storage/lifetime-часть `CNpc` исторического GameServer.
//!
//! Constructor `CNpc::CNpc` и пустой decoder подтверждены точной парой
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`; исходный owner —
//! `server/gameserver/appserver/npc.h/.cpp`. Constructor создаёт `CMoveShape`,
//! назначает type `500`, пустой byte-string script и `show-list = true`.
//! Остальные scalar-поля constructor не записывает: safe Rust хранит live/born
//! как `Option`, а region spawn обязан назначить live-time до публикации NPC.
//! Это явно сохраняет неизвестность вместо выдуманного нулевого default-а.
//!
//! Специализированная ветвь `CBaseObject::CreateObject(500,id)` живёт у factory
//! owner-а в `baseobject.rs`. `Vec<u8>` сохраняет legacy script без UTF-8.
//! Player interaction использует owned script path и immutable shape-view;
//! NPC virtual figure для distance остаётся нулевой, как достигнутый base shape.
//! Клиентский снимок `CNpc::AddToByteArray` точно совпадает с базовым shape-
//! префиксом и формируется здесь, у владельца конкретной категории.
//! Сценарный `NpcTalk` также формирует свой точный `0xBF801` здесь; `CGame`
//! оставляет за собой только spatial delivery вокруг принадлежащего региону NPC.
//! `AI` lifetime predicate вызывается из row-major active-shape scan; `CGame`
//! публикует `0xBF504(type,id,0)` и сразу удаляет NPC из region owner-а, как
//! virtual `DeleteChildObject` исходника. Полная shape serialization и Talk
//! для остальных категорий и Talk остаются RAW до подключения их цепочек.

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

    /// Формирует точный кадр сценарной функции `3301 / NpcTalk`. Переданное
    /// сценарием имя является частью wire и намеренно не заменяется именем
    /// объекта из region storage.
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

    /// Точный виртуальный `CNpc::AddToByteArray`: NPC не добавляет полей к
    /// базовому shape-префиксу. Параметр дочерних данных сохраняется для
    /// контракта виртуальной цепочки, хотя достигнутые базовые owners его не
    /// используют.
    pub(crate) fn encode_client_snapshot(&self, include_child: bool) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.move_shape
            .shape()
            .encode_to_byte_array(&mut payload, include_child)
            .then_some(payload)
    }

    /// Факты для виртуального `CMoveShape::SetTileXY`: NPC всегда занимает
    /// новую клетку независимо от здоровья, что отдельно учитывает общий
    /// позиционный механизм для типа `500`.
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

    /// Сохраняет wrapping `GetTickCount` и строгое сравнение `live < elapsed`.
    /// До region spawn live-time неинициализирован и predicate не применяется.
    pub(crate) const fn lifetime_expired(&self, now_ms: u32) -> bool {
        match (self.live_time_ms, self.born_time_ms) {
            (Some(live_time), Some(born_time)) if live_time != 0 => {
                live_time < now_ms.wrapping_sub(born_time)
            }
            _ => false,
        }
    }

    /// Exact `CNpc::DecordFromByteArray` ничего не читает и возвращает true.
    pub(crate) const fn decord_from_byte_array(
        &mut self,
        _source: &[u8],
        _cursor: &mut usize,
        _include_child: bool,
    ) -> bool {
        true
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.h

// ============================================================================
// FUNCTION: CNpc::AI
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: scalar predicate — `lifetime_expired`, wire/removal caller —
// `CGame::run_region_npc_ai`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp:62
// RVA: 0x001D3DD0
// ADDRESS: 005d3dd0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::CNpc
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp:17
// RVA: 0x001D3E90
// ADDRESS: 005d3e90
// PROTOTYPE: undefined __thiscall CNpc(void)
//
// /* public: __thiscall CNpc::CNpc(void) */
//
// IMPLEMENTED выше; ABI/vtable и MSVC SSO заменены typed composition/`Vec`.

// ============================================================================
// FUNCTION: CNpc::LossHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.h:42
// RVA: 0x001D3ED0
// ADDRESS: 005d3ed0
// PROTOTYPE: ushort __thiscall LossHP(long param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::~CNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp:23
// RVA: 0x001D3EE0
// ADDRESS: 005d3ee0
// PROTOTYPE: void __thiscall ~CNpc(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::Talk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp:38
// RVA: 0x001D3F50
// ADDRESS: 005d3f50
// PROTOTYPE: void __thiscall Talk(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::DecordFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp:33
// RVA: 0x001E9840
// ADDRESS: 005e9840
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// /* public: virtual bool __thiscall CNpc::DecordFromByteArray(unsigned char *,long &,bool) */
//
// IMPLEMENTED выше: source/cursor/include-child намеренно не читаются.

// COMPONENT_VARIANT_END: GameServer
