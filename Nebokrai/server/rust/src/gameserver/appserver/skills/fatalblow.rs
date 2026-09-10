//! Смертельный удар боевого духа `CFatalBlow` (`0x21C`).
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fatalblow.cpp`. Владелец сохраняет проверки цели и пути,
//! задержку повторного использования, расход MP боевого духа, время подготовки,
//! визуальные пакеты и создание
//! `CFatalBlowPhalanx`. `CGame` предоставляет владельцев, регион, регистрацию
//! снаряда и доставку. Координатный и пустой `Begin` сохраняют исходный отказ
//! `10 + ZHGS0045` и завершающий visual action `3`; затем планировщик выдаёт
//! общий 4,2. Сам Begin (0x0051e3c0) не выдаёт ранний 4,2, в отличие от
//! BloodLoss/PoisonArrow. Отказ уже запущенного AI не получает ответ расписания.
//! В Rust единственный внешний 4,2 отправляет координатор после общего End(0);
//! concrete Begin сохраняет собственную диагностику, а visual action 3
//! принадлежит общему End зарегистрированного экземпляра.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; подготовка
//! сравнивает unsigned now с wrapping(start + delay), cmp/jb 0x0051f330.
//! После virtual Summon(+0x90) в 0x0051f53d AI без проверки результата
//! вызывает End(1) в 0x0051f549; отказ создания не отменяет AfterUse/reuse.
//! Visual Update(mode=1, wire action 2) в AI (0x0051f530) предшествует Summon. Отсутствие
//! GetWarSoulGoods внутри Summon (0x0051f701..0x0051f708 → 0x0051f8f6)
//! возвращает 0 без 4,2; вызывающий AI всё равно выполняет End(1).
//! Lifetime снаряда принадлежит отдельному владельцу.
//! CFatalBlowEffect занимает только базовые 0xC байт: allocation 0x0051E3EC,
//! CVisualEffect constructor 0x0051E406, vtable 0x00656EC4. Присваивание
//! skill+0x34 и BeginVisualEffect(1) (0x0051E426) предшествуют CheckCastCondition
//! (0x0051E42E); две другие перегрузки Begin сохраняют тот же порядок.
//! Ресурс принадлежит навыку и доступен диагностике failed Begin без payload.
//! Update (0x0051E480) требует свой concrete skill, !Effect.ended и GetUser;
//! общий base-tail 0x0051E98A вызывается даже при проваленных gates. Mode 1
//! читает flying-time из skill+0x4C (0x0051E61E), поэтому скаляр хранится в
//! FatalBlowExecutionState, а не копируется в Effect вместе с source/target.
//! End(int) 0x0051E370 обнуляет +0x54/+0x50 и flying-time +0x4C до mode 3
//! и AfterUse (0x0051E37A..0x0051E380). Общий BF-пролог выключает concrete
//! фазу через Idle, а этот hook очищает только flying-time; base ended,
//! source/target и ресурс сохраняются до общего хвоста End.
//! Native временно меняет type источника на 700 и возвращает 400 в 0x0051E97A.
//! Around (0x0041460F) определяет CPlayer через RTTI, не это поле; общий
//! publisher пишет 700 в wire и сохраняет живую форму для дальнего team-tail.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairytransfer::send_goods_update;
use super::fatalblowphalanx::CFatalBlowPhalanx;
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, BattleFairyExecution,
    SkillExecutionKernel, SkillStage,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const FATAL_BLOW_SKILL_ID: u32 = 0x21c;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0x67;
const DENIED_STATE_C: u32 = 0xd2;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FatalBlowExecutionState {
    kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    missile_flying_time: u32,
}

impl FatalBlowExecutionState {
    pub(crate) const fn begin(dispatch: BattleFairySkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            missile_flying_time: 0,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<BattleFairySkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn missile_flying_time(&self) -> u32 {
        self.missile_flying_time
    }

    pub(crate) const fn set_missile_flying_time(&mut self, missile_flying_time: u32) {
        self.missile_flying_time = missile_flying_time;
    }

    pub(crate) fn prepare_derived_end(&mut self) {
        self.missile_flying_time = 0;
    }
}

fn reject(
    game: &mut CGame,
    player_id: i32,
    action: u8,
    string_id: &[u8],
) -> QueuedSkillExecutionOutcome {
    game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, u32::from(action));
    if !string_id.is_empty() {
        game.send_skill_system_info(player_id, string_id);
    }
    terminal(QueuedSkillExecutionState::Rejected)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

pub(crate) fn execute_battle_fairy_fatal_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let reject_before_ai = |game: &mut CGame, action: u8, text: &[u8]| {
        if action != 2 { game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, u32::from(action)); }
        if !text.is_empty() { game.send_skill_system_info(player_id, text); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let (skill_level, target) = match dispatch {
        BattleFairySkillDispatch::SelfTarget {
            skill_id: FATAL_BLOW_SKILL_ID,
            ..
        } | BattleFairySkillDispatch::Point {
            skill_id: FATAL_BLOW_SKILL_ID,
            ..
        } => return reject_before_ai(game, 10, b"ZHGS0045"),
        BattleFairySkillDispatch::Object {
            skill_id: FATAL_BLOW_SKILL_ID,
            skill_level,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => (skill_level, target),
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(FATAL_BLOW_SKILL_ID, skill_level) else {
        return reject_before_ai(game, 2, b"");
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let missile_step_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let damage_factor = properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32;
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let skill_name = properties.skill_name().to_vec();
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.battle_fairy_execution(player_id, FATAL_BLOW_SKILL_ID).is_none() {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            return reject_before_ai(game, 10, b"ZHGS0045");
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
            || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B)
        {
            game.send_skill_system_info(player_id, b"ZHGS0046");
            return reject_before_ai(game, 2, b"");
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C) {
            game.send_skill_system_info(player_id, b"ZHGS0047");
            return reject_before_ai(game, 2, b"");
        }
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.battle_fairy_skill_last_used_ms(player_id, FATAL_BLOW_SKILL_ID), cooldown_ms, cooldown_now_ms) {
            return reject_before_ai(game, 0x0d, b"ZHGS0048");
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            return reject_before_ai(game, 10, b"ZHGS0045");
        };
        let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source_view.tile_x,
            source_view.tile_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            return reject_before_ai(game, 0x0b, b"ZHGS0049");
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, 0x0f);
            game.send_skill_system_info_with_text(
                player_id,
                b"ZHGS0051",
                game.periodic_state_target_name(region_id, target),
            );
            return reject_before_ai(game, 2, b"");
        }
        let war_soul_mana = game
            .find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()));
        let Some(war_soul_mana) = war_soul_mana else {
            return reject_before_ai(game, 2, b"");
        };
        if mp_loss != 0 {
            if i64::from(war_soul_mana) - i64::from(mp_loss) < 0 {
                game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    battle_fairy_mana_text_cost(mp_loss),
                );
                return reject_before_ai(game, 2, b"");
            }
        }
        game.insert_battle_fairy_execution(player_id, FatalBlowExecutionState::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.battle_fairy_execution(player_id, FATAL_BLOW_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.battle_fairy_execution(player_id, FATAL_BLOW_SKILL_ID)
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        if game.periodic_state_target_dead(region_id, target) {
            return reject(game, player_id, 10, b"ZHGS0050");
        }
        if mp_loss != 0 {
            let goods_factory = game.goods_factory().clone();
            let da_kong_key = game.globe_setup().da_kong_key();
            let update = game.find_player_mut(player_id).and_then(|player| {
                player.spend_war_soul_mana(mp_loss, &goods_factory, da_kong_key)
            });
            let Some(update) = update else {
                game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    battle_fairy_mana_text_cost(mp_loss),
                );
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            send_goods_update(game, &update);
        }
        game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, 0);
        if let Some(execution) = game.battle_fairy_execution_mut(player_id, FATAL_BLOW_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.battle_fairy_execution(player_id, FATAL_BLOW_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение смертельного удара создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        return reject(game, player_id, 10, b"ZHGS0050");
    };
    if game.periodic_state_target_dead(region_id, target) {
        return reject(game, player_id, 10, b"ZHGS0050");
    }
    let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
        None,
    );
    if maximum_distance != 0 && path.len() > maximum_distance as usize {
        return reject(game, player_id, 0x0b, b"ZHGS0049");
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, 0x0f);
        game.send_skill_system_info_with_text(player_id, b"ZHGS0053", &skill_name);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let missile_flying_time = missile_step_ms.wrapping_mul(path.len() as u32);
    if let Some(BattleFairyExecution::FatalBlow(state)) = game.battle_fairy_execution_state_mut(player_id, FATAL_BLOW_SKILL_ID) {
        state.set_missile_flying_time(missile_flying_time);
    }
    game.update_player_skill_visual(player_id, FATAL_BLOW_SKILL_ID, 1);
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    };
    if player.war_soul_goods(game.goods_factory()).is_none() {
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    }
    let master = master_info(player);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CFatalBlowPhalanx::new(
        summon_id,
        master,
        summon_started_at_ms,
        lifetime_ms,
        skill_level,
        damage_factor,
        target,
        minimum_attack,
        maximum_attack,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let result = game.add_fatal_blow_phalanx(
        region_id,
        phalanx,
        target_view.tile_x,
        target_view.tile_y,
        summon_started_at_ms,
        runtime,
    );
    let summoned = result.is_some_and(|result| result.is_ok());
    if summoned {
        let _ = game.send_fatal_blow_phalanx_entry(region_id, summon_id, runtime);
    }
    if let Some(execution) = game.battle_fairy_execution_mut(player_id, FATAL_BLOW_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::RejectedAfterUse
    })
}
