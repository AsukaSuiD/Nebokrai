//! Региональный снаряд базовой стрельбы GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/archeryphalanx.cpp`. Два раздельных чтения часов,
//! строгие границы срока жизни и задержки атаки сохранены. В отличие от
//! базовой магии `End` лишь ставит `CS_DELETE`: отдельный немедленный пакет
//! выхода здесь не отправляется. Расчёт трёх типов урона и RNG принадлежит
//! `CGame`, где доступен живой владелец-игрок.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArcheryPhalanxTick {
    Pending,
    Attack {
        target: ShapeIdentity,
        sampled_at_ms: u32,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CArcheryPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl CArcheryPhalanx {
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        attack_delay_ms: u32,
        target: ShapeIdentity,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            skill_level,
            attack_delay_ms,
            target,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        get_attack_now_ms: impl FnOnce() -> u32,
    ) -> ArcheryPhalanxTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ArcheryPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if attack_now_ms.wrapping_sub(self.started_at_ms) > self.attack_delay_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ArcheryPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        ArcheryPhalanxTick::Pending
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp

// ============================================================================
// FUNCTION: CArcheryPhalanx::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:270
// RVA: 0x001E47A0
// ADDRESS: 005e47a0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:281
// RVA: 0x001EB070
// ADDRESS: 005eb070
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::CArcheryPhalanx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:28
// RVA: 0x00201AA0
// ADDRESS: 00601aa0
// PROTOTYPE: undefined __thiscall CArcheryPhalanx(tagMasterInfo * param_1, ulong param_2, long param_3, long param_4, long param_5, long param_6, long param_7, long param_8, long param_9)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::~CArcheryPhalanx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:46
// RVA: 0x00201BA0
// ADDRESS: 00601ba0
// PROTOTYPE: void __thiscall ~CArcheryPhalanx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:291
// RVA: 0x00201C10
// ADDRESS: 00601c10
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:196
// RVA: 0x00201C40
// ADDRESS: 00601c40
// PROTOTYPE: void __thiscall CalculateAttackPower(tagAttackInformation * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:173
// RVA: 0x00201F00
// ADDRESS: 00601f00
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryPhalanx::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:66
// RVA: 0x00202000
// ADDRESS: 00602000
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
