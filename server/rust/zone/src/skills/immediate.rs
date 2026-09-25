//! Правила зарегистрированного цикла immediate-состояний TaiJi, Origin,
//! трёх Enlarge, четырёх Swordship и пяти WuXing.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb` (пара
//! `4F5C98E0…`, RSDS match), `appserver/skills/{taiji,origin,enlargefullmiss,
//! enlargemaxhp,enlargemaxmp,swordship,swordship2,swordship3,swordship4,wuxing*}.cpp`.
//!
//! MATCH по машинной разведке порции №4 (запись в docs/status/audit.md
//! 26.09.2026): общий Begin-цикл пятнадцати навыков выполняет Check чтением
//! `[skill+0x64]`, при отказе — End(0), при успехе — phase=1; reuse, MP,
//! visual и Move в цикле отсутствуют. AI читает свойства до GetU и допускает
//! при NULL U подстановку GetS, кроме Swordship, которому нужен именно U.
//! `CTaiJi::AI` VA `0x005AF770` и `COrigin::AI` VA `0x005AEA70`: создание и
//! первичный Begin нового состояния до поиска первого старого ID, затем End
//! старого, destroy свежего остатка и установка в ту же позицию,
//! UpdateProperty и End(1). Три Enlarge исполняют End первого ID до чтения
//! прибавки и добавляют новое состояние в конец, UpdateProperty безусловно
//! даже при отказе Begin; Swordship1-4 завершают AI вызовом End(0), включая
//! успешную установку. Остальные immediate-семейства дают End(1) включая
//! отказ Begin состояния. UNKNOWN честные: writer/reader тела
//! Serialize/Unserialize состояний (шапки данных — zone/effects).
//!
//! Живой обход Game (поиск участников, арена состояний, публикация, часы
//! машины) остаётся за переходным `appserver/skills/immediatestate*.rs`;
//! здесь ID-карта семьи, подстановка S, ветка установки, End-политика и
//! подготовка payload по таблице свойств. Swordship читает MIN перед MAX —
//! порядок подтверждён property callback и writer состояний.

use crate::effects::{
    ENLARGE_FULL_MISS_STATE_ID, ENLARGE_MAX_HP_STATE_ID, ENLARGE_MAX_MP_STATE_ID,
    EnlargeFullMissState, EnlargeMaxHpState, EnlargeMaxMpState, ORIGIN_STATE_ID, OriginState,
    SwordshipState, TAIJI_STATE_ID, TaiJiState, is_swordship_state_id,
};

use super::wuxing::is_wuxing_skill;

const TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;
const ELEMENT_MODIFY_GAIN: u32 = 115;
const FULL_MISS_GAIN: u32 = 115;
const MAX_HP_GAIN: u32 = 118;
const MAX_MP_GAIN: u32 = 115;
const TARGET_MIN_ATK_GAIN: u32 = 0x74;
const TARGET_MAX_ATK_GAIN: u32 = 0x75;

/// Пятнадцать навыков общего цикла: TaiJi, Origin, три Enlarge,
/// четыре Swordship и пять WuXing.
pub const fn is_immediate_state_skill(skill_id: u32) -> bool {
    matches!(skill_id, TAIJI_STATE_ID | ORIGIN_STATE_ID | ENLARGE_FULL_MISS_STATE_ID
        | ENLARGE_MAX_HP_STATE_ID | ENLARGE_MAX_MP_STATE_ID)
        || is_swordship_state_id(skill_id) || is_wuxing_skill(skill_id)
}

/// AI Swordship передаёт End(0) включая успех; остальные — End(1) даже при
/// отказе Begin состояния. Внешние End(1/4/0) эта политика не подменяет.
pub const fn immediate_completion_end_argument(skill_id: u32) -> i32 {
    if is_swordship_state_id(skill_id) { 0 } else { 1 }
}

/// Swordship допускает только GetU; остальные семейства при NULL U читают GetS.
pub const fn immediate_ai_sufferer_fallback(skill_id: u32) -> bool {
    !is_swordship_state_id(skill_id)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImmediateStatePlacement {
    /// TaiJi/Origin/Swordship: новый payload и его Begin до поиска первого
    /// старого ID, затем замена в прежней позиции.
    ReplaceAtOldPosition,
    /// Enlarge: End/destructor первого ID до чтения прибавки, append в конец.
    EndFirstThenAppend,
}

/// Ветка установки по ID навыка; WuXing её не проходит — у него собственная
/// подготовка параметров (zone skills/wuxing) и тот же replace-установщик Game.
pub const fn immediate_state_placement(skill_id: u32) -> Option<ImmediateStatePlacement> {
    if matches!(skill_id, TAIJI_STATE_ID | ORIGIN_STATE_ID) || is_swordship_state_id(skill_id) {
        Some(ImmediateStatePlacement::ReplaceAtOldPosition)
    } else if matches!(skill_id, ENLARGE_FULL_MISS_STATE_ID | ENLARGE_MAX_HP_STATE_ID | ENLARGE_MAX_MP_STATE_ID) {
        Some(ImmediateStatePlacement::EndFirstThenAppend)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImmediateStatePayload {
    TaiJi(TaiJiState),
    Origin(OriginState),
    FullMiss(EnlargeFullMissState),
    MaxHp(EnlargeMaxHpState),
    MaxMp(EnlargeMaxMpState),
    Swordship(SwordshipState),
}

impl ImmediateStatePayload {
    /// Выбор ветки состояния и чтение прибавки из таблицы навыка. Момент
    /// вызова — у ветки установки Game: replacement до End старого, Enlarge
    /// после него; выбор не фильтрует RTTI или ended.
    pub fn new(skill_id: u32, mut query_property: impl FnMut(u32) -> u32) -> Option<Self> {
        Some(match skill_id {
            TAIJI_STATE_ID => Self::TaiJi(TaiJiState::new(query_property(TARGET_ELEMENT_RESISTANT_GAIN) as i32)),
            ORIGIN_STATE_ID => Self::Origin(OriginState::new(query_property(ELEMENT_MODIFY_GAIN) as i32)),
            ENLARGE_FULL_MISS_STATE_ID => Self::FullMiss(EnlargeFullMissState::new(query_property(FULL_MISS_GAIN) as i32)),
            ENLARGE_MAX_HP_STATE_ID => Self::MaxHp(EnlargeMaxHpState::new(query_property(MAX_HP_GAIN) as i32)),
            ENLARGE_MAX_MP_STATE_ID => Self::MaxMp(EnlargeMaxMpState::new(query_property(MAX_MP_GAIN) as i32)),
            id if is_swordship_state_id(id) => Self::Swordship(swordship_state(id, query_property)),
            _ => return None,
        })
    }
}

/// MIN читается перед MAX; исходный ctor принимает MAX/MIN, а Rust хранит их
/// в именованных полях.
pub fn swordship_state(
    skill_id: u32, mut query_property: impl FnMut(u32) -> u32,
) -> SwordshipState {
    let minimum = query_property(TARGET_MIN_ATK_GAIN) as i32;
    let maximum = query_property(TARGET_MAX_ATK_GAIN) as i32;
    SwordshipState::new(skill_id, minimum, maximum)
}
