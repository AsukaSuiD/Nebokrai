//! Региональная ловушка стрел `CHeartLessArrowPhalanx2/3` (`0xE5/0xE6`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/heartlessarrowphalanx2.cpp` и
//! `appserver/skills/heartlessarrowphalanx3.cpp`. Оба варианта различаются
//! только идентификатором навыка. Форма сохраняет исходный срок жизни,
//! сканирует одну клетку в региональном порядке и исчезает после первого
//! допустимого попадания. Урон читает текущие свойства игрока-владельца и
//! сохраняет ровно два вызова генератора: физический урон и критический шанс.
//! Процентный damage factor сохраняется в `f32` после расширенного x87-
//! умножения, а критический урон усекается к нулю при записи в `i32`.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
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

    /// Точный клиентский `AddToByteArray` обоих вариантов читает `skill_id`,
    /// `skill_level` и сохранённые `master_type/master_id`; `damage_factor` и
    /// `critical_chance` остаются только серверными параметрами расчёта атаки.
    pub(crate) fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            self.skill_id as i32,
            self.skill_level,
            self.master.master_type,
            self.master.master_id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }

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
        now: &mut dyn FnMut() -> u32,
    ) {
        super::heartlessarrow::apply_daub_poison(
            game,
            self.master.master_id,
            region_id,
            target,
            now,
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
        damage_factor: (f64::from(phalanx.damage_factor) * f64::from(0.01_f32)) as f32,
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
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(rate),
            );
        }
    }
    Some((attack, combat, occupation, level))
}
