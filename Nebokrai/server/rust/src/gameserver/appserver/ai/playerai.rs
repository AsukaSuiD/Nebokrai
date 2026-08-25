//! Достигнутая client-destination часть `CPlayerAI` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/ai/playerai.cpp`. Трёхаргументный virtual `MoveTo` RVA
//! `0x0010A480` для живого player owner-а очищает emotion, удаляет старейшие
//! destination до длины не более трёх и затем добавляет `(direction, is_run)`;
//! так очередь после вызова содержит не более четырёх элементов. Owner/health
//! и ClearEmotion остаются у caller-а, чтобы не хранить raw pointers внутри AI.
//! Owner теперь принадлежит canonical `CPlayer`: quest movement и оба skill
//! message family кладут typed dispatch в его FIFO, а reached `CMoveShape::AI`
//! получает именно этот owner и может потребить очереди без shadow map.
//! Хвост `CPlayerAI::Run` после ещё внешних `CBaseAI::Run` и auto-exp теперь
//! хранит собственный energy clock, использует persisted player/faction facts,
//! exact unsigned due-check и возвращает изменение для адресного `0xBF72C`.
//! Четырёхаргументный pathfinding `MoveTo`, target/skill execution и остальные
//! методы ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use std::collections::VecDeque;

use crate::gameserver::appserver::player::{
    BattleFairySkillDispatch, CPlayer, PlayerSkillDispatch,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAiDestination {
    pub(crate) direction: i32,
    pub(crate) is_run: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerAI {
    destinations: VecDeque<PlayerAiDestination>,
    player_skills: VecDeque<PlayerSkillDispatch>,
    battle_fairy_skills: VecDeque<BattleFairySkillDispatch>,
    auto_inc_energy_last_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEnergyRegeneration {
    pub(crate) player_id: i32,
    pub(crate) sampled_at_ms: u32,
    pub(crate) increment: u32,
    pub(crate) previous_energy: u32,
    pub(crate) current_energy: u32,
}

impl CPlayerAI {
    pub(crate) fn destinations(&self) -> &VecDeque<PlayerAiDestination> {
        &self.destinations
    }

    pub(crate) fn queue_client_destination(&mut self, direction: i32, is_run: bool) {
        while 3 < self.destinations.len() {
            self.destinations.pop_front();
        }
        self.destinations
            .push_back(PlayerAiDestination { direction, is_run });
    }

    pub(crate) fn queue_player_skill(&mut self, dispatch: PlayerSkillDispatch) {
        self.player_skills.push_back(dispatch);
    }

    pub(crate) fn queue_battle_fairy_skill(&mut self, dispatch: BattleFairySkillDispatch) {
        self.battle_fairy_skills.push_back(dispatch);
    }

    pub(crate) fn player_skills(&self) -> &VecDeque<PlayerSkillDispatch> {
        &self.player_skills
    }

    pub(crate) fn battle_fairy_skills(&self) -> &VecDeque<BattleFairySkillDispatch> {
        &self.battle_fairy_skills
    }

    /// Exact energy tail `CPlayerAI::Run`: первый живой tick только заводит
    /// clock; full energy не двигает его дальше. Due comparison намеренно не
    /// wrap-safe (`last < now - interval`) — это наблюдаемая native-семантика.
    pub(crate) fn regenerate_player_energy(
        &mut self,
        player: &mut CPlayer,
        interval_ms: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerEnergyRegeneration> {
        if player.is_dead() {
            return None;
        }
        if self.auto_inc_energy_last_time_ms == 0 {
            self.auto_inc_energy_last_time_ms = get_tick_ms();
        }
        let previous_energy = player.energy();
        if previous_energy == player.maximum_energy() {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_energy_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_energy_last_time_ms = get_tick_ms();

        let faction_bonus = if player.faction_id() == 0 {
            0.0
        } else {
            (f64::from(player.level()) * f64::from(0.01_f32))
                .min(1.0)
                .mul_add(f64::from(player.faction_level()) * 0.5, 0.0)
                .max(1.0)
        };
        // MSVC меняет x87 rounding mode на truncation перед `__ftol2`.
        let increment =
            ((f64::from(player.level()) * f64::from(0.1_f32) - 1.0) * 5.0 + faction_bonus + 10.0)
                .trunc() as u32;
        if increment == 0 {
            return None;
        }
        player.set_energy(previous_energy.wrapping_add(increment));
        let current_energy = player.energy();
        (current_energy != previous_energy).then_some(PlayerEnergyRegeneration {
            player_id: player.player_id(),
            sampled_at_ms,
            increment,
            previous_energy,
            current_energy,
        })
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp

// ============================================================================
// FUNCTION: CPlayerAI::Tracing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:414
// RVA: 0x00108DC0
// ADDRESS: 00508dc0
// PROTOTYPE: int __thiscall Tracing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnChangeSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:608
// RVA: 0x00108E40
// ADDRESS: 00508e40
// PROTOTYPE: int __thiscall OnChangeSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnMoving
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:658
// RVA: 0x00108E90
// ADDRESS: 00508e90
// PROTOTYPE: int __thiscall OnMoving(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnStanding
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:685
// RVA: 0x00108ED0
// ADDRESS: 00508ed0
// PROTOTYPE: int __thiscall OnStanding(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::MoveTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:952
// RVA: 0x00108F10
// ADDRESS: 00508f10
// PROTOTYPE: void __thiscall MoveTo(CRegion * param_1, long param_2, long param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:450
// RVA: 0x00109130
// ADDRESS: 00509130
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnLoseTargetWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:484
// RVA: 0x001091B0
// ADDRESS: 005091b0
// PROTOTYPE: int __thiscall OnLoseTargetWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnFightingWithWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:517
// RVA: 0x00109230
// ADDRESS: 00509230
// PROTOTYPE: int __thiscall OnFightingWithWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnFighting
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:563
// RVA: 0x001092B0
// ADDRESS: 005092b0
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnChangeSkillWithWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:629
// RVA: 0x00109310
// ADDRESS: 00509310
// PROTOTYPE: int __thiscall OnChangeSkillWithWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::IsCanMoveTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:1057
// RVA: 0x00109360
// ADDRESS: 00509360
// PROTOTYPE: int __thiscall IsCanMoveTo(long param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Run
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:27
// RVA: 0x001093E0
// ADDRESS: 005093e0
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnScheduleAboutWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:143
// RVA: 0x00109730
// ADDRESS: 00509730
// PROTOTYPE: void __thiscall OnScheduleAboutWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:256
// RVA: 0x001098D0
// ADDRESS: 005098d0
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::CPlayerAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:19
// RVA: 0x00109B70
// ADDRESS: 00509b70
// PROTOTYPE: undefined __thiscall CPlayerAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:711
// RVA: 0x00109FF0
// ADDRESS: 00509ff0
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:841
// RVA: 0x0010A230
// ADDRESS: 0050a230
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
