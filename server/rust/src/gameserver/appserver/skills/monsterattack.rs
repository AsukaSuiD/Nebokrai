//! Доставка рассчитанного удара монстра и допуск целей его навыков.
//! Источник: gameserver.exe + GameServer.pdb, CMoveShape::OnBeenAttacked
//! и CMonster::IsAttackAble.
//!
//! Формула, RNG, выбор целей и End остаются у навыка. Во время попадания
//! настоящий производный регион публикуется целиком: общий получатель
//! видит живые защиты, HP/MP, источник и выбранный AI цели. Возврат из
//! callback требует заново получить регион и объекты, а не применять
//! сохранённые до защиты снимки здоровья.
//! Клеточный обход использует общий resolver полного региона. Ограничения
//! типов принадлежат навыкам; снимок цели не принимает решение IsAttackAble.

use super::kernel::SkillStage;
use super::skillfactory::CSkillFactory;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeIdentity, ShapeView,
};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, RegionShapeResolver, ServerRegionOwner};
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

/// Возвращает живой упорядоченный снимок одной клетки для конкретного удара.
pub(crate) fn monster_attack_cell_candidates(
    game: &CGame,
    owner: &ServerRegionOwner,
    source_monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    let (area_width, area_height) = game.area_dimensions();
    let resolver = RegionShapeResolver { game, owner };
    let region = owner.base();
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
            matches!(identity.object_type, 400 | 500 | 600 | 1_100 | 1_200)
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
    pub(crate) monster_property: Option<MonsterProperties>,
}

pub(crate) fn resolve_owned_monster_attack_target(
    game: &CGame,
    owner: &ServerRegionOwner,
    identity: ShapeIdentity,
) -> Option<OwnedMonsterAttackTarget> {
    let region = owner.base();
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            let view = player.shape_view()?;
            Some(OwnedMonsterAttackTarget {
                shape: player.shape().clone(),
                view,
                player_properties: Some(player.combat_properties()),
                dead: player.is_dead(),
                god: player.is_god_mode(),
                city_dead: player.city_war_died_state(),
                monster_property: None,
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
                monster_property: Some(property),
            })
        }
        500 | 1_100 | 1_200 => {
            // CNpc наследует GetHP == 0: его IsDied истинен независимо от action.
            let (move_shape, dead) = if identity.object_type == 500 {
                (region.find_npc_by_id(identity.id)?.move_shape(), true)
            } else {
                let build = owner.stationary_build(identity)?;
                (build.move_shape(), build.hp() == 0)
            };
            Some(OwnedMonsterAttackTarget {
                shape: move_shape.shape().clone(),
                view: game.shape_view_in_owner(owner, identity)?,
                player_properties: None,
                dead,
                god: move_shape.is_god(),
                city_dead: false,
                monster_property: None,
            })
        }
        _ => None,
    }
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
