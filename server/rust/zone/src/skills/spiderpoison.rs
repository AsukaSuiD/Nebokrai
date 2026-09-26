//! Ядовитый удар `CSpiderPoison` (`0x191`): собственные CheckCastCondition,
//! AI, прямой удар с формулой и позднее наложение яда. Общая обвязка
//! `stateskill` (Begin-скелет, End, оркестрация player/monster путей и
//! публикация visual) остаётся hub прежнего пакета; состояния принадлежат
//! аренам получателей, а не исполнению навыка.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/spiderpoison.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/spiderpoison.rs`; тела Check/AI/Attack/
//! Calculate/apply_poison перенесены буквально (кластер D полосы Monster
//! 0x19x «трупная/ядовая state-линия», карта — запись аудита «Zone skills:
//! карта полосы Monster 0x19x — 5 кластеров волн», 26 сентября 2026).
//!
//! Машинная разведка порции по этой паре (запись `.local/recon-de/notes/
//! D5-spiderpoison.md`, тела `.local/recon-de/disasm/CSpiderPoison.txt`);
//! сопоставление с перенесённым кодом — MATCH по всем пунктам:
//!
//! - vtable эффекта `0x65B188`, класса `0x25B0F4`: Begin-скелет трёх форм
//!   (`0x5853E0`/`0x5854B0`/`0x5855B0`) = форвард `CAttackSkill::Begin` →
//!   new effect 0xC → `[+0x34]` → `BeginVisualEffect(1)` → Check слот `+0x60`;
//!   провал — visual(2) → End(0), успех — `[+0x4C]=1`, `[+0x50]=0` (обвязка
//!   hub `stateskill`). Check `0x585BB0`: null U/S/props → 0 без кадров;
//!   reuse (10005, `CSkill::IsRestored`) → visual(13); путь `vcall+0x58` и
//!   `Query(5003) != 0 && size > max` → visual(11); SetMoveable(0) на U →
//!   ret 1.
//! - AI `0x586020`: `[+0x4C]==0` → out; props null → End(0); U/S null →
//!   End(0); **IsDied только у S** → visual(10) → End(0); Begin-фаза один
//!   раз: `Query(10006)` → `[+0x3C]`, вычисление `Y(S), X(S), Y(U), X(U)` →
//!   GetLineDir → SetDir на U → visual(0); задержка `Query(10001)+[+0x2C]`
//!   unsigned → out; **`SetMoveable(1)` перед повторной дистанционной
//!   проверкой**, длинный путь → visual(11) → End(0); иначе visual(1) →
//!   `Attack(U, S)` → `End(1)` — все исходы состояния (как свойства, так и
//!   бросок яда) завершаются `End(1)`.
//! - Attack `0x585F10`: `tagAttackInformation` ctor-дефолт (id `0x7FFFFFFF`,
//!   level 1) не переписывается; U.type==400 → MasterInfo-поля игрока
//!   (+0x278..+0x27C, +0xB20/+0xB28/+0xB78) → `CalculateAttackPower` →
//!   общий virtual `+0x15C` приёмника.
//! - Calculate `0x585CE0`: props null → info остаётся ctor-дефолтом;
//!   `[+0x18]=0` hit, `[+0x1C]=1.0f` factor, `[+0x20]=0` modifier; physical
//!   kind 1: `|max − min| + 1` (cdq-abs) → один RNG → min + random →
//!   jns-clamp; element kind 3: `vcall+0x118` (у монстра 0) clamp; soul
//!   kind 4: `vcall+0x11C` **movzx WORD** clamp; **2 RNG суммарно**: второй
//!   `random(100)` обязателен даже при нулевом `vcall+0x114` монстра,
//!   **movzx WORD cch**, знаковое `<`; x87-крит с глобалкой float
//!   `0xEF3E5C` (принятая x87/f64-модель combat.md через `truncate_original`).
//! - Поздний бросок яда: `DoesStateExist(0x131)` на S → пропуск;
//!   `random(100) > Query(40001)` **signed jg** → пропуск; master живого U
//!   **с country = 0**; ctor `CSpiderPoisonState` (`0x5E90C0`) с порядком
//!   вычисления Query const(20010) → freq(6001) → keep(10002) **до** замены;
//!   скан первого `0x191` в `S+0x11C` → End `vcall+0x1C` → deleting-dtor
//!   `vcall+0x10(1)` → slot = 0 → `Begin(U, S)` → тот же слот, иначе append.
//!
//! Payload состояния — `CSpiderPoisonState`: кодек `zone/effects/poison.rs`
//! (VERIFIED раньше), живой AI/фабрика — hub `states/poison.rs` и
//! `statefactory` (оба вне этого файла). `spiderpoisonstate.rs` прежнего
//! пакета остаётся тонким alias.
//!
//! Объявленные швы переноса (не расхождения): трейт `SpiderPoisonGame`
//! расширяет hub `baseattackruntime::BaseAttackGame` единственным чтением
//! `GetTargetPath`; трейт `SpiderPoisonStateArena` — живой Cure-факт и
//! семейная замена первого состояния 0x191 (`states/state.rs` прежнего
//! пакета: find/end/placement + primary Begin); `SpiderPoisonMoveShape`
//! добавляет `SetMoveable` к фасаду фигуры. Потребление статическое
//! (generic), dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).
//! Последний вариант замены — abort при неразрешимой позиции слота
//! (внутренний отказ арены: запись найдена, а локализация офсета нет) —
//! семантика прежнего sibling-кастa `corpseptomaine` сохранена для обеих
//! ветвей вместо прежнего D1-append двойника; машинной ветви этому отказу
//! не соответствует, она возникает только при несогласованности арены Rust.

