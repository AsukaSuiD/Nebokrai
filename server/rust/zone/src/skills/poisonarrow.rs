//! Отравленная стрела боевого духа `CPoisonArrow` (`0x21E`): собственные
//! CheckCastCondition и AI, общая с `CBloodLoss` (`0x21D`) обёртка
//! `execute_periodic_battle_fairy_arrow`, позднее наложение периодического
//! яда `CPoisonArrowState`.
//!
//! Источник: `gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`; RVA
//! истинные `off pub + 0x1000`) + `GameServer.pdb` (RSDS
//! `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match), исходный владелец
//! `appserver/skills/poisonarrow.cpp`. Машинный разбор тела —
//! `.local/recon-de/notes/D6-poisonarrow.md` и
//! `.local/recon-de/disasm/CPoisonArrow.txt`.
//!
//! - Check `0x519750` (U,S): null S/props → 0; null-target/self (U==S) →
//!   visual(10) + ZHGS0045; конфликт сканируется **по позиции** исходного
//!   вектора состояний S: первый id из `{0x192 → ZHGS0046, 0xD2 → ZHGS0047,
//!   0x67 → ZHGS0046}` → системное сообщение игроку, ret 0 (тело hub
//!   `battlefairyskill::check_battle_fairy_target_states`);
//!   reuse(10005) → visual(13) + ZHGS0048; путь
//!   `vcall+0x58`: Query(5003)!=0 и cells > max (jbe проход) → visual(11) +
//!   ZHGS0049; клетка third==2 → visual(15) + ZHGS0051 (SSO-имя цели);
//!   Query(2) == 0 (GENESIS MP0) → **ret 1**; GetWarSoulGoods null →
//!   тихий ret 0; `GetAddonPropertyValues(0x9A,1)` − Query(2) < 0 (js) →
//!   visual(7) + ZHGS0052 (число, fild/fmul 0.01 → ftrunc → %u); иначе
//!   ret 1.
//! - AI `0x519D70`: `[+0x4C]==0` → out; props null → End(0); U/S null →
//!   End(0); IsDied(S) → visual(10) → End(0). Фаза 0, player only (RTTI):
//!   GetWarSoulGoods null → **тихий выход без End (Pending)**; нехватка MP
//!   → visual(7) + ZHGS0052 → End(0); иначе SetAddon(1,0x9A,остаток)
//!   signed → SerializeForOldClient → кадр `0xBF918` (long player, guid,
//!   count, payload; доставка — объявленный шов `send_goods_update`, якоря
//!   в шапке `skills/battlefairyskill.rs`);
//!   CAN(10006) → visual(0) → `[+0x50]=1`; delay 10001 unsigned → out;
//!   повторный путь: дальность → visual(11) + ZHGS0049 → End(0), клетка 2 →
//!   visual(15) + ZHGS0051 → End(0); visual(1); MasterInfo живого U с
//!   **country = 0**; PK до состояния: player(U) && region([S+0x40]) &&
//!   player(S) → `CPKSys::OnFirstSkill` с TileX/TileY(U) (объявленный шов
//!   контакта); ctor `0x5E3140` (Query eval: const 20010 → freq 6001 →
//!   keep 10002) → первый старый state End `vcall+0x1C` → destructor
//!   свежего остатка → Begin(U,S) `vcall+0x08` → прежний слот либо append;
//!   все исходы → **End(1)**. RNG, UpdateProperty и второго End при отказе
//!   state Begin нет.
//! - End навыка по vtable — тело второй половины общего End-контракта
//!   координатора (`0x5DFBD0`, hub прежнего пакета `states/skill.rs`,
//!   зафиксировано шапкой `skills/battlefairyskill.rs`); собственной
//!   логики CPoisonArrow не добавляет.
//!
//! Payload состояния — `CPoisonArrowState` (ctor `0x5E3140`, Calculate `kind 5`
//! с единственным clamp отрицательного HP-loss → 0, vtable `0x0065F24C`):
//! данные и кодек — Zone `effects/poison.rs` (`PoisonState<0x21E>`, VERIFIED
//! четырёхсторонний кодек), живой AI/Begin и семейная замена первого слота —
//! hub прежнего пакета `states/poison.rs` + `states/periodicattack.rs`
//! (объявленный шов `PoisonArrowStateArena` ниже, zone-код здесь их не
//! дублирует).
//!
//! Общая обёртка `execute_periodic_battle_fairy_arrow` живёт в Zone
//! целиком; `CBloodLoss` вызывает её через hub-делегат старого пакета
//! (`appserver/skills/poisonarrow.rs`) без правок.
//!
//! Объявленные швы переноса (не расхождения): hub
//! `battlefairyskill::BattleFairyGame` (реестр, WarSoul/equipment,
//! документированный путь S и регион, BF918-доставка, ZHGS-строки);
//! трейты `PoisonArrowStateArena` (замена первого слота ID и primary Begin
//! периодического яда прежних hub `states/state.rs`/`states/poison.rs`) и
//! `PoisonArrowContact` (PK `OnFirstSkill` живого main loop) реализуются
//! в файле-делегате старого пакета. Потребление статическое (generic),
//! dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).

