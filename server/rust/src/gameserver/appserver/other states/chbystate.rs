//! Владелец состояния преображения `CHBYState` GameServer.
//!
//! Контракт подтверждён точной парой `gameserver.exe + GameServer.pdb` и
//! исходным owner-ом `appserver/other states/chbystate.cpp`. Реализация хранит
//! byte-exact 120-байтовый `tagCHBYState`, время, пять временных навыков и
//! сохранённые hotkey 12..23. Системный wrapping tick передаётся caller-ом;
//! Rust-владение заменяет raw `CState*`. Ниже сохранены ещё не полностью
//! перенесённые конструктор/перегрузки Begin; их RAW не входит в runtime.
//! Доступ к little-endian полям делегирован общему legacy codec поверх `bytes`.
//! `Serialize` записывает остаток обратно в keeptime без перезапуска clock;
//! клиентский снимок этого не делает. AI0x005DAAA0 при keep=0 не читает часы;
//! иначе один clock и строгое unsigned сравнение wrapping deadline < now.
//! После срока GetSufferer(+18) определяет адресата GS1145, затем вызывается
//! End(+1C), независимо от хранения строки уведомления. Holder не заменяет
//! адресата. Native unchecked CPlayer-получатель вне живого player подавляет
//! только небезопасную отправку адаптера; достигнутый End остаётся отдельным.
//! Payload принадлежит общей арене CMoveShape; decode_at читает одну
//! фабрично подтверждённую запись и сохраняет её точный offset, без byte-scan.
//! restart_change_body_state переносит object Begin0x005DB000 после смерти
//! и Begin(NULL, holder, 1)0x005DADB0 в обычном StartAllStates. Первый требует
//! CPlayer RTTI, второй native делает unchecked player-cast; безопасный
//! адаптер явно отказывает non-player, не изображая определённый native результат.
//! Base Begin сохраняет timestamp, затем visual/очистка emotion/mode, Update0
//! и base visual tail, append пяти ID в m_vskill без очистки, hotkeys.
//! Только двухаргументный Begin при !online сохраняет/обнуляет hotkeys12..23.
//! Первые пять накопленных ID задают hotkeys12..16; нулевые ID не шлют пакет.
//! Visual0x005DA390 добавляет навыки последовательно через общий AddSkill,
//! затем читает реальный GetSkill и его virtual+70/+74/+78. ID0 занимает
//! шесть ULONG0; missing skill/properties не создаёт placeholder. Клиентское
//! время0x005DA030 читает часы1/2/3 раза; Unserialize выставляет online=true.
//! Чистый visual-снимок не копирует накопленный m_vskill и не владеет состоянием.
//! Конструктор 0x005DAC90 не читает часы и оставляет m_vskill пустым.
//! Destructor0x005DAA30 освобождает вектор, затем всегда вызывает базовый
//! destructor; Vec/арена владеют тем же ресурсным хвостом без End/пакетов.
//! Произвольный User и флаг0 в Begin(U,S,flag)0x005DADB0 пока не перенесены;
//! реализован только достигнутый restart(NULL,player,1), RAW overload сохранён.
//! Первичный object Begin0x005DB000 проверяет sufferer/CPlayer и при User=self
//! читает один base clock. До append выполняются ClearEmotion/mode, loop1
//! visual Update0 с пятью AddSkill, накопление ID и hotkeys. Единственный
//! подготовленный payload остаётся вне арены через эти callbacks, как native;
//! его копия не регистрируется раньше Begin. Caller после успеха переносит
//! payload в арену и связывает User/Sufferer с сохранённой в базе identity.
//! Loop1 visual не меняет ended в base tail; его итоговое владение создаёт
//! существующий каталог арены при append, без повторной отправки Begin.
//! Новый cache-record кодируется после Begin без Serialize и без часов.
//! Writer по общему span обновляет все поля, включая mode/old_hotkeys,
//! сохраняя padding загруженных tagCHBYState по смещениям +1 и +70..71.
//! Общий Save читает native remaining-getter один раз и передаёт одно значение
//! writer-у и commit живого keepTime до обработки следующего экземпляра.
//! End0x005DA240 реализован в CGame::end_move_shape_change_body_state:
//! visual Update1 при наличии ресурса, свежий Sufferer, обнуление режима
//! игрока и payload, двенадцать восстановленных hotkeys/BF908, пять DelSkill
//! по параметрам (не по накопленному вектору), затем RemoveState. Base End
//! и запись state.ended здесь отсутствуют. Visual ended/NULL S подавляют
//! пакет, но не base visual tail; unsafe native non-player tail не эмулируется.
//! OnChangeRegion0x005DAB80 в CGame::set_change_body_state_region пишет только
//! User-region перед флагами и возможным notice/End; Sufferer не меняется.
//! Выделение длинной строки уведомления не отменяет последующий End.
//! OnUpdateProperties0x005DA0F0 требует sufferer, но не user/Begin; для игрока
//! сначала присваивает ненулевой mode, затем ненулевые добавки с DWORD wrapping
//! до ограничения INT_MAX и WORD wrapping. Этот callback не вызывает visual
//! и не читает часы; расчёт заимствует payload без копии накопленного m_vskill.

