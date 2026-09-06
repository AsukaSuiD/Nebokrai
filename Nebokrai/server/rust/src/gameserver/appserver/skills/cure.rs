//! Очищение `CCure` (`0x131`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/cure.cpp`. Модуль сохраняет двойную проверку MP,
//! время восстановления, путь и препятствия, задержку, направление, точную
//! вероятность и один вызов генератора MSVCRT на каждое подходящее состояние
//! в порядке исходного вектора состояний. Из уже типизированных состояний
//! достигнуты `0x67`, `0x73`, `0x7C`, `0xC9`, `0xD2`, `0x138`, `0x191`,
//! `0x192`, активный `CStateSkill` `0x198`, эффекты `0x199`, `0x1A6` и
//! `0x1F8`; неизвестные старые записи
//! остаются нетронутыми. `CGame` только разрешает владельцев и выполняет
//! доставку. Координатная перегрузка `Begin` использует точный базовый
//! `CState::GetSufferer` и fallback к `GetUser`, когда цель не найдена;
//! `DoesTargetEffective` допускает игрока либо только carriage-монстра, не
//! подменяя обычного или приручённого монстра заклинателем.
//! Порог очищения сохраняет расширенное вычисление x87 и усечение к нулю
//! перед исходным целочисленным умножением. Восстановление использует
//! абсолютные сроки `CSkill::IsRestored` и cast-delay (cmp/jb по 0x005AE373).
//! После списания MP вызывается OnChangeStates (0x005AE2F1) до поворота
//! и визуализации каста, а не обновление общего боевого статуса.
//! AI (`0x005AE110`) сначала выполняет Begin нового Cure (`0x005AE49F`),
//! затем завершает только первый прежний (`0x005AE50A`) и устанавливает
//! новый в освободившийся слот (`0x005AE53A`). End пересчитывает свойства
//! без обоих экземпляров; остальные Cure сохраняются. При отсутствии
//! прежней записи новый экземпляр добавляется в конец без UpdateProperty.
//! CastCure (0x005ADB10) допускает 0x198 (cmp в 0x005ADC0A) без
//! исключения для самого заклинателя и вызывает End состояния через +0x1C
//! в 0x005ADC58. На время очищения извлечённый AI публикуется в CPlayer:
//! завершение собственного SpiderMist видит тот же экземпляр и возвращает
//! изменённый AI, сохраняя Cure и остальные независимые исполнения.
//! SpiderMist использует CState::End() 0x005DBCE0, а не End(int):
//! RemoveState (0x004CDAB0) удаляет экземпляр и вызывает UpdateProperty.
//! Деструкторы 0x005406D0 → 0x005E0F20 → 0x004D81A0 не возвращают
//! движение и не фиксируют reuse. Rust удаляет только совпавшее исполнение
//! и его state-запись; ссылку выбора на удалённый экземпляр обнуляет безопасно,
//! не сохраняя native dangling pointer. Созданный phalanx остаётся независимым.

use super::fightdefense::truncate_original;
use super::bossbluequakestate::{
    BOSS_BLUE_QUAKE_STATE_ID, BossBlueQuakeState,
    finish_player_boss_blue_quake_state_on_cure, send_boss_blue_quake_state_visual,
};
use super::boalockstate::{
    BOA_LOCK_STATE_ID, BoaLockState, send_boa_lock_state_visual,
};
use super::curestate::{CureState, send_cure_state_visual};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::stateskill::finish_state_skill;
use super::knockoutstate::{
    KNOCK_OUT_STATE_ID, KnockOutState, finish_player_knock_out_state_on_defense,
    send_knock_out_state_visual,
};
use super::knightcutstate::{
    KNIGHT_CUT_STATE_ID, KnightCutState, finish_player_knight_cut_state_on_cure,
    send_knight_cut_state_visual,
};
use super::spiderpoison::SPIDER_POISON_SKILL_ID;
use super::spidermist::SPIDER_MIST_SKILL_ID;
use super::spiderpoisonstate::{
    SpiderPoisonState, finish_player_spider_poison_state_on_cure,
    send_spider_poison_state_visual,
};
use super::spriteburn::SPRITE_BURN_SKILL_ID;
use super::spriteburnstate::{
    SpriteBurnState, finish_player_sprite_burn_state_on_cure,
    send_sprite_burn_state_visual,
};
use super::spiderweb::SPIDER_WEB_SKILL_ID;
use super::spiderwebstate::{
    SpiderWebState, finish_player_spider_web_state_on_defense,
    send_spider_web_state_visual,
};
use super::sealstate::{SEAL_STATE_ID, SealState, send_seal_state_visual};
use super::poisonfogstate::{POISON_FOG_STATE_ID, PoisonFogState, send_poison_fog_state_visual};
use super::rushstate::{RUSH_STATE_ID, RushState, send_rush_state_visual};
use super::rushstate2::{RUSH_2_STATE_ID, Rush2State, send_rush_2_state_visual};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_coordinate_sufferer, resolve_identity_sufferer, resolve_state_user,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;