use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::content::goods::GAP_BF_MP;
use crate::effects::PoisonState;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    check_battle_fairy_target_states, execute_registered_battle_fairy_state,
    send_battle_fairy_goods_update,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::state::StateKey;

pub const POISON_ARROW_SKILL_ID: u32 = 0x21e;

/// Payload `CPoisonArrowState` семейного периодического яда (данные и кодек
/// Zone `effects/poison.rs`, VERIFIED четырёхсторонний; живая обвязка — шов
/// арены).
pub type PoisonArrowState = PoisonState<POISON_ARROW_SKILL_ID>;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;

/// Эффект применения стрелы боевого духа: уже разрешённые U/S, MasterInfo
/// (country = 0) и таблица свойств, захваченная на входе AI.
pub struct ArrowEffect {
    pub user: (i32, ShapeIdentity),
    pub target: (i32, ShapeIdentity),
    pub master: MasterInfo,
    pub properties: CSkillBaseProperties,
}

/// Арена состояний стрел боевого духа (общая для `CPoisonArrow` и
/// `CBloodLoss`): замена первого слота своего ID и primary Begin нового
/// состояния. Реализация — прежний hub `states/state.rs` (скан первого ID →
/// локализация офсета до End → End + destructor остатка) и прежний hub
/// `states/poison.rs` (primary Begin общего механизма периодических ядов).
pub trait PoisonArrowStateArena {
    /// Замена первого состояния `state_id` у цели: `Some(Some(location))` —
    /// слот найден и завершён (End + destructor), `Some(None)` — совпадения
    /// нет (append), `None` — разрешение фигуры либо локализация офсета не
    /// удалась (caller пропускает применение, как прежний ранний return).
    fn arrow_state_replacement_slot(
        &mut self,
        target: (i32, ShapeIdentity),
        state_id: u32,
    ) -> Option<Option<(usize, usize)>>;

    /// Primary Begin `CPoisonArrowState` в прежнюю позицию либо append
    /// (прежний hub `states/poison.rs`).
    fn begin_primary_poison_arrow_state(
        &mut self,
        holder_region: i32,
        holder: ShapeIdentity,
        user: Option<(i32, ShapeIdentity)>,
        sufferer: Option<(i32, ShapeIdentity)>,
        state: PoisonArrowState,
        placement: Option<(usize, usize)>,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<StateKey>;
}

/// Контакт с runtime игрового хода: PK-блок первого удара до применения
/// состояния (вызов `player_on_first_skill_at_position` прежнего владельца).
pub trait PoisonArrowContact<Runtime> {
    /// `CPKSys::OnFirstSkill` с регионом цели и клеткой источника.
    fn poison_arrow_first_skill_at_position(
        &mut self,
        attacker_id: i32,
        victim_id: i32,
        region_id: i32,
        x: i32,
        y: i32,
        runtime: &mut Runtime,
    );
}

fn fail<Game: BattleFairyGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    mode: u32,
    text: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    game.send_skill_system_info(player_id, text);
}

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost));
}

