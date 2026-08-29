//! Владелец жизненного цикла прирученного монстра `CPet`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/pet.cpp`. Контроллер сохраняет секундный поиск хозяина,
//! шестичасовой счётчик жизни, переходы режима/действия, возврат и одичание.
//! Поиск живых владельцев, пространственное перемещение, пакеты и удаление остаются у
//! `CGame`; состояние хранится ровно один раз внутри `CMonster`.
//!
//! Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
//! Декомпилятор: Ghidra 12.1.2
//! Сохранены ещё не сопоставленные боевые и событийные ветви `CPet`.

const SEEK_MASTER_INTERVAL_MS: u32 = 1_000;
const LIFE_CYCLE_INTERVAL_MS: u32 = 21_600_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleState {
    seek_master_ms: u32,
    life_cycle_ms: u32,
    life_cycle_counter: u32,
    invalid_master_ms: u32,
    master_logout: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleFacts {
    pub(crate) now_ms: u32,
    pub(crate) wild_time_ms: u32,
    pub(crate) master_present: bool,
    pub(crate) master_close: bool,
    pub(crate) safe_cell: bool,
    pub(crate) reclaimable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PetLifecycleNotice {
    AgeWarning,
    AgeExpired,
    BecameWild,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleOutcome {
    pub(crate) notice: Option<PetLifecycleNotice>,
    pub(crate) reclaim: bool,
    pub(crate) vanish: bool,
    pub(crate) action: Option<i32>,
    pub(crate) mode: Option<i32>,
    pub(crate) clear_target: bool,
}

impl PetLifecycleState {
    pub(crate) fn tick(
        &mut self,
        facts: PetLifecycleFacts,
        current_mode: i32,
        has_target: bool,
    ) -> PetLifecycleOutcome {
        let mut outcome = PetLifecycleOutcome::default();
        if self.seek_master_ms != 0
            && facts.now_ms.wrapping_sub(self.seek_master_ms) < SEEK_MASTER_INTERVAL_MS
        {
            return outcome;
        }
        if self.life_cycle_ms == 0 {
            self.life_cycle_ms = facts.now_ms;
        }
        if facts.now_ms.wrapping_sub(self.life_cycle_ms) >= LIFE_CYCLE_INTERVAL_MS {
            self.life_cycle_counter = self.life_cycle_counter.wrapping_add(1);
            self.life_cycle_ms = facts.now_ms;
            if self.life_cycle_counter < 4 {
                if facts.master_present {
                    outcome.notice = Some(PetLifecycleNotice::AgeWarning);
                }
            } else {
                if facts.master_present {
                    outcome.notice = Some(PetLifecycleNotice::AgeExpired);
                }
                outcome.vanish = true;
                return outcome;
            }
        }
        self.seek_master_ms = facts.now_ms;
        if !facts.master_present {
            self.master_logout = true;
            if self.invalid_master_ms == 0 {
                outcome.action = Some(2);
                outcome.clear_target = facts.safe_cell || current_mode == 2;
                outcome.mode = Some(if facts.safe_cell { 0 } else { 1 });
                self.seek_master_ms = 0;
                self.invalid_master_ms = facts.now_ms;
            }
        } else {
            if self.master_logout && facts.reclaimable {
                outcome.clear_target = current_mode == 2 || has_target;
                self.invalid_master_ms = 0;
                self.seek_master_ms = 0;
                outcome.mode = Some(1);
                outcome.action = Some(1);
                self.master_logout = false;
                outcome.reclaim = true;
            }
            if facts.master_close {
                self.invalid_master_ms = 0;
                return outcome;
            }
            if self.invalid_master_ms == 0 {
                self.invalid_master_ms = facts.now_ms;
            }
        }
        if self.invalid_master_ms != 0
            && facts.now_ms.wrapping_sub(self.invalid_master_ms) >= facts.wild_time_ms
        {
            if facts.master_present {
                outcome.notice = Some(PetLifecycleNotice::BecameWild);
            }
            outcome.vanish = true;
        }
        outcome
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp

// ============================================================================
// FUNCTION: CPet::CPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:19
// RVA: 0x000E9400
// ADDRESS: 004e9400
// PROTOTYPE: undefined __thiscall CPet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::~CPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:23
// RVA: 0x000E9450
// ADDRESS: 004e9450
// PROTOTYPE: void __thiscall ~CPet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::WhenBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:490
// RVA: 0x000E94E0
// ADDRESS: 004e94e0
// PROTOTYPE: void __thiscall WhenBeenHurted(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnMoving
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:537
// RVA: 0x000E9540
// ADDRESS: 004e9540
// PROTOTYPE: int __thiscall OnMoving(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnIdle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:631
// RVA: 0x000E9580
// ADDRESS: 004e9580
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:653
// RVA: 0x000E95F0
// ADDRESS: 004e95f0
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::SetTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:666
// RVA: 0x000E9630
// ADDRESS: 004e9630
// PROTOTYPE: void __thiscall SetTarget(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnStayingSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:399
// RVA: 0x000E9650
// ADDRESS: 004e9650
// PROTOTYPE: void __thiscall OnStayingSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnSearchEnemy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:556
// RVA: 0x000E98A0
// ADDRESS: 004e98a0
// PROTOTYPE: int __thiscall OnSearchEnemy(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnAttackingSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:235
// RVA: 0x000E9A20
// ADDRESS: 004e9a20
// PROTOTYPE: void __thiscall OnAttackingSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
