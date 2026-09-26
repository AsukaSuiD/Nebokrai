//! Передача ресурсов CHuoxieshu/CLingzhishu: Check/AI.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! `appserver/skills/{huoxieshu,lingzhishu}.cpp`; тела перенесены буквально.
//!
//! Общий registered-вход сохраняет base Begin, visual loop1 и исходные часы;
//! Check требует CPlayer и ненулевой GetS, но не предмет или регион. Нулевая
//! цена допускается без чтения ресурса. Health оставляет одно HP, Mana в
//! Check допускает точную цену, а в AI также требует остаток не меньше одного.
//! Разность проверяется как signed DWORD, с исходным wrapping.
//!
//! AI больше не читает S и не проверяет смерть. Отсутствие CPlayer либо
//! equipment[10] оставляет ожидание; NULL региона U и нехватка ресурса дают
//! End(0). Расход → OnChangeStates → CAN → visual0 → condition; затем живой
//! condition и абсолютный start+delay. Отдельного начального clock нет.
//! Оба AI при нехватке используют ZHGS0052 с ценой без масштабирования,
//! даже Health; его Check использует ZHGS0054 с ценой плюс один.
//!
//! По сроку visual1 предшествует свежему GetWarSoulGoods. При отсутствии
//! предмета AI повторит этот visual в следующем такте без повторного расхода.
//! Восстановление читает current → поздний gain из таблицы начала AI → max,
//! при превышении максимума повторяет GetMax, затем пишет один раз.
//! Общий setter перезагружает уже существующие fairy-проекции до Serialize.
//! BF918 отправляется и при отказе Serialize — точечно, решение C
//! (якоря `0x501BDE..0x501C61`/`0x51F249`, шапка координатора
//! `skills/battlefairyskill.rs`). Собственный End(bool) с visual3 принадлежит
//! координатору, не дублируется внешним End(int).
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame`; Health/Mana игрока — фасад `BattleFairyPlayer`; доступ к
//! предмету — швы war-soul (marker-допуск) и equipment слота 10.

use crate::content::CSkillBaseProperties;
use crate::content::goods::{GAP_BF_HP, GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP};

use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    execute_registered_battle_fairy_state, send_battle_fairy_goods_update,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};

pub const HUOXIESHU_SKILL_ID: u32 = 0x222;
pub const LINGZHISHU_SKILL_ID: u32 = 0x223;

const SKILL_USAGE_USER_HP_LOSE: u32 = 1;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_HP_GAIN: u32 = 31;
const SKILL_USAGE_TARGET_MP_GAIN: u32 = 32;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyTransferKind {
    Health,
    Mana,
}

impl BattleFairyTransferKind {
    pub const fn skill_id(self) -> u32 {
        match self {
            Self::Health => HUOXIESHU_SKILL_ID,
            Self::Mana => LINGZHISHU_SKILL_ID,
        }
    }

    const fn cost_usage(self) -> u32 {
        match self {
            Self::Health => SKILL_USAGE_USER_HP_LOSE,
            Self::Mana => SKILL_USAGE_USER_MP_LOSE,
        }
    }

    const fn gain_usage(self) -> u32 {
        match self {
            Self::Health => SKILL_USAGE_TARGET_HP_GAIN,
            Self::Mana => SKILL_USAGE_TARGET_MP_GAIN,
        }
    }

    const fn target_properties(self) -> (i32, i32) {
        match self {
            Self::Health => (GAP_BF_HP, GAP_BF_MAX_HP),
            Self::Mana => (GAP_BF_MP, GAP_BF_MAX_MP),
        }
    }

    const fn failure_action(self) -> u32 {
        match self {
            Self::Health => 6,
            Self::Mana => 7,
        }
    }

    /// Health вызывает QueryProperty до виртуального GetHP; Mana читает
    /// поле MP до QueryProperty. Этот порядок одинаков в Check и AI.
    fn cost_and_current<Game: BattleFairyGame>(
        self, game: &Game, player_id: i32, properties: &CSkillBaseProperties,
    ) -> Option<(u32, u32)> {
        Some(match self {
            Self::Health => {
                let cost = properties.query_property(self.cost_usage());
                (cost, game.find_player(player_id)?.health())
            }
            Self::Mana => {
                let current = game.find_player(player_id)?.mana();
                (properties.query_property(self.cost_usage()), current)
            }
        })
    }

    fn deduct(self, player: &mut impl super::battlefairyskill::BattleFairyPlayer, remaining: u32) {
        match self {
            Self::Health => player.set_health(remaining),
            Self::Mana => player.set_mana(remaining),
        }
    }
}

fn fail_resource<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties, kind: BattleFairyTransferKind, checking: bool,
) {
    game.update_registered_skill_visual(instance, kind.failure_action());
    let cost = properties.query_property(kind.cost_usage());
    let (text, amount) = if checking && kind == BattleFairyTransferKind::Health {
        (b"ZHGS0054".as_slice(), cost.wrapping_add(1))
    } else {
        (b"ZHGS0052".as_slice(), cost)
    };
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    kind: BattleFairyTransferKind,
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
    if properties.query_property(kind.cost_usage()) != 0 {
        let Some((cost, current)) = kind.cost_and_current(game, player_id, &properties) else {
            return false;
        };
        let reserve = u32::from(kind == BattleFairyTransferKind::Health);
        if (current.wrapping_sub(cost).wrapping_sub(reserve) as i32) < 0 {
            fail_resource(game, instance, player_id, &properties, kind, true);
            return false;
        }
    }
    true
}

/// visual1 → свежий GetWarSoulGoods: current → поздний gain таблицы → max с
/// повторным чтением при превышении → единственная запись общим setter-ом.
fn restore_goods_resource<Game: BattleFairyGame>(
    game: &mut Game, player_id: i32, kind: BattleFairyTransferKind,
    properties: &CSkillBaseProperties,
) -> Option<()> {
    let (property, maximum_property) = kind.target_properties();
    let current = game.battle_fairy_war_soul_addon(player_id, property)? as u32;
    let gain = properties.query_property(kind.gain_usage());
    let mut restored = current.wrapping_add(gain);
    let maximum = game.battle_fairy_war_soul_addon(player_id, maximum_property)? as u32;
    if maximum < restored {
        restored = game.battle_fairy_war_soul_addon(player_id, maximum_property)? as u32;
    }
    let _ = game.set_battle_fairy_equipment_addon(player_id, property, restored as i32)?;
    Some(())
}

pub fn execute_battle_fairy_transfer<Game: BattleFairyGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, kind: BattleFairyTransferKind,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != kind.skill_id() {
        return BattleFairySkillOutcome::Rejected;
    }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, kind, runtime, now),
        |game, instance, runtime| run_ai(game, instance, kind, runtime, now),
    )
}

fn run_ai<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, kind: BattleFairyTransferKind,
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
    let Some(user) = game.resolve_state_move_shape(region, user)
        .map(|shape| shape.shape().identity()).filter(|user| user.object_type == 400)
    else { return BattleFairySkillOutcome::Pending; };
    let Some(player) = game.find_player(user.id) else {
        return BattleFairySkillOutcome::Pending;
    };
    if !player.shape().is_assigned_to_server_region() {
        return BattleFairySkillOutcome::Rejected;
    }
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if !game.battle_fairy_equipment_present(user.id) {
            return BattleFairySkillOutcome::Pending;
        }
        let Some((cost, current)) = kind.cost_and_current(game, user.id, &properties) else {
            return BattleFairySkillOutcome::Pending;
        };
        let remaining = current.wrapping_sub(cost);
        if (remaining.wrapping_sub(1) as i32) < 0 {
            fail_resource(game, instance, user.id, &properties, kind, false);
            return BattleFairySkillOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(user.id) { kind.deduct(player, remaining); }
        let _ = game.publish_player_states(user.id);
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
    if restore_goods_resource(game, user.id, kind, &properties).is_none() {
        return BattleFairySkillOutcome::Pending;
    }
    if let Some((ex_id, payload)) = game.battle_fairy_equipment_payload(user.id) {
        send_battle_fairy_goods_update(game, user.id, ex_id, &payload);
    }
    BattleFairySkillOutcome::Completed
}