use nebokrai_shared::runtime::get_line_direction;

use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original,
};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::baseattackruntime::{
    BaseAttackGame, BaseAttackMoveShape, BaseAttackPlayer, SKILL_USAGE_CAN_BE_BREAKED,
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::cure::CURE_SKILL_ID;
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::spidermist::{SKILL_USAGE_STATE_PERSIST_TIME, SKILL_USAGE_TARGET_AFFECT_FREQUENCY};
use super::statefactory::SpiderPoisonState;

pub use super::statefactory::SPIDER_POISON_SKILL_ID;

const HP_LOSS: u32 = 20_010;
const PROBABILITY: u32 = 40_001;

/// Диспетчерская форма player-cast ядовитого удара: любая форма цели с
/// этим ID (object/point — реальные вызовы, self — предикат диспетчера).
pub const fn is_player_spider_poison_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == SPIDER_POISON_SKILL_ID
}

/// Целевая форма конкретного Begin: Object-форма несёт уже разрешённую пару
/// (её невозможность — провал до Check), Point-форма разрешается общим
/// `GetSufferer` записи внутри CheckCastCondition (без fallback к user).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpiderPoisonBeginTarget {
    Object(Option<(i32, ShapeIdentity)>),
    Resolved,
}

/// Живая фигура-участник ядовой линии: `SetMoveable` U до/после каста.
pub trait SpiderPoisonMoveShape: BaseAttackMoveShape {
    fn set_moveable(&mut self, moveable: bool);
}

/// Арена состояний ядовой линии (общая для `CSpiderPoison` и
/// `CCorpsePtomaine`): факт состояния на живой фигуре и семейная замена
/// первого состояния 0x191. Реализация — прежний hub `states/state.rs`
/// (find первого 0x191 → End → destructor остатка → primary Begin в тот же
/// слот либо append), `end_and_destroy_state_at` и primary Begin общего
/// механизма ядов.
pub trait SpiderPoisonStateArena {
    /// Наличие состояния навыка у живой фигуры; неразрешённая фигура —
    /// `None` (caller пропускает кандидата, как прежний ранний return).
    fn poison_shape_has_state(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        skill_id: u32,
    ) -> Option<bool>;

    /// Замена первого состояния 0x191 у держателя либо append нового;
    /// `user`/`sufferer` — уже разрешённые пары начала AI. `false` — Begin
    /// не состоялся либо арена не смогла локализовать слот (см. шапку).
    fn replace_or_begin_spider_poison_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        user: Option<(i32, ShapeIdentity)>,
        sufferer: Option<(i32, ShapeIdentity)>,
        state: SpiderPoisonState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool;
}

/// Hub-шов исполнения `CSpiderPoison`: hub `baseattackruntime` плюс единый
/// читатель пути цели записи навыка.
pub trait SpiderPoisonGame: BaseAttackGame {
    /// Общий `GetTargetPath` записи навыка (`vcall+0x58` живого экземпляра).
    fn skill_target_path(
        &self,
        lifecycle: &super::lifecycle::SkillLifecycle,
    ) -> Vec<(i32, i32, u8)>;
}

