//! Круговая атака `CMonsterRangeAttack` (ID `0x2ef`): player-путь
//! (Check/AI/Calc) и монстровые Begin/prepare/контакт маски 7x7 вокруг
//! источника. На время прямого удара настоящий CPlayerAI опубликован в
//! CPlayer: вложенные обработчики смерти видят и изменяют ту же очередь
//! источника.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/monsterrangeattack.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/monsterrangeattack.rs` (кластер A2
//! полосы Monster, 26 сентября 2026); hub оркестрации монстра остаётся у
//! `execute_owned_monster_base_attack` прежнего пакета.
//!
//! Машинная база по этой паре (VERIFIED, тела `.local/recon-a2/out/`);
//! сопоставление с перенесённым кодом — MATCH по всем пунктам:
//!
//! - ctor (RVA `0x111590`): `[+4] = 0x2ef`; статические
//!   `g_dwLength/g_dwHeight == 7`, маска `g_bScope` по `0x006A0ECC`.
//! - CheckCastCondition (RVA `0x111CF0`): param null / props null → ret 0
//!   без кадра; reuse (10005) → `{0xBFE01, 0, 13}` + GS1143 только у
//!   dyn-CPlayer; **non-player проходит без MP и без SetMoveable** (je
//!   0x511E1F); player: **нулевая стоимость MP → молчаливый ret 0** (jbe
//!   0x511EA1), MP < cost → `{0, 7}` + GS1144(`%u`) → ret 0; успех —
//!   SetMoveable(param, 0).
//! - AI (RVA `0x112500`): `[+0x4C] == 0` → выход; props null → `End(0)`
//!   (0x512900); U null → `End(0)`; первая фаза повторно списывает MP
//!   только у dyn-CPlayer (`MP - Query(2)`, js → `{0, 7}` + GS + End(0);
//!   `SetMP` + vt+0x164 OnChangeStates), start-кадр (mode 0) **без
//!   поворота**, `[+0x3C] = QueryProperty(10006)`; delay — абсолютный
//!   unsigned-срок `[+0x2C] + QueryProperty(10001)`; fire-кадр (mode 1) с
//!   центром U; регион из `[U+0x40]` (null → `End(1)`); обход
//!   g_dwLength×g_dwHeight: X внешний, Y внутренний, индекс маски **x+7y**,
//!   начало `tile - 3`; клетка читается после предыдущих ударов;
//!   `IsAttackAble(U)` кандидата **перед** дедупликацией; цель добавляется
//!   в список **после** Attack; конец обхода — `End(1)` со штампом reuse.
//! - Calculate (RVA `0x112170`): id/уровень записываются в seed; hit =
//!   `Query(20001)`; damage_factor = `U->vt+0x184(S->vt+0x110())` float;
//!   **span `abs(max-min) + 1`, вид 3**; значение = AddElementAtk(U) +
//!   min(20008) + random(span из 20009−20008) + **trunc(unsigned(EM=20015)
//!   × 0.01f(const 0x64DBD0) × ElementModify)** (unsigned-переход через
//!   fild+fadd 2^32, `fimul` по EC); критический roll только у dyn-CPlayer
//!   (`random(100) < vt+0x114`, `×[player+0x414]` с x87-усечением); монстр
//!   player-ветвей не выполняет — его ElementModify/AddElementAtk == 0.
//! - Attack (RVA `0x1123E0`): RP атакующего **не** увеличивается (в отличие
//!   от базовой атаки семьи). End общего владельца 0x146090: нули фаз,
//!   SetMoveable(U, 1), аргумент в CAttackSkill::End.
//!
//! Объявленные швы переноса: hub-трейты `skills/monsterattack.rs`; derived
//! `g_dwLength/g_dwHeight` read через константы ниже; часы — fn-параметр
//! `now_milliseconds` делегата старого main loop.

use crate::app::game_message::CMessage;
use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original,
};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::baseattackruntime::{
    BaseAttackContact, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER,
};
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillStage, SkillTermination, skill_is_restored};
use super::monsterattack::{
    MonsterCombatContact, MonsterCombatGame, MonsterCombatOutcome, MonsterCombatPlayer,
    apply_owned_monster_attack_hit, monster_attack_cell_candidates,
    resolve_owned_monster_attack_target,
};
use crate::ai::monsterai::MonsterSkillCallOutcome;