pub(crate) const CURE_SKILL_ID: u32 = 0x131;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;
const CONST: u32 = 20_010;
const EM_MODIFIER: u32 = 20_015;
const BASE_PROBABILITY: u32 = 40_001;

#[derive(Clone)]
struct CureTarget {
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    dead: bool,
    display_name: Vec<u8>,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn caster_identity(player_id: i32) -> ShapeIdentity {
    ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<CureTarget> {
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            let tile_x = player.shape().get_tile_x().ok()?;
            let tile_y = player.shape().get_tile_y().ok()?;
            Some(CureTarget {
                identity,
                tile_x,
                tile_y,
                dead: player.is_dead(),
                display_name: player.player_name().to_vec(),
            })
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let tile_x = monster.move_shape().shape().get_tile_x().ok()?;
            let tile_y = monster.move_shape().shape().get_tile_y().ok()?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            if !monster.is_carriage(property) {
                return None;
            }
            Some(CureTarget {
                identity,
                tile_x,
                tile_y,
                dead: monster.hit_points() == 0,
                display_name: monster.display_name().to_vec(),
            })
        }
        _ => None,
    }
}

fn requested_target(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
    player_id: i32,
) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id: CURE_SKILL_ID, .. } => Some(caster_identity(player_id)),
        PlayerSkillDispatch::Point { skill_id: CURE_SKILL_ID, x, y } => {
            resolve_coordinate_sufferer(game, region_id, x, y)
                .or_else(|| resolve_state_user(game, region_id, caster_identity(player_id)))
        }
        PlayerSkillDispatch::Object { skill_id: CURE_SKILL_ID, target: target @ ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } } => {
            resolve_identity_sufferer(game, region_id, target)
                .or_else(|| resolve_state_user(game, region_id, caster_identity(player_id)))
        }
        _ => None,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_cast(game: &mut CGame, player_id: i32, target: &CureTarget, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(CURE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(target.identity.object_type);
        message.add_long(target.identity.id);
        message.add_long(target.tile_x);
        message.add_long(target.tile_y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_cure<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_skill_used(CURE_SKILL_ID, now_ms));
}

fn abort_player_cure(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn complete_player_cure<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(CURE_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_cure(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_cure<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(CURE_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_cure(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn cure_threshold(element_modify: i32, base_probability: u32, constant: u32, em_modifier: u32) -> i32 {
    let scaled = truncate_original(
        f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    (scaled as u32).wrapping_mul(constant).wrapping_add(base_probability) as i32
}

fn curable_state_ids(game: &CGame, region_id: i32, target: ShapeIdentity) -> Vec<u32> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::curable_state_ids).unwrap_or_default(),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).map(|monster| monster.move_shape().curable_state_ids()).unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn finish_active_spider_mist_on_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    _runtime: &mut Runtime,
) -> bool {
    let Some(mut player_ai) = game.find_player_mut(player_id).map(CPlayer::take_player_ai) else {
        return false;
    };
    let finished = player_ai.player_skill_execution(SPIDER_MIST_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
        .is_some_and(|dispatch| player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled));
    if let Some(player) = game.find_player_mut(player_id) {
        player.restore_player_ai(player_ai);
        if finished {
            player.finish_curable_skill_state(SPIDER_MIST_SKILL_ID);
            if player.current_skill_id() == Some(SPIDER_MIST_SKILL_ID) {
                player.set_current_skill_id(None);
            }
        }
    }
    if finished {
        let _ = game.update_player_properties(player_id);
    }
    finished
}

enum RemovedMonsterCurableState {
    BoaLock(BoaLockState),
    Rush(RushState),
    Rush2(Rush2State),
    Seal(SealState),
    SpiderPoison(SpiderPoisonState),
    SpriteBurn(SpriteBurnState),
    SpiderWeb(SpiderWebState),
    KnockOut(KnockOutState),
    BossBlueQuake(BossBlueQuakeState),
    KnightCut(KnightCutState),
    PoisonFog(PoisonFogState),
    ActiveSpiderMist,
}

fn finish_monster_curable_state(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    state_id: u32,
    now_ms: u32,
) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else { return false };
    let removed = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        let removed = match state_id {
            BOA_LOCK_STATE_ID => {
                let state = monster.move_shape_mut().take_boa_lock_state()?;
                monster.move_shape_mut().set_moveable(true);
                RemovedMonsterCurableState::BoaLock(state)
            }
            RUSH_STATE_ID => {
                let state = monster.move_shape_mut().take_rush_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::Rush(state)
            }
            RUSH_2_STATE_ID => {
                let state = monster.move_shape_mut().take_rush_2_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::Rush2(state)
            }
            SEAL_STATE_ID => {
                let state = monster.move_shape_mut().take_seal_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::Seal(state)
            }
            SPIDER_POISON_SKILL_ID => RemovedMonsterCurableState::SpiderPoison(monster.move_shape_mut().take_spider_poison_state()?),
            SPRITE_BURN_SKILL_ID => RemovedMonsterCurableState::SpriteBurn(monster.move_shape_mut().take_sprite_burn_state()?),
            SPIDER_WEB_SKILL_ID => {
                let state = monster.move_shape_mut().take_spider_web_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::SpiderWeb(state)
            }
            KNOCK_OUT_STATE_ID => {
                let state = monster.move_shape_mut().take_knock_out_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::KnockOut(state)
            }
            BOSS_BLUE_QUAKE_STATE_ID => {
                let state = monster.move_shape_mut().take_boss_blue_quake_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::BossBlueQuake(state)
            }
            KNIGHT_CUT_STATE_ID => {
                let state = monster.move_shape_mut().take_knight_cut_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::KnightCut(state)
            }
            POISON_FOG_STATE_ID => RemovedMonsterCurableState::PoisonFog(monster.move_shape_mut().take_poison_fog_state()?),
            SPIDER_MIST_SKILL_ID => {
                if !monster.remove_curable_attack_cast(SPIDER_MIST_SKILL_ID) {
                    return None;
                }
                RemovedMonsterCurableState::ActiveSpiderMist
            }
            _ => return None,
        };
        Some((removed, monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?))
    });
    game.restore_region_owner(owner);
    let Some((removed, identity, tile_x, tile_y)) = removed else { return false };
    match removed {
        RemovedMonsterCurableState::BoaLock(state) => send_boa_lock_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
        ),
        RemovedMonsterCurableState::Rush(state) => send_rush_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
        ),
        RemovedMonsterCurableState::Rush2(state) => send_rush_2_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, now_ms,
        ),
        RemovedMonsterCurableState::Seal(state) => send_seal_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
        ),
        RemovedMonsterCurableState::SpiderPoison(state) => send_spider_poison_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
        RemovedMonsterCurableState::SpriteBurn(state) => send_sprite_burn_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
        RemovedMonsterCurableState::SpiderWeb(state) => send_spider_web_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
        ),
        RemovedMonsterCurableState::KnockOut(state) => send_knock_out_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
        ),
        RemovedMonsterCurableState::BossBlueQuake(state) => send_boss_blue_quake_state_visual(
            game,
            region_id,
            identity,
            tile_x,
            tile_y,
            state,
            false,
            || now_ms,
        ),
        RemovedMonsterCurableState::KnightCut(state) => send_knight_cut_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
        ),
        RemovedMonsterCurableState::PoisonFog(state) => send_poison_fog_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
        // У `CSpiderMist` нет собственного override `CState::End`: native
        // cure-path только удаляет active state-skill из owner-а.
        RemovedMonsterCurableState::ActiveSpiderMist => {}
    }
    true
}

