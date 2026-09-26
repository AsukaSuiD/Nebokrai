//! Яростный прорыв `CRageBreak` (0x6E): Check, AI и порядок наложения
//! состояний. Источник: `gameserver.exe` (SHA-256 `4F5C98E0…`)
//! + `GameServer.pdb` (RSDS match), исходный владелец
//! `appserver/skills/ragebreak.cpp/.h`. Общий stateskill старого пакета
//! обслуживает три Begin, registered visual loop1 и полный End (навигация
//! делегата); здесь — конкретные Check/AI, порядок состояний и wire-visual.
//!
//! Машинные якоря (VA = RVA + 0x400000): ctor `0x59F880` (0x54 байта,
//! фабричный индекс `0x15`, `CGoodsContainer::Add` `0x469C4B`), vtable
//! `0x65BDB4`; Begin триада `0x59F9E0` / `0x59F910` / `0x59FAD0`;
//! CheckCastCondition `0x59FF10`; AI `0x5A00F0`; dtor `0x59FAB0`;
//! DoesTargetEffective `0x5AFCE0` (всегда 1); End — общий хвост
//! `0x546090` (vcall vtable+0x68; `[+0x4C]/[+0x50]` в 0, свежему U
//! возвращается движение `0x4CCEE0(1)`, базовый вызов `0x5DFBD0`);
//! `CRageBreakEffect` vtable `0x65BE48`, UpdateVisualEffect `0x59FB90`.
//!
//! Check (`0x59FF10`): reuse `0x2715` (visual13 + GS0278 при неготовности);
//! RTTI-гейт игрока; стоимость RP `query(3) == 0` — тихий отказ без visual;
//! signed-дефицит RP — visual8 + GS0289 с суммой; успех запрещает движение
//! U (`0x4CCEE0(0)`).
//!
//! AI (`0x5A00F0`): `[+0x4C] == 0` — чистый возврат; таблица и U обязаны
//! (отсутствие → End(0)); `IsDied(U)` → visual2 + End(1); первый проход
//! `[+0x50] == 0` расходует RP игрока (дефицит → visual8 + GS0289 + End(0)),
//! пишет CAN `0x2716` → `[+0x3C]`, даёт visual0; задержка `0x2711` сверяется
//! абсолютной (`start + delay`; unsigned). После визуал1 порядок состояний:
//! End+dtor первого прежнего 0x6E → `new CRageBreakState` (ctor `0x5FD1C0`;
//! gain = `query(105)`, keep = `query(0x2712)`; Begin(U,U), append в хвост)
//! → свип девяти id {0x138, 0xD2, 0xC9, 0x67, 0x192, 0x191, 0x198, 0x199,
//! 0x1A6} (End + dtor + обнуление слота) → у первого 0x131 только End()
//! (без destructor и обнуления) → `new CCureState` (ctor `0x5E9E50`;
//! keep = `query(0x2712)`; Begin(U,U), append в хвост) → UpdateProperty
//! vcall +0x9C → End(1). RP/fail диагностика и visual идут
//! `skills/fury.rs`; обновление срока прежнего 0x6E отсутствует (это ветвь
//! Fury, не RageBreak).
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::StateCastGame`
//! и RP-шов `fury::RageCastPlayer` реализованы у прежнего владельца;
//! разрешение `begin_target`, материализация исполнения и общий полный End
//! остаются в `stateskill.rs` делегата — точки прежнего
//! `end_state_skill(...)` помечаются `StateCastExecutionOutcome`. Исход
//! apply-ветки: успех — EndCompleted (`end_state_skill(1)`), ранний отказ
//! участников — Pending, как у прежнего тела.

use super::curestate::begin_primary_cure_state;
use super::fury::{
    RageRpPolicy, check_rage_skill_cast, prepare_rage_skill_effect,
    remove_reached_conflict_states,
};
use super::ragebreakstate::begin_primary_rage_break_state;
use super::statecast::{
    RageCastVisualContract, StateCastExecutionOutcome, StateCastGame,
    StateCastMoveShape, StateCastVisualTarget, publish_rage_cast_visual,
};
use super::visualeffect::SkillVisualEffectKind;
use crate::effects::{CURE_STATE_SKILL_ID, CureState, RAGE_BREAK_STATE_ID, RageBreakState};

pub const RAGE_BREAK_SKILL_ID: u32 = RAGE_BREAK_STATE_ID;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_ATTACK_GAIN: u32 = 105;

static RAGE_BREAK_VISUAL: RageCastVisualContract = RageCastVisualContract {
    skill_id: RAGE_BREAK_SKILL_ID,
    kind: SkillVisualEffectKind::RageBreak,
    failures: &[2, 7, 8, 13],
    dword_failures: &[8],
    target: StateCastVisualTarget::User,
};

/// Visual навыка `0x000BFE01` `CRageBreakEffect`: отказы 2/7/8/13 только
/// игроку (mode 8 — `[dword 0][byte 8]`), modes 0/1 — around-кадр с U.
pub fn publish_rage_break_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    publish_rage_cast_visual(game, skill, mode, &RAGE_BREAK_VISUAL);
}

/// CheckCastCondition `0x59FF10`: общая RP-проверка с запретом нулевой
/// стоимости (`RageRpPolicy::RequirePositive`).
pub fn check_rage_break_cast<Game>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    Game: StateCastGame,
    Game::Player: super::fury::RageCastPlayer,
{
    check_rage_skill_cast(game, address, RageRpPolicy::RequirePositive, now)
}

/// AI `0x5A00F0`: подготовка `skills/fury.rs`, затем порядок состояний —
/// End+dtor прежнего 0x6E, новый `CRageBreakState`, свип конфликтов,
/// только-End прежнего 0x131, новый `CCureState`, UpdateProperty, End(1).
pub fn run_rage_break_ai<Game>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> StateCastExecutionOutcome
where
    Game: StateCastGame,
    Game::Player: super::fury::RageCastPlayer,
{
    let effect = match prepare_rage_skill_effect(game, address, now) {
        std::ops::ControlFlow::Continue(effect) => effect,
        std::ops::ControlFlow::Break(outcome) => return outcome,
    };
    match apply_rage_break_effect(game, effect, now) {
        Some(_argument) => StateCastExecutionOutcome::EndCompleted,
        None => StateCastExecutionOutcome::Pending,
    }
}

/// Порядок состояний AI (успех возвращает аргумент End — всегда 1):
/// прежний 0x6E завершается полностью (End + destructor свежей позиции),
/// прежний 0x131 — только первым End без внешнего destructor; новый
/// экземпляр всегда добавляется в хвост после позднего запроса срока.
fn apply_rage_break_effect<Game: StateCastGame>(
    game: &mut Game,
    effect: super::fury::RageSkillEffect,
    now: &mut dyn FnMut() -> u32,
) -> Option<i32> {
    let super::fury::RageSkillEffect { source, properties } = effect;
    let previous = game.resolve_state_move_shape(source.0, source.1)?
        .find_state_position(|state| state.state_id() == RAGE_BREAK_STATE_ID);
    if let Some((position, _)) = previous {
        let _ = game.end_and_destroy_state_at(source.0, source.1, position);
    }
    let keep = properties.query_property(STATE_PERSIST_TIME);
    let gain = properties.query_property(TARGET_ATTACK_GAIN) as i32;
    let state = RageBreakState::new(keep, gain);
    let _ = begin_primary_rage_break_state(
        game, source.0, source.1, Some(source), Some(source), state, now,
    );
    remove_reached_conflict_states(game, source.0, source.1);

    // Для прежнего Cure вызывается только первый End, без внешнего destructor;
    // новый экземпляр всегда добавляется в хвост после позднего запроса срока.
    let previous = game.resolve_state_move_shape(source.0, source.1)?
        .find_state_position(|state| state.state_id() == CURE_STATE_SKILL_ID);
    if let Some((_, key)) = previous {
        let _ = game.end_move_shape_state(source.0, source.1, key);
    }
    let cure = CureState::new(properties.query_property(STATE_PERSIST_TIME));
    let _ = begin_primary_cure_state(
        game, source.0, source.1, Some(source), Some(source), cure, now,
    );
    game.update_move_shape_properties(source.0, source.1);
    Some(1)
}

pub const fn is_rage_break_dispatch(dispatch: super::dispatch::PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == RAGE_BREAK_SKILL_ID
}
