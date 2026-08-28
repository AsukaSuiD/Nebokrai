//! Снаряд смертельного удара боевого духа `CFatalBlowPhalanx`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fatalblowphalanx.cpp`. Снаряд хранит снимок владельца,
//! цель, уровень и коэффициент навыка. На первом AI-проходе после создания он
//! рассчитывает один физический удар и удаляется; до этого проверка срока жизни
//! имеет приоритет. Формула выполняет ровно один вызов `legacy MSVCRT RNG`.
//! Поиск цели, проверка `IsAttackAble`, защита и сетевые последствия остаются
//! у исполняющего владельца, которому требуется доступ к нескольким сущностям.

use super::fatalblow::FATAL_BLOW_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FatalBlowPhalanxTick {
    Ready(ShapeIdentity),
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFatalBlowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    damage_factor: i32,
    target: ShapeIdentity,
    minimum_attack: i32,
    maximum_attack: i32,
}

impl CFatalBlowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        damage_factor: i32,
        target: ShapeIdentity,
        minimum_attack: i32,
        maximum_attack: i32,
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
            damage_factor,
            target,
            minimum_attack,
            maximum_attack,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub(crate) const fn master(&self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn skill_level(&self) -> i32 {
        self.skill_level
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> FatalBlowPhalanxTick {
        if now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            FatalBlowPhalanxTick::Expired
        } else {
            FatalBlowPhalanxTick::Ready(self.target)
        }
    }

    pub(crate) fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
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
            writer.write_i32(FATAL_BLOW_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.target.object_type);
            writer.write_i32(self.target.id);
            writer.write_u32(remained);
        }
        self.shape
            .encode_to_byte_array(&mut payload, true)
            .then_some(payload)
    }

    pub(crate) fn calculate_attack(
        &self,
        battle_fairy_attack: i32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) -> AttackInformation {
        let delta = self.maximum_attack.wrapping_sub(self.minimum_attack);
        let width = if delta < 0 {
            delta.wrapping_neg()
        } else {
            delta
        }
        .wrapping_add(1);
        let damage = battle_fairy_attack
            .wrapping_add(random_below(width))
            .wrapping_add(self.minimum_attack)
            .max(0);
        AttackInformation {
            skill_id: FATAL_BLOW_SKILL_ID,
            skill_level: self.skill_level as u8,
            attacker_type: self.master.master_type,
            attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id,
            attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id,
            hit_modifier: 0,
            damage_factor: self.damage_factor as f32 * 0.01,
            damage_modifier: 0,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: damage,
                mp_damage: 0,
            }],
        }
    }
}


// Восстановление снаряда из массива байтов пока не достигнуто и сохранено RAW.
// ============================================================================
// FUNCTION: CFatalBlowPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fatalblowphalanx.cpp:181
// RVA: 0x001ED890
// ADDRESS: 005ed890
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