pub(crate) fn finish_curable_state(game: &mut CGame, region_id: i32, target: ShapeIdentity, state_id: u32, now_ms: u32) -> bool {
    match (target.object_type, state_id) {
        (PLAYER_TYPE, BOA_LOCK_STATE_ID) => {
            let removed = game.find_player_mut(target.id).and_then(|player| {
                let state = player.take_boa_lock_state()?;
                player.set_skill_moveable(true);
                Some((state, player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
            });
            let Some((state, region, x, y)) = removed else { return false };
            send_boa_lock_state_visual(game, region, target, x, y, state, false, || now_ms);
            true
        }
        (PLAYER_TYPE, RUSH_STATE_ID) => {
            let removed = game.find_player_mut(target.id).and_then(|player| {
                let state = player.take_rush_state()?;
                player.set_skill_moveable(true);
                player.set_skill_fightable(true);
                Some((state, player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
            });
            let Some((state, region, x, y)) = removed else { return false };
            send_rush_state_visual(game, region, target, x, y, state, false, || now_ms);
            true
        }
        (PLAYER_TYPE, RUSH_2_STATE_ID) => {
            let removed = game.find_player_mut(target.id).and_then(|player| {
                let state = player.take_rush_2_state()?;
                player.set_skill_moveable(true);
                player.set_skill_fightable(true);
                Some((state, player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
            });
            let Some((state, region, x, y)) = removed else { return false };
            send_rush_2_state_visual(game, region, target, x, y, state, false, now_ms);
            true
        }
        (PLAYER_TYPE, SPIDER_POISON_SKILL_ID) => finish_player_spider_poison_state_on_cure(game, target.id, now_ms),
        (PLAYER_TYPE, SPRITE_BURN_SKILL_ID) => finish_player_sprite_burn_state_on_cure(game, target.id, now_ms),
        (PLAYER_TYPE, SPIDER_WEB_SKILL_ID) => finish_player_spider_web_state_on_defense(game, target.id, now_ms),
        (PLAYER_TYPE, KNOCK_OUT_STATE_ID) => finish_player_knock_out_state_on_defense(game, target.id, now_ms),
        (PLAYER_TYPE, BOSS_BLUE_QUAKE_STATE_ID) => finish_player_boss_blue_quake_state_on_cure(game, target.id, now_ms),
        (PLAYER_TYPE, KNIGHT_CUT_STATE_ID) => finish_player_knight_cut_state_on_cure(game, target.id, now_ms),
        (PLAYER_TYPE, POISON_FOG_STATE_ID) => { let removed = game.find_player_mut(target.id).and_then(|player| { let region = player.server_region_id()?; let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?; let state = player.take_poison_fog_state()?; Some((region, x, y, state)) }); let Some((region, x, y, state)) = removed else { return false }; send_poison_fog_state_visual(game, region, target, x, y, state, false, now_ms); true },
        (MONSTER_TYPE, _) => finish_monster_curable_state(game, region_id, target.id, state_id, now_ms),
        _ => false,
    }
}

fn install_cure_state(game: &mut CGame, region_id: i32, target: &CureTarget, state: CureState) -> bool {
    match target.identity.object_type {
        PLAYER_TYPE => {
            let Some(player) = game.find_player(target.identity.id)
                .filter(|player| player.server_region_id() == Some(region_id)) else { return false };
            let old = player.cure_state();
            send_cure_state_visual(game, target.identity.id, state, true);
            if old.is_some() {
                let Some(offset) = game.find_player_mut(target.identity.id)
                    .and_then(|player| player.move_shape_mut().cure_state_replacement_offset())
                    else { return false };
                if !super::curestate::end_player_cure_state(game, target.identity.id) {
                    return false;
                }
                let Some(player) = game.find_player_mut(target.identity.id) else { return false };
                player.move_shape_mut().insert_replacement_cure_state(state, offset);
            } else if let Some(player) = game.find_player_mut(target.identity.id) {
                player.push_cure_state(state);
            }
            true
        }
        MONSTER_TYPE => {
            let Some(mut owner) = game.take_region_owner(region_id) else { return false };
            let installed = if let Some(monster) = owner.base().find_monster_by_id(target.identity.id) {
                let old = monster.move_shape().cure_state();
                let shape = monster.move_shape().shape().clone();
                super::curestate::send_cure_state_visual_in_region(game, owner.base(), &shape, state, true);
                if old.is_some() {
                    let offset = owner.base().find_monster_by_id(target.identity.id)
                        .and_then(|monster| monster.move_shape().cure_state_replacement_offset());
                    if let Some(offset) = offset {
                        if super::curestate::end_monster_cure_state_at(game, owner.base_mut(), target.identity.id, 0) {
                            owner.base_mut().find_monster_by_id_mut(target.identity.id)
                                .expect("End Cure сохраняет монстра")
                                .move_shape_mut().insert_replacement_cure_state(state, offset);
                            true
                        } else { false }
                    } else { false }
                } else {
                    owner.base_mut().find_monster_by_id_mut(target.identity.id)
                        .expect("публикация не удаляет владельца Cure")
                        .move_shape_mut().push_cure_state(state);
                    true
                }
            } else { false };
            game.restore_region_owner(owner);
            installed
        }
        _ => false,
    }
}

pub(crate) const fn is_cure_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: CURE_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: CURE_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: CURE_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }
    )
}

pub(crate) fn execute_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(CURE_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(requested_identity) = requested_target(game, region_id, dispatch, player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CURE_SKILL_ID, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(DELAY_TIME);
    let _keep_time_ms = properties.query_property(STATE_PERSIST_TIME);
    let reuse_delay_ms = properties.query_property(REUSE_DELAY_TIME);
    let constant = properties.query_property(CONST);
    let em_modifier = properties.query_property(EM_MODIFIER);
    let base_probability = properties.query_property(BASE_PROBABILITY);
    let _can_be_breaked = properties.query_property(CAN_BE_BREAKED);

    if player_ai.player_skill_execution(CURE_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let Some(initial_target) = target_snapshot(game, region_id, requested_identity) else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(player_ai.skill_last_used_ms(CURE_SKILL_ID), reuse_delay_ms, cooldown_now_ms) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = if requested_identity == caster_identity(player_id) { Vec::new() } else { game.base_magic_path(region_id, source_x, source_y, initial_target.tile_x, initial_target.tile_y, None) };
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info_with_text(player_id, b"GS0295", &initial_target.display_name);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(CURE_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_execution(CURE_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let target = match target_snapshot(game, region_id, requested_identity) {
        Some(target) => target,
        None => { abort_player_cure(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
    };
    if target.dead {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_cure(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.player_skill_execution(CURE_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_cure(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.publish_player_states(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target.tile_x, target.tile_y));
        }
        send_cast(game, player_id, &target, level, false);
        if let Some(execution) = player_ai.player_skill_execution_mut(CURE_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = player_ai.player_skill_execution(CURE_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение очищения создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }

    send_cast(game, player_id, &target, level, true);
    let element_modify = game.find_player(player_id).map(|player| player.combat_properties().element_modify).unwrap_or_default();
    let threshold = cure_threshold(element_modify, base_probability, constant, em_modifier);
    let properties_changed = game.with_published_player_ai(player_id, player_ai, |game| {
        let mut properties_changed = false;
        for state_id in curable_state_ids(game, region_id, target.identity) {
            if game.skill_random_below(100) < threshold {
                // Пакеты завершения этих состояний не содержат время; дополнительное
                // чтение часов между вызовами генератора MSVCRT исходный `CastCure` не делал.
                properties_changed |= if state_id == SPIDER_MIST_SKILL_ID
                    && target.identity.object_type == PLAYER_TYPE
                {
                    finish_active_spider_mist_on_cure(game, target.identity.id, runtime)
                } else {
                    finish_curable_state(game, region_id, target.identity, state_id, 0)
                };
            }
        }
        properties_changed
    });
    if properties_changed && target.identity.object_type == PLAYER_TYPE { let _ = game.update_player_properties(target.identity.id); }
    let installed = install_cure_state(
        game,
        region_id,
        &target,
        CureState::new(caster_identity(player_id), target.identity).begin_now(),
    );
    if let Some(execution) = player_ai.player_skill_execution_mut(CURE_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_cure(game, player_id, player_ai, runtime);
    terminal(if installed { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