fn participant<Game: BaseAttackGame>(
    game: &Game,
    value: (i32, ShapeIdentity),
) -> Option<(i32, ShapeIdentity)> {
    let shape = game.resolve_state_move_shape(value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

/// MasterInfo живого источника удара: полные PK-поля только у игрока.
fn poison_master<Game: BaseAttackGame>(
    game: &Game,
    source: (i32, ShapeIdentity),
) -> Option<MasterInfo> {
    if source.1.object_type != PLAYER_TYPE {
        return Some(MasterInfo {
            master_type: source.1.object_type,
            master_id: source.1.id,
            ..MasterInfo::default()
        });
    }
    let player = game.find_player(source.1.id)?;
    let permissions = player.pk_permissions();
    Some(MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: source.1.id,
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    })
}

/// Границы урона живого источника: боевые свойства игрока либо границы
/// монстра после состояний (`state_attack_bounds` владельца).
fn poison_attack_bounds<Game: BaseAttackGame>(
    game: &Game,
    source: (i32, ShapeIdentity),
) -> Option<(u32, u32)> {
    if source.1.object_type == PLAYER_TYPE {
        let combat = game.find_player(source.1.id)?.combat_properties();
        Some((combat.minimum_attack, combat.maximum_attack))
    } else {
        game.monster_base_attack_bounds(source)
    }
}

/// `CheckCastCondition` `0x585BB0`: reuse → visual 13, длинный путь →
/// visual 11, иначе SetMoveable(0) на U и разрешение Begin.
pub fn check_spider_poison_cast<Game: SpiderPoisonGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    begin_target: SpiderPoisonBeginTarget,
    now_ms: u32,
) -> bool
where
    Game::MoveShape: SpiderPoisonMoveShape,
{
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let (id, level, lifecycle, last_used_ms) =
        (skill.id(), skill.level(), *skill.lifecycle(), skill.last_used_ms());
    let Some(user) = participant(game, lifecycle.user()) else { return false; };
    let resolved = match begin_target {
        SpiderPoisonBeginTarget::Object(target) => target,
        SpiderPoisonBeginTarget::Resolved => game.resolve_skill_sufferer(&lifecycle),
    };
    if resolved.is_none() { return false; }
    let Some(properties) = game.skill_base_properties(id, level) else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if !skill_is_restored(last_used_ms, reuse, now_ms) {
        game.update_registered_skill_visual(instance, 13);
        return false;
    }
    let path = game.skill_target_path(&lifecycle);
    if maximum != 0 && maximum < path.len() as u32 {
        game.update_registered_skill_visual(instance, 11);
        return false;
    }
    if let Some(shape) = game.resolve_state_move_shape_mut(user.0, user.1) {
        shape.set_moveable(false);
    }
    true
}