pub const MONSTER_RANGE_ATTACK_SKILL_ID: u32 = 0x2ef;

pub const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;

const RANGE_SCOPE_SIDE: i32 = 7;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;

const CAST_VISUAL_MESSAGE: i32 = 0x000b_fe01;

/// `g_bScope` по адресу 0x006A0ECC при `g_dwLength/g_dwHeight == 7`.
const RANGE_SCOPE: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0,
    0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
];

/// Порядок обхода маски AI: X внешний, Y внутренний, индекс `x + 7y`,
/// начало окна на (tile−3, tile−3) от источника.
pub fn range_attack_scope_cells() -> impl Iterator<Item = (i32, i32)> {
    (0..RANGE_SCOPE_SIDE).flat_map(|x| {
        (0..RANGE_SCOPE_SIDE).filter_map(move |y| {
            let index = (x + RANGE_SCOPE_SIDE * y) as usize;
            (RANGE_SCOPE[index] != 0).then_some((x - 3, y - 3))
        })
    })
}

/// Кадр выпуска `0x000BFE01` круговой атаки: action 2, нулевая объектная
/// часть и центр источника (message-owner mode 1; fire user-center).
pub fn range_attack_fire_message(
    skill_level: u16,
    actor_type: i32,
    actor_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(CAST_VISUAL_MESSAGE);
    message.add_byte(2);
    message.add_long(MONSTER_RANGE_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

/// Кадр начала круговой атаки: action 1 и направление «как есть» (AI не
/// поворачивает источника круговой атаки — машинный старт без SetDir).
fn range_cast_start_message(skill_level: i32, actor_type: i32, actor_id: i32, direction: i32) -> CMessage {
    let mut message = CMessage::new(CAST_VISUAL_MESSAGE);
    message.add_byte(1);
    message.add_long(MONSTER_RANGE_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    message.add_long(direction);
    message
}

/// Исходный упорядоченный снимок одной клетки маски монстра. Следующая
/// клетка читается только после применения предыдущих ударов и их
/// последствий смерти, как у прежнего владельца.
pub fn range_attack_cell_candidates<Game: MonsterCombatGame>(
    game: &Game,
    region_owner: &Game::RegionOwner,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    monster_attack_cell_candidates(game, region_owner, monster_id, tile_x, tile_y)
}

/// Расчёт кругового удара монстра: порядок чтений машинного Calculate —
/// span `abs(max-min)+1`, элементальный вид 3; виртуальный
/// `CMonster::GetAddElementAtk` и ElementModify равны нулю, поэтому
/// EM-бонус и критические ветви player здесь не разыгрываются.
pub fn calculate_monster_range_attack(
    properties: &CSkillBaseProperties,
    skill_level: u16,
    monster_id: i32,
    random_below: &mut dyn FnMut(i32) -> i32,
) -> AttackInformation {
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
            hp_damage: skill_damage.max(0),
            mp_damage: 0,
        }],
    }
}

/// Живой снимок разрешённого fire круговой атаки монстра: центр окна и
/// уровень записаны тиком fire; таблица хранится здесь до применения хвоста
/// hub-планировщика (обход owner End(0) при потере свойств — 0x512900).
#[derive(Clone, Debug)]
pub struct MonsterRangeAttackDispatch {
    pub monster_id: i32,
    pub skill_level: u16,
    properties: CSkillBaseProperties,
    pub center_x: i32,
    pub center_y: i32,
}

/// Begin монстровой круговой атаки: sufferer — сам источник, reuse-момент
/// отклоняет как `BeginRejected` с возвратом движения (non-player Check не
/// блокирует движение машинно); kernel остаётся в Begin без поворота и
/// стартового пакета этого owner-а.
pub fn begin_owned_monster_range_cast<Game: MonsterCombatGame>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    started_at_ms: u32,
    now_milliseconds: fn() -> u32,
) -> MonsterSkillCallOutcome {
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(facts) = game.monster_combat_facts(region, monster_id, MONSTER_RANGE_ATTACK_SKILL_ID) else {
        return MonsterSkillCallOutcome::NotHandled;
    };
    if !skill_is_restored(facts.last_used_ms, reuse, now_milliseconds()) {
        game.monster_set_moveable(region, monster_id, true);
        return MonsterSkillCallOutcome::BeginRejected;
    }
    let target = facts.source.identity();
    let target_object = Some((facts.source.get_region_id(), target));
    game.monster_install_cast(
        region, monster_id, target, MONSTER_RANGE_ATTACK_SKILL_ID, skill_level,
        started_at_ms, target_object,
    );
    MonsterSkillCallOutcome::Handled
}

