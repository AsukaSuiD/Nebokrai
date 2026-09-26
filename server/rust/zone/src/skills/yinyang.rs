//! Параметры и маски областей CYinYang и CYinYang2.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/yinyang{,2}.cpp и yinyangphalanx{,2}.cpp/.h.
//! Опорные адреса Summon и конструкторов областей —
//! docs/reconstruction/gameserver-skills.md#zonalcast-скелет-областных-призывов.

pub const YIN_YANG_SKILL_ID: u32 = 0x139;
pub const YIN_YANG_2_SKILL_ID: u32 = 0x146;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;
const LIFETIME_PROPERTY: u32 = 30_001;
const FULL: [bool; 9] = [true; 9];
const SINGLE: [bool; 1] = [true];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct YinYangSummonParameters {
    pub skill_id: u32,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub lifetime_ms: u32,
}

impl YinYangSummonParameters {
    /// Текущий экземпляр читается после двух свойств атаки и до срока.
    pub fn read(
        mut query_property: impl FnMut(u32) -> u32,
        current_skill: impl FnOnce() -> Option<(u32, i32)>,
    ) -> Option<Self> {
        let maximum_attack = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum_attack = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let (skill_id, skill_level) = current_skill()?;
        let lifetime_ms = query_property(LIFETIME_PROPERTY);
        Some(Self { skill_id, skill_level, minimum_attack, maximum_attack, lifetime_ms })
    }
}

pub fn yin_yang_scope(skill_id: u32) -> (i32, i32, &'static [bool]) {
    if skill_id == YIN_YANG_2_SKILL_ID { (1, 1, &SINGLE) }
    else { (3, 3, &FULL) }
}

use crate::regions::ShapeIdentity;
use super::masked_area::{MaskedAreaPulse, MaskedElementPhalanx};
use super::zonalcast::{ZonalCastContact, ZonalCastMoveShape, prepare_element_summon};
use super::{ElementPhalanxAttack, ElementSummonLiveField};

/// Тело Summon CYinYang/CYinYang2: Master(country0)/Player EM→свежая
/// таблица→usage20015/FISTP (префикс `prepare_element_summon` hub-а) →
/// CCH WORD → GetAddElementAttack; ключи и порядок дальнейших свойств — в
/// `YinYangSummonParameters::read`. SetTile выполняется до свежего actual
/// region captured U; затем Add и inherited encode/BF502 выполняются
/// независимо от результата регистрации (швы `ZonalCastContact` в
/// `skills/zonalcast.rs`). Обхода прежних областей и вызова
/// ReplaceAffectRegion в Summon нет. Разные маски выбирает `yin_yang_scope`
/// по исходному skill ID.
pub fn summon_yin_yang<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    destination: (i32, i32),
    runtime: &mut Runtime,
    now_milliseconds: &mut dyn FnMut(&mut Runtime) -> u32,
)
where
    Game: ZonalCastContact<Runtime>,
{
    let Some((master, properties, scaled_element)) = prepare_element_summon(game, instance, source) else { return; };
    let Some(cch) = game.zonal_source_property(source, ElementSummonLiveField::CriticalChance) else { return; };
    let cch = i32::from(cch as u16);
    let Some(element) = game.zonal_source_property(source, ElementSummonLiveField::AddElementAttack) else { return; };
    let element = (element as i32).wrapping_add(scaled_element);
    let Some(parameters) = YinYangSummonParameters::read(
        |property| properties.query_property(property),
        || game.registered_skill(instance).map(|skill| (skill.id(), i32::from(skill.level()))),
    ) else { return; };
    let started = now_milliseconds(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = MaskedElementPhalanx::new(
        id, ElementPhalanxAttack {
            master, skill_id: parameters.skill_id, skill_level: parameters.skill_level,
            minimum: parameters.minimum_attack, maximum: parameters.maximum_attack,
            element, critical_chance: cch,
        },
        started, parameters.lifetime_ms, MaskedAreaPulse::Once,
        yin_yang_scope(parameters.skill_id),
    );
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(destination.0) + 0.5) as f32, (f64::from(destination.1) + 0.5) as f32,
    );
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_masked_element_phalanx(region, phalanx, started, runtime);
}
