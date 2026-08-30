//! Одноклеточная область небесного огня `CTianhuoPhalanx` (`0x21A`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/tianhuophalanx.cpp`. Пока срок жизни не истёк, область
//! на каждом проходе просматривает свою клетку в исходном порядке региона.
//! После каждой допустимой атаки она помечается на удаление и немедленно
//! отправляет `0xBF504`; один проход всё ещё обрабатывает уже полученный
//! снимок клетки, поэтому пакет удаления может повториться. Формула делает ровно один
//! вызов legacy RNG до чтения боевого духа и свойств навыка. Поиск целей и
//! применение результата к независимым владельцам остаются у `CGame`.

use super::tianhuo::{TIANHUO_SKILL_ID, TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_SPRITE;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TianhuoPhalanxTick {
    Scan { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTianhuoPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    _element_modifier: i32,
}

pub(crate) fn tianhuo_targets(game: &CGame, region_id: i32, phalanx: &CTianhuoPhalanx) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (Ok(tile_x), Ok(tile_y)) = (phalanx.shape().get_tile_x(), phalanx.shape().get_tile_y()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut shapes = Vec::new();
    if region.get_shapes(tile_x, tile_y, area_width, area_height, game, &mut shapes).is_err() { return Vec::new() }
    shapes.into_iter().map(|shape| shape.identity).filter(|identity| {
        *identity != phalanx.shape().identity()
            && !(identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
            && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
    }).collect()
}

pub(crate) fn calculate_owned_tianhuo_attack(game: &mut CGame, phalanx: &CTianhuoPhalanx) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    if master.master_type != PLAYER_TYPE || master.master_id == 0 { return None }
    let player = game.find_player(master.master_id)?;
    let sprite = player.war_soul_goods(game.goods_factory())?.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1);
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let target_damage_factor = game.skill_base_properties(TIANHUO_SKILL_ID, phalanx.skill_level())?.query_property(TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY);
    Some(phalanx.calculate_attack(sprite, combat, occupation, attacker_level, target_damage_factor, &mut |maximum| game.skill_random_below(maximum)))
}

impl CTianhuoPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            skill_level,
            minimum_attack,
            maximum_attack,
            _element_modifier: element_modifier,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    pub(crate) fn tick(&mut self, now_ms: u32) -> TianhuoPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.finish();
            TianhuoPhalanxTick::Expired
        } else {
            TianhuoPhalanxTick::Scan {
                sampled_at_ms: now_ms,
            }
        }
    }

    pub(crate) fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape
            .add_to_byte_array(&mut payload, true)
            .then_some(payload)
    }

    pub(crate) fn calculate_attack(
        &self,
        sprite: i32,
        combat: PlayerCombatProperties,
        occupation: u8,
        attacker_level: u8,
        target_damage_factor: u32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) -> (AttackInformation, PlayerCombatProperties, u8, u8) {
        let delta = self.maximum_attack.wrapping_sub(self.minimum_attack);
        let width = delta.wrapping_abs().wrapping_add(1);
        let rolled_attack = random_below(width).wrapping_add(self.minimum_attack);
        let damage = (f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6
            + f64::from(rolled_attack))
        .round_ties_even() as i32;
        (
            AttackInformation {
                skill_id: TIANHUO_SKILL_ID,
                skill_level: self.skill_level as u8,
                attacker_type: self.master.master_type,
                attacker_id: self.master.master_id,
                attacker_team_id: self.master.master_team_id,
                attacker_faction_id: self.master.master_guild_id,
                attacker_union_id: self.master.master_union_id,
                hit_modifier: 100,
                damage_factor: 1.0,
                damage_modifier: 0,
                critical: false,
                blast_attack: false,
                full_miss: 0,
                damages: vec![AttackPower {
                    kind: AttackPowerType::Element,
                    hp_damage: damage.max(0),
                    mp_damage: 0,
                }],
            },
            combat,
            occupation,
            attacker_level,
        )
    }
}
