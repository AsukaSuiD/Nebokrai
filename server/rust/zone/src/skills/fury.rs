//! Общая RP-подготовка яростных навыков CFury (0x1A3) и CRageBreak (0x6E).
//! Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match), исходный владелец
//! `appserver/skills/fury.cpp/.h`; CFury::AI VA `0x536C43`–`0x536CCE`,
//! разделяемые с CRageBreak Check/AI (`0x59FF10`/`0x5A00F0`) исходят из
//! `appserver/skills/{fury,ragebreak}.cpp`. RP-хелперы перенесены буквально:
//! тело Check и подготовка AI нужны сразу двум владельцам, а Zone не зависит
//! от старого пакета. Сам CFury остаётся у прежнего владельца.
//!
//! Check (`0x59FF10`): абсолютный срок reuse (`0x2715`), затем игрок-RTTI;
//! стоимость RP проверяется ДО чтения RP — нулевая стоимость у RageBreak
//! отклоняется тихо (без visual), а signed-дефицит RP даёт visual8 и
//! GS0289 с суммой стоимости. Расход RP в AI идёт только первым проходом
//! игрока и завершается OnChangeStates; CAN `0x2716` ложится в [+0x3C],
//! задержка — абсолютная `start + 0x2711` по unsigned-сравнению.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//! реализован у владельца старого пакета; RP-операции `CPlayer` объявлены
//! собственным швом `RageCastPlayer` (реализация у делегата старого пакета).
//! Исход делегата `End(0/1)` моделируется `StateCastExecutionOutcome`:
//! точки прежнего `end_state_skill(...)` помечаются EndRejected/EndCompleted.

use std::ops::ControlFlow;

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::lifecycle::{SkillStage, skill_is_restored};
use super::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastMoveShape, state_cast_participant,
};

const RP_LOSS: u32 = 3;
const CAN_BREAK: u32 = 10_006;
const DELAY: u32 = 10_001;
const REUSE: u32 = 10_005;

/// Девять ID состояний (`CFury::AI` / `CRageBreak::AI`, безусловный свип
/// перед созданием Cure: End + deleting dtor + обнуление слота живого
/// вектора, без полного destroy извне).
pub fn is_fury_conflicting_state_id(state_id: u32) -> bool {
    matches!(
        state_id,
        0x138 | 0xd2 | 0xc9 | 0x67 | 0x192 | 0x191 | 0x198 | 0x199 | 0x1a6
    )
}

/// RP-операции прежнего `CPlayer` для яростных навыков: `rp()` читает
/// WORD-поле, `set_rp` — его запись после signed-проверки дефицита.
pub trait RageCastPlayer {
    fn rp(&self) -> u16;

    fn set_rp(&mut self, rp: u16);
}

/// Различие двух RP-навыков касается только первого Check, не расхода в AI:
/// у RageBreak нулевая стоимость RP — тихий отказ, у Fury она допустима.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RageRpPolicy {
    AllowZero,
    RequirePositive,
}

pub struct RageSkillEffect {
    pub source: (i32, ShapeIdentity),
    pub properties: CSkillBaseProperties,
}

fn fail_rp<Game: StateCastGame>(
    game: &mut Game,
    address: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(address, 8);
    if source.1.object_type == PLAYER_TYPE {
        game.send_skill_system_info_with_unsigned(
            source.1.id, b"GS0289", properties.query_property(RP_LOSS),
        );
    }
}

