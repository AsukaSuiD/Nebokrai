//! Общий короткий runtime немедленных состояний игрока и монстра.
//! Monster active/background используют единый выбор concrete owner-а.
//! Он не выполняет Begin и не меняет active FIFO: допуск и фазы остаются
//! у caller-а; WuXing сохраняет End(0), Swordship — собственный порядок End.
//!
//! Владелец объединяет только подтверждённую одинаковую последовательность
//! `Begin → Check → Calculate → Attack → Apply`. Идентификатор usage,
//! формула значения и конкретное каноническое состояние остаются у пяти
//! навыков семейства; завершение сохраняет общий для конкретного skill ID
//! reuse-clock. Точные `CEnlargeFullMiss/MaxMp/MaxHp::AI` RVA
//! `0x00116720/0x001169B0/0x00116BF0` принимают и target, и sufferer owner-а,
//! заменяют одноимённое состояние и публикуют `OnChangeStates`; их
//! `OnUpdateProperties` меняет характеристики только при type `400`, поэтому
//! monster хранит и сериализует состояние без придуманного property-effect.
//! `CGame` предоставляет canonical shape owner, свойства навыка, пересчёт
//! производных характеристик и фактическую публикацию состояния.
//! Входной reuse-gate сохраняет общий `CSkill::IsRestored`, включая нулевой
//! timestamp нового экземпляра навыка.
//! `End(1)` записывает reuse, но не создаёт событие активного ИИ: оно
//! принадлежит вызывающему `OnFighting`, а не фоновой очереди состояний.
//! Автоматический и активный входы используют один AI. End (0x005AFA40)
//! передаёт аргумент в CStateSkill::End; успешный Enlarge/TaiJi/Origin
//! вызывает End(1), включая AfterUseSkill (0x0053CF30) до записи cooldown.
//! Поэтому фоновое применение не пропускает оружейный эффект завершения.
//! Успешный Begin возвращает Begun до наложения состояния; авто-вход уже
//! имеет kernel от AddObject и не повторяет Begin или его reuse-проверку.
//! Все 14 вариантов MonsterImmediateSkill имеют End по адресу 0x005AFA40
//! (включая четыре Swordship и пять WuXing). Внешний Stiffen вызывает End(4),
//! поэтому CMonster отмечает ended и reuse даже для вариантов с обычным
//! AI-End(0). Это завершение навыка, а не End наложенного state-объекта;
//! состояние и порядок удаления записи фоновой очереди остаются независимыми.

use super::baseattack::SKILL_USAGE_REUSE_DELAY_TIME;
use super::enlargefullmiss::{ENLARGE_FULL_MISS_SKILL_ID, SKILL_USAGE_FULL_MISS_GAIN};
use super::enlargefullmissstate::EnlargeFullMissState;
use super::enlargemaxhp::{ENLARGE_MAX_HP_SKILL_ID, SKILL_USAGE_MAX_HP_GAIN};
use super::enlargemaxhpstate::EnlargeMaxHpState;
use super::enlargemaxmp::{ENLARGE_MAX_MP_SKILL_ID, SKILL_USAGE_MAX_MP_GAIN};
use super::enlargemaxmpstate::EnlargeMaxMpState;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::stateskill::finish_state_skill;
use super::origin::{ORIGIN_SKILL_ID, SKILL_USAGE_ELEMENT_MODIFY_GAIN};
use super::originstate::OriginState;
use super::taiji::{SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN, TAIJI_SKILL_ID};
use super::wuxing::{execute_player_wuxing, is_wuxing_skill};
use super::taijistate::TaiJiState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

enum ImmediateStateKind {
    TaiJi,
    EnlargeFullMiss,
    EnlargeMaxHp,
    EnlargeMaxMp,
    Origin,
}

pub(crate) enum MonsterImmediateSkill {
    State,
    Swordship,
    PlayerOnly,
}

impl MonsterImmediateSkill {
    pub(crate) fn from_skill_id(skill_id: u32) -> Option<Self> {
        if super::swordship::is_swordship_skill(skill_id) {
            Some(Self::Swordship)
        } else if is_wuxing_skill(skill_id) {
            Some(Self::PlayerOnly)
        } else if is_immediate_state_skill(skill_id) {
            Some(Self::State)
        } else {
            None
        }
    }

    pub(crate) const fn has_effect(&self) -> bool {
        !matches!(self, Self::PlayerOnly)
    }

    pub(crate) fn execute(
        self, game: &CGame, region: &mut CServerRegion, monster_id: i32,
        skill_id: u32, skill_level: i32, now_ms: u32,
    ) -> bool {
        match self {
            Self::State => execute_monster_immediate_state(
                game, region, monster_id, skill_id, skill_level, now_ms,
            ),
            Self::Swordship => super::swordship::execute_monster_auto_start_swordship(
                game, region, monster_id, skill_id, skill_level,
            ),
            Self::PlayerOnly => {
                let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
                    return false;
                };
                monster.move_shape_mut().finish_immediate_skill(skill_id);
                true
            }
        }
    }
}

