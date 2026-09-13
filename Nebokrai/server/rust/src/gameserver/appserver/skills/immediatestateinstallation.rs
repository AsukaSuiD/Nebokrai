//! Установка постоянных состояний TaiJi, Origin и трёх Enlarge.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/{taiji,origin,
//! enlargefullmiss,enlargemaxhp,enlargemaxmp}.cpp и объектный CState::Begin.
//! TaiJi/Origin создают и начинают новое состояние до поиска первого старого
//! ID, затем заменяют его в прежней позиции. Enlarge сначала выполняют
//! End/destructor первого ID, лишь затем читают прибавку и добавляют новый
//! экземпляр в конец. Выбор не фильтрует RTTI или ended.
//! Первичный Begin читает часы и сохраняет фактические U/S, но до установки
//! новый payload не виден списку и callback-ам старого End. У этих пяти Begin
//! нет visual; timestamp не читается их AI, свойствами или восьмибайтным codec.
//! SlotMap/Vec и существующий DB-cache заменяют указатели/STL. Формулы и
//! перезапуск остаются у раздельных payload. Внешний UpdateProperty у Enlarge
//! безусловен после Begin, у TaiJi/Origin требует его успеха; OnChangeStates
//! и завершение навыка не принадлежат этой операции.

use super::enlargefullmiss::{ENLARGE_FULL_MISS_SKILL_ID, SKILL_USAGE_FULL_MISS_GAIN};
use super::enlargefullmissstate::EnlargeFullMissState;
use super::enlargemaxhp::{ENLARGE_MAX_HP_SKILL_ID, SKILL_USAGE_MAX_HP_GAIN};
use super::enlargemaxhpstate::EnlargeMaxHpState;
use super::enlargemaxmp::{ENLARGE_MAX_MP_SKILL_ID, SKILL_USAGE_MAX_MP_GAIN};
use super::enlargemaxmpstate::EnlargeMaxMpState;
use super::origin::{ORIGIN_SKILL_ID, SKILL_USAGE_ELEMENT_MODIFY_GAIN};
use super::originstate::OriginState;
use super::skillbaseproperties::CSkillBaseProperties;
use super::taiji::{TAIJI_SKILL_ID, SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN};
use super::taijistate::TaiJiState;
use crate::gameserver::appserver::moveshape::{AppliedState, CMoveShape, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;

enum ImmediateStatePayload {
    TaiJi(TaiJiState), Origin(OriginState), FullMiss(EnlargeFullMissState),
    MaxHp(EnlargeMaxHpState), MaxMp(EnlargeMaxMpState),
}

impl ImmediateStatePayload {
    fn new(skill_id: u32, properties: &CSkillBaseProperties) -> Option<Self> {
        Some(match skill_id {
            TAIJI_SKILL_ID => Self::TaiJi(TaiJiState::new(properties.query_property(SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN) as i32)),
            ORIGIN_SKILL_ID => Self::Origin(OriginState::new(properties.query_property(SKILL_USAGE_ELEMENT_MODIFY_GAIN) as i32)),
            ENLARGE_FULL_MISS_SKILL_ID => Self::FullMiss(EnlargeFullMissState::new(properties.query_property(SKILL_USAGE_FULL_MISS_GAIN) as i32)),
            ENLARGE_MAX_HP_SKILL_ID => Self::MaxHp(EnlargeMaxHpState::new(properties.query_property(SKILL_USAGE_MAX_HP_GAIN) as i32)),
            ENLARGE_MAX_MP_SKILL_ID => Self::MaxMp(EnlargeMaxMpState::new(properties.query_property(SKILL_USAGE_MAX_MP_GAIN) as i32)),
            _ => return None,
        })
    }

    fn install(self, holder: &mut CMoveShape, placement: Option<(usize, usize)>) -> Option<StateKey> {
        fn install<T: AppliedState>(holder: &mut CMoveShape, state: T, record: &[u8], placement: Option<(usize, usize)>) -> Option<StateKey> {
            match placement {
                Some(location) => holder.insert_replacement_state_record(state, record, location),
                None => Some(holder.append_applied_state_record(state, record)),
            }
        }
        match self {
            Self::TaiJi(state) => install(holder, state, &state.encoded(), placement),
            Self::Origin(state) => install(holder, state, &state.encoded(), placement),
            Self::FullMiss(state) => install(holder, state, &state.encoded(), placement),
            Self::MaxHp(state) => install(holder, state, &state.encoded(), placement),
            Self::MaxMp(state) => install(holder, state, &state.encoded(), placement),
        }
    }
}

fn primary_begin<Runtime: GameMainLoopRuntime>(
    game: &CGame, source: (i32, ShapeIdentity), runtime: &mut Runtime,
) -> Option<((i32, ShapeIdentity), (i32, ShapeIdentity))> {
    let _ = runtime.now_milliseconds();
    let participant = || {
        let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    Some((participant()?, participant()?))
}

fn publish(
    game: &mut CGame, source: (i32, ShapeIdentity), state: ImmediateStatePayload,
    participants: ((i32, ShapeIdentity), (i32, ShapeIdentity)), placement: Option<(usize, usize)>,
) -> bool {
    let Some(holder) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    let Some(key) = state.install(holder, placement) else { return false; };
    holder.set_applied_state_user(key, Some(participants.0));
    holder.set_applied_state_sufferer(key, Some(participants.1));
    true
}

pub(super) fn apply_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) -> bool {
    if matches!(skill_id, TAIJI_SKILL_ID | ORIGIN_SKILL_ID) {
        let Some(state) = ImmediateStatePayload::new(skill_id, properties) else { return false; };
        let Some(participants) = primary_begin(game, source, runtime) else { return false; };
        let Some(holder) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
        let previous = holder.find_state_position(|state| state.state_id() == skill_id);
        let placement = match previous {
            Some((index, key)) => {
                let Some(location) = holder.applied_state_replacement_location(key) else { return false; };
                if end_and_destroy_state_at(game, source.0, source.1, index).is_none() { return false; }
                Some(location)
            }
            None => None,
        };
        if !publish(game, source, state, participants, placement) { return false; }
        let _ = game.update_move_shape_properties(source.0, source.1);
        true
    } else if matches!(skill_id, ENLARGE_FULL_MISS_SKILL_ID | ENLARGE_MAX_HP_SKILL_ID | ENLARGE_MAX_MP_SKILL_ID) {
        let Some(holder) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
        if let Some((index, _)) = holder.find_state_position(|state| state.state_id() == skill_id)
            && end_and_destroy_state_at(game, source.0, source.1, index).is_none()
        { return false; }
        let Some(state) = ImmediateStatePayload::new(skill_id, properties) else { return false; };
        let begun = primary_begin(game, source, runtime);
        let installed = begun.is_some_and(|participants| publish(game, source, state, participants, None));
        let _ = game.update_move_shape_properties(source.0, source.1);
        installed
    } else { false }
}