/// Формула Calculate `0x585CE0` поверх живого источника; порядок чтения
/// границ — max → min → повторный min после броска (чистые чтения владельца).
fn calculate_spider_poison_attack<Game: BaseAttackGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return };
    if game.skill_base_properties(skill.id(), skill.level()).is_none() { return; }
    attack.damage_modifier = 0;
    attack.damage_factor = 1.0;
    attack.hit_modifier = 0;
    let Some((_, maximum)) = poison_attack_bounds(game, source) else { return };
    let Some((minimum, _)) = poison_attack_bounds(game, source) else { return };
    let span = (maximum as i32).wrapping_sub(minimum as i32).unsigned_abs().wrapping_add(1) as i32;
    let random = game.skill_random_below(span);
    let Some((minimum, _)) = poison_attack_bounds(game, source) else { return };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Physical,
        hp_damage: (minimum as i32).wrapping_add(random).max(0),
        mp_damage: 0,
    });
    let element = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return };
        player.combat_properties().add_element_attack as i32
    } else {
        // `GetAddElementAtk` монстра (vcall+0x118) возвращает ноль.
        0
    };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Element,
        hp_damage: element.max(0),
        mp_damage: 0,
    });
    let soul = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return };
        player.combat_properties().add_soul_attack
    } else {
        let Some(soul) = game.monster_base_attack_soul_attack(source) else { return };
        soul
    };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Soul,
        hp_damage: i32::from(soul),
        mp_damage: 0,
    });
    let critical_chance = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return };
        player.combat_properties().cch
    } else {
        // Нулевой `GetCCH` монстра не отменяет второй RNG Calculate.
        0
    };
    if game.skill_random_below(100) < i32::from(critical_chance) {
        attack.critical = true;
        let rate = game.critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

/// Прямой удар Attack `0x585F10`: пустой ctor-deфолт информации, формула и
/// общий virtual контакт; NULL таблица Calculate не отменяет приём.
fn spider_poison_attack<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) where
    Game: BaseAttackGame + super::baseattackruntime::BaseAttackContact<Runtime>,
{
    let Some(master) = poison_master(game, source) else { return };
    let mut attack = AttackInformation::for_master(master);
    calculate_spider_poison_attack(game, instance, source, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

/// Позднее наложение яда после удара: живой Cure-гейт 0x131, знаковый бросок
/// вероятности 40001, master живого U с нулевой страной и семейная замена
/// первого состояния 0x191 (ctor до замены).
fn apply_spider_poison<Game: BaseAttackGame + SpiderPoisonStateArena>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    now: &mut dyn FnMut() -> u32,
) {
    if game.poison_shape_has_state(target.0, target.1, CURE_SKILL_ID) != Some(false) { return; }
    // DWORD probability сравнивается знаково (jg); равенство сохраняет успех.
    let probability = properties.query_property(PROBABILITY);
    if game.skill_random_below(100) > probability as i32 { return; }
    let Some(master) = poison_master(game, source) else { return };
    // В отличие от прямого удара, поздний MasterInfo не получает country.
    let master = MasterInfo { master_country_id: 0, ..master };
    let state = SpiderPoisonState::new(
        master,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
        properties.query_property(HP_LOSS),
    );
    let _ = game.replace_or_begin_spider_poison_state(
        target.0, target.1, Some(source), Some(target), state, now,
    );
}

/// Собственный AI `0x586020` одного тика. `Some(0/1)` — буквальный
/// `End(argument)` общего механизма (вызывает hub `stateskill`); `None`
/// оставляет экземпляр следующему тику. Часы приходят fn-параметром
/// делегата старого main loop (`game_tick_milliseconds`, как у соседей).
pub fn execute_spider_poison_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> Option<i32>
where
    Game: SpiderPoisonGame + SpiderPoisonStateArena
        + super::baseattackruntime::BaseAttackContact<Runtime>,
    Game::MoveShape: SpiderPoisonMoveShape,
{
    let skill = game.registered_skill(instance)?;
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) { return None; }
    let (id, level, lifecycle) =
        (skill.id(), skill.level(), *skill.lifecycle());
    let Some(properties) = game.skill_base_properties(id, level).cloned() else { return Some(0) };
    let Some(source) = participant(game, lifecycle.user()) else { return Some(0) };
    let Some(target) = game
        .resolve_skill_sufferer(&lifecycle)
        .and_then(|value| participant(game, value))
    else { return Some(0) };
    // IsDied только у S: мёртвая цель завершает без удара.
    if game.base_magic_target_dead(target.0, target.1) {
        game.update_registered_skill_visual(instance, 10);
        return Some(0);
    }
    if game.registered_skill(instance)?.execution_stage() == Some(SkillStage::Begin) {
        game
            .registered_skill_mut(instance)?
            .lifecycle_mut()
            .set_available(properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0);
        let target_y = game
            .resolve_state_move_shape(target.0, target.1)?
            .shape()
            .get_tile_y()
            .unwrap_or(i32::MIN);
        let target_x = game
            .resolve_state_move_shape(target.0, target.1)?
            .shape()
            .get_tile_x()
            .unwrap_or(i32::MIN);
        let source_y = game
            .resolve_state_move_shape(source.0, source.1)?
            .shape()
            .get_tile_y()
            .unwrap_or(i32::MIN);
        let source_x = game
            .resolve_state_move_shape(source.0, source.1)?
            .shape()
            .get_tile_x()
            .unwrap_or(i32::MIN);
        game.resolve_state_move_shape_mut(source.0, source.1)?
            .shape_mut()
            .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let started = game.registered_skill(instance)?.lifecycle().started_at_ms();
    if now_milliseconds() < started.wrapping_add(delay) { return None; }
    // SetMoveable(1) на U до повторной дистанционной проверки.
    if let Some(shape) = game.resolve_state_move_shape_mut(source.0, source.1) {
        shape.set_moveable(true);
    }
    let path = game.skill_target_path(game.registered_skill(instance)?.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
    {
        game.update_registered_skill_visual(instance, 11);
        return Some(0);
    }
    // Путь и visual разрешают S заново; gameplay использует S начала AI.
    game.update_registered_skill_visual(instance, 1);
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
        let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
    }
    spider_poison_attack(game, instance, source, target, runtime);
    // После удара нет IsDied/IsAttackAble: только живой Cure и бросок яда.
    apply_spider_poison(
        game,
        source,
        target,
        &properties,
        &mut || now_milliseconds(),
    );
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Attack, SkillStage::Apply);
    }
    Some(1)
}