/// Monster-ветвь пяти подтверждённых immediate-state `AI`: owner навыка
/// является sufferer-ом, поэтому состояние заменяется на самом монстре.
/// `TaiJi/Origin` участвуют в monster combat getters; `601..603` сохраняют
/// исходный player-only property gate, но остаются видимы в state snapshot.
pub(crate) fn execute_monster_immediate_state(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    skill_id: u32,
    skill_level: i32,
    now_ms: u32,
) -> bool {
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return false;
    };
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    match skill_id {
        TAIJI_SKILL_ID => {
            let gain = properties.query_property(SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN) as i32;
            let _ = monster
                .move_shape_mut()
                .replace_taiji_state(TaiJiState::new(gain));
        }
        ENLARGE_FULL_MISS_SKILL_ID => {
            let gain = properties.query_property(SKILL_USAGE_FULL_MISS_GAIN) as i32;
            let _ = monster
                .move_shape_mut()
                .replace_enlarge_full_miss_state(EnlargeFullMissState::new(gain));
        }
        ENLARGE_MAX_HP_SKILL_ID => {
            let gain = properties.query_property(SKILL_USAGE_MAX_HP_GAIN) as i32;
            let _ = monster
                .move_shape_mut()
                .replace_enlarge_max_hp_state(EnlargeMaxHpState::new(gain));
        }
        ENLARGE_MAX_MP_SKILL_ID => {
            let gain = properties.query_property(SKILL_USAGE_MAX_MP_GAIN) as i32;
            let _ = monster
                .move_shape_mut()
                .replace_enlarge_max_mp_state(EnlargeMaxMpState::new(gain));
        }
        ORIGIN_SKILL_ID => {
            let gain = properties.query_property(SKILL_USAGE_ELEMENT_MODIFY_GAIN) as i32;
            let _ = monster
                .move_shape_mut()
                .replace_origin_state(OriginState::new(gain));
        }
        _ => return false,
    }
    monster.mark_immediate_skill_used(skill_id, now_ms);
    let _ = game.publish_owned_monster_states(region, monster_id);
    true
}

pub(crate) const fn is_immediate_state_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        TAIJI_SKILL_ID
            | ENLARGE_MAX_HP_SKILL_ID
            | ENLARGE_MAX_MP_SKILL_ID
            | ENLARGE_FULL_MISS_SKILL_ID
            | ORIGIN_SKILL_ID
    ) || is_wuxing_skill(skill_id)
}


pub(crate) fn execute_player_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let dispatch_skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    if is_wuxing_skill(dispatch_skill_id) {
        return execute_player_wuxing(game, player_id, dispatch, player_ai, runtime);
    }
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if is_immediate_state_skill(skill_id) => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    if game.find_player(player_id).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let skill_level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(skill_id));
    let Some(properties) = game.skill_base_properties(skill_id, skill_level).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);

    if player_ai.player_skill_execution(skill_id).is_none() {
        let cooldown_now_ms = runtime.now_milliseconds();
        let last_used_ms = player_ai.skill_last_used_ms(skill_id);
        if !skill_is_restored(last_used_ms, reuse_delay_ms, cooldown_now_ms) {
            game.send_base_magic_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai
        .player_skill_execution(skill_id)
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let (usage, state_kind) = match skill_id {
        TAIJI_SKILL_ID => (
            SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN,
            ImmediateStateKind::TaiJi,
        ),
        ENLARGE_FULL_MISS_SKILL_ID => (
            SKILL_USAGE_FULL_MISS_GAIN,
            ImmediateStateKind::EnlargeFullMiss,
        ),
        ENLARGE_MAX_HP_SKILL_ID => (SKILL_USAGE_MAX_HP_GAIN, ImmediateStateKind::EnlargeMaxHp),
        ENLARGE_MAX_MP_SKILL_ID => (SKILL_USAGE_MAX_MP_GAIN, ImmediateStateKind::EnlargeMaxMp),
        ORIGIN_SKILL_ID => (SKILL_USAGE_ELEMENT_MODIFY_GAIN, ImmediateStateKind::Origin),
        _ => unreachable!(),
    };
    let gain = properties.query_property(usage) as i32;
    if let Some(player) = game.find_player_mut(player_id) {
        match state_kind {
            ImmediateStateKind::TaiJi => {
                let _ = player.replace_taiji_state(TaiJiState::new(gain));
            }
            ImmediateStateKind::EnlargeFullMiss => {
                let _ = player.replace_enlarge_full_miss_state(EnlargeFullMissState::new(gain));
            }
            ImmediateStateKind::EnlargeMaxHp => {
                let _ = player.replace_enlarge_max_hp_state(EnlargeMaxHpState::new(gain));
            }
            ImmediateStateKind::EnlargeMaxMp => {
                let _ = player.replace_enlarge_max_mp_state(EnlargeMaxMpState::new(gain));
            }
            ImmediateStateKind::Origin => {
                let _ = player.replace_origin_state(OriginState::new(gain));
            }
        }
    }
    let _ = game.publish_player_states(player_id);
    let _ = game.update_player_properties(player_id);
    if let Some(state) = player_ai.player_skill_execution_mut(skill_id) {
        let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(skill_id, now_ms);
    });
    terminal(QueuedSkillExecutionState::Completed)
}
