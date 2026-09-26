//! Восстановление здоровья CWangsheng (0x221): Check/AI и числовое правило,
//! без создания WangshengState.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! `appserver/skills/wangsheng.cpp/.h` (конструктор ID и vtable VA
//! 0x0051D4E0–0x0051D514, AI через слот +0x90 VA 0x0051DBD0, участок лечения
//! VA 0x0051E01D–0x0051E043). Подтверждённый State owner 0x221 этим
//! навыком не создаётся; живые callbacks сохранённого CWangshengState
//! остаются hub-lifecycle прежнего `wangshengstate.rs`.
//!
//! Общий вход сохраняет зарегистрированный экземпляр и часы base Begin.
//! Check принимает исходного игрока, требует GetS и проверяет reuse; MP0
//! допускается без предмета, но ненулевая цена требует GetWarSoulGoods.
//! Первый AI разрешает свежего U, требует его region-link, но не проверяет S
//! или смерть.
//!
//! AI сначала списывает MP у equipment[10] через общий setter с reload
//! существующих fairy-проекций, затем повторно проверяет
//! GetWarSoulGoods. Отказ сохраняет списание и ожидание без отката.
//! Serialize не подавляет BF918 при false; после отправки CAN/visual0/condition
//! предшествуют абсолютной задержке. После visual1 читаются HP и величина
//! лечения: wrapping-сумма проходит SetHP и OnChangeStates. Единственный
//! собственный End(bool), отличный от внешнего End(int), выполняет координатор
//! (`skills/battlefairyskill.rs`).
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame`; доставка BF918 — точечный `send_battle_fairy_goods_update`
//! (якоря в шапке координатора).

use crate::content::CSkillBaseProperties;
use crate::content::goods::GAP_BF_MP;

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    execute_registered_battle_fairy_state, send_battle_fairy_goods_update,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};

pub const WANGSHENG_SKILL_ID: u32 = 0x221;
const TARGET_HP_GAIN: u32 = 31;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

/// Текущий HP читается до свойства; сумма передаётся setter-у без ограничения.
pub fn wangsheng_restored_health(
    current: u32, mut query_property: impl FnMut(u32) -> u32,
) -> u32 {
    current.wrapping_add(query_property(TARGET_HP_GAIN))
}

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(
        player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost),
    );
}

fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    if game.resolve_skill_sufferer(skill.lifecycle()).is_none() { return false; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now(runtime)) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(player_id, b"ZHGS0048");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) != 0 {
        let Some(current) = game.battle_fairy_war_soul_addon(player_id, GAP_BF_MP)
        else { return false; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(cost as i32) < 0 {
            fail_mana(game, instance, player_id, &properties);
            return false;
        }
    }
    true
}

pub fn execute_battle_fairy_wangsheng<Game: BattleFairyGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != WANGSHENG_SKILL_ID {
        return BattleFairySkillOutcome::Rejected;
    }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, runtime, now),
        |game, instance, runtime| run_ai(game, instance, runtime, now),
    )
}

fn run_ai<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let (region, user) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(region, user)
        .map(|shape| shape.shape().identity()).filter(|source| source.object_type == 400)
    else { return BattleFairySkillOutcome::Pending; };
    if game.find_player(source.id).is_none_or(|player| {
        !player.shape().is_assigned_to_server_region()
    }) {
        return BattleFairySkillOutcome::Rejected;
    }
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        let Some(current) = game.battle_fairy_equipment_addon(source.id, GAP_BF_MP)
        else { return BattleFairySkillOutcome::Pending; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, source.id, &properties);
            return BattleFairySkillOutcome::Rejected;
        }
        let Some(_stored) = game.set_battle_fairy_equipment_addon(source.id, GAP_BF_MP, remaining)
        else { return BattleFairySkillOutcome::Pending; };

        // Маркер проверяется после записи: неподходящий предмет теряет MP,
        // но condition не устанавливается, и следующий AI повторяет попытку.
        if let Some((ex_id, payload)) = game.battle_fairy_war_soul_payload(source.id) {
            send_battle_fairy_goods_update(game, source.id, ex_id, &payload);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return BattleFairySkillOutcome::Pending;
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if now(runtime) < started.wrapping_add(delay) {
        return BattleFairySkillOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    let Some(current) = game.find_player(source.id).map(|player| player.health()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    let restored = wangsheng_restored_health(current, |key| properties.query_property(key));
    if let Some(player) = game.find_player_mut(source.id) {
        player.set_health(restored);
    }
    let _ = game.publish_player_states(source.id);
    BattleFairySkillOutcome::Completed
}