use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
    resolve_state_move_shape, resolve_applied_state_sufferer, change_body_client_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{
    CHANGE_BODY_SKILL_TYPE, CHANGE_BODY_STATE_BYTES, CHANGE_BODY_STATE_ID, ChangeBodyState,
};

/// Тонкая оболочка переходного Game: фабрика навыков остаётся у старого
/// владельца, а данные и 124-байтная запись — в Zone `effects/changebody.rs`.
pub(crate) fn change_body_state_from_factory(
    level: u32,
    factory: &CSkillFactory,
) -> Option<ChangeBodyState> {
    let properties =
        factory.query_skill_base_properties(CHANGE_BODY_SKILL_TYPE, level as i32)?;
    ChangeBodyState::from_properties(level, |usage| Some(properties.query_property(usage)))
}

pub(crate) fn update_change_body_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ChangeBodyState>(key))
    else { return false };
    let mode = state.mode;
    let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false };
    if target.object_type != 400 { return true }
    if mode != 0 {
        let Some(player) = game.find_player_mut(target.id) else { return false };
        let (head, face, _) = player.appearance_and_mode();
        player.restore_appearance_and_mode(head, face, mode);
    }
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ChangeBodyState>(key))
    else { return false };
    let Some(player) = game.find_player(target.id) else { return false };
    let properties = CPlayer::apply_active_change_body_state_properties(
        player.combat_properties(), state,
    );
    let Some(player) = game.find_player_mut(target.id) else { return false };
    player.update_state_combat_properties(|_| properties);
    true
}

/// Достигнутый AddCHBY вызывает object Begin(self, self) до append.
/// Возвращается адрес, записанный базой до visual/skill callbacks, а не
/// повторно снятый регион после возможных побочных действий этих callbacks.
pub(crate) fn begin_primary_change_body_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &mut ChangeBodyState,
    now: &mut dyn FnMut() -> u32,
) -> Option<(i32, ShapeIdentity)> {
    if holder.object_type != 400 || game.find_player(holder.id).is_none() {
        return None;
    }
    state.started_ms = now();
    let player = game.find_player_mut(holder.id)?;
    let participant = (
        player.shape().get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..player.shape().identity() },
    );
    player.clear_emotion_state();
    let (head, face, _) = player.appearance_and_mode();
    player.restore_appearance_and_mode(head, face, state.mode);

    let _ = send_change_body_begin_visual(game, region_id, holder, (&*state).into(), None, now);
    state.begun_skill_ids.extend(state.skills.iter().map(|(id, _)| *id));
    if !state.online {
        let Some(player) = game.find_player_mut(holder.id) else { return Some(participant) };
        for index in 0..12 {
            state.old_hotkeys[index] = player.hotkey((index + 12) as u8).unwrap_or_default();
            let _ = player.set_hotkey((index + 12) as u8, 0);
        }
    }
    for index in 0..5 {
        let Some(skill_id) = state.begun_skill_ids.get(index).copied() else { break };
        if skill_id != 0 {
            game.set_script_player_hotkey(holder.id, (index + 12) as u8, u32::from(skill_id) | 0x8000_0000);
        }
    }
    game.set_script_player_hotkey(holder.id, 17, 0x8000_031f);
    Some(participant)
}

