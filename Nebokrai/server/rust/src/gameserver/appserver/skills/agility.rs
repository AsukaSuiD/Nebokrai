//! Общее исполнение семейства `CAgility/CAgility2/CNatural/CRapture`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/agility.cpp` и `agility2.cpp`. Общая часть сохраняет
//! отдельные часы восстановления, повторную проверку и расход MP, дополнительный
//! `UpdateCurrentState`, задержку и сетевой формат навыка на себя. Нулевая цена MP
//! намеренно не ставит запрет движения, хотя завершение всё равно снимает его.
//! Три постоянных состояния взаимно заменяются, а временная `CAgility2`
//! заменяет только себя. Различающиеся свойства и жизненный цикл принадлежат
//! `CanonicalStateStorage` и вызывающему `CGame`; после замены состояние
//! немедленно входит в полный `UpdateProperty`, а не ждёт постороннего
//! пересчёта. Клиентская отмена проходит через тот же семейный владелец и не
//! откатывает уже выполненный расход MP.
//! Reuse каждого ID проверяется общим absolute deadline `CSkill::IsRestored`;
//! задержка исполнения остаётся отдельным elapsed-интервалом.
//! Begin заканчивается возвратом Begun после создания исполнения. Проверки
//! и эффекты первого AI остаются после этой границы; координатор вызывает AI
//! в том же Run после постановки Attack, не сдвигая исходное время Begin.

use super::agility2::begin_agility_2_state;
pub(crate) use super::agility2::AGILITY_2_SKILL_ID;
use super::agilitystate::{
    send_agility_family_state_visual, AgilityState, PersistentAgilityFamilyState,
};
use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::stateskill::finish_state_skill;
use super::natural::{NATURAL_SKILL_ID, SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN};
use super::naturalstate::NaturalState;
use super::rapture::{RAPTURE_SKILL_ID, SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN};
use super::rapturestate::RaptureState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};

pub(crate) const AGILITY_SKILL_ID: u32 = 218;
pub(crate) const AGILITY_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_TARGET_FULL_MISS_GAIN: u32 = 127;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityFamilyExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}

impl AgilityFamilyExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

fn restore_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn abort_player_agility(game: &mut CGame, player_id: i32) {
    restore_movement(game, player_id);
}

fn finish_player_agility<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    restore_movement(game, player_id);
    finish_state_skill(game, player_id, skill_id, runtime);
}

pub(crate) fn cancel_player_agility_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<AgilityFamilyExecutionState>(player_id, execution_skill_id).copied()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    let skill_id = dispatch.skill_id();
    finish_player_agility(game, player_id, skill_id, player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_agility_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if matches!(
                skill_id,
                AGILITY_SKILL_ID | AGILITY_2_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID
            ) => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if player.server_region_id().is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let skill_level = player.learned_skill_level(skill_id, game.skill_factory());
    let initial_mana = player.mana();
    let Some(properties) = game.skill_base_properties(skill_id, skill_level)
    else {
        if game.player_skill_state::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()).copied().is_some() {
            abort_player_agility(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    enum FamilyBonus {
        FullMiss(u16),
        ElementResistance(u16),
        BlastAttack(u16),
    }
    let (bonus, insufficient_mana_message): (FamilyBonus, &[u8]) = match skill_id {
        NATURAL_SKILL_ID => (
            FamilyBonus::ElementResistance(
                properties.query_property(SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN) as u16,
            ),
            b"GS0288",
        ),
        RAPTURE_SKILL_ID => (
            FamilyBonus::BlastAttack(
                properties.query_property(SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN) as u16,
            ),
            b"GS0279",
        ),
        _ => (
            FamilyBonus::FullMiss(properties.query_property(SKILL_USAGE_TARGET_FULL_MISS_GAIN) as u16),
            b"GS0279",
        ),
    };
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()).copied().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let cooldown_now_ms = runtime.now_milliseconds();
        let last_used_ms = game.player_skill_last_used_ms(player_id, skill_id);
        if !skill_is_restored(last_used_ms, reuse_delay_ms, cooldown_now_ms) {
            game.send_self_state_skill_failure(AGILITY_EFFECT_MESSAGE, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            abort_player_agility(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && initial_mana < mp_loss {
            game.send_self_state_skill_failure(AGILITY_EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(
                player_id,
                insufficient_mana_message,
                mp_loss,
            );
            abort_player_agility(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, AgilityFamilyExecutionState::begin(
            dispatch,
            started_at_ms,
        ));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()).copied()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.find_player(player_id).is_some_and(CPlayer::is_dead) {
        game.send_self_state_skill_failure(AGILITY_EFFECT_MESSAGE, player_id, 2);
        finish_player_agility(game, player_id, skill_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.player_skill_state::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()).copied()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            game.send_self_state_skill_failure(AGILITY_EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(
                player_id,
                insufficient_mana_message,
                mp_loss,
            );
            abort_player_agility(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        let _ = game.update_player_criminal_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
            runtime,
        );
        game.send_self_state_skill_cast(
            AGILITY_EFFECT_MESSAGE,
            player_id,
            skill_id,
            skill_level,
            1,
        );
        if let Some(state) = game.player_skill_state_mut::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_state::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()).copied()
        .map(|state| state.kernel().started_at_ms())
        .expect("исполнение семейства ловкости создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.send_self_state_skill_cast(
        AGILITY_EFFECT_MESSAGE,
        player_id,
        skill_id,
        skill_level,
        2,
    );

    let removed_skill_id = if skill_id == AGILITY_2_SKILL_ID {
        game.find_player_mut(player_id)
            .and_then(CPlayer::take_agility_state_2)
            .map(|state| state.skill_id())
    } else {
        game.find_player_mut(player_id)
            .and_then(CPlayer::take_persistent_agility_family_state)
            .map(PersistentAgilityFamilyState::skill_id)
    };
    if let Some(removed_skill_id) = removed_skill_id {
        if removed_skill_id != AGILITY_2_SKILL_ID {
            send_agility_family_state_visual(
                game,
                player_id,
                removed_skill_id,
                false,
                0,
            );
        }
        let _ = game.publish_player_states(player_id);
    }

    let state_started_at_ms = runtime.now_milliseconds();
    let client_time = if skill_id == AGILITY_2_SKILL_ID {
        let FamilyBonus::FullMiss(full_miss) = bonus else { unreachable!() };
        begin_agility_2_state(
            game,
            player_id,
            full_miss,
            state_started_at_ms,
            keep_time_ms,
            runtime,
        )
    } else {
        let state = match bonus {
            FamilyBonus::FullMiss(full_miss) => PersistentAgilityFamilyState::Agility(
                AgilityState::persistent(full_miss),
            ),
            FamilyBonus::ElementResistance(gain) => {
                PersistentAgilityFamilyState::Natural(NaturalState::new(gain))
            }
            FamilyBonus::BlastAttack(gain) => {
                PersistentAgilityFamilyState::Rapture(RaptureState::new(gain))
            }
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.begin_persistent_agility_family_state(state);
        }
        0
    };
    send_agility_family_state_visual(game, player_id, skill_id, true, client_time);
    let _ = game.publish_player_states(player_id);
    let _ = game.update_player_properties(player_id);
    if let Some(state) = game.player_skill_state_mut::<AgilityFamilyExecutionState>(player_id, dispatch.skill_id()) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_agility(game, player_id, skill_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только посторонний недостигнутый callback контейнера; skill-путь материализован полностью.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.h

// ============================================================================
// FUNCTION: CContainerListener::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:33
// RVA: 0x001AFCE0
// ADDRESS: 005afce0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//











// COMPONENT_VARIANT_END: GameServer