/// Путь повторно разрешается через текущий навык; дальность и BLOCK_UNFLY
/// дают visual(11/15) с текстами игрока, имя цели читается живой фигурой.
fn path_allowed<Game: BattleFairyGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: Option<i32>,
    target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    obstacle_string: &[u8],
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        game.update_registered_skill_visual(instance, 11);
        if let Some(player_id) = player_id {
            game.send_skill_system_info(player_id, b"ZHGS0049");
        }
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_registered_skill_visual(instance, 15);
        if let Some(player_id) = player_id {
            if let Some(shape) = game.resolve_state_move_shape(target.0, target.1) {
                game.send_skill_system_info_with_text(
                    player_id, obstacle_string, shape.shape().base_object().get_name(),
                );
            }
        }
        return false;
    }
    true
}

/// `CheckCastCondition` `0x519750`: self → конфликт позиций → reuse → путь →
/// GENESIS MP0-проход → WarSoul-манометр, тексты отказов ZHGS0045..0052.
fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let user = player.shape().identity();
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let Some(target) = begin_target.filter(|(_, target)| {
        target.object_type != user.object_type || target.id != user.id
    }) else {
        fail(game, instance, player_id, 10, b"ZHGS0045");
        return false;
    };
    if !check_battle_fairy_target_states(game, player_id, target) { return false; }
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(|skill| skill.last_used_ms()) else {
        return false;
    };
    if !skill_is_restored(last_used, reuse, now(runtime)) {
        fail(game, instance, player_id, 13, b"ZHGS0048");
        return false;
    }
    if !path_allowed(game, instance, Some(player_id), target, &properties, b"ZHGS0051") {
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) == 0 { return true; }
    let Some(current) = game.battle_fairy_war_soul_addon(player_id, GAP_BF_MP) else {
        return false;
    };
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    if current.wrapping_sub(cost as i32) < 0 {
        fail_mana(game, instance, player_id, &properties);
        return false;
    }
    true
}

/// Общий зарегистрированный вход стрел боевого духа: начальный visual 2 при
/// отказе Check, затем единственный AI-тик; исходная S передаётся из Begin.
pub fn execute_periodic_battle_fairy_arrow<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    obstacle_string: &[u8],
    apply: impl FnOnce(&mut Game, ArrowEffect, &mut Runtime),
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome
where
    Game: BattleFairyGame + PoisonArrowContact<Runtime>,
{
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, Some(2),
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime, now),
        |game, instance, runtime| run_ai(game, instance, runtime, obstacle_string, apply, now),
    )
}

/// Вход `CPoisonArrow`: guard ID и фамильная строка препятствия ZHGS0051.
pub fn execute_battle_fairy_poison_arrow<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome
where
    Game: BattleFairyGame + PoisonArrowContact<Runtime> + PoisonArrowStateArena,
{
    if dispatch.skill_id() != POISON_ARROW_SKILL_ID {
        return BattleFairySkillOutcome::Rejected;
    }
    execute_periodic_battle_fairy_arrow(
        game, player_id, instance, dispatch, begin_target, runtime, b"ZHGS0051",
        |game, effect, runtime| apply_poison_arrow(game, effect, runtime, now), now,
    )
}