/// Продолжение монстровой круговой атаки из активного cast: старт с прежним
/// направлением ровно один тик, затем delay-absolute и fire с центром U.
/// Отсутствие свойств после Begin — End(0) у caller-а hub-владельца.
pub fn prepare_owned_monster_range_cast<Game: MonsterCombatGame>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
    properties: &CSkillBaseProperties,
    dispatch: &mut Option<MonsterRangeAttackDispatch>,
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(facts) = game.monster_combat_facts(region, monster_id, MONSTER_RANGE_ATTACK_SKILL_ID) else {
        return false;
    };
    let Some(cast) = facts.cast else { return false; };
    if cast.skill_id != MONSTER_RANGE_ATTACK_SKILL_ID {
        return false;
    }
    if cast.stage == SkillStage::Begin {
        // Старт-кадр один тик с направлением «как есть» (AI без поворота),
        // затем свежие часы delay, как 0x5125D8..0x512602 машинного AI.
        let start = range_cast_start_message(
            i32::from(cast.skill_level), MONSTER_TYPE, monster_id, facts.source.get_direction(),
        );
        game.send_visual_around(region, &facts.source, &start);
        game.monster_advance_cast(
            region, monster_id, MONSTER_RANGE_ATTACK_SKILL_ID, SkillStage::Begin, SkillStage::Check,
        );
    }
    if !skill_is_restored(cast.started_at_ms, properties.query_property(SKILL_USAGE_DELAY_TIME), now_milliseconds()) {
        return true;
    }
    let (Ok(tile_x), Ok(tile_y)) = (facts.source.get_tile_x(), facts.source.get_tile_y()) else {
        return true;
    };
    game.monster_advance_cast(
        region, monster_id, MONSTER_RANGE_ATTACK_SKILL_ID, SkillStage::Check, SkillStage::Calculate,
    );
    let fire = range_attack_fire_message(cast.skill_level, MONSTER_TYPE, monster_id, tile_x, tile_y);
    game.send_visual_around(region, &facts.source, &fire);
    *dispatch = Some(MonsterRangeAttackDispatch {
        monster_id,
        skill_level: cast.skill_level,
        properties: properties.clone(),
        center_x: tile_x,
        center_y: tile_y,
    });
    true
}

/// Контакт одной живой цели маски монстра: повторное разрешение на каждом
/// такте, IsAttackAble перед дедупликацией, цель добавляется в список после
/// Attack (caller хранит обход и вектор `attacked`).
pub fn execute_owned_monster_range_target<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    dispatch: &MonsterRangeAttackDispatch,
    identity: ShapeIdentity,
    attacked: &[ShapeIdentity],
    runtime: &mut Runtime,
) -> bool
where
    Game: MonsterCombatContact<Runtime> + BaseAttackContact<Runtime>,
{
    let Some(region_owner) = owner.as_ref() else { return false; };
    if resolve_owned_monster_attack_target(game, region_owner, identity).is_none() {
        return false;
    }
    let source = ShapeIdentity {
        object_type: MONSTER_TYPE,
        id: dispatch.monster_id,
        ex_id: nebokrai_shared::values::CGuid::GUID_INVALID,
    };
    if !game.live_skill_target_attackable_in(region_owner, source, identity) {
        return false;
    }
    if attacked.contains(&identity) {
        return false;
    }
    let mut random = |maximum| game.skill_random_below(maximum);
    let attack = calculate_monster_range_attack(
        &dispatch.properties,
        dispatch.skill_level,
        dispatch.monster_id,
        &mut random,
    );
    apply_owned_monster_attack_hit(game, owner, runtime, identity, attack);
    true
}

