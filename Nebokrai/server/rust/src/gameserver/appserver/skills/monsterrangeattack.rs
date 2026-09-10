//! Круговая атака `CMonsterRangeAttack` (ID `0x2ef`) для игрока и монстра.
//! На время прямого удара настоящий CPlayerAI опубликован в CPlayer:
//! вложенные обработчики смерти видят и изменяют ту же очередь источника.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/skills/monsterrangeattack.cpp`. Player и monster хранят kernel
//! и reuse в собственных зарегистрированных экземплярах навыка CMoveShape.
//! Все варианты Begin центрируют эффект на источнике, а не на цели запроса.
//! CheckCastCondition (VA `0x00511cf0`) проверяет reuse; игрок дополнительно
//! требует ненулевую стоимость MP и достаточный запас. Нулевой cost отвергается
//! (VA `0x00511df4`), non-player проходит без MP (VA `0x00511de7`).
//! Первая AI-фаза повторно проверяет и списывает MP, вызывает OnChangeStates,
//! затем посылает начало без изменения направления. Ошибки reuse/MP сохраняют
//! failure 13/7 и строки GS1143/GS1144; Begin не добавляет failure 2.
//!
//! AI (VA `0x00512500`) использует абсолютный wrapping-срок delay. Fire
//! `0xbfe01` предшествует обходу подтверждённой маски 7x7 по X, затем Y.
//! Каждая клетка читается после предыдущих повреждений; IsAttackAble вызывается
//! перед дедупликацией, цель добавляется в список после Attack. Общий регион,
//! Vec и kernel заменяют только указатели, STL и хранение исполнения.
//!
//! Расчёт (VA `0x00512170`) сохраняет RNG `abs(max-min)+1`, элементальный
//! урон и x87-усечение EM-бонуса с исходной константой `0.01_f32`.
//! Игрок добавляет ElementModify/GetAddElementAtk, оружейный фактор по уровню
//! цели и личный критический множитель; для монстра эти player-ветви отсутствуют.
//! Унаследованный GetLevel построек/ворот равен 1 (VA `0x004cfb30`).
//! Защита, HP, death-script и wire используют существующих owners урона.
//! Attack (VA `0x005123e0`) не увеличивает RP атакующего, в отличие от
//! базовой атаки; RP защищающегося не меняется этим различием.
//! End (VA `0x00546090`) возвращает движение; только успешный исход
//! выполняет AfterUseSkill с износом оружия и фиксирует reuse.
//! Monster Begin не наследует поворот и стартовый пакет базовой атаки:
//! его sufferer — сам источник, kernel остаётся в Begin. Non-player
//! CheckCastCondition не блокирует движение; отказ reuse всё равно вызывает
//! End(0) и возвращает BeginRejected общему расписанию. Первый AI
//! (0x00512569..0x005125D8) посылает старт с прежним направлением, затем
//! читает свежие часы delay. Отсутствие свойств после Begin — End(0)
//! (0x00512900). Ранние отказы schedule до Begin остаются у caller-а.

//! Цепочка попадания передаёт Option владельца региона до синхронной смерти.
//! Заимствование базы не переживает эту границу; продолжение заново получает
//! оставшегося владельца, не создавая замену исчезнувшему региону.

use crate::gameserver::gameserver::game::ServerRegionOwner;

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::basemagic::{SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use crate::gameserver::appserver::ai::monsterai::MonsterSkillCallOutcome;
use crate::gameserver::appserver::monster::{MonsterBaseAttackCast, MonsterBaseAttackDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::skills::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::skills::kernel::SkillTermination;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::skills::monsterattack::{
    apply_owned_monster_attack_hit,
    defend_owned_monster_attack, monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const RANGE_SCOPE_SIDE: i32 = 7;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;

pub(crate) const MONSTER_RANGE_ATTACK_SKILL_ID: u32 = 0x2ef;

fn player_range_outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn end_player_range_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, runtime: &mut Runtime, success: bool,
) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    if success { game.after_use_player_skill(player_id, MONSTER_RANGE_ATTACK_SKILL_ID, runtime); }
}

pub(crate) fn finish_player_monster_range_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime, success: bool,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, MONSTER_RANGE_ATTACK_SKILL_ID).map(|kernel| kernel.dispatch()) else { return false };
    end_player_range_attack(game, player_id, runtime, success);
    game.finish_player_skill(player_id, ai, dispatch, if success { SkillTermination::Completed } else { SkillTermination::Cancelled })
}

