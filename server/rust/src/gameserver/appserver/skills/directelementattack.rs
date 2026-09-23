//! Прямое элементальное попадание молний, печатей и пошаговых снарядов.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightning.cpp,
//! chainlightning.cpp, infernol.cpp, seal.cpp, soulmirror.cpp,
//! energybolt.cpp, snakebolt.cpp и zombieclaw.cpp.
//! Допуск, смерть цели и список повторных попаданий принадлежат AI/Attack
//! владельца. Здесь остаются PK источника, живые чтения, свежая таблица
//! Calculate и сырой OnBeenAttacked; числовое правило находится в Zone.
//! Отсутствие таблицы оставляет исходный ID и пустой урон.
//! ChainLightning, Infernol и SoulMirror читают уровень цели и оружейный множитель;
//! только ChainLightning после контакта начисляет RP источнику. Lightning и Seal
//! сохраняют единичный множитель без этих чтений. Infernol и SoulMirror не читают
//! usage 20002: final modifier остаётся нулём.
//! EM игрока захватывается до таблицы. Порядок масштабирования, запросов
//! MAX→MIN→RNG→свежего MIN→живого AddElement задаёт Zone, затем CCH→RNG100.
//! Seal и SoulMirror завершают расчёт после AddElement: без CCH и второго RNG.
//! Критический множитель усекается к нулю; промежуточного округления EM к f32 нет.
//! Пошаговые снаряды после снимка PK расходуют души до Calculate; Energy/Snake
//! читают оружейный множитель Player после EM, но до таблицы; при её отсутствии
//! этот множитель всё равно сохраняется. Zombie его не читает.
//! Их единственный RNG предшествует усилению от душ в Zone, без критического хвоста и RP.

use super::chainlightning::CHAIN_LIGHTNING_SKILL_ID;
use super::infernol::INFERNOL_SKILL_ID;
use super::lightning::LIGHTNING_SKILL_ID;
use super::seal::SEAL_SKILL_ID;
use super::soulmirror::SOUL_MIRROR_SKILL_ID;
use super::soulcollectstate::consume_soul_collect_for_attack;
use super::weaponattack::{SourceProperty, apply_weapon_critical, source_master, source_property};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::{DirectElementLiveField, DirectElementProfile};

fn direct_element_profile(skill_id: u32) -> Option<DirectElementProfile> {
    match skill_id {
        LIGHTNING_SKILL_ID => Some(DirectElementProfile::Lightning),
        CHAIN_LIGHTNING_SKILL_ID => Some(DirectElementProfile::ChainLightning),
        INFERNOL_SKILL_ID => Some(DirectElementProfile::Infernol),
        SEAL_SKILL_ID => Some(DirectElementProfile::Seal),
        SOUL_MIRROR_SKILL_ID => Some(DirectElementProfile::SoulMirror),
        super::energybolt::ENERGY_BOLT_SKILL_ID | super::snakebolt::SNAKE_BOLT_SKILL_ID => Some(DirectElementProfile::PathWeapon),
        super::zombieclaw::ZOMBIE_CLAW_SKILL_ID => Some(DirectElementProfile::PathUnarmed),
        _ => None,
    }
}

fn calculate(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), souls: Option<i32>, attack: &mut AttackInformation,
) {
    let Some(profile) = game.registered_skill(instance)
        .and_then(|skill| direct_element_profile(skill.id())) else { return; };
    let element_modify = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().element_modify
    } else { 0 };
    if profile == DirectElementProfile::PathWeapon && source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
        let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
        attack.damage_factor = player.weapon_modifier(game.goods_factory(), i32::from(level), divisor, minimum);
    }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    profile.begin_calculation(attack, skill.id(), skill.level() as u8,
        |property| properties.query_property(property));
    if profile.uses_weapon_modifier() {
        let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
        attack.damage_factor = if source.1.object_type == 400 {
            let Some(player) = game.find_player(source.1.id) else { return; };
            let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
            player.weapon_modifier(game.goods_factory(), i32::from(level), divisor, minimum)
        } else { 1.0 };
    }
    if profile.roll_damage(attack, element_modify, souls,
        |property| properties.query_property(property),
        |field| match field {
            DirectElementLiveField::RandomBelow(width) => Some(game.skill_random_below(width)),
            DirectElementLiveField::AddElementAttack => source_property(game, source, SourceProperty::Element).map(|value| value as i32),
        },
    ).is_none() { return; }
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
        .and_then(|skill| direct_element_profile(skill.id()));
    let mut attack = AttackInformation::for_master(master);
    let souls = profile.filter(|profile| profile.consumes_souls())
        .map(|_| consume_soul_collect_for_attack(game, source));
    calculate(game, instance, source, target, souls, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if profile.is_some_and(DirectElementProfile::increases_player_rp)
        && source.1.object_type == 400
    {
        game.increase_owned_player_rp(source.1.id, true, 0);
    }
}
