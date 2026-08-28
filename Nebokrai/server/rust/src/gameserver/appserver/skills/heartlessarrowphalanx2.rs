//! Региональная ловушка стрел `CHeartLessArrowPhalanx2/3` (`0xE5/0xE6`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/heartlessarrowphalanx2.cpp` и
//! `appserver/skills/heartlessarrowphalanx3.cpp`. Оба варианта различаются
//! только идентификатором навыка. Форма сохраняет исходный срок жизни,
//! сканирует одну клетку в региональном порядке и исчезает после первого
//! допустимого попадания. Урон читает текущие свойства игрока-владельца и
//! сохраняет ровно два вызова генератора: физический урон и критический шанс.

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
pub(crate) enum HeartlessArrowPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CHeartlessArrowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_id: u32,
    skill_level: i32,
    damage_factor: i32,
    critical_chance: i32,
}

impl CHeartlessArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля повторяют состояние исходной призванной формы")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_id: u32,
        skill_level: i32,
        damage_factor: i32,
        critical_chance: i32,
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
            skill_id,
            skill_level,
            damage_factor,
            critical_chance,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) fn tick(&mut self, now_ms: u32) -> HeartlessArrowPhalanxTick {
        if now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            HeartlessArrowPhalanxTick::Expired
        } else {
            HeartlessArrowPhalanxTick::Scan
        }
    }

    pub(crate) fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn accepts_candidate(&self, target: ShapeIdentity) -> bool {
        matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
            && target != self.shape.identity()
            && (target.object_type != self.master.master_type || target.id != self.master.master_id)
    }

    pub(crate) fn prepare_target(
        &self,
        game: &mut CGame,
        region_id: i32,
        target: ShapeIdentity,
        now_ms: u32,
    ) {
        super::heartlessarrow::apply_daub_poison(
            game,
            self.master.master_id,
            region_id,
            target,
            now_ms,
        );
    }
}

pub(crate) fn calculate_owned_heartless_arrow_attack(
    game: &mut CGame,
    phalanx: &CHeartlessArrowPhalanx,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master.master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let level = player.level();
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let delta = maximum.wrapping_sub(minimum);
    let width = if delta < 0 { delta.wrapping_neg() } else { delta }.wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(width));
    let mut attack = AttackInformation {
        skill_id: phalanx.skill_id,
        skill_level: phalanx.skill_level as u8,
        attacker_type: phalanx.master.master_type,
        attacker_id: phalanx.master.master_id,
        attacker_team_id: phalanx.master.master_team_id,
        attacker_faction_id: phalanx.master.master_guild_id,
        attacker_union_id: phalanx.master.master_union_id,
        hit_modifier: 0,
        damage_factor: phalanx.damage_factor as f32 * 0.01,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < phalanx.critical_chance {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * rate).round_ties_even() as i32;
        }
    }
    Some((attack, combat, occupation, level))
}
