//! CTeamState, gameserver.exe + GameServer.pdb, исходный owner
//! appserver/other states/teamstate.cpp. Общая арена CMoveShape хранит
//! экземпляр, стороны и visual; Vec/Drop заменяют строки и ручной lifetime.
//! Object Begin0x005BF9A0 требует sufferer: base self/self clock → visual
//! loop1/Update0 → lastcheck=0 → append у caller0x0048D8A2. NULL-user restart
//! сохраняет User и не читает часы. Base timestamp здесь не потребляется.
//! Visual0x005BFAD0 использует actual sufferer; absent/ended подавляют пакет,
//! но существующий ресурс получает base tail. BFE03 содержит type/id,
//! state ID100006, time0, живой GetAdditionalData и имя как C-строку.
//! Getter0x005BFDD0 берёт число разрешённых team plugs либо1; password-bit16
//! зависит от длины строки, а не её первого байта. Первичный Begin использует
//! тот же размер существующей команды, не постоянную единицу.
//! AI0x005BFD20: clock1 → unsigned last+5000<=now → clock2/write stamp →
//! actual sufferer/Player. Нет цели или не Player — End; team0, отсутствующая
//! session и текущий лидер сохраняют состояние. Смерть отдельно не проверяется.
//! End0x005FD420: optional visual/BFE04 → свежий sufferer → общий RemoveState,
//! без state.ended и удаления чужой арены. Vtable0x0065D2B4 SetRegion меняет
//! только sufferer-region. Общий span сохраняет переменную длину записи.
//! Ctor0x005BFE60 сохраняет C-prefix имени/пароля. Serialize0x005BFA50 и
//! Unserialize0x005BFF20: ID и две C-строки, без часов;
//! bounded decode допускает до255 байт на строку. Неинициализированный native
//! default-ctor stamp безопасно равен0 до Begin; отказ allocator не эмулируется.
//! Координатные Begin0x005BF800/0x005BF8D0 остаются только в локальном исследовательском корпусе.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_applied_state_visual_base,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{CTeamState};

const TEAM_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const TEAM_STATE_UPDATE_MESSAGE: i32 = 0x000b_fe05;

pub(crate) fn begin_primary_team_state(
    game: &mut CGame,
    player_id: i32,
    team_name: Vec<u8>,
    team_password: Vec<u8>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let state = CTeamState::new(team_name, team_password);
    game.find_player(player_id)?;
    let _ = now();
    let player = game.find_player(player_id)?;
    let participant = (
        player.shape().get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..player.shape().identity() },
    );
    let teammates = team_member_count(game, participant.1);
    let message = team_state_begin_message(participant.1, &state, teammates);
    let _ = game.send_move_shape_around(participant.0, participant.1, &message);
    let record = state.encoded_for_install();
    let shape = game.find_player_mut(player_id)?.move_shape_mut();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    Some(key)
}

pub(crate) fn restart_team_recruitment_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CTeamState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_team_state_begin_visual(game, region_id, holder, key);
    }
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<CTeamState>(key))
    {
        state.reset_check();
    }
    true
}

fn team_member_count(game: &CGame, target: ShapeIdentity) -> usize {
    if target.object_type == 400 {
        game.find_player(target.id)
            .map(|player| game.team_state_member_count(player.team_id()))
            .unwrap_or(1)
    } else {
        1
    }
}

fn team_state_begin_message(
    target: ShapeIdentity, state: &CTeamState, teammates: usize,
) -> CMessage {
    let mut message = CMessage::new(TEAM_STATE_BEGIN_MESSAGE);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.state_id());
    message.add_long(state.client_state_time());
    message.add_ulong(state.additional_data(teammates));
    message.base_mut().add(state.team_name());
    message.add_byte(0);
    message
}

fn update_team_state_begin_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(ended) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_visual_ended(key))
    else { return false };
    if !ended {
        if let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key) {
            let teammates = resolve_applied_state_sufferer(game, region_id, holder, key)
                .map(|(_, target)| team_member_count(game, target)).unwrap_or(1);
            let message = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<CTeamState>(key))
                .map(|state| team_state_begin_message(target, state, teammates));
            if let Some(message) = message {
                let _ = game.send_move_shape_around(target_region, target, &message);
            }
        }
    }
    update_applied_state_visual_base(game, region_id, holder, key);
    true
}

pub(crate) fn update_team_recruitment_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let sampled_at_ms = runtime.now_milliseconds();
    let due = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CTeamState>(key))
        .is_some_and(|state| state.check_due(sampled_at_ms));
    if !due { return false; }
    let recorded_at_ms = runtime.now_milliseconds();
    let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<CTeamState>(key))
    else { return false; };
    state.record_check(recorded_at_ms);
    if let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key) {
        if target.object_type == 400 {
            if let Some(player) = game.find_player(target.id) {
                let team_id = player.team_id();
                let team_leader_id = (team_id != 0)
                    .then(|| game.get_team_session_id(team_id as u32))
                    .and_then(|session_id| game.session_factory().query_team(session_id))
                    .map(|team| team.leader_id());
                if !CTeamState::ends_for_team(target.id, team_id, team_leader_id) {
                    return false;
                }
            }
        }
    }
    end_team_recruitment_state(game, region_id, holder, key)
}

pub(crate) fn end_team_recruitment_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CTeamState>(key))
    else { return false; };
    let bytes = 6 + state.team_name().len() + state.team_password().len();
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, bytes)
}

pub(crate) fn team_state_update_message(
    player_id: i32,
    state: &CTeamState,
    teammate_count: usize,
) -> CMessage {
    let mut message = CMessage::new(TEAM_STATE_UPDATE_MESSAGE);
    message.add_long(player_id);
    message.add_long(player_id);
    message.add_long(state.state_id());
    message.add_ulong(state.additional_data(teammate_count));
    message
}