fn calculate_player_range_attack(
    game: &mut CGame, player_id: i32, region_id: i32, target: ShapeIdentity,
    level: i32, properties: &CSkillBaseProperties,
) -> Option<(crate::gameserver::appserver::masterinfo::MasterInfo, AttackInformation)> {
    use crate::gameserver::appserver::states::attackpower::{AttackPower, AttackPowerType};
    use super::fightdefense::truncate_original;
    let target_level = if matches!(target.object_type, 1100 | 1200) { 1 }
        else { super::flash::target_level(game, region_id, target)? };
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = super::lordfastattack::master_info(player);
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let damage_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let bonus = truncate_original(f64::from(properties.query_property(SKILL_USAGE_EM_MODIFIER))
        * f64::from(0.01_f32) * f64::from(combat.element_modify));
    let damage = (combat.add_element_attack as i32).wrapping_add(minimum)
        .wrapping_add(game.skill_random_below(span)).wrapping_add(bonus).max(0);
    let critical = game.skill_random_below(100) < i32::from(combat.cch);
    let damage = if critical { truncate_original(f64::from(damage) * f64::from(combat.critical_rate())) } else { damage };
    Some((master, AttackInformation {
        skill_id: MONSTER_RANGE_ATTACK_SKILL_ID, skill_level: level as u8,
        attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor, damage_modifier: 0, critical, blast_attack: false, full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    }))
}

pub(crate) fn execute_player_monster_range_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
    use super::kernel::skill_is_restored;
    let rejected = || player_range_outcome(QueuedSkillExecutionState::Rejected);
    if dispatch.skill_id() != MONSTER_RANGE_ATTACK_SKILL_ID { return rejected(); }
    let Some((region_id, level, mana)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.learned_skill_level(MONSTER_RANGE_ATTACK_SKILL_ID, game.skill_factory()), player.mana()))
    }) else { return rejected() };
    let Some(properties) = game.skill_base_properties(MONSTER_RANGE_ATTACK_SKILL_ID, level).cloned() else {
        end_player_range_attack(game, player_id, runtime, false);
        return rejected();
    };
    let mp_loss = properties.query_property(2);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(super::basemagic::SKILL_USAGE_CAN_BE_BREAKED);
    let mp_failure = |game: &CGame| {
        game.send_self_state_skill_failure(0x000b_fe01, player_id, 7);
        game.send_skill_system_info_with_unsigned(player_id, b"GS1144", mp_loss);
    };
    if game.player_skill_execution(player_id, MONSTER_RANGE_ATTACK_SKILL_ID).is_none() {
        let now = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, MONSTER_RANGE_ATTACK_SKILL_ID), reuse, now) {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 13);
            game.send_skill_system_info(player_id, b"GS1143");
            end_player_range_attack(game, player_id, runtime, false);
            return rejected();
        }
        if mp_loss == 0 || (mana.wrapping_sub(mp_loss) as i32) < 0 {
            if mp_loss != 0 { mp_failure(game); }
            end_player_range_attack(game, player_id, runtime, false);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(MONSTER_RANGE_ATTACK_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now));
        return player_range_outcome(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, MONSTER_RANGE_ATTACK_SKILL_ID).is_none_or(|kernel| kernel.dispatch() != dispatch) {
        return rejected();
    }
    if game.player_skill_execution(player_id, MONSTER_RANGE_ATTACK_SKILL_ID).is_some_and(|kernel| kernel.stage() == SkillStage::Begin) {
        let remaining = game.find_player(player_id).map_or(0, |player| player.mana()).wrapping_sub(mp_loss);
        if (remaining as i32) < 0 {
            mp_failure(game);
            end_player_range_attack(game, player_id, runtime, false);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(remaining); }
        let _ = game.publish_player_states(player_id);
        super::lordfastattack::send_start(game, player_id, MONSTER_RANGE_ATTACK_SKILL_ID, level);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, MONSTER_RANGE_ATTACK_SKILL_ID) { let _ = kernel.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = game.player_skill_execution(player_id, MONSTER_RANGE_ATTACK_SKILL_ID).map(|kernel| kernel.started_at_ms()).expect("круговая атака хранит начало");
    if !skill_is_restored(started, delay, runtime.now_milliseconds()) {
        return player_range_outcome(QueuedSkillExecutionState::Pending);
    }
    let Some(view) = game.find_player(player_id).and_then(|player| player.shape_view()) else {
        end_player_range_attack(game, player_id, runtime, false);
        return rejected();
    };
    let mut fire = CMessage::new(0x000b_fe01);
    fire.add_byte(2); fire.add_long(MONSTER_RANGE_ATTACK_SKILL_ID as i32); fire.add_short(level as i16);
    fire.add_long(PLAYER_TYPE); fire.add_long(player_id); fire.add_long(0); fire.add_long(0);
    fire.add_long(view.tile_x); fire.add_long(view.tile_y);
    let _ = game.send_player_shape_around(player_id, None, &fire);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, MONSTER_RANGE_ATTACK_SKILL_ID) { let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate); }
    let mut attacked = Vec::new();
    for (dx, dy) in range_attack_scope_cells() {
        for target in super::flash::cell_views(game, region_id, view.tile_x.wrapping_add(dx), view.tile_y.wrapping_add(dy)) {
            let target = target.identity;
            let Some(master) = game.find_player(player_id).map(super::lordfastattack::master_info) else { break };
            let attackable = if matches!(target.object_type, 1100 | 1200) {
                game.stationary_build_attackable_by_player(player_id, region_id, target)
            } else { game.owned_player_skill_target_attackable(master, target, region_id) };
            if !attackable || attacked.contains(&target) { continue; }
            if !(target.object_type == PLAYER_TYPE && target.id == player_id)
                && let Some((master, attack)) = calculate_player_range_attack(game, player_id, region_id, target, level, &properties)
            {
                match target.object_type {
                    PLAYER_TYPE => game.with_published_player_ai(player_id, player_ai, |game| game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime)),
                    MONSTER_TYPE => game.with_published_player_ai(player_id, player_ai, |game| game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime)),
                    1100 | 1200 => game.with_published_player_ai(player_id, player_ai, |game| game.apply_owned_skill_attack_to_stationary_build(player_id, region_id, target, attack, runtime)),
                    _ => {}
                }
            }
            attacked.push(target);
        }
    }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, MONSTER_RANGE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    end_player_range_attack(game, player_id, runtime, true);
    player_range_outcome(QueuedSkillExecutionState::Completed)
}

