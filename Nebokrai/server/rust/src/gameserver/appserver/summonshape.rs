//! Общая достигнутая часть идентичности и жизненного цикла `CSummonShape`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/summonshape.cpp`. Все summoned shapes получают process-wide
//! знаковый ID с переполнением из прежнего `g_lID`, тип `1000`, начальные часы
//! и срок жизни. Конкретная область, атака и игровые эффекты остаются у
//! производного владельца. Счётчик принадлежит `CGame`, а не региону, поэтому смена региона
//! не создаёт повторные legacy IDs.

pub(crate) const SUMMON_SHAPE_TYPE: i32 = 1000;

use crate::gameserver::appserver::skills::basemagicphalanx::CBaseMagicPhalanx;
use crate::gameserver::appserver::skills::battlefairybasemagicphalanx::CBattleFairyBaseMagicPhalanx;
use crate::gameserver::appserver::shape::CShape;
use crate::gameserver::appserver::masterinfo::MasterInfo;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SummonedSkillShape {
    BaseMagic(CBaseMagicPhalanx),
    BattleFairyBaseMagic(CBattleFairyBaseMagicPhalanx),
}

impl SummonedSkillShape {
    pub(crate) const fn shape(&self) -> &CShape {
        match self {
            Self::BaseMagic(shape) => shape.shape(),
            Self::BattleFairyBaseMagic(shape) => shape.shape(),
        }
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        match self {
            Self::BaseMagic(shape) => shape.shape_mut(),
            Self::BattleFairyBaseMagic(shape) => shape.shape_mut(),
        }
    }

    pub(crate) const fn master(&self) -> MasterInfo {
        match self {
            Self::BaseMagic(shape) => shape.master(),
            Self::BattleFairyBaseMagic(shape) => shape.master(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct NextSummonShapeId(i32);

impl NextSummonShapeId {
    pub(crate) fn take(&mut self) -> i32 {
        let id = self.0;
        self.0 = self.0.wrapping_add(1);
        id
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.h

// ============================================================================
// FUNCTION: CSummonShape::GetSkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.h:44
// RVA: 0x001E1030
// ADDRESS: 005e1030
// PROTOTYPE: tagSkillID __thiscall GetSkillID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonShape::CSummonShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:15
// RVA: 0x001E97A0
// ADDRESS: 005e97a0
// PROTOTYPE: undefined __thiscall CSummonShape(tagMasterInfo * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonShape::~CSummonShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:26
// RVA: 0x001E9850
// ADDRESS: 005e9850
// PROTOTYPE: void __thiscall ~CSummonShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonShape::GetRemainedTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:134
// RVA: 0x001E9870
// ADDRESS: 005e9870
// PROTOTYPE: ulong __thiscall GetRemainedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScope::CScope
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:143
// RVA: 0x001E98B0
// ADDRESS: 005e98b0
// PROTOTYPE: undefined __thiscall CScope(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScope::~CScope
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:154
// RVA: 0x001E9900
// ADDRESS: 005e9900
// PROTOTYPE: void __thiscall ~CScope(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScope::Get
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:161
// RVA: 0x001E9920
// ADDRESS: 005e9920
// PROTOTYPE: int __thiscall Get(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScope::Set
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:169
// RVA: 0x001E9960
// ADDRESS: 005e9960
// PROTOTYPE: void __thiscall Set(uchar * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScope::Set
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:175
// RVA: 0x001E9990
// ADDRESS: 005e9990
// PROTOTYPE: void __thiscall Set(uchar param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CScope::GetOverlappingField
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:202
// RVA: 0x001E99C0
// ADDRESS: 005e99c0
// PROTOTYPE: int __cdecl GetOverlappingField(tagRetangle * param_1, tagRetangle param_2, tagRetangle * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonShape::ForceMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:35
// RVA: 0x001E9AF0
// ADDRESS: 005e9af0
// PROTOTYPE: void __thiscall ForceMove(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonShape::MoveStep
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:81
// RVA: 0x001E9C20
// ADDRESS: 005e9c20
// PROTOTYPE: void __thiscall MoveStep(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSummonShape::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\summonshape.cpp:124
// RVA: 0x001E9DC0
// ADDRESS: 005e9dc0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: GameServer
