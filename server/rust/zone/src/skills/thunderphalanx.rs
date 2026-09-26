//! Периодический гром CThunderPhalanx — живая область боевого духа навыка
//! CThunder (0x21F; вызывающий путь — `skills/thunder.rs`).
//!
//! Отдельный flat-модуль: Initialize и AddToByteArray у фаланги ICF-фолднуты
//! с `CGodThunderPhalanx2`, но ctor/Calc/Attack/AI собственные — класс живёт
//! рядом с `godthunder`, а не внутри него; общая маска 7×7 берётся из
//! `godthunder` без дублирования (одна на все уровни — свойство сборки).
//!
//! Машинные quirks: нулевая frequency заменяется единицей конструктором;
//! 49-ячеечные окна Initialize — массив для клиента, не серверный выбор.
//! PARTIAL: полный маппинг 9 аргументов ctor не досмотрен.
//!
//! Швы: hub `battlefairyskill::BattleFairyGame` (игрок, WarSoul, таблица,
//! RNG); оружейный шов `ThunderPhalanxGame` — делегат старого пакета
//! `appserver/skills/thunderphalanx.rs`. Run-делегации (Attack-обход,
//! регистрация) остаются у прежнего владельца.
//!
//! Исходный владелец PDB: `appserver/skills/thunderphalanx.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#области-cthunderphalanx-cleimingphalanx2-ctianhuophalanx

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::values::CGuid;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
};
use crate::content::goods::GAP_BF_SPRITE;
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE};

use super::battlefairyskill::{BattleFairyGame, BattleFairyPlayer};
use super::godthunder::{ROUNDED_THUNDER_SCOPE, ROUNDED_THUNDER_SCOPE_SIDE};
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_prefix};
use super::thunder::{
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, THUNDER_SKILL_ID,
    THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY, thunder_base_damage,
};

/// Оружейные hub-фасады прежнего владельца `CGame` для Calc CThunderPhalanx;
/// открывают только прежние обращения, имена сохраняют исходную операцию.
/// Реализация — у делегата старого пакета `appserver/skills/thunderphalanx.rs`.
pub trait ThunderPhalanxGame: BattleFairyGame {
    /// Пара `(divisor, minimum)` оружейных факторов globe-установок.
    fn thunder_phalanx_weapon_damage_factors(&self) -> (f32, f32);

    /// Живой `weapon_modifier` игрока против уровня цели; чтение фабрики
    /// предметов остаётся у делегата.
    fn thunder_phalanx_weapon_modifier(
        &self,
        player_id: i32,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> Option<f32>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderPhalanxTick {
    Pending,
    Attack { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CThunderPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    frequency_ms: u32,
    minimum_attack: i32,
    maximum_attack: i32,
    _element_modifier: i32,
    _target_count: u32,
    _cch: i32,
    last_attack_ms: u32,
    cells: Vec<(i32, i32)>,
}

pub fn calculate_owned_thunder_attack<Game: ThunderPhalanxGame>(
    game: &mut Game, phalanx: &CThunderPhalanx, target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let (weapon_divisor, weapon_minimum) = game.thunder_phalanx_weapon_damage_factors();
    let weapon_damage_factor = game.thunder_phalanx_weapon_modifier(
        master.master_id, i32::from(target_level), weapon_divisor, weapon_minimum,
    )?;
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = THUNDER_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.damage_factor = weapon_damage_factor;
    attack.hit_modifier = 100;
    let constructor_width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let _ = game.skill_random_below(constructor_width);
    let sprite = game.battle_fairy_war_soul_addon(master.master_id, GAP_BF_SPRITE);
    let properties = game.skill_base_properties(THUNDER_SKILL_ID, phalanx.skill_level);
    if let (Some(sprite), Some(properties)) = (sprite, properties) {
        let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
        let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
        let target_damage_factor = properties.query_property(THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY);
        let base_damage = thunder_base_damage(target_damage_factor, sprite);
        let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
        let damage = base_damage.wrapping_add(game.skill_random_below(width))
            .wrapping_add(minimum).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
    }
    Some((attack, combat, occupation, attacker_level))
}

impl CThunderPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        frequency_ms: u32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
        target_count: u32,
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
            frequency_ms: frequency_ms.max(1),
            minimum_attack,
            maximum_attack,
            _element_modifier: element_modifier,
            _target_count: target_count,
            _cch: cch,
            last_attack_ms: 0,
            cells: Vec::new(),
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

    pub fn tick(
        &mut self, lifetime_now_ms: u32, now: &mut dyn FnMut() -> u32,
    ) -> ThunderPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ThunderPhalanxTick::Expired;
        }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now() {
            self.last_attack_ms = now();
            return ThunderPhalanxTick::Attack { sampled_at_ms: self.last_attack_ms };
        }
        ThunderPhalanxTick::Pending
    }

    pub fn scope_cells(&self) -> impl Iterator<Item = (i32, i32)> {
        let origin_x = self.shape.get_tile_x().unwrap_or(i32::MIN).wrapping_sub(3);
        let origin_y = self.shape.get_tile_y().unwrap_or(i32::MIN).wrapping_sub(3);
        (0..ROUNDED_THUNDER_SCOPE_SIDE).flat_map(move |x| {
            (0..ROUNDED_THUNDER_SCOPE_SIDE).filter_map(move |y| {
                (ROUNDED_THUNDER_SCOPE[(x * ROUNDED_THUNDER_SCOPE_SIDE + y) as usize] != 0)
                    .then_some((origin_x.wrapping_add(x), origin_y.wrapping_add(y)))
            })
        })
    }

    pub fn initialize(
        &mut self,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) {
        let attack_windows = self.lifetime_ms / self.frequency_ms;
        let total_cells = attack_windows.wrapping_mul(49) as usize;
        self.cells = vec![(0, 0); total_cells];
        let origin_x = self.shape.get_tile_x().unwrap_or(i32::MIN).wrapping_sub(3);
        let origin_y = self.shape.get_tile_y().unwrap_or(i32::MIN).wrapping_sub(3);
        for window in 0..attack_windows {
            for target in 0..self._target_count.min(49) {
                let (x, y) = loop {
                    let x = random_below(ROUNDED_THUNDER_SCOPE_SIDE);
                    let y = random_below(ROUNDED_THUNDER_SCOPE_SIDE);
                    let index = x.wrapping_add(ROUNDED_THUNDER_SCOPE_SIDE.wrapping_mul(y)) as usize;
                    if ROUNDED_THUNDER_SCOPE.get(index).copied().unwrap_or_default() != 0 {
                        break (x, y);
                    }
                };
                let index = window
                    .wrapping_mul(49)
                    .wrapping_add(target) as usize;
                if let Some(cell) = self.cells.get_mut(index) {
                    *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
                }
            }
        }
    }

    pub fn encode_client_snapshot(
        &self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let mut payload = encode_related_phalanx_prefix(
            THUNDER_SKILL_ID as i32, self.skill_level, self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, &mut now_milliseconds,
        );
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_u32(self.lifetime_ms);
            writer.write_u32(self.frequency_ms);
            let serialized_count =
                (self.lifetime_ms / self.frequency_ms).wrapping_mul(self._target_count);
            writer.write_u32(serialized_count);
            for &(x, y) in self.cells.iter().take(serialized_count as usize) {
                writer.write_i32(x);
                writer.write_i32(y);
            }
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }

}
