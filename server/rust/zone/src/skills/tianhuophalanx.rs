//! Небесный огонь CTianhuoPhalanx — живая область боевого духа навыка
//! CTianhuo (0x21A; вызывающий путь — `skills/tianhuo.rs`).
//!
//! Машинные quirks: на каждом проходе область сканирует свою клетку в
//! исходном порядке региона; после допустимой атаки помечается на удаление и
//! немедленно шлёт `0xBF504`; один проход обрабатывает уже полученный снимок
//! клетки, поэтому пакет удаления может повториться. Формула — единственный
//! вызов legacy RNG при наличии предмета в слоте 10, x87-сумма до одного
//! усечения в i64 с чтением младших 32 бит. PARTIAL: маппинг 6 аргументов
//! ctor не досмотрен.
//!
//! Швы: hub `battlefairyskill::BattleFairyGame` (игрок, слот-10 equipment,
//! WarSoul, таблица, RNG); скан клетки, отправка `0xBF504` и свёртка старой
//! области остаются у прежнего владельца.
//!
//! Исходный владелец PDB: `appserver/skills/tianhuophalanx.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#области-cthunderphalanx-cleimingphalanx2-ctianhuophalanx

use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original_i64_low,
};
use crate::content::goods::GAP_BF_SPRITE;
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};

use super::battlefairyskill::{BattleFairyGame, BattleFairyPlayer};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use super::tianhuo::{TIANHUO_SKILL_ID, TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TianhuoPhalanxTick {
    Scan { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CTianhuoPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    _element_modifier: i32,
}


pub fn calculate_owned_tianhuo_attack<Game: BattleFairyGame>(game: &mut Game, phalanx: &CTianhuoPhalanx) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let _ = game.battle_fairy_equipment_addon(master.master_id, GAP_BF_SPRITE)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = TIANHUO_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.hit_modifier = 100;
    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let rolled_attack = game.skill_random_below(width).wrapping_add(phalanx.minimum_attack);
    let sprite = game.battle_fairy_war_soul_addon(master.master_id, GAP_BF_SPRITE);
    let properties = game.skill_base_properties(TIANHUO_SKILL_ID, phalanx.skill_level);
    let damage = if let (Some(sprite), Some(properties)) = (sprite, properties) {
        let target_damage_factor = properties.query_property(TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY);
        truncate_original_i64_low(
            f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6
                + f64::from(rolled_attack),
        ).max(0)
    } else {
        0
    };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Element,
        hp_damage: damage,
        mp_damage: 0,
    });
    Some((attack, combat, occupation, attacker_level))
}

impl CTianhuoPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub fn new(
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

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.skill_level }

    pub fn set_center(&mut self, x: i32, y: i32) {
        self.shape.set_pos_xy_move_order(
            (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
        );
    }

    pub fn tick(&mut self, now_ms: u32) -> TianhuoPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.finish();
            TianhuoPhalanxTick::Expired
        } else {
            TianhuoPhalanxTick::Scan {
                sampled_at_ms: now_ms,
            }
        }
    }

    pub fn finish(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, TIANHUO_SKILL_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}
