//! Однократная область грома `CLeimingPhalanx2` (`0x21B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunder2phalanx.cpp`. Все три подтверждённые маски и их
//! размеры равны одной активной ячейке. По истечении срока область обходит
//! эту ячейку, поражает каждую найденную цель один раз и удаляется. Формула
//! урона делает ровно один вызов legacy RNG. Поиск целей и применение атаки
//! к независимым владельцам остаются у `CGame`. Виртуальный
//! `ReplaceAffectRegion` имеет подтверждённую формулу взаимного вычитания
//! областей, но его реальный вызывающий владелец пока не достигнут, поэтому
//! этот неподключённый путь здесь не подменён придуманным вызовом.

use super::thunder2::LEIMING2_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Leiming2PhalanxTick {
    Pending,
    AttackAndExpire { sampled_at_ms: u32 },
}
// ============================================================================
// FUNCTION: CLeimingPhalanx2::ReplaceAffectRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\thunder2phalanx.cpp:187
// RVA: 0x001E4520
// ADDRESS: 005e4520
// PROTOTYPE: void __thiscall ReplaceAffectRegion(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CLeimingPhalanx2 {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    _element_modifier: i32,
    _cch: i32,
}

impl CLeimingPhalanx2 {
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
        cch: i32,
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
            _cch: cch,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    pub(crate) fn tick(&mut self, now_ms: u32) -> Leiming2PhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return Leiming2PhalanxTick::AttackAndExpire {
                sampled_at_ms: now_ms,
            };
        }
        Leiming2PhalanxTick::Pending
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now {
            0
        } else {
            let second_now = now_milliseconds();
            self.lifetime_ms
                .wrapping_sub(second_now)
                .wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(LEIMING2_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.identity().object_type);
            writer.write_i32(self.shape.identity().id);
            writer.write_u32(remained);
        }
        self.shape
            .encode_to_byte_array(&mut payload, true)
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
        let base_damage = (f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6)
            .round_ties_even() as i32;
        let delta = self.maximum_attack.wrapping_sub(self.minimum_attack);
        let width = delta.wrapping_abs().wrapping_add(1);
        let damage = base_damage
            .wrapping_add(random_below(width))
            .wrapping_add(self.minimum_attack)
            .max(0);
        (
            AttackInformation {
                skill_id: LEIMING2_SKILL_ID,
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
                    hp_damage: damage,
                    mp_damage: 0,
                }],
            },
            combat,
            occupation,
            attacker_level,
        )
    }
}