/// Check яростной пары: reuse, RTTI-гейт игрока, стоимость и дефицит RP,
/// запрет движения U. Порядок запросов свойств сохраняет исходный (см. шапку).
pub fn check_rage_skill_cast<Game>(
    game: &mut Game,
    address: Game::SkillAddress,
    policy: RageRpPolicy,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    Game: StateCastGame,
    Game::Player: RageCastPlayer,
{
    let Some(skill) = game.registered_skill(address) else { return false; };
    let Some(source) = state_cast_participant(game, skill.lifecycle().user()) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, now()) {
        game.update_registered_skill_visual(address, 13);
        if source.1.object_type == PLAYER_TYPE {
            game.send_skill_system_info(source.1.id, b"GS0278");
        }
        return false;
    }
    if source.1.object_type != PLAYER_TYPE { return true; }
    // RageBreak проверяет стоимость до чтения RP, затем запрашивает её вновь.
    // Fury сразу читает RP и только после этого делает единственный запрос.
    if matches!(policy, RageRpPolicy::RequirePositive) && properties.query_property(RP_LOSS) == 0 {
        return false;
    }
    let Some(rp) = game.find_player(source.1.id).map(|player| player.rp()) else { return false; };
    if (u32::from(rp).wrapping_sub(properties.query_property(RP_LOSS)) as i32) < 0 {
        fail_rp(game, address, source, &properties);
        return false;
    }
    if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
        user.set_moveable(false);
    }
    true
}

/// Общая подготовка Fury/RageBreak заканчивается visual1. Таблица этого AI
/// и его U передаются обработчику состояний, не разрешаясь заново по ID навыка.
/// IsDied U завершает навык с End(1) после visual2; дефицит RP — с End(0).
pub fn prepare_rage_skill_effect<Game>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> ControlFlow<StateCastExecutionOutcome, RageSkillEffect>
where
    Game: StateCastGame,
    Game::Player: RageCastPlayer,
{
    let Some(skill) = game.registered_skill(address) else {
        return ControlFlow::Break(StateCastExecutionOutcome::Rejected);
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return ControlFlow::Break(StateCastExecutionOutcome::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return ControlFlow::Break(StateCastExecutionOutcome::EndRejected);
    };
    let Some(source) = state_cast_participant(game, skill.lifecycle().user()) else {
        return ControlFlow::Break(StateCastExecutionOutcome::EndRejected);
    };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        // Смерть U завершает оба навыка с AfterUse, хотя усиления не создаются.
        game.update_registered_skill_visual(address, 2);
        return ControlFlow::Break(StateCastExecutionOutcome::EndCompleted);
    }
    if game.registered_skill(address).map(|skill| skill.execution_stage()) == Some(Some(SkillStage::Begin)) {
        if source.1.object_type == PLAYER_TYPE {
            let Some(rp) = game.find_player(source.1.id).map(|player| player.rp()) else {
                return ControlFlow::Break(StateCastExecutionOutcome::EndRejected);
            };
            let remaining = u32::from(rp).wrapping_sub(properties.query_property(RP_LOSS));
            if (remaining as i32) < 0 {
                fail_rp(game, address, source, &properties);
                return ControlFlow::Break(StateCastExecutionOutcome::EndRejected);
            }
            if let Some(player) = game.find_player_mut(source.1.id) {
                player.set_rp(remaining as u16);
            }
            game.publish_player_states(source.1.id);
        }
        if let Some(skill) = game.registered_skill_mut(address) {
            skill.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
        }
        game.update_registered_skill_visual(address, 0);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(address).map(|skill| skill.execution_stage()) != Some(Some(SkillStage::Check)) {
        return ControlFlow::Break(StateCastExecutionOutcome::Pending);
    }
    let delay = properties.query_property(DELAY);
    let Some(started) = game.registered_skill(address).map(|skill| skill.lifecycle().started_at_ms()) else {
        return ControlFlow::Break(StateCastExecutionOutcome::Rejected);
    };
    if started.wrapping_add(delay) > now() {
        return ControlFlow::Break(StateCastExecutionOutcome::Pending);
    }
    game.update_registered_skill_visual(address, 1);
    ControlFlow::Continue(RageSkillEffect { source, properties })
}

/// Свип конфликтующих состояний по живым позициям вектора (End + dtor
/// каждой найденной позиции; порядок — возрастание индекса, сдвига после
/// удаления нет, позиция читается заново).
pub fn remove_reached_conflict_states<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
) {
    let mut position = 0;
    loop {
        let Some(shape) = game.resolve_state_move_shape(region_id, holder) else { return; };
        if position >= shape.state_slot_count() { break; }
        if shape.state_at(position).is_some_and(|(_, state)| is_fury_conflicting_state_id(state.state_id())) {
            let _ = game.end_and_destroy_state_at(region_id, holder, position);
        }
        position += 1;
    }
}