/// Player-расчёт кругового удара: уровень построек/ворот унаследован равным
/// 1 (0x004CFB30); оружейный фактор по уровню цели, `trunc(unsigned(EM)
/// × 0.01f × ElementModify)` и личный критический множитель `[player+0x414]`.
fn calculate_player_range_attack<Game: MonsterCombatGame>(
    game: &mut Game,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    properties: &CSkillBaseProperties,
) -> Option<(MasterInfo, AttackInformation)> {
    let target_level = if matches!(target.object_type, 1_100 | 1_200) {
        1
    } else {
        game.monster_combat_target_level(region_id, target)?
    };
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = player.master_info();
    let damage_factor = game.monster_combat_weapon_modifier(player_id, i32::from(target_level));
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let bonus = truncate_original(
        f64::from(properties.query_property(SKILL_USAGE_EM_MODIFIER))
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let damage = (combat.add_element_attack as i32)
        .wrapping_add(minimum)
        .wrapping_add(game.skill_random_below(span))
        .wrapping_add(bonus)
        .max(0);
    let critical = game.skill_random_below(100) < i32::from(combat.cch);
    let damage = if critical {
        truncate_original(f64::from(damage) * f64::from(combat.critical_rate()))
    } else {
        damage
    };
    Some((master, AttackInformation {
        skill_id: MONSTER_RANGE_ATTACK_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor,
        damage_modifier: 0,
        critical,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    }))
}

/// Хвост исхода player-ветви: движение возвращается всегда (derived End
/// 0x146090 вызывает SetMoveable(U, 1) до CAttackSkill::End), успех —
/// `AfterUseSkill` с износом оружия и reuse-штампом.
fn end_player_cast<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    runtime: &mut Runtime,
    success: bool,
) where
    Game: MonsterCombatContact<Runtime>,
{
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    if success {
        game.monster_combat_after_use_player_skill(player_id, MONSTER_RANGE_ATTACK_SKILL_ID, runtime);
    }
}

/// Внешнее завершение player-cast по dispatch регистра.
pub fn finish_player_monster_range_attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    success: bool,
) -> bool
where
    Game: MonsterCombatContact<Runtime>,
{
    let Some(dispatch) = game
        .player_kernel(player_id, MONSTER_RANGE_ATTACK_SKILL_ID)
        .map(|kernel| kernel.dispatch())
    else {
        return false;
    };
    end_player_cast(game, player_id, runtime, success);
    game.finish_monster_player_skill(
        player_id,
        player_ai,
        dispatch,
        if success { SkillTermination::Completed } else { SkillTermination::Cancelled },
    )
}