// `g_bScope` по адресу 0x006A0ECC при `g_dwLength/g_dwHeight == 7`.
const RANGE_SCOPE: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0,
    0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
];

pub(crate) fn range_attack_fire_message(
    skill_level: u16,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(MONSTER_RANGE_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

pub(crate) fn range_attack_scope_cells() -> impl Iterator<Item = (i32, i32)> {
    (0..RANGE_SCOPE_SIDE).flat_map(|x| {
        (0..RANGE_SCOPE_SIDE).filter_map(move |y| {
            let index = (x + RANGE_SCOPE_SIDE * y) as usize;
            (RANGE_SCOPE[index] != 0).then_some((x - 3, y - 3))
        })
    })
}

/// Возвращает исходный упорядоченный снимок одной клетки. Следующая клетка
/// читается только после применения предыдущих ударов и их последствий смерти.
pub(crate) fn range_attack_cell_candidates(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    monster_attack_cell_candidates(game, region, monster_id, tile_x, tile_y)
}

pub(crate) fn calculate_monster_range_attack(
    properties: &CSkillBaseProperties,
    skill_level: u16,
    monster_id: i32,
    random_below: &mut dyn FnMut(i32) -> i32,
) -> crate::gameserver::appserver::states::attackpower::AttackInformation {
    use crate::gameserver::appserver::states::attackpower::{
        AttackInformation, AttackPower, AttackPowerType,
    };

    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let skill_span = maximum
        .wrapping_sub(minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let skill_damage = minimum.wrapping_add(random_below(skill_span));
    let _element_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER);

    AttackInformation {
        skill_id: MONSTER_RANGE_ATTACK_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            // Виртуальный `CMonster::GetAddElementAtk` возвращает ноль;
            // `SKILL_USAGE_EM_MODIFIER` умножается на нулевой ElementModify.
            hp_damage: skill_damage.max(0),
            mp_damage: 0,
        }],
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MonsterRangeAttackDispatch {
    pub(crate) monster_id: i32,
    pub(crate) skill_level: u16,
    properties: CSkillBaseProperties,
    property: crate::setup::monsterlist::MonsterProperties,
    attacker_master: crate::gameserver::appserver::masterinfo::MasterInfo,
    attacker_tamed: bool,
    pub(crate) center_x: i32,
    pub(crate) center_y: i32,
    now_ms: u32,
}

pub(crate) fn begin_owned_monster_range_cast<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    started_at_ms: u32,
    factory: &super::skillfactory::CSkillFactory,
    runtime: &mut Runtime,
) -> MonsterSkillCallOutcome {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return MonsterSkillCallOutcome::NotHandled;
    };
    if !super::kernel::skill_is_restored(
        monster.skill_last_used_ms(MONSTER_RANGE_ATTACK_SKILL_ID, factory),
        properties.query_property(super::baseattack::SKILL_USAGE_REUSE_DELAY_TIME),
        runtime.now_milliseconds(),
    ) {
        monster.move_shape_mut().set_moveable(true);
        return MonsterSkillCallOutcome::BeginRejected;
    }
    let target = monster.move_shape().shape().identity();
    let target_object = Some((monster.move_shape().shape().get_region_id(), target));
    monster.install_base_attack_cast(MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
        target,
        skill_id: MONSTER_RANGE_ATTACK_SKILL_ID,
        skill_level,
    }, started_at_ms), target_object, factory);
    MonsterSkillCallOutcome::Handled
}