pub(crate) fn restart_change_body_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    after_death: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if holder.object_type != 400 { return false }
    let Some(mode) = game.find_player(holder.id)
        .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key))
        .map(|state| state.mode)
    else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if !begin_applied_state_visual(game, region_id, holder, key, 1) { return true }
    let Some(player) = game.find_player_mut(holder.id) else { return true };
    player.clear_emotion_state();
    let (head, face, _) = player.appearance_and_mode();
    player.restore_appearance_and_mode(head, face, mode);

    let snapshot = game.find_player(holder.id)
        .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key))
        .map(ChangeBodyBeginVisualSnapshot::from);
    if let Some(snapshot) = snapshot {
        let _ = send_change_body_begin_visual(game, region_id, holder, snapshot, Some(key), now);
    }
    let _ = update_applied_state_visual_base(game, region_id, holder, key);

    let Some(player) = game.find_player_mut(holder.id) else { return true };
    let Some(state) = player.move_shape_mut().applied_state_mut::<ChangeBodyState>(key)
    else { return true };
    state.begun_skill_ids.extend(state.skills.iter().map(|(id, _)| *id));
    let save_hotkeys = after_death && !state.online;
    if save_hotkeys {
        for index in 0..12 {
            let old = player.hotkey((index + 12) as u8).unwrap_or_default();
            let Some(state) = player.move_shape_mut().applied_state_mut::<ChangeBodyState>(key)
            else { return true };
            state.old_hotkeys[index] = old;
            let _ = player.set_hotkey((index + 12) as u8, 0);
        }
    }
    for index in 0..5 {
        let Some(skill_id) = game.find_player(holder.id)
            .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key))
            .and_then(|state| state.begun_skill_ids.get(index)).copied()
        else { return true };
        if skill_id != 0 {
            game.set_script_player_hotkey(holder.id, (index + 12) as u8, u32::from(skill_id) | 0x8000_0000);
        }
    }
    if game.find_player(holder.id)
        .and_then(|player| player.move_shape().applied_state::<ChangeBodyState>(key)).is_some()
    {
        game.set_script_player_hotkey(holder.id, 17, 0x8000_031f);
    }
    true
}

#[derive(Clone, Copy, Debug)]
struct ChangeBodyBeginVisualSnapshot {
    started_ms: u32,
    keep_time_ms: u32,
    mode: u32,
    level: u32,
    visual_effect: u16,
    continue_after_death: bool,
    skills: [(u16, u16); 5],
}

impl From<&ChangeBodyState> for ChangeBodyBeginVisualSnapshot {
    fn from(state: &ChangeBodyState) -> Self {
        Self {
            started_ms: state.started_ms,
            keep_time_ms: state.keep_time_ms,
            mode: state.mode,
            level: state.level,
            visual_effect: state.visual_effect,
            continue_after_death: state.continue_after_death,
            skills: state.skills,
        }
    }
}


fn send_change_body_begin_visual(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    snapshot: ChangeBodyBeginVisualSnapshot,
    key: Option<StateKey>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder).is_none() { return false }
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(CHANGE_BODY_STATE_ID as i32);
    message.add_ulong(change_body_client_time(snapshot.started_ms, snapshot.keep_time_ms, &mut *now));
    message.add_long(0);
    message.add_ulong(snapshot.mode);
    message.add_ulong(snapshot.level);
    message.add_ulong(u32::from(snapshot.visual_effect));
    message.add_byte(u8::from(snapshot.continue_after_death));
    for index in 0..5 {
        let (skill_id, level) = if let Some(key) = key {
            let Some(skills) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ChangeBodyState>(key))
                .map(|state| state.skills)
            else { return false };
            skills[index]
        } else { snapshot.skills[index] };
        if skill_id == 0 {
            for _ in 0..6 { message.add_ulong(0); }
            continue;
        }
        let _ = game.add_move_shape_skill(region_id, holder, u32::from(skill_id), i32::from(level));
        let (skill_id, level) = if let Some(key) = key {
            let Some(skills) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ChangeBodyState>(key))
                .map(|state| state.skills)
            else { return false };
            skills[index]
        } else { (skill_id, level) };
        let skill = game.registered_move_shape_skill(region_id, holder, u32::from(skill_id));
        let properties = game.skill_base_properties(u32::from(skill_id), i32::from(level));
        if let (Some(skill), Some(properties)) = (skill, properties)
            && let Some(parameters) = game.registered_skill_client_parameters(skill)
        {
            message.base_mut().add_short(skill_id as i16);
            message.base_mut().add_short(level as i16);
            message.add_ulong(properties.query_property(10_005));
            for value in parameters { message.add_ulong(value); }
        }
    }
    let _ = game.send_move_shape_around(region_id, holder, &message);
    true
}