/// Player-вход `0x2ef`: reuse с failure 13+GS1143, нулевая стоимость MP —
/// молчаливый отказ (jbe машинного Check), повторное списание MP в первой AI-
/// фазе с failure 7+GS1144, delay-absolute, fire с центром игрока и обход
/// маски с IsAttackAble до дедупликации и записью цели после попадания.
pub fn execute_player_monster_range_attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> MonsterCombatOutcome
where
    Game: MonsterCombatContact<Runtime>,
{
    if dispatch.skill_id() != MONSTER_RANGE_ATTACK_SKILL_ID {
        return MonsterCombatOutcome::Rejected;
    }
    let Some((region_id, level, mana)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            game.monster_combat_player_skill_level(player_id, MONSTER_RANGE_ATTACK_SKILL_ID)?,
            player.mana(),
        ))
    }) else {
        return MonsterCombatOutcome::Rejected;
    };
    let Some(properties) = game
        .skill_base_properties(MONSTER_RANGE_ATTACK_SKILL_ID, level)
        .cloned()
    else {
        end_player_cast(game, player_id, runtime, false);
        return MonsterCombatOutcome::Rejected;
    };
    let mp_loss = properties.query_property(2);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let mp_failure = |game: &Game| {
        game.send_cast_failure(player_id, 7);
        game.send_skill_system_info_unsigned(player_id, b"GS1144", mp_loss);
    };
    if game.player_kernel(player_id, MONSTER_RANGE_ATTACK_SKILL_ID).is_none() {
        let now = now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, MONSTER_RANGE_ATTACK_SKILL_ID),
            reuse,
            now,
        ) {
            game.send_cast_failure(player_id, 13);
            game.send_skill_system_info(player_id, b"GS1143");
            end_player_cast(game, player_id, runtime, false);
            return MonsterCombatOutcome::Rejected;
        }
        if mp_loss == 0 || (mana.wrapping_sub(mp_loss) as i32) < 0 {
            if mp_loss != 0 {
                mp_failure(game);
            }
            end_player_cast(game, player_id, runtime, false);
            return MonsterCombatOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(MONSTER_RANGE_ATTACK_SKILL_ID));
        }
        game.begin_player_kernel(
            player_id,
            super::lifecycle::SkillExecutionKernel::begin(dispatch, now),
        );
        return MonsterCombatOutcome::Begun;
    }
    if game
        .player_kernel(player_id, MONSTER_RANGE_ATTACK_SKILL_ID)
        .is_none_or(|kernel| kernel.dispatch() != dispatch)
    {
        return MonsterCombatOutcome::Rejected;
    }
    if game
        .player_kernel(player_id, MONSTER_RANGE_ATTACK_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let remaining = game.find_player(player_id).map_or(0, |player| player.mana())
            .wrapping_sub(mp_loss);
        if (remaining as i32) < 0 {
            mp_failure(game);
            end_player_cast(game, player_id, runtime, false);
            return MonsterCombatOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(remaining);
        }
        game.publish_player_states(player_id);
        let direction = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction());
        if let Some(direction) = direction {
            let message = range_cast_start_message(level, PLAYER_TYPE, player_id, direction);
            game.send_player_visual(player_id, &message);
        }
        if let Some(kernel) = game.player_kernel_mut(player_id, MONSTER_RANGE_ATTACK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = game
        .player_kernel(player_id, MONSTER_RANGE_ATTACK_SKILL_ID)
        .map(|kernel| kernel.started_at_ms())
        .expect("круговая атака хранит начало");
    if !skill_is_restored(started, delay, now_milliseconds()) {
        return MonsterCombatOutcome::Pending;
    }
    let Some(view) = game.find_player(player_id).and_then(|player| player.shape_view()) else {
        end_player_cast(game, player_id, runtime, false);
        return MonsterCombatOutcome::Rejected;
    };
    let fire = range_attack_fire_message(level as u16, PLAYER_TYPE, player_id, view.tile_x, view.tile_y);
    game.send_player_visual(player_id, &fire);
    if let Some(kernel) = game.player_kernel_mut(player_id, MONSTER_RANGE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    let mut attacked = Vec::new();
    for (dx, dy) in range_attack_scope_cells() {
        for view in game.monster_combat_cell_views(
            region_id,
            view.tile_x.wrapping_add(dx),
            view.tile_y.wrapping_add(dy),
        ) {
            let target = view.identity;
            let Some(master) = game.find_player(player_id).map(|player| player.master_info()) else {
                break;
            };
            let attackable = if matches!(target.object_type, 1_100 | 1_200) {
                game.stationary_build_attackable_by_player(player_id, region_id, target)
            } else {
                game.owned_player_skill_target_attackable(master, target, region_id)
            };
            // IsAttackAble перед дедупликацией; цель добавляется в список
            // после Attack, повторный удар той же цели пропускается.
            if !attackable || attacked.contains(&target) {
                continue;
            }
            if !(target.object_type == PLAYER_TYPE && target.id == player_id)
                && let Some((master, attack)) = calculate_player_range_attack(
                    game, player_id, region_id, target, level, &properties,
                )
            {
                match target.object_type {
                    PLAYER_TYPE => game.with_published_player_ai(player_id, player_ai, |game| {
                        game.monster_combat_apply_attack_to_player(master, target.id, region_id, attack, runtime)
                    }),
                    MONSTER_TYPE => game.with_published_player_ai(player_id, player_ai, |game| {
                        game.monster_combat_apply_attack_to_monster(master, target.id, region_id, attack, runtime)
                    }),
                    1_100 | 1_200 => game.with_published_player_ai(player_id, player_ai, |game| {
                        game.monster_combat_apply_attack_to_build(region_id, target, attack, runtime)
                    }),
                    _ => {}
                }
            }
            attacked.push(target);
        }
    }
    if let Some(kernel) = game.player_kernel_mut(player_id, MONSTER_RANGE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    end_player_cast(game, player_id, runtime, true);
    MonsterCombatOutcome::Completed
}
