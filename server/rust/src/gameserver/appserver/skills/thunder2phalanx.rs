//! Однократный гром CLeimingPhalanx2, gameserver.exe/GameServer.pdb,
//! appserver/skills/thunder2phalanx.cpp. Все уровни имеют одну активную ячейку.
//! По истечении собственного срока область обходит
//! эту ячейку, поражает каждую найденную цель один раз и удаляется. Формула
//! урона делает один вызов legacy RNG лишь при наличии боевого духа и таблицы
//! свойств; их отсутствие оставляет заполненные метаданные без составляющих урона.
//! End помечает удаление только после всех попаданий. ReplaceAffectRegion
//! выключает совпавшую клетку; создание Leiming2 и общий AddObject его не
//! вызывают. Wire содержит Master и живое оставшееся время перед CShape.
//! Базовый урон использует общий с `CThunderPhalanx` расширенный порядок x87,
//! усечение в `i64` и чтение младших 32 бит.

use super::basemagic::{SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use super::thunder::thunder_base_damage;
use super::thunder2::{LEIMING2_SKILL_ID, LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_SPRITE;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Leiming2PhalanxTick {
    Pending,
    AttackAndExpire { sampled_at_ms: u32 },
}
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
    scope_active: bool,
}


pub(crate) fn calculate_owned_leiming2_attack(game: &mut CGame, phalanx: &CLeimingPhalanx2) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = LEIMING2_SKILL_ID;
    attack.skill_level = phalanx.skill_level as u8;
    attack.hit_modifier = 100;
    let goods = player.war_soul_goods(game.goods_factory());
    let properties = game.skill_base_properties(LEIMING2_SKILL_ID, phalanx.skill_level);
    if let (Some(goods), Some(properties)) = (goods, properties) {
        let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
        let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
        let sprite = goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1);
        let target_damage_factor = properties.query_property(LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY);
        let base_damage = thunder_base_damage(target_damage_factor, sprite);
        let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
        let damage = base_damage
            .wrapping_add(game.skill_random_below(width))
            .wrapping_add(minimum)
            .max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: damage,
            mp_damage: 0,
        });
    }
    Some((attack, combat, occupation, attacker_level))
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
            scope_active: true,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    pub(crate) fn set_center(&mut self, x: i32, y: i32) {
        self.shape.set_pos_xy_move_order(
            (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
        );
    }

    pub(crate) fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) {
        if self.shape.get_tile_x() == Ok(tile_x) && self.shape.get_tile_y() == Ok(tile_y) {
            self.scope_active = false;
        }
    }

    pub(crate) const fn scope_active(&self) -> bool { self.scope_active }

    pub(crate) fn tick(&mut self, now_ms: u32) -> Leiming2PhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            return Leiming2PhalanxTick::AttackAndExpire {
                sampled_at_ms: now_ms,
            };
        }
        Leiming2PhalanxTick::Pending
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, LEIMING2_SKILL_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}