/// Собственный AI `0x519D70` одного тика: WarSoul-списание и BF918 фазы 0,
/// CAN, visual 0/1, повторный путь, PK до состояния и внешний apply.
fn run_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    obstacle_string: &[u8],
    apply: impl FnOnce(&mut Game, ArrowEffect, &mut Runtime),
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome
where
    Game: BattleFairyGame + PoisonArrowContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return BattleFairySkillOutcome::Rejected; };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let user = game.resolve_state_move_shape(region, identity)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let target = game.resolve_skill_sufferer(skill.lifecycle());
    let (Some(user), Some(target)) = (user, target) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if game.base_magic_target_dead(target.0, target.1) {
        game.update_registered_skill_visual(instance, 10);
        return BattleFairySkillOutcome::Rejected;
    }
    let player_id = (user.1.object_type == PLAYER_TYPE).then_some(user.1.id);
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if let Some(player_id) = player_id {
            // AI требует WarSoul даже после MP0-прохода Check: null goods —
            // тихий выход без End (GENESIS-различие разведки).
            let Some(current) = game.battle_fairy_war_soul_addon(player_id, GAP_BF_MP) else {
                return BattleFairySkillOutcome::Pending;
            };
            let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
            let remaining = current.wrapping_sub(cost as i32);
            if remaining < 0 {
                fail_mana(game, instance, player_id, &properties);
                return BattleFairySkillOutcome::Rejected;
            }
            if game.set_battle_fairy_equipment_addon(player_id, GAP_BF_MP, remaining).is_none() {
                return BattleFairySkillOutcome::Pending;
            }
            // Setter сохраняет предмет в equipment. Повторный WarSoul-gate
            // здесь отсутствует; отказ Serialize не подавляет BF918.
            let Some((ex_id, payload)) = game.battle_fairy_equipment_payload(player_id) else {
                return BattleFairySkillOutcome::Pending;
            };
            send_battle_fairy_goods_update(game, player_id, ex_id, &payload);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return BattleFairySkillOutcome::Pending;
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if now(runtime) < started.wrapping_add(delay) {
        return BattleFairySkillOutcome::Pending;
    }
    if !path_allowed(game, instance, player_id, target, &properties, obstacle_string) {
        return BattleFairySkillOutcome::Rejected;
    }
    game.update_registered_skill_visual(instance, 1);
    let mut master = MasterInfo {
        master_type: user.1.object_type, master_id: user.1.id, ..MasterInfo::default()
    };
    if let Some(player) = player_id.and_then(|player_id| game.find_player(player_id)) {
        let permissions = player.battle_fairy_pk_permissions();
        master.master_guild_id = player.faction_id();
        master.master_team_id = player.team_id();
        master.master_union_id = player.union_id();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
    }
    let target_region = game.resolve_state_move_shape(target.0, target.1).and_then(|shape| {
        let shape = shape.shape();
        shape.is_assigned_to_server_region().then_some(shape.get_region_id())
    }).filter(|region| game.battle_fairy_region_exists(*region));
    if let (Some(player_id), Some(target_region)) = (player_id, target_region) {
        if target.1.object_type == PLAYER_TYPE {
            if let Some(player) = game.find_player(player_id) {
                let y = player.shape().get_tile_y().unwrap_or(i32::MIN);
                let x = player.shape().get_tile_x().unwrap_or(i32::MIN);
                game.poison_arrow_first_skill_at_position(
                    player_id, target.1.id, target_region, x, y, runtime,
                );
            }
        }
    }
    apply(game, ArrowEffect { user, target, master, properties }, runtime);
    BattleFairySkillOutcome::Completed
}

/// Позднее наложение яда: сохранённая таблица (const → freq → keep), ctor
/// `0x5E3140`, замена первого старого state, primary Begin в прежний слот
/// либо append; отказ Begin не отменяет внешний End(1).
fn apply_poison_arrow<Game, Runtime>(
    game: &mut Game,
    effect: ArrowEffect,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) where
    Game: PoisonArrowStateArena,
{
    let hp_loss = effect.properties.query_property(SKILL_USAGE_CONST);
    let frequency = effect.properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let keep = effect.properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state = PoisonArrowState::new(effect.master, keep, frequency, hp_loss);
    let Some(placement) = game.arrow_state_replacement_slot(effect.target, POISON_ARROW_SKILL_ID)
    else { return; };
    let _ = game.begin_primary_poison_arrow_state(
        effect.target.0, effect.target.1, Some(effect.user), Some(effect.target), state, placement,
        &mut || now(runtime),
    );
}
