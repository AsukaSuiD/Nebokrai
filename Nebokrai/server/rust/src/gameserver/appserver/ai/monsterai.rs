//! ИИ обычного монстра GameServer.
//!
//! Точная `CMonsterAI::SelectAttackSkill` из `gameserver.exe/.pdb` делает один
//! вызов `random(10000)`, проходит список навыков в исходном порядке и выбирает
//! первый ID, для которого бросок не больше накопленной суммы `odds`. Если
//! сумма не покрыла бросок, назначается стандартная атака владельца.
//! Выбранный ID хранится каноническим `CMoveShape::current_skill_id`; конкретный
//! владелец навыка разрешает уровень и исполняет стадии. Остальной корпус ниже
//! остаётся `UNKNOWN` (исследовательский декомпилят хранится локально) до достижения соответствующих AI-ветвей.

use crate::setup::monsterlist::MonsterSkill;

/// Сохраняет точный порядок и границу сравнения `SelectAttackSkill`.
/// `roll` получает вызывающая сторона из исходного генератора случайных чисел,
/// а стандартный навык вычисляет владелец формы по категориям установленных
/// навыков.
pub(crate) fn select_attack_skill(
    skills: &[MonsterSkill],
    roll: i32,
    default_skill_id: u16,
) -> u16 {
    let mut cumulative_odds = 0_i32;
    for skill in skills {
        cumulative_odds = cumulative_odds.wrapping_add(i32::from(skill.odds));
        if roll <= cumulative_odds {
            return skill.id;
        }
    }
    default_skill_id
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp

// ============================================================================
// FUNCTION: CMonsterAI::CMonsterAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:15
// RVA: 0x001DCB80
// ADDRESS: 005dcb80
// PROTOTYPE: undefined __thiscall CMonsterAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::~CMonsterAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:20
// RVA: 0x001DCBA0
// ADDRESS: 005dcba0
// PROTOTYPE: void __thiscall ~CMonsterAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::Run
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:25
// RVA: 0x001DCBB0
// ADDRESS: 005dcbb0
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::OnChangeSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:167
// RVA: 0x001DCBC0
// ADDRESS: 005dcbc0
// PROTOTYPE: int __thiscall OnChangeSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::HasTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:262
// RVA: 0x001DCC20
// ADDRESS: 005dcc20
// PROTOTYPE: int __thiscall HasTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::OnLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:267
// RVA: 0x001DCC30
// ADDRESS: 005dcc30
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::SetTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:272
// RVA: 0x001DCC40
// ADDRESS: 005dcc40
// PROTOTYPE: void __thiscall SetTarget(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::Hibernate
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CMonster::hibernate_ai` делегирует canonical `CBaseAI`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:284
// RVA: 0x001DCC50
// ADDRESS: 005dcc50
// PROTOTYPE: void __thiscall Hibernate(void)
//
// ============================================================================
// FUNCTION: CMonsterAI::OnIdle
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CMonster::hibernate_ai` и
// `execute_owned_monster_base_attack` сохраняют проверку соседних игроков,
// спящий переход и назначение текущего навыка до поиска противника для
// полностью достигнутых списков. Случайное блуждание и очередь ожидания RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:34
// RVA: 0x001DCC60
// ADDRESS: 005dcc60
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::WhenBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:218
// RVA: 0x001DCE00
// ADDRESS: 005dce00
// PROTOTYPE: void __thiscall WhenBeenHurted(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:252
// RVA: 0x001DCEC0
// ADDRESS: 005dcec0
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::WakeUp
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CMonster::wake_ai` и `CGame::wake_owned_monsters_around_area`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:296
// RVA: 0x001DCEE0
// ADDRESS: 005dcee0
// PROTOTYPE: void __thiscall WakeUp(void)
//
// ============================================================================
// FUNCTION: CMonsterAI::OnSchedule
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_monster_base_attack` сохраняет проверку цели,
// выбор текущего навыка, преследование, интервал атаки и запуск пяти
// достигнутых владельцев. Общий событийный автомат и прочие навыки RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:102
// RVA: 0x001DCF80
// ADDRESS: 005dcf80
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
