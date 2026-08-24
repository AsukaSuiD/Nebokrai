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
//! Полная shape serialization, around-message, deferred delete и Talk остаются
//! RAW ниже до подключения соответствующих owner-цепочек.

use super::moveshape::CMoveShape;
use super::shape::{ShapeFigure, ShapeView};

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
// FUNCTION: CNpc::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\npc.cpp:27
// RVA: 0x001D3DC0
// ADDRESS: 005d3dc0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
