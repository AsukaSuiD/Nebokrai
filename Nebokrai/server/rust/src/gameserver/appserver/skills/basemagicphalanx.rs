//! Владелец снаряда базовой магической атаки GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/basemagicphalanx.cpp`. Объект типа `1000` принадлежит
//! `CServerRegion`: первое чтение часов проверяет срок жизни строго через `>`,
//! второе отдельное чтение проверяет задержку атаки тем же строгим правилом.
//! После единственной попытки атаки объект отправляет exit и становится
//! `SHAPE_CHANGE_DELETE`; цель может исчезнуть без побочного эффекта. Формула
//! и два исходных вызова RNG выполняются в `CGame`, где доступны канонические
//! владельцы игрока, защиты и сети.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{
    CShape, SHAPE_CHANGE_DELETE, ShapeIdentity,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseMagicPhalanxTick {
    Pending,
    Attack {
        target: ShapeIdentity,
        sampled_at_ms: u32,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBaseMagicPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl CBaseMagicPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют constructor BaseMagicPhalanx")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
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
            minimum_attack,
            maximum_attack,
            element_modifier,
            attack_delay_ms,
            target,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub(crate) const fn master(&self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn skill_level(&self) -> i32 {
        self.skill_level
    }

    pub(crate) const fn minimum_attack(&self) -> i32 {
        self.minimum_attack
    }

    pub(crate) const fn maximum_attack(&self) -> i32 {
        self.maximum_attack
    }

    pub(crate) const fn element_modifier(&self) -> i32 {
        self.element_modifier
    }

    pub(crate) const fn target(&self) -> ShapeIdentity {
        self.target
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        get_attack_now_ms: impl FnOnce() -> u32,
    ) -> BaseMagicPhalanxTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return BaseMagicPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if attack_now_ms.wrapping_sub(self.started_at_ms) > self.attack_delay_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return BaseMagicPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        BaseMagicPhalanxTick::Pending
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagicphalanx.cpp

// ============================================================================
// FUNCTION: CBaseMagicPhalanx::CBaseMagicPhalanx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagicphalanx.cpp:27
// RVA: 0x002020B0
// ADDRESS: 006020b0
// PROTOTYPE: undefined __thiscall CBaseMagicPhalanx(tagMasterInfo * param_1, ulong param_2, long param_3, long param_4, long param_5, long param_6, long param_7, long param_8, long param_9)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagicPhalanx::~CBaseMagicPhalanx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagicphalanx.cpp:45
// RVA: 0x002021B0
// ADDRESS: 006021b0
// PROTOTYPE: void __thiscall ~CBaseMagicPhalanx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagicPhalanx::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagicphalanx.cpp:195
// RVA: 0x00202240
// ADDRESS: 00602240
// PROTOTYPE: void __thiscall CalculateAttackPower(tagAttackInformation * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagicPhalanx::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagicphalanx.cpp:172
// RVA: 0x00202440
// ADDRESS: 00602440
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagicPhalanx::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagicphalanx.cpp:65
// RVA: 0x00202540
// ADDRESS: 00602540
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
