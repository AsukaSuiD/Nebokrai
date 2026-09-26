//! Снаряд базовой атаки боевой феи `CBFBaseAttackPhalanx` GameServer.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! исходный владелец `appserver/skills/battlefairybasemagicphalanx.cpp`.
//! Прежний переходный владелец — `src/gameserver/appserver/skills/
//! battlefairybasemagicphalanx.rs`; форма, тики, клиентский снимок и формула
//! перенесены буквально порцией №6b «BF-ядро». Жизненный цикл и два
//! раздельных чтения часов совпадают с другими `CSummonShape`, но формула
//! использует `GAP_BF_SPRITE`, единичный коэффициент оружия и реальную
//! критическую ставку игрока. Формула, wrapping и два исходных вызова RNG
//! принадлежат этому owner-у; `CGame` передаёт снимок владельца и применяет
//! рассчитанную атаку к независимому владельцу цели.
//! Sprite-scale усекается через исходный 64-битный `fistp`, а критический
//! множитель — к `int`; оба преобразования используют truncation к нулю.
//! Exact точки `0x005E2698..0x005E26DC` и `0x005E27BC..0x005E27E6`
//! сохраняют x87-произведения до соответствующих `FISTP`.
//! Срок жизни и время попадания сравниваются как абсолютные wrapping-суммы;
//! общий End помечает удаление после попытки Attack, не до защиты цели.
//! Клиентский снимок передаёт Master, а не отдельную цель снаряда.
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame`; sprite предмета — шов `battle_fairy_war_soul_addon`,
//! globe-границы — `base_combat_scales`, RNG — `skill_random_below`.

use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original, truncate_original_i64_low,
};
use crate::content::goods::GAP_BF_SPRITE;
use crate::regions::ShapeIdentity;
use crate::regions::shape::CShape;

use super::battlefairyskill::{BattleFairyGame, BattleFairyPlayer};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};

pub const BATTLE_FAIRY_BASE_MAGIC_SKILL_ID: u32 = 0x224;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyPhalanxTick {
    Pending,
    Attack {
        target: ShapeIdentity,
        sampled_at_ms: u32,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CBattleFairyBaseMagicPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl CBattleFairyBaseMagicPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору CBFBaseAttackPhalanx")]
    pub fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
        attack_delay_ms: u32,
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
            minimum_attack,
            maximum_attack,
            element_modifier,
            attack_delay_ms,
            target,
        }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.skill_level }
    pub const fn minimum_attack(&self) -> i32 { self.minimum_attack }
    pub const fn maximum_attack(&self) -> i32 { self.maximum_attack }
    pub const fn element_modifier(&self) -> i32 { self.element_modifier }

    pub fn set_center(&mut self, x: i32, y: i32) {
        self.shape.set_pos_xy_move_order(
            (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
        );
    }

    /// Точный клиентский `AddToByteArray` совпадает на уровне одного адреса
    /// машинного кода с базовой магией и снарядом базовой стрельбы.
    pub fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            BATTLE_FAIRY_BASE_MAGIC_SKILL_ID as i32,
            self.skill_level,
            self.master.master_type,
            self.master.master_id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }

    pub fn tick(
        &mut self,
        lifetime_now_ms: u32,
        get_attack_now_ms: impl FnOnce() -> u32,
    ) -> BattleFairyPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms {
            return BattleFairyPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if self.started_at_ms.wrapping_add(self.attack_delay_ms) < attack_now_ms {
            return BattleFairyPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        BattleFairyPhalanxTick::Pending
    }
}

pub fn calculate_owned_battle_fairy_base_magic_attack<Game: BattleFairyGame>(
    game: &mut Game,
    phalanx: &CBattleFairyBaseMagicPhalanx,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    if master.master_id == 0 {
        return None;
    }
    let player = game.find_player(master.master_id)?;
    let sprite_raw = game.battle_fairy_war_soul_addon(master.master_id, GAP_BF_SPRITE)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let sprite = truncate_original_i64_low(f64::from(sprite_raw) * 0.0001);
    let combat_scales = game.base_combat_scales();
    calculate_battle_fairy_base_magic_attack(
        phalanx,
        combat,
        occupation,
        attacker_level,
        sprite,
        combat_scales,
        |maximum| game.skill_random_below(maximum),
    )
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют входы исходной формулы")]
pub fn calculate_battle_fairy_base_magic_attack(
    phalanx: &CBattleFairyBaseMagicPhalanx,
    mut combat: PlayerCombatProperties,
    occupation: u8,
    attacker_level: u8,
    sprite: i32,
    combat_scales: [f32; 5],
    mut random_below: impl FnMut(i32) -> i32,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    if master.master_id == 0 {
        return None;
    }
    let width_delta = phalanx
        .maximum_attack()
        .wrapping_sub(phalanx.minimum_attack());
    let width = if width_delta < 0 {
        width_delta.wrapping_neg()
    } else {
        width_delta
    }
    .wrapping_add(1);
    let random_damage = random_below(width);
    let element_damage = phalanx
        .element_modifier()
        .wrapping_mul(sprite)
        .wrapping_div(100)
        .wrapping_add(random_damage)
        .wrapping_add(phalanx.minimum_attack())
        .max(0);
    let mut attack = AttackInformation {
        skill_id: BATTLE_FAIRY_BASE_MAGIC_SKILL_ID,
        skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type,
        attacker_id: master.master_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: 100,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: element_damage,
            mp_damage: 0,
        }],
    };
    if random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = combat.critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(critical_rate),
            );
        }
    }
    let [blast_attack, blast_defense, element_blast_attack, element_blast_defense, full_miss] =
        combat_scales;
    if combat.blast_attack_scale() < 1.0 {
        combat.blast_attack_scale_bits = blast_attack.max(1.0).to_bits();
    }
    if combat.blast_defense_scale() < 0.01 {
        combat.blast_defense_scale_bits = blast_defense.max(0.01).to_bits();
    }
    if combat.element_blast_attack_scale() < 1.0 {
        combat.element_blast_attack_scale_bits = element_blast_attack.max(1.0).to_bits();
    }
    if combat.element_blast_defense_scale() < 0.01 {
        combat.element_blast_defense_scale_bits = element_blast_defense.max(0.01).to_bits();
    }
    if combat.full_miss_scale() < 0.01 {
        combat.full_miss_scale_bits = full_miss.max(0.01).to_bits();
    }
    Some((attack, combat, occupation, attacker_level))
}
