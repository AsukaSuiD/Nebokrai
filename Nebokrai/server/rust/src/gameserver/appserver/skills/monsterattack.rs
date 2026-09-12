//! Доставка рассчитанного удара монстра и допуск целей его навыков.
//! Источник: gameserver.exe + GameServer.pdb, CMoveShape::OnBeenAttacked
//! и CMonster::IsAttackAble.
//!
//! Формула, RNG, выбор целей и End остаются у навыка. Во время попадания
//! настоящий производный регион публикуется целиком: общий получатель
//! видит живые защиты, HP/MP, источник и выбранный AI цели. Возврат из
//! callback требует заново получить регион и объекты, а не применять
//! сохранённые до защиты снимки здоровья.

use super::kernel::SkillStage;
use super::skillfactory::CSkillFactory;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeIdentity, ShapeResolver, ShapeView,
};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;

pub(crate) fn end_owned_monster_skill_without_reuse(
    region: &mut CServerRegion,
    monster_id: i32,
    skill_id: u32,
    factory: &CSkillFactory,
) -> bool {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false };
    if monster.base_attack_cast(skill_id, factory).is_none() {
        return false;
    }
    let _ = monster.finish_base_attack_cast_without_reuse(skill_id, factory);
    true
}

pub(crate) fn finish_owned_monster_attack_impact<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    skill_id: u32,
    factory: &CSkillFactory,
    runtime: &mut Runtime,
) {
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Calculate, SkillStage::Attack, factory);
        let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Attack, SkillStage::Apply, factory);
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast_with_clock(skill_id, factory, || runtime.now_milliseconds());
    }
}

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

#[derive(Clone, Debug)]
pub(crate) struct OwnedMonsterAttackTarget {
    pub(crate) shape: CShape,
    pub(crate) view: ShapeView,
    pub(crate) player_properties: Option<PlayerCombatProperties>,
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
            let view = player.shape_view()?;
            (player.server_region_id() == Some(region.id)).then(|| OwnedMonsterAttackTarget {
                shape: player.shape().clone(),
                view,
                player_properties: Some(player.combat_properties()),
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
            let view = monster.shape_view(&property)?;
            Some(OwnedMonsterAttackTarget {
                shape: monster.move_shape().shape().clone(),
                view,
                player_properties: None,
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
        return game.player_attackable_by_monster(
            target_player_id,
            region_id,
            attacker_property,
            attacker_tamed,
            attacker_master,
        );
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
        return game.player_attackable_by_monster(
            target_identity.id,
            region_id,
            attacker_property,
            attacker_tamed,
            attacker_master,
        );
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

pub(crate) fn apply_owned_monster_attack_hit<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    runtime: &mut Runtime,
    target: ShapeIdentity,
    attack: AttackInformation,
) {
    let Some(region_id) = owner.as_ref().map(ServerRegionOwner::region_id) else { return; };
    let master = MasterInfo {
        master_type: attack.attacker_type,
        master_id: attack.attacker_id,
        ..MasterInfo::default()
    };
    let _ = game.with_published_region(owner, |game| {
        game.apply_owned_skill_contact(master, target, region_id, attack, runtime);
    });
}
