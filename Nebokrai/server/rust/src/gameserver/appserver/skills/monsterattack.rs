//! Общая узкая ветвь применения уже рассчитанного удара монстра.
//!
//! Конкретный навык сохраняет формулу, RNG и выбор целей у своего владельца.
//! Здесь остаются общие `CFightDefense::PreDefense`, изменение цели, точные
//! пакеты ранения и смерти и семантические хвосты. Смерть игрока передаётся
//! наружу после возврата владельца региона; смерть любого монстра остаётся
//! в его `CBaseAI` как пассивный `Died` и завершается runtime-владельцем AI.

use super::fightdefense::{
    defend_monster_from_monster_base_attack, defend_player_from_monster_base_attack,
};
use super::knockoutstate::finish_blind_states_on_defense;
use crate::gameserver::appserver::ai::cityguardwithbow::retarget_city_bow_guard_after_hurt;
use crate::gameserver::appserver::ai::guardcountry::retarget_special_guard_after_hurt;
use crate::gameserver::appserver::ai::smartgladiator::apply_monster_hurt_response;
use crate::gameserver::appserver::ai::jiumai::retarget_jiumai_after_hurt;
use crate::gameserver::appserver::ai::vilcouguardwithbow::retarget_village_bow_guard_after_hurt;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::{MonsterCombatProperties, MonsterKillingAttack};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::{CPlayer, PlayerCombatProperties};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeIdentity, ShapeResolver, ShapeView,
};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, PlayerKillingBlow};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;

struct MonsterAttackShapeResolver<'a> {
    game: &'a CGame,
    region: &'a CServerRegion,
}

impl ShapeResolver for MonsterAttackShapeResolver<'_> {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        match identity.object_type {
            PLAYER_TYPE => self.game.find_player(identity.id)?.shape_view(),
            MONSTER_TYPE => {
                let monster = self.region.find_monster_by_id(identity.id)?;
                let property = self
                    .game
                    .find_monster_property_by_origin_name(monster.base_property_key()?)?;
                monster.shape_view(property)
            }
            _ => None,
        }
    }
}

