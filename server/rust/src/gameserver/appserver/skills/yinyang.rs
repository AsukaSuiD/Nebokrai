//! Призыв областей CYinYang и CYinYang2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/yinyang.cpp и
//! yinyang2.cpp. Общий Begin/Check/AI/visual/End находится в zonalcast.
//! Summon: Master(country0)/Player EM→свежая таблица→usage20015/FISTP→CCH WORD
//! →GetAddElementAttack; ключи и порядок дальнейших свойств — в zone/skills/yinyang.rs.
//! SetTile выполняется до свежего actual region captured U. Затем Add и
//! inherited encode/BF502 выполняются независимо от результата регистрации.
//! В Summon нет обхода прежних областей и вызова ReplaceAffectRegion.
//! Разные маски выбирает Zone по исходному skill ID.

use super::weaponattack::{SourceProperty, source_property};
use super::yinyangphalanx::new_yin_yang_phalanx;
use super::zonalcast::prepare_element_summon;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::YinYangSummonParameters;

pub(crate) use nebokrai_zone::skills::YIN_YANG_SKILL_ID;

pub(super) fn summon_yin_yang<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some((master, properties, scaled_element)) = prepare_element_summon(game, instance, source) else { return; };
    let Some(cch) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    let cch = i32::from(cch as u16);
    let Some(element) = source_property(game, source, SourceProperty::Element) else { return; };
    let element = (element as i32).wrapping_add(scaled_element);
    let Some(parameters) = YinYangSummonParameters::read(
        |property| properties.query_property(property),
        || game.registered_skill(instance).map(|skill| (skill.id(), i32::from(skill.level()))),
    ) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = new_yin_yang_phalanx(
        id, master, started, element, cch, parameters,
    );
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_masked_element_phalanx(region, phalanx, started, runtime, None);
}
