//! Снаряд смертельного удара боевого духа `CFatalBlowPhalanx`.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! исходный владелец `appserver/skills/fatalblowphalanx.cpp`;
//! форма, тики, клиентский снимок и формула перенесены буквально.
//! Снаряд хранит снимок владельца, цель, уровень и коэффициент
//! навыка. На первом AI-проходе после создания он пытается нанести физический
//! удар; отсутствие master либо отказ IsAttackAble оставляют его до следующего
//! прохода. Удаление следует после Attack, а строгий абсолютный wrapping-срок
//! имеет приоритет. При наличии боевого духа и таблицы свойств формула
//! выполняет один вызов `legacy MSVCRT RNG`; иначе сохраняется атака без
//! составляющих урона, но с уже записанными идентификатором и коэффициентом навыка.
//! Поиск цели, проверка `IsAttackAble`, защита и сетевые последствия остаются
//! у исполняющего владельца, которому требуется доступ к нескольким сущностям.
//! Атака боевого духа усекается через исходный `i64` с чтением младших 32 бит;
//! процентный damage factor сохраняется в `f32` только после x87-умножения.
//! Конструктор хранит CCH, но min/max формула получает только из живой таблицы.
//! Wire передаёт Master, а не цель снаряда; отказ AddObject не отменяет BF502.
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame`; sprite-чтение предмета — шов `battle_fairy_war_soul_addon`,
//! RNG — `skill_random_below` прежнего общего потока.

use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original_i64_low,
};
use crate::content::goods::GAP_BF_ATTACK;
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};

use super::battlefairyskill::{BattleFairyGame, BattleFairyPlayer};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};

pub const FATAL_BLOW_SKILL_ID: u32 = 0x21c;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FatalBlowPhalanxTick {
    Ready(ShapeIdentity),
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CFatalBlowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    damage_factor: i32,
    target: ShapeIdentity,
    _cch: i32,
}

pub fn calculate_owned_fatal_blow_attack<Game: BattleFairyGame>(
    game: &mut Game,
    phalanx: &CFatalBlowPhalanx,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = FATAL_BLOW_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_factor = (f64::from(phalanx.damage_factor) * f64::from(0.01_f32)) as f32;
    let war_soul_attack = game.battle_fairy_war_soul_addon(master.master_id, GAP_BF_ATTACK);
    let properties = game.skill_base_properties(FATAL_BLOW_SKILL_ID, phalanx.skill_level);
    if let (Some(fairy_attack_raw), Some(properties)) = (war_soul_attack, properties) {
        let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
        let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
        let fairy_attack = truncate_original_i64_low(f64::from(fairy_attack_raw) * 0.0001);
        let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
        let damage = fairy_attack
            .wrapping_add(game.skill_random_below(width))
            .wrapping_add(minimum)
            .max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Physical,
            hp_damage: damage,
            mp_damage: 0,
        });
    }
    Some((attack, combat, occupation, attacker_level))
}

impl CFatalBlowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        damage_factor: i32,
        cch: i32,
        target: ShapeIdentity,
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
            _cch: cch,
        }
    }

    pub const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub const fn master(&self) -> MasterInfo {
        self.master
    }

    pub const fn skill_level(&self) -> i32 {
        self.skill_level
    }

    pub fn set_center(&mut self, x: i32, y: i32) {
        self.shape.set_pos_xy_move_order(
            (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
        );
    }

    pub fn tick(&mut self, now_ms: u32) -> FatalBlowPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            FatalBlowPhalanxTick::Expired
        } else {
            FatalBlowPhalanxTick::Ready(self.target)
        }
    }

    pub fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            FATAL_BLOW_SKILL_ID as i32,
            self.skill_level,
            self.master.master_type,
            self.master.master_id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }
}

// UNKNOWN: восстановление снаряда из массива байтов пока не перенесено —
// `CFatalBlowPhalanx::DecordFromByteArray` (RVA `0x001ED890`,
// fatalblowphalanx.cpp:181, `bool __thiscall DecordFromByteArray(uchar*, long*, bool)`).
