//! Небесный огонь CTianhuoPhalanx, gameserver.exe/GameServer.pdb,
//! appserver/skills/tianhuophalanx.cpp. Пока собственный срок жизни не истёк, область
//! на каждом проходе просматривает свою клетку в исходном порядке региона.
//! После каждой допустимой атаки она помечается на удаление и немедленно
//! отправляет `0xBF504`; один проход всё ещё обрабатывает уже полученный
//! снимок клетки, поэтому пакет удаления может повториться. При наличии предмета
//! в слоте 10 формула делает один вызов legacy RNG до повторного чтения боевого
//! духа и свойств навыка; поздний отказ сохраняет нулевую запись урона. Поиск целей и
//! применение результата к независимым владельцам остаются у `CGame`.
//! Слагаемое боевого духа и случайная база складываются в x87 до единственного
//! усечения в `i64`, после которого читаются младшие 32 бита.
//! Wire содержит ID/level/Master и живое оставшееся время перед CShape;
//! совпавшая старая область завершается до регистрации новой.

use super::tianhuo::{TIANHUO_SKILL_ID, TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY};
use super::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_SPRITE;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

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


pub(crate) fn calculate_owned_tianhuo_attack(game: &mut CGame, phalanx: &CTianhuoPhalanx) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let player = game.find_player(master.master_id)?;
    let equipment = player.equipment().get_goods(10)?;
    let _ = equipment.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1);
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
    let goods = game.find_player(master.master_id)
        .and_then(|player| player.war_soul_goods(game.goods_factory()));
    let properties = game.skill_base_properties(TIANHUO_SKILL_ID, phalanx.skill_level);
    let damage = if let (Some(goods), Some(properties)) = (goods, properties) {
        let sprite = goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1);
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

    pub(crate) fn set_center(&mut self, x: i32, y: i32) {
        self.shape.set_pos_xy_move_order(
            (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
        );
    }

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

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, TIANHUO_SKILL_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}
