//! Идентификаторы и подготовка параметров пяти состояний У-син.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb` (пара
//! `4F5C98E0…`, RSDS match), `appserver/skills/wuxingmetal.cpp`, `wuxingwood.cpp`,
//! `wuxingwater.cpp`, `wuxingfire.cpp`, `wuxingearth.cpp`.
//!
//! MATCH по машинной разведке порции №4 (запись в docs/status/audit.md
//! 26.09.2026): общий зарегистрированный AI читает все двадцать четыре
//! свойства в приведённом ниже порядке ещё до выбора GetU/GetS, затем
//! допускает только источник-игрока. Water сохраняет полный signed MAX_HP,
//! остальные элементы сужают его до short; дополнительный MAX_MP читает
//! только Metal (usage 119). После установки — UpdateProperty и отдельный
//! RestoreHpMp, не зависящий от его результата; успешный AI вызывает End(1)
//! даже при отказе Begin состояния, отсутствие U/S или иной тип — End(0).
//! UNKNOWN честные: тела `CWuXing*::AI` Fire/Water/Wood дизассемблом не
//! сняты — их конструкторы разделяют общий порядок по Metal/Earth, а
//! x87-промежуточная точность последующей формулы остаётся PARTIAL
//! (зафиксировано в шапке zone/effects/wuxing.rs).
//!
//! Живой обход Game (создание состояния, replace-установщик, восстановление
//! HP/MP) остаётся за переходным `appserver/skills/wuxing.rs`; данные и
//! 96-байтная запись — zone/effects/wuxing.rs. Здесь ID-карта и табличная
//! подготовка параметров по делегированному чтению свойств.

use crate::effects::{WuXingKind, WuXingStateParameters, kind_for_skill_id};

use crate::effects::{
    WUXING_EARTH_STATE_ID, WUXING_FIRE_STATE_ID, WUXING_METAL_STATE_ID, WUXING_WATER_STATE_ID,
    WUXING_WOOD_STATE_ID,
};

/// Пять навыков У-син общего immediate-цикла.
pub const fn is_wuxing_skill(skill_id: u32) -> bool {
    matches!(skill_id, WUXING_METAL_STATE_ID | WUXING_WOOD_STATE_ID
        | WUXING_WATER_STATE_ID | WUXING_FIRE_STATE_ID | WUXING_EARTH_STATE_ID)
}

/// Набор параметров строится до разрешения участников; конструктор состояния
/// и его первичный Begin остаются после проверки типа источника.
pub fn prepare_wuxing_parameters(
    skill_id: u32, mut query_property: impl FnMut(u32) -> u32,
) -> Option<WuXingStateParameters> {
    let kind = kind_for_skill_id(skill_id)?;
    let query = &mut query_property;
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
