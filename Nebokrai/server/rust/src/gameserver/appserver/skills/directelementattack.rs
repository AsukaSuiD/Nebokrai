//! Прямое элементальное попадание Lightning, ChainLightning, Infernol, Seal и SoulMirror.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightning.cpp,
//! chainlightning.cpp, infernol.cpp, seal.cpp и soulmirror.cpp.
//! Допуск, смерть цели и список повторных попаданий принадлежат AI/Attack
//! владельца. Здесь сохраняются PK источника, свежая таблица Calculate и сырой
//! OnBeenAttacked; отсутствие таблицы оставляет исходный UNKNOWN/1 и пустой урон.
//! ChainLightning, Infernol и SoulMirror читают уровень цели и оружейный множитель;
//! только ChainLightning после контакта начисляет RP источнику. Lightning и Seal
//! сохраняют единичный множитель без этих чтений. Infernol и SoulMirror не читают
//! usage 20002: final modifier остаётся нулём.
//! EM игрока захватывается до таблицы. Его масштабирование и усечение выполнены
//! до MAX→MIN→RNG→свежего MIN→живого AddElement, затем CCH→RNG100.
//! Seal и SoulMirror завершают расчёт после AddElement: без CCH и второго RNG.
//! Расширенное вычисление EM и критического множителя
//! усекается к нулю; промежуточного округления EM к f32 нет. Vec владеет уроном.

use super::chainlightning::CHAIN_LIGHTNING_SKILL_ID;
use super::fightdefense::truncate_original;
use super::infernol::INFERNOL_SKILL_ID;
use super::lightning::LIGHTNING_SKILL_ID;
use super::seal::SEAL_SKILL_ID;
use super::soulmirror::SOUL_MIRROR_SKILL_ID;
use super::weaponattack::{SourceProperty, apply_weapon_critical, source_master, source_property};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectElementProfile {
    Lightning,
    ChainLightning,
    Infernol,
    Seal,
    SoulMirror,
}

impl DirectElementProfile {
    fn from_skill_id(skill_id: u32) -> Option<Self> {
        match skill_id {
            LIGHTNING_SKILL_ID => Some(Self::Lightning),
            CHAIN_LIGHTNING_SKILL_ID => Some(Self::ChainLightning),
            INFERNOL_SKILL_ID => Some(Self::Infernol),
            SEAL_SKILL_ID => Some(Self::Seal),
            SOUL_MIRROR_SKILL_ID => Some(Self::SoulMirror),
            _ => None,
        }
    }

    const fn uses_weapon_modifier(self) -> bool {
        matches!(self, Self::ChainLightning | Self::Infernol | Self::SoulMirror)
    }

    const fn reads_damage_modifier(self) -> bool {
        !matches!(self, Self::Infernol | Self::SoulMirror)
    }

    const fn increases_player_rp(self) -> bool {
        matches!(self, Self::ChainLightning)
    }

    const fn uses_critical(self) -> bool {
        !matches!(self, Self::Seal | Self::SoulMirror)
    }
}

fn calculate(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let element_modify = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().element_modify
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(profile) = DirectElementProfile::from_skill_id(skill.id()) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = if profile.reads_damage_modifier() {
        properties.query_property(20_002) as i32
    } else { 0 };
    if profile.uses_weapon_modifier() {
        let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
        attack.damage_factor = if source.1.object_type == 400 {
            let Some(player) = game.find_player(source.1.id) else { return; };
            let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
            player.weapon_modifier(game.goods_factory(), i32::from(level), divisor, minimum)
        } else { 1.0 };
    }
    attack.hit_modifier = properties.query_property(20_001) as i32;
    let modifier = properties.query_property(20_015);
    let bonus = truncate_original(f64::from(modifier) * f64::from(0.01_f32) * f64::from(element_modify));
    let maximum = properties.query_property(20_009) as i32;
    let minimum = properties.query_property(20_008) as i32;
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random = game.skill_random_below(width);
    let minimum = properties.query_property(20_008) as i32;
    let Some(element) = source_property(game, source, SourceProperty::Element) else { return; };
    let damage = (element as i32).wrapping_add(random).wrapping_add(minimum).wrapping_add(bonus).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
    if profile.uses_critical() {
        let Some(chance) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
        apply_weapon_critical(game, i32::from(chance as u16), attack);
    }
}

pub(super) fn apply_direct_element_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(source_shape) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if std::ptr::eq(source_shape, target_shape) { return; }
    let Some(master) = source_master(game, source) else { return; };
    let profile = game.registered_skill(instance)
        .and_then(|skill| DirectElementProfile::from_skill_id(skill.id()));
    let mut attack = AttackInformation::for_master(master);
    calculate(game, instance, source, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if profile.is_some_and(DirectElementProfile::increases_player_rp)
        && source.1.object_type == 400
    {
        game.increase_owned_player_rp(source.1.id, true, 0);
    }
}