/// Возвращает живой упорядоченный снимок одной клетки для конкретного удара.
pub(crate) fn monster_attack_cell_candidates(
    game: &CGame,
    region: &CServerRegion,
    source_monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    let (area_width, area_height) = game.area_dimensions();
    let resolver = MonsterAttackShapeResolver { game, region };
    let mut shapes = Vec::new();
    if region
        .get_shapes(
            tile_x,
            tile_y,
            area_width,
            area_height,
            &resolver,
            &mut shapes,
        )
        .is_err()
    {
        return Vec::new();
    }
    shapes
        .into_iter()
        .map(|shape| shape.identity)
        .filter(|identity| {
            matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                && !(identity.object_type == MONSTER_TYPE && identity.id == source_monster_id)
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MonsterAttackDeath {
    Player(PlayerKillingBlow),
}

#[derive(Clone, Debug)]
pub(crate) struct OwnedMonsterAttackTarget {
    pub(crate) shape: CShape,
    pub(crate) health: u32,
    pub(crate) mana: u32,
    pub(crate) war_soul_mana: Option<i32>,
    pub(crate) player_properties: Option<PlayerCombatProperties>,
    pub(crate) monster_properties: Option<MonsterCombatProperties>,
    pub(crate) dead: bool,
    pub(crate) god: bool,
    pub(crate) city_dead: bool,
    pub(crate) master: Option<MasterInfo>,
    pub(crate) monster_property: Option<MonsterProperties>,
    pub(crate) tamed: bool,
    pub(crate) carriage: bool,
}

pub(crate) fn resolve_owned_monster_attack_target(
    game: &CGame,
    region: &CServerRegion,
    identity: ShapeIdentity,
) -> Option<OwnedMonsterAttackTarget> {
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            (player.server_region_id() == Some(region.id)).then(|| OwnedMonsterAttackTarget {
                shape: player.shape().clone(),
                health: player.health(),
                mana: player.mana(),
                war_soul_mana: player.war_soul_mana(game.goods_factory()),
                player_properties: Some(player.combat_properties()),
                monster_properties: None,
                dead: player.is_dead(),
                god: player.is_god_mode(),
                city_dead: player.city_war_died_state(),
                master: None,
                monster_property: None,
                tamed: false,
                carriage: false,
            })
        }
        MONSTER_TYPE => {
            let monster = region.find_monster_by_id(identity.id)?;
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some(OwnedMonsterAttackTarget {
                shape: monster.move_shape().shape().clone(),
                health: monster.hit_points(),
                mana: 0,
                war_soul_mana: None,
                player_properties: None,
                monster_properties: Some(monster.combat_properties(&property)),
                dead: CMoveShape::is_died(monster.hit_points()),
                god: monster.move_shape().is_god(),
                city_dead: false,
                master: Some(monster.master_info()),
                monster_property: Some(property.clone()),
                tamed: monster.is_tamed(),
                carriage: monster.is_carriage(&property),
            })
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments, reason = "аргументы задают обе стороны исходного CMonster::IsAttackAble")]
pub(crate) fn monster_attackable_by_monster(
    game: &CGame,
    attacker_property: &MonsterProperties,
    attacker_tamed: bool,
    attacker_master: MasterInfo,
    target_property: &MonsterProperties,
    target_tamed: bool,
    target_master: MasterInfo,
    region_id: i32,
) -> bool {
    if target_property.kind == 5 {
        if !attacker_tamed {
            return attacker_property.kind != 5;
        }
        return (attacker_master.master_type != PLAYER_TYPE || attacker_master.master_id == 0)
            || game.find_player(attacker_master.master_id).is_some_and(|master| {
                master.is_badman(game.globe_setup().pk_count_per_kill())
            });
    }
    if target_tamed {
        let Some(target_player_id) = (target_master.master_type == PLAYER_TYPE
            && target_master.master_id != 0)
            .then_some(target_master.master_id)
        else {
            return true;
        };
        if attacker_tamed {
            if attacker_master.master_type != PLAYER_TYPE || attacker_master.master_id == 0 {
                return true;
            }
            if let Some((string_id, limit)) = game.player_base_attack_level_block(
                attacker_master.master_id,
                target_player_id,
            ) {
                game.send_base_attack_level_block(
                    attacker_master.master_id,
                    string_id,
                    limit,
                );
                return false;
            }
            return game.player_base_attackable(attacker_master.master_id, target_player_id);
        }
        return attacker_property.kind != 5
            || game.guard_monster_attackable(target_player_id, region_id, attacker_property);
    }
    attacker_tamed || attacker_property.kind == 5
}

/// Применяет исходную `CMonster::IsAttackAble` к уже разрешённой цели. Проверка
/// уровня хозяев питомцев сохраняет немедленное клиентское уведомление.
#[allow(clippy::too_many_arguments, reason = "аргументы задают владельца и разрешённую цель без параллельного снимка")]
pub(crate) fn owned_monster_attackable(
    game: &CGame,
    region_id: i32,
    attacker_property: &MonsterProperties,
    attacker_tamed: bool,
    attacker_master: MasterInfo,
    target_identity: ShapeIdentity,
    target: &OwnedMonsterAttackTarget,
) -> bool {
    if target_identity.object_type == PLAYER_TYPE {
        if attacker_tamed
            && attacker_master.master_type == PLAYER_TYPE
            && attacker_master.master_id != 0
        {
            if let Some((string_id, limit)) =
                game.player_base_attack_level_block(attacker_master.master_id, target_identity.id)
            {
                game.send_base_attack_level_block(attacker_master.master_id, string_id, limit);
                return false;
            }
            return game.player_base_attackable(attacker_master.master_id, target_identity.id);
        }
        return attacker_property.kind != 5
            || game.guard_monster_attackable(target_identity.id, region_id, attacker_property);
    }
    if target.carriage {
        return game.carriage_attackable_by_monster(
            attacker_property,
            attacker_tamed,
            attacker_master,
            target.master.unwrap_or_default(),
            region_id,
        );
    }
    target.monster_property.as_ref().is_some_and(|property| {
        monster_attackable_by_monster(
            game,
            attacker_property,
            attacker_tamed,
            attacker_master,
            property,
            target.tamed,
            target.master.unwrap_or_default(),
            region_id,
        )
    })
}

#[allow(clippy::too_many_arguments, reason = "поля сохраняют атомарный снимок цели исходного OnBeenAttacked")]
pub(crate) fn defend_owned_monster_attack(
    game: &mut CGame,
    target: ShapeIdentity,
    target_mana: u32,
    target_war_soul_mana: Option<i32>,
    target_player_properties: Option<PlayerCombatProperties>,
    target_monster_properties: Option<MonsterCombatProperties>,
    mut attack: AttackInformation,
) -> AttackInformation {
    let mut defense_shields = (target.object_type == PLAYER_TYPE)
        .then(|| game.find_player_mut(target.id))
        .flatten()
        .map(CPlayer::take_defense_shields)
        .unwrap_or_default();
    let pillar_damage_factor = game.find_player(target.id)
        .and_then(CPlayer::pillar_state).map(|state| state.damage_factor());
    let globe_setup = game.globe_setup().clone();
    let mut random = |maximum| game.skill_random_below(maximum);
    if let Some(properties) = target_player_properties {
        defend_player_from_monster_base_attack(
            &mut attack,
            properties,
            target_mana,
            target_war_soul_mana,
            &globe_setup,
            &mut random,
            &mut defense_shields,
            pillar_damage_factor,
        );
    } else if let Some(properties) = target_monster_properties {
        defend_monster_from_monster_base_attack(
            &mut attack,
            properties,
            &globe_setup,
            &mut random,
        );
    }
    if let Some(player) = game.find_player_mut(target.id) {
        player.restore_defense_shields(defense_shields);
    }
    attack
}

#[allow(clippy::too_many_arguments, reason = "поля сохраняют атомарный снимок цели исходного OnBeenAttacked")]
pub(crate) fn apply_owned_monster_attack_hit<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    attacker_master: MasterInfo,
    target: ShapeIdentity,
    target_shape: &CShape,
    target_health: u32,
    target_mana: u32,
    _target_master: Option<MasterInfo>,
    target_monster_property: Option<MonsterProperties>,
    target_tamed: bool,
    target_carriage: bool,
    attack: AttackInformation,
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    let (damage, mana_damage) = CGame::applied_attack_damage(&attack, target_health, target_mana);
    if attack.full_miss != 0 {
        let mut missed = CMessage::new(0x000b_f612);
        missed.add_byte(attack.full_miss);
        missed.add_long(target.object_type);
        missed.add_long(target.id);
        let _ = game.send_game_shape_around(region, target_shape, None, &missed);
        return;
    }
    if damage == 0 && mana_damage == 0 {
        return;
    }
    let current_health = target_health - damage;
    let (attacker_is_tamed, passive_attacker_is_owned_creature) = region
        .find_monster_by_id(monster_id)
        .and_then(|attacker| {
            let property = game
                .find_monster_property_by_origin_name(attacker.base_property_key()?)?;
            Some((
                attacker.is_tamed(),
                attacker.is_tamed() || attacker.is_carriage(property),
            ))
        })
        .unwrap_or((false, false));
    if target.object_type == PLAYER_TYPE {
        if let Some(player) = game.find_player_mut(target.id) {
            player.set_health(current_health);
            player.set_mana(target_mana - mana_damage);
            player
                .movement_shape_mut()
                .set_action(if current_health == 0 { 6 } else { 5 });
        }
    } else if let Some(monster) = region.find_monster_by_id_mut(target.id) {
        monster.set_hit_points(current_health);
        monster
            .move_shape_mut()
            .shape_mut()
            .set_action(if current_health == 0 { 6 } else { 5 });
        if current_health == 0 {
            monster.when_been_killed(now_ms);
            monster.set_killed_by(MonsterKillingAttack {
                attacker_type: MONSTER_TYPE,
                attacker_id: monster_id,
                skill_id: attack.skill_id,
                skill_level: attack.skill_level,
                critical: attack.critical,
                blast_attack: attack.blast_attack,
            });
        } else {
            let attacker = ShapeIdentity {
                object_type: MONSTER_TYPE,
                id: monster_id,
                ex_id: CGuid::GUID_INVALID,
            };
            if target_tamed {
                monster.when_pet_been_hurted_by(attacker, now_ms);
            } else if target_monster_property
                .as_ref()
                .is_some_and(|property| property.ai == 2)
            {
                // Реакция AI2 требует разрешить отдельного владельца атакующего
                // после освобождения изменяемого заимствования цели.
            } else if target_monster_property
                .as_ref()
                .is_some_and(|property| property.ai == 16)
            {
                // Поиск AI16 читает соседние категории после освобождения
                // изменяемого заимствования цели.
            } else if target_monster_property
                .as_ref()
                .is_some_and(|property| property.ai == 11)
            {
                // Поиск AI11 читает владельца города и соседние категории
                // после освобождения изменяемого заимствования цели.
            } else if target_monster_property
                .as_ref()
                .is_some_and(|property| property.ai == 0x65)
            {
                // AI101 разрешает атакующего и связывает близнеца после
                // освобождения изменяемого заимствования цели.
            } else if target_monster_property
                .as_ref()
                .is_some_and(|property| property.ai == 1)
            {
                monster.when_passive_gladiator_hurted_by(
                    attacker,
                    now_ms,
                    passive_attacker_is_owned_creature,
                );
            } else if target_monster_property
                .as_ref()
                .is_some_and(|property| matches!(property.ai, 8 | 13 | 14 | 20))
            {
                monster.when_been_hurted(now_ms);
            } else {
                monster.when_been_hurted_by(attacker, attacker_is_tamed, now_ms);
            }
        }
        if !target_tamed
            && !target_carriage
            && attacker_master.master_type == PLAYER_TYPE
            && attacker_master.master_id != 0
        {
            let protection_ms = game.globe_setup().attack_monster_protection_ms();
            let _ = monster.register_attacking_player(
                attacker_master.master_id,
                now_ms,
                protection_ms,
            );
        }
    }
    if current_health != 0
        && !target_tamed
        && target_monster_property
            .as_ref()
            .is_some_and(|property| property.ai == 2)
    {
        apply_monster_hurt_response(game, region, target.id, monster_id, now_ms);
    }
    if current_health != 0
        && !target_tamed
        && target_monster_property
            .as_ref()
            .is_some_and(|property| property.ai == 16)
        && let Some(property) = target_monster_property.as_ref()
    {
        retarget_village_bow_guard_after_hurt(game, region, target.id, property, now_ms);
    }
    if current_health != 0
        && !target_tamed
        && target_monster_property
            .as_ref()
            .is_some_and(|property| property.ai == 11)
        && let Some(property) = target_monster_property.as_ref()
    {
        retarget_city_bow_guard_after_hurt(game, region, target.id, property, now_ms);
    }
    if current_health != 0
        && !target_tamed
        && target_monster_property
            .as_ref()
            .is_some_and(|property| property.ai == 0x65)
    {
        let _ = retarget_jiumai_after_hurt(
            game,
            region,
            target.id,
            ShapeIdentity {
                object_type: MONSTER_TYPE,
                id: monster_id,
                ex_id: CGuid::GUID_INVALID,
            },
            now_ms,
        );
    }
    if current_health != 0
        && !target_tamed
        && target_monster_property
            .as_ref()
            .is_some_and(|property| matches!(property.ai, 8 | 13 | 14 | 20))
        && let Some(property) = target_monster_property.as_ref()
    {
        retarget_special_guard_after_hurt(game, region, target.id, property);
    }
    if current_health != 0 {
        let _ = finish_blind_states_on_defense(game, region, target, now_ms);
    }
    if current_health == 0 {
        let mut died = CMessage::new(0x000b_f60b);
        died.add_long(MONSTER_TYPE);
        died.add_long(monster_id);
        died.add_long(target.object_type);
        died.add_long(target.id);
        died.add_ulong(damage);
        died.base_mut().add_char(1);
        CGame::append_base_attack_tail(&mut died, &attack);
        let _ = game.send_game_shape_around(region, target_shape, None, &died);
        if target.object_type == PLAYER_TYPE {
            deaths.push(MonsterAttackDeath::Player(PlayerKillingBlow {
                victim_id: target.id,
                attacker_type: MONSTER_TYPE,
                attacker_id: monster_id,
                attacker_faction_id: 0,
            }));
        }
    } else {
        let mut hurt = CMessage::new(0x000b_f60a);
        hurt.add_long(MONSTER_TYPE);
        hurt.add_long(monster_id);
        hurt.add_long(target.object_type);
        hurt.add_long(target.id);
        CGame::append_hurt_damage_records(&mut hurt, damage, mana_damage);
        hurt.add_ulong(current_health);
        CGame::append_base_attack_tail(&mut hurt, &attack);
        let _ = game.send_game_shape_around(region, target_shape, None, &hurt);
        if target.object_type == PLAYER_TYPE {
            let pets = game
                .find_player(target.id)
                .map(|player| player.active_pets().to_vec())
                .unwrap_or_default();
            for pet in pets {
                if pet.object_type == MONSTER_TYPE
                    && let Some(monster) = region.find_monster_by_id_mut(pet.id)
                {
                    let _ = monster.retarget_passive_pet(ShapeIdentity {
                        object_type: MONSTER_TYPE,
                        id: monster_id,
                        ex_id: CGuid::GUID_INVALID,
                    });
                }
            }
            game.damage_player_armor(target.id, runtime);
        }
    }
}
