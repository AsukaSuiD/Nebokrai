//! Параметры и установка постоянных состояний CWuXingMetal/Wood/Water/Fire/Earth.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/wuxing*.cpp.
//! Общий immediatestate отвечает за Begin/AI/End. WuXing собирает весь набор
//! свойств до GetU/GetS и player-gate; Water сохраняет полный signed MAX_HP,
//! остальные сужают его до short, только Metal читает MAX_MP.
//! После player-gate новый объект получает первичный Begin до поиска первого
//! старого ID. Общая установка сохраняет прежнюю позицию, исполняет End и
//! destructor свежего остатка, затем UpdateProperty. RestoreHpMp следует
//! отдельно и не зависит от результата UpdateProperty. Успешный AI вызывает
//! End1 даже при отказе нового Begin; отсутствие U/S или иной тип даёт End0.

use super::immediatestateinstallation::replace_immediate_state;
use super::skillbaseproperties::CSkillBaseProperties;
use super::wuxingearth::WUXING_EARTH_SKILL_ID;
use super::wuxingfire::WUXING_FIRE_SKILL_ID;
use super::wuxingmetal::WUXING_METAL_SKILL_ID;
use super::wuxingstate::{kind_for_skill_id, WuXingKind, WuXingState, WuXingStateParameters};
use super::wuxingwater::WUXING_WATER_SKILL_ID;
use super::wuxingwood::WUXING_WOOD_SKILL_ID;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const fn is_wuxing_skill(skill_id: u32) -> bool {
    matches!(skill_id, WUXING_METAL_SKILL_ID | WUXING_WOOD_SKILL_ID
        | WUXING_WATER_SKILL_ID | WUXING_FIRE_SKILL_ID | WUXING_EARTH_SKILL_ID)
}

/// Набор параметров строится до разрешения участников; конструктор состояния
/// и его первичный Begin остаются после проверки типа источника.
pub(super) fn prepare_wuxing_parameters(
    skill_id: u32, properties: &CSkillBaseProperties,
) -> Option<WuXingStateParameters> {
    let kind = kind_for_skill_id(skill_id)?;
    let query = |usage| properties.query_property(usage);
    let element_modify = query(115) as i16;
    let minimum_attack = query(116) as i16;
    let maximum_attack = query(117) as i16;
    let defense = query(109) as i16;
    let element_resistance = query(112) as i16;
    let strength = query(101) as i32;
    let dexterity = query(102) as i32;
    let constitution = query(103) as i32;
    let intelligence = query(104) as i32;
    let maximum_hp = query(118);
    let maximum_hp = if kind == WuXingKind::Water { maximum_hp as i32 } else { i32::from(maximum_hp as i16) };
    let maximum_mp = if kind == WuXingKind::Metal { query(119) } else { 0 };
    Some(WuXingStateParameters {
        element_modify, minimum_attack, maximum_attack, defense, element_resistance,
        strength, dexterity, constitution, intelligence, maximum_hp, maximum_mp,
        blast_attack_scale_bits: (query(80_011) as i32 as f32).to_bits(),
        blast_defense_scale_bits: (query(80_012) as i32 as f32).to_bits(),
        critical_rate_bits: (query(80_016) as i32 as f32).to_bits(),
        element_blast_attack_scale_bits: (query(80_013) as i32 as f32).to_bits(),
        element_blast_defense_scale_bits: (query(80_014) as i32 as f32).to_bits(),
        full_miss_scale_bits: (query(80_015) as i32 as f32).to_bits(),
        resume_hp_peace: query(80_017) as i32,
        resume_mp_peace: query(80_018) as i32,
        resume_hp_fight: query(80_019) as i32,
        resume_mp_fight: query(80_020) as i32,
        restored_hp_peace: query(80_021) as i32,
        restored_mp_peace: query(80_022) as i32,
        restored_hp_fight: query(80_023) as i32,
        restored_mp_fight: query(80_024) as i32,
    })
}

pub(super) fn apply_wuxing_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    parameters: WuXingStateParameters, runtime: &mut Runtime,
) -> bool {
    let Some(kind) = kind_for_skill_id(skill_id) else { return false; };
    let state = WuXingState::new(skill_id, kind, parameters);
    let record = state.encoded();
    let installed = replace_immediate_state(game, source, skill_id, state, &record, runtime);
    if installed && source.1.object_type == 400 && game.find_player(source.1.id).is_some() {
        let _ = game.restore_player_hp_mp_states(source.1.id);
    }
    installed
}