pub(crate) fn prepare_owned_monster_range_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    dispatch: &mut Option<MonsterRangeAttackDispatch>,
) -> bool {
    let Some((shape, property, cast, attacker_master, attacker_tamed)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.current_active_attack_cast(game.skill_factory())?,
                monster.master_info(),
                monster.is_tamed(),
            ))
        })
    else {
        return false;
    };
    if cast.dispatch().skill_id != MONSTER_RANGE_ATTACK_SKILL_ID {
        return false;
    }
    if cast.stage() == SkillStage::Begin {
        let mut start = CMessage::new(0x000b_fe01);
        start.add_byte(1);
        start.add_long(MONSTER_RANGE_ATTACK_SKILL_ID as i32);
        start.add_short(cast.dispatch().skill_level as i16);
        start.add_long(MONSTER_TYPE);
        start.add_long(monster_id);
        start.add_long(shape.get_direction());
        let _ = game.send_game_shape_around(region, &shape, None, &start);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(MONSTER_RANGE_ATTACK_SKILL_ID, SkillStage::Begin, SkillStage::Check, game.skill_factory());
        }
    }
    let delay_ms = properties.query_property(super::baseattack::SKILL_USAGE_DELAY_TIME);
    if !super::kernel::skill_is_restored(cast.started_at_ms(), delay_ms, runtime.now_milliseconds()) {
        return true;
    }
    let (Ok(tile_x), Ok(tile_y)) = (shape.get_tile_x(), shape.get_tile_y()) else {
        return true;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(MONSTER_RANGE_ATTACK_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
    }
    let fire = range_attack_fire_message(
        cast.dispatch().skill_level,
        monster_id,
        tile_x,
        tile_y,
    );
    let _ = game.send_game_shape_around(region, &shape, None, &fire);
    *dispatch = Some(MonsterRangeAttackDispatch {
        monster_id,
        skill_level: cast.dispatch().skill_level,
        properties: properties.clone(),
        property,
        attacker_master,
        attacker_tamed,
        center_x: tile_x,
        center_y: tile_y,
        now_ms,
    });
    true
}

pub(crate) fn execute_owned_monster_range_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    dispatch: &MonsterRangeAttackDispatch,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
) -> bool {
    let Some(region) = owner.as_mut().map(ServerRegionOwner::base_mut) else { return false; };
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return false;
    };
    if target.dead || target.god || target.city_dead {
        return false;
    }
    if !owned_monster_attackable(
        game,
        region.id,
        &dispatch.property,
        dispatch.attacker_tamed,
        dispatch.attacker_master,
        identity,
        &target,
    ) {
        return false;
    }
    let mut random = |maximum| game.skill_random_below(maximum);
    let attack = calculate_monster_range_attack(
        &dispatch.properties,
        dispatch.skill_level,
        dispatch.monster_id,
        &mut random,
    );
    let attack = defend_owned_monster_attack(
        game,
        identity,
        target.mana,
        target.war_soul_mana,
        target.player_properties,
        target.monster_properties,
        attack,
    );
    apply_owned_monster_attack_hit(
        game,
        owner,
        runtime,
        dispatch.now_ms,
        dispatch.monster_id,
        dispatch.attacker_master,
        identity,
        &target.shape,
        target.health,
        target.mana,
        target.master,
        target.monster_property,
        target.tamed,
        target.carriage,
        attack,
    );
    true
}
