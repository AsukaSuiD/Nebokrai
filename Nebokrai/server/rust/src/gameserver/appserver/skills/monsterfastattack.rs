//! Двухударная быстрая атака (`CMonsterFastAttack`, ID `0x2d1`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/monsterfastattack.cpp`. Достигнутый вызов монстра хранит
//! визуальную фазу, первый удар и сроки `delay → first_time → second_time` в
//! этом модуле; общий проход ближней атаки сохраняет выбор цели, RNG, защиту, урон и
//! последствия смерти. Значения свойств `15001/15002` подтверждены прямыми
//! аргументами `QueryProperty` в RVA `0x00113810`.
//!
//! Ниже сохранены RAW только для недостигнутых координатных `Begin` и
//! ветвей с источником-игроком в `CheckCastCondition`, `CalculateAttackPower`,
//! `UpdateVisualEffect` и `AI`; их нельзя удалить до появления реального
//! вызывающего пути игрока.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.h

// ============================================================================
// FUNCTION: CMonsterFastAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp:152
// RVA: 0x001129A0
// ADDRESS: 005129a0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterFastAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp:171
// RVA: 0x00112A70
// ADDRESS: 00512a70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterFastAttackEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp:445
// RVA: 0x00112C70
// ADDRESS: 00512c70
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterFastAttack::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp:37
// RVA: 0x001131D0
// ADDRESS: 005131d0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterFastAttack::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp:371
// RVA: 0x00113490
// ADDRESS: 00513490
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterFastAttack::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterfastattack.cpp:206
// RVA: 0x00113810
// ADDRESS: 00513810
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use crate::nets::netserver::message::CMessage;

pub(crate) const MONSTER_FAST_ATTACK_SKILL_ID: u32 = 0x2d1;
pub(crate) const SKILL_USAGE_FIRST_TIME: u32 = 15_001;
pub(crate) const SKILL_USAGE_SECOND_TIME: u32 = 15_002;

pub(crate) fn fast_attack_fire_message(
    skill_level: u16,
    monster_id: i32,
    target_x: i32,
    target_y: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(MONSTER_FAST_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(600);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    message
}

/// Состояние двух последовательных ударов принадлежит конкретному навыку;
/// общий `SkillExecutionKernel` по-прежнему хранит начало и основные стадии.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterFastAttackProgress {
    visual_started: bool,
    first_attack_done: bool,
}

impl MonsterFastAttackProgress {
    pub(crate) const fn visual_started(self) -> bool {
        self.visual_started
    }

    pub(crate) const fn first_attack_done(self) -> bool {
        self.first_attack_done
    }

    pub(crate) const fn mark_visual_started(&mut self) {
        self.visual_started = true;
    }

    pub(crate) const fn mark_first_attack_done(&mut self) {
        self.first_attack_done = true;
    }
}
