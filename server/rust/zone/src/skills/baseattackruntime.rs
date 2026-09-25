//! Исполнение базовой атаки `CBaseAttack` (навык 1) игроком и монстром.
//!
//! Источник: gameserver.exe + GameServer.pdb, исходный владелец
//! `appserver/skills/baseattack.cpp`. Прежние переходные места —
//! `src/gameserver/appserver/skills/baseattackruntime.rs` (player Begin/AI/
//! Attack/visual) и runtime-часть `src/gameserver/appserver/skills/baseattack.rs`
//! (AI и Attack монстра, abort-формы, общий terminal End). Тела перенесены
//! буквально; неизменными остались порядок стадий, чтения определений, выбор
//! визуалов и обращения к общему потоку случайных чисел.
//!
//! Точная пара: `original/server/Miracle_server/GameServer/gameserver.exe`
//! (SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`)
//! + `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2; совпадение подтверждено `.local/evidence/symbols.py identity`).
//!
//! Машинная разведка порции по этой паре (запись аудита «Zone skills:
//! машинная разведка baseattackruntime», 26 сентября 2026) сняла класс
//! целиком: vtable `0x25CB0C`, AI `0x001B39B0`, Restart `0x00113E00`, свойства
//! по ключам 5003/10001/10006/20001, часы `WINMM!timeGetTime`. Сопоставление с
//! перенесённым кодом — MATCH по всем пунктам:
//!
//! - `execute_stage`: гейт active → props → user → dead-check → Begin-стадия
//!   ровно раз (direction/visual(0)/stage=1) → fall-through в тот же тик →
//!   delay-wrap unsigned → visual(1) → Attack со свежим GetSufferer ПОСЛЕ
//!   visual → End(1);
//! - `calculate_attack`: порядок GetMaxAttack → GetMinAttack → span → RNG →
//!   +Min → clamp, виды 1/3/4, критический хвост без промежуточной записи
//!   float;
//! - `attack`: guards включая отказ object type 500, master fill, вызван общий
//!   virtual +15C (..., false), затем безусловный caller `IncreaseRp(1,0)`; у
//!   `CMonster` IncreaseRp — пустое тело (ICF-факт, ошибкой не является);
//! - `publish_base_attack_visual` (`0x000BFE01`): 16-входный switch, area
//!   0→u8=1, 1→u8=2 с target/dest, personal {2,7,10,11,13,14,15}→[u8=0]
//!   [u8=mode]; режимы 3,4,5,6,8,9,12 ничего не посылают;
//! - terminal/abort: End(1) изнашивает оружие только у игрока и фиксирует
//!   reuse, End(0) не делает ни того, ни другого; OnChangeRegion — End(0).
//!
//! Объявленные швы переноса (не расхождения): kernel ведёт учёт стадий,
//! определения перечитываются (данные уровня статичны), часы приходят от
//! делегата старого main loop. Трейты ниже — переходные фасады прежнего
//! владельца `CGame`, реализация остаётся у него в файле-делегате; швы
//! потребляются статически (generic), dyn-совместимость и `Send`-контракт не
//! вводятся (ADR-0013). Гейт приёмника `+0x2A` отложен порции приёмника.
//!
//! Открытый UNKNOWN сохранён честно: достижимость отдельного `Restart`
//! (`0x00113E00`) — диспетчер vtbl+0x20 не установлен; повторный Begin не
//! считается его реализацией. Метаданные исследования — в конце файла.

use crate::app::game_message::CMessage;
use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original,
};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{NPC_TYPE, PLAYER_TYPE};
use crate::regions::shape::{CShape, ShapeView};
use nebokrai_shared::runtime::get_line_direction;

use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::skillfactory::SkillOwner;
use super::{
    PlayerSkillDispatch, SkillExecutionKernel, SkillLifecycle, SkillStage, SkillTermination,
    SkillVisualEffect, SkillVisualEffectKind,
};

pub const BASE_ATTACK_SKILL_ID: u32 = 1;

pub const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5003;
pub const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub const SKILL_USAGE_USER_HIT_MODIFIER: u32 = 20_001;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

const BASE_ATTACK_VISUAL_MESSAGE: i32 = 0x000b_fe01;

pub type BaseAttackExecutionState = SkillExecutionKernel<PlayerSkillDispatch>;

/// Стадии результата одного тика исполнения; обёртка очереди с полем
/// `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseAttackExecutionOutcome {
    Begun,
    Pending,
    Completed,
    Rejected,
    /// Применение отклонено, но владелец требует `End(1)`.
    RejectedAfterUse,
}

/// PK-допуски источника для полей `permitted_to_kill_*` `tagMasterInfo`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseAttackPkPermissions {
    pub player: bool,
    pub teammate: bool,
    pub guild_member: bool,
    pub criminal: bool,
}

/// Игрок-источник базовой атаки: переходный фасад старого `CPlayer`.
pub trait BaseAttackPlayer {
    /// Форма игрока (identity, клетки, direction).
    fn shape(&self) -> &CShape;

    /// Изменяемая форма для поворота Begin-стадии.
    fn movement_shape_mut(&mut self) -> &mut CShape;

    fn shape_view(&self) -> Option<ShapeView>;

    fn set_skill_moveable(&mut self, moveable: bool);

    fn combat_properties(&self) -> PlayerCombatProperties;

    fn faction_id(&self) -> i32;

    fn team_id(&self) -> i32;

    fn union_id(&self) -> i32;

    fn country(&self) -> u8;

    fn pk_permissions(&self) -> BaseAttackPkPermissions;
}

/// Живая фигура стороны базовой атаки: переходный фасад старого `CMoveShape`.
pub trait BaseAttackMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;
}

/// Переходные фасады прежнего владельца `CGame`, открывающие исполнению
/// `CBaseAttack` только прежние обращения; имена сохраняют исходную операцию.
pub trait BaseAttackGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для исполнения.
    type SkillAddress: Copy;

    type Player: BaseAttackPlayer;

    type MoveShape: BaseAttackMoveShape;

    /// AI игрока, публикуемый на время callbacks; непрозрачен для исполнения.
    type PlayerAi;

    /// Извлекаемый владелец региона старого хоста.
    type RegionOwner;

    // Реестр и исполнение экземпляра (фасады `CGame`).
    fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<Self::SkillAddress>;

    fn registered_move_shape_skill(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        skill_id: u32,
    ) -> Option<Self::SkillAddress>;

    fn registered_skill(
        &self,
        address: Self::SkillAddress,
    ) -> Option<&RegisteredSkillRecord<Self::MonsterExecution>>;

    fn registered_skill_mut(
        &mut self,
        address: Self::SkillAddress,
    ) -> Option<&mut RegisteredSkillRecord<Self::MonsterExecution>>;

    fn player_skill_execution(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>>;

    fn player_skill_execution_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>>;

    fn player_skill_lifecycle(&self, player_id: i32, skill_id: u32) -> Option<&SkillLifecycle>;

    fn begin_player_skill_execution(
        &mut self,
        player_id: i32,
        kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool;

    fn replace_player_skill_visual_effect(
        &mut self,
        player_id: i32,
        skill_id: u32,
        effect: SkillVisualEffect,
    ) -> bool;

    fn update_player_skill_visual(&mut self, player_id: i32, skill_id: u32, mode: u32);

    fn update_registered_skill_visual(&mut self, address: Self::SkillAddress, mode: u32);

    // Определения уровня и разрешение сторон.
    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    /// Точный общий `GetSufferer`: сохранённая identity, затем клетка региона.
    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)>;

    /// Живой `GetUser` по сохранённым region/type/id.
    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&Self::MoveShape>;

    fn resolve_state_move_shape_mut(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&mut Self::MoveShape>;

    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut Self::Player>;

    /// Общий факт `GetSufferer` для объектной цели (HP/god у движущихся).
    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView>;

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8>;

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        user: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    // Формула и случайность.
    fn weapon_damage_factors(&self) -> (f32, f32);

    /// `weapon_modifier` игрока против уровня цели; чтение уровня реестра и
    /// фабрики предметов остаётся у владельца.
    fn base_attack_weapon_modifier(
        &self,
        player_id: i32,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> Option<f32>;

    fn critical_rate(&self) -> f32;

    fn skill_random_below(&mut self, maximum: i32) -> i32;

    // Монстр-источник: hub-чтение `CMonster` и его таблицы свойств.
    /// Держатель (регион, identity) активного монстра извлекаемого owner-а.
    fn base_attack_monster_holder(
        &self,
        owner: &Self::RegionOwner,
        monster_id: i32,
    ) -> Option<(i32, ShapeIdentity)>;

    fn monster_base_attack_bounds(&self, source: (i32, ShapeIdentity)) -> Option<(u32, u32)>;

    fn monster_base_attack_soul_attack(&self, source: (i32, ShapeIdentity)) -> Option<u16>;

    // Доставка visual `0x000BFE01`: кадр строится здесь, маршруты — у владельца.
    /// Personal-ветвь по numeric identity игрока.
    fn send_base_attack_visual_to_player(&self, player_id: i32, message: &CMessage);

    /// Around-ветвь региона формы; отсутствующий регион пропускает отправку.
    fn send_base_attack_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);

    // Terminal и публикация контекстов.
    /// Публикует AI игрока на время callback и возвращает его обратно.
    fn with_published_player_ai<Output>(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Output;

    /// Публикует извлечённый регион на время callback и возвращает его обратно.
    fn with_published_region<Output>(
        &mut self,
        owner: &mut Option<Self::RegionOwner>,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Option<Output>;

    /// Общий зарегистрированный `CSkill::End` текущего пакета; признак
    /// завершения у caller-ов отбрасывается, как и раньше.
    fn end_registered_instance<Runtime>(
        &mut self,
        address: Self::SkillAddress,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
    );

    /// `End(0)` без AfterUse и часов reuse.
    fn end_registered_instance_without_after_use(
        &mut self,
        address: Self::SkillAddress,
        termination: SkillTermination,
    );

    fn finish_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool;

    fn finish_registered_player_command(
        &mut self,
        address: Option<Self::SkillAddress>,
        player_ai: &mut Self::PlayerAi,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool;

    /// `IncreaseRp(1, 0)` источника после возврата приёмника.
    fn increase_owned_player_rp(&mut self, player_id: i32, attacking: bool, damage: u16);
}

/// Контактная стадия с runtime игрового хода: общий virtual +15C (..., false)
/// приёмника. Отделена, потому что тип хода принадлежит старому main loop,
/// а не самой атаке.
pub trait BaseAttackContact<Runtime> {
    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );
}

fn skill_level<Game: BaseAttackGame>(game: &Game, player_id: i32) -> Option<i32> {
    game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID)
        .and_then(|address| game.registered_skill(address))
        .map(RegisteredSkillRecord::level)
}

fn target<Game: BaseAttackGame>(game: &Game, player_id: i32) -> Option<(i32, ShapeIdentity)> {
    let lifecycle = game.player_skill_lifecycle(player_id, BASE_ATTACK_SKILL_ID)?;
    let (region, identity) = game.resolve_skill_sufferer(lifecycle)?;
    let shape = game.resolve_state_move_shape(region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub fn publish_base_attack_visual<Game: BaseAttackGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if skill.owner() != SkillOwner::CBaseAttack
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::BaseAttack || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(BASE_ATTACK_VISUAL_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_base_attack_visual_to_player(source.identity().id, &message);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        let target = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity));
        let (x, y) = if let Some(target) = target {
            let shape = target.shape();
            let (Ok(x), Ok(y)) = (shape.get_tile_x(), shape.get_tile_y()) else { return; };
            (x, y)
        } else {
            skill.lifecycle().destination()
        };
        message.add_long(target.map_or(0, |target| target.shape().identity().object_type));
        message.add_long(target.map_or(0, |target| target.shape().identity().id));
        message.add_long(x);
        message.add_long(y);
    }
    game.send_base_attack_visual_around(source.get_region_id(), source, &message);
}

fn calculate_attack<Game: BaseAttackGame>(
    game: &mut Game,
    player_id: i32,
    target: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let Some(level) = skill_level(game, player_id) else { return; };
    let Some(properties) = game.skill_base_properties(BASE_ATTACK_SKILL_ID, level) else { return; };
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let (divisor, minimum_factor) = game.weapon_damage_factors();
    let Some(damage_factor) =
        game.base_attack_weapon_modifier(player_id, i32::from(target_level), divisor, minimum_factor)
    else { return; };
    attack.damage_factor = damage_factor;
    attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let Some(player) = game.find_player(player_id) else { return; };
    let maximum = player.combat_properties().maximum_attack as i32;
    let minimum = player.combat_properties().minimum_attack as i32;
    let span = maximum.wrapping_sub(minimum).max(0);
    let minimum = player.combat_properties().minimum_attack as i32;
    // В отличие от Strike, верхняя граница исключена и отрицательная ширина
    // обнуляется. Повторный GetMinAttack предшествует RNG, а не следует за ним.
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    let Some(player) = game.find_player(player_id) else { return; };
    let element = (player.combat_properties().add_element_attack as i32).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    let soul = i32::from(player.combat_properties().add_soul_attack);
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    let critical = player.combat_properties().cch;
    if game.skill_random_below(100) < i32::from(critical) {
        attack.critical = true;
        let rate = game.critical_rate();
        for power in &mut attack.damages {
            // Между x87 multiply и усечением к int нет промежуточной записи float.
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

fn attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) where
    Game: BaseAttackGame + BaseAttackContact<Runtime>,
{
    let Some(target) = target else { return; };
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return; };
    if (target.1.object_type == user.object_type && target.1.id == user.id)
        || target.1.object_type == NPC_TYPE
        || !game.live_skill_target_attackable(target.0, user, target.1)
    { return; }
    let Some(player) = game.find_player(player_id) else { return; };
    let permissions = player.pk_permissions();
    let master = MasterInfo {
        master_type: user.object_type,
        master_id: user.id,
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    };
    // CBaseAttack не записывает skill ID/level в tagAttackInformation:
    // даже успешный Calculate сохраняет конструкторские UNKNOWN и 1.
    let mut attack = AttackInformation::for_master(master);
    calculate_attack(game, player_id, target, &mut attack);
    // Вызван именно общий virtual +15C (..., false), включая постройки.
    // Его внутренний отказ не отменяет последующий caller IncreaseRp.
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    game.increase_owned_player_rp(player_id, true, 0);
}

pub fn execute_player_base_attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> BaseAttackExecutionOutcome
where
    Game: BaseAttackGame + BaseAttackContact<Runtime>,
{
    let instance = game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID);
    let result = execute_stage(game, player_id, dispatch, player_ai, runtime, now_milliseconds);
    let end = match result {
        BaseAttackExecutionOutcome::Begun | BaseAttackExecutionOutcome::Pending => None,
        BaseAttackExecutionOutcome::Completed => Some((1, SkillTermination::Completed)),
        BaseAttackExecutionOutcome::Rejected => Some((0, SkillTermination::Rejected)),
        BaseAttackExecutionOutcome::RejectedAfterUse => Some((1, SkillTermination::Rejected)),
    };
    if let Some((instance, (argument, termination))) = instance.zip(end)
        && game.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended())
    {
        game.with_published_player_ai(player_id, player_ai, |game| {
            game.end_registered_instance(instance, argument, termination, runtime);
        });
    }
    result
}

fn execute_stage<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> BaseAttackExecutionOutcome
where
    Game: BaseAttackGame + BaseAttackContact<Runtime>,
{
    if game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).is_none() {
        game.replace_player_skill_visual_effect(
            player_id, BASE_ATTACK_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::BaseAttack, 1),
        );
        if game.find_player(player_id).is_none() { return BaseAttackExecutionOutcome::Rejected; }
        let Some(level) = skill_level(game, player_id) else { return BaseAttackExecutionOutcome::Rejected; };
        if game.skill_base_properties(BASE_ATTACK_SKILL_ID, level).is_none() {
            return BaseAttackExecutionOutcome::Rejected;
        }
        let Some(started) = game.player_skill_lifecycle(player_id, BASE_ATTACK_SKILL_ID)
            .map(|lifecycle| lifecycle.started_at_ms())
        else { return BaseAttackExecutionOutcome::Rejected; };
        game.begin_player_skill_execution(player_id, BaseAttackExecutionState::begin(dispatch, started));
        return BaseAttackExecutionOutcome::Begun;
    }
    let Some(level) = skill_level(game, player_id) else { return BaseAttackExecutionOutcome::Rejected; };
    let Some(properties) = game.skill_base_properties(BASE_ATTACK_SKILL_ID, level) else {
        return BaseAttackExecutionOutcome::Rejected;
    };
    let Some(player) = game.find_player(player_id) else { return BaseAttackExecutionOutcome::Rejected; };
    let target = target(game, player_id);
    if target.is_some_and(|(region, target)| game.base_magic_target_dead(region, target)) {
        game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 2);
        return BaseAttackExecutionOutcome::Completed;
    }
    if game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(source_view) = player.shape_view() else { return BaseAttackExecutionOutcome::Rejected; };
        let Some(destination) = game.player_skill_lifecycle(player_id, BASE_ATTACK_SKILL_ID)
            .map(|lifecycle| lifecycle.destination())
        else { return BaseAttackExecutionOutcome::Rejected; };
        let target_view = target.and_then(|(region, target)| {
            if target.object_type == PLAYER_TYPE {
                game.find_player(target.id).and_then(BaseAttackPlayer::shape_view)
            } else {
                game.base_magic_target_view(region, target)
            }
        });
        let distance = target_view.map_or_else(
            || player.shape().real_distance_to_point(destination.0, destination.1),
            |target| source_view.real_distance(Some(target)),
        );
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
            && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < distance as u32
        {
            game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 11);
            return BaseAttackExecutionOutcome::Rejected;
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        let (target_x, target_y) = if let Some((region, identity)) = target {
            let Some(target) = game.resolve_state_move_shape(region, identity) else {
                return BaseAttackExecutionOutcome::Rejected;
            };
            let (Ok(x), Ok(y)) = (target.shape().get_tile_x(), target.shape().get_tile_y()) else {
                return BaseAttackExecutionOutcome::Rejected;
            };
            (x, y)
        } else { destination };
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return BaseAttackExecutionOutcome::Rejected; };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(BASE_ATTACK_SKILL_ID, level) else {
        return BaseAttackExecutionOutcome::Rejected;
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID)
        .map(BaseAttackExecutionState::started_at_ms)
    else { return BaseAttackExecutionOutcome::Rejected; };
    if now_milliseconds() < started.wrapping_add(delay) {
        return BaseAttackExecutionOutcome::Pending;
    }
    game.update_player_skill_visual(player_id, BASE_ATTACK_SKILL_ID, 1);
    // Attack заново разрешает S после visual; первоначальная цель AI нужна
    // только для ранней смерти, дальности и поворота.
    let target = self::target(game, player_id);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
    }
    game.with_published_player_ai(player_id, player_ai, |game| {
        attack(game, player_id, target, runtime)
    });
    if let Some(kernel) = game.player_skill_execution_mut(player_id, BASE_ATTACK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    // OnFirstAttack принадлежит receiver; общий legacy first_contact вызвал бы
    // здесь лишний OnFirstSkill уже после попадания.
    BaseAttackExecutionOutcome::Completed
}

fn monster_sufferer<Game: BaseAttackGame>(
    game: &Game,
    instance: Game::SkillAddress,
) -> Option<(i32, ShapeIdentity)> {
    let target = game.resolve_skill_sufferer(game.registered_skill(instance)?.lifecycle())?;
    let shape = game.resolve_state_move_shape(target.0, target.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

fn calculate_monster_attack<Game: BaseAttackGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.damage_modifier = 0;
    // CMonster наследует GetWeaponModifier = 1; GetCCH и GetAddElementAtk
    // возвращают ноль. Calculate не записывает skill ID/level в seed.
    attack.damage_factor = 1.0;
    attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let Some((_, maximum)) = game.monster_base_attack_bounds(source) else { return; };
    let Some((minimum, _)) = game.monster_base_attack_bounds(source) else { return; };
    let span = (maximum as i32).wrapping_sub(minimum as i32).max(0);
    let Some((minimum, _)) = game.monster_base_attack_bounds(source) else { return; };
    let physical = (minimum as i32).wrapping_add(game.skill_random_below(span)).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 });
    let Some(soul) = game.monster_base_attack_soul_attack(source) else { return; };
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(soul), mp_damage: 0 });
    // В отличие от MonsterBaseAttack, этот RNG вызывается и при CCH == 0.
    let _ = game.skill_random_below(100);
}

fn attack_by_monster<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) where
    Game: BaseAttackGame + BaseAttackContact<Runtime>,
{
    let Some(target) = target else { return; };
    if game.resolve_state_move_shape(source.0, source.1).is_none()
        || (source.1.object_type == target.1.object_type && source.1.id == target.1.id)
        || target.1.object_type == NPC_TYPE
        || !game.live_skill_target_attackable(source.0, source.1, target.1)
    { return; }
    let master = MasterInfo { master_type: source.1.object_type, master_id: source.1.id, ..MasterInfo::default() };
    let mut attack = AttackInformation::for_master(master);
    calculate_monster_attack(game, instance, source, &mut attack);
    // Исчезнувшая таблица оставляет пустой UNKNOWN/1 seed, но не отменяет
    // вызов +15C(..., false). CMonster::IncreaseRp после него — пустой RET8.
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

fn execute_monster_stage<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> Option<i32>
where
    Game: BaseAttackGame + BaseAttackContact<Runtime>,
{
    let skill = game.registered_skill(instance)?;
    let kernel = *skill.monster_kernel()?;
    if kernel.lifecycle().is_ended() { return None; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return Some(0); };
    let lifecycle = *skill.lifecycle();
    let Some(source_shape) = game.resolve_state_move_shape(lifecycle.user().0, lifecycle.user().1) else { return Some(0); };
    let source = (source_shape.shape().get_region_id(), source_shape.shape().identity());
    let target = monster_sufferer(game, instance);
    if target.is_some_and(|target| game.base_magic_target_dead(target.0, target.1)) {
        game.update_registered_skill_visual(instance, 2);
        return Some(1);
    }
    if kernel.stage() == SkillStage::Begin {
        let Some(source_view) = game.base_magic_target_view(source.0, source.1) else { return Some(0); };
        let distance = if let Some(target) = target {
            let Some(target_view) = game.base_magic_target_view(target.0, target.1) else { return Some(0); };
            source_view.real_distance(Some(target_view))
        } else {
            source_shape.shape().real_distance_to_point(lifecycle.destination().0, lifecycle.destination().1)
        };
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
            && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < distance as u32
        {
            game.update_registered_skill_visual(instance, 11);
            return Some(0);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        game.registered_skill_mut(instance)?.lifecycle_mut().set_available(can_break != 0);
        let (target_x, target_y) = if let Some(target) = target {
            let shape = game.resolve_state_move_shape(target.0, target.1)?.shape();
            (shape.get_tile_x().ok()?, shape.get_tile_y().ok()?)
        } else {
            lifecycle.destination()
        };
        let source_shape = game.resolve_state_move_shape(source.0, source.1)?.shape();
        let source_y = source_shape.get_tile_y().ok()?;
        let source_x = source_shape.get_tile_x().ok()?;
        game.resolve_state_move_shape_mut(source.0, source.1)?.shape_mut().set_direction(
            get_line_direction(source_x, source_y, target_x, target_y),
        );
        game.update_registered_skill_visual(instance, 0);
        let _ = game.registered_skill_mut(instance)?.monster_kernel_mut()?.advance(SkillStage::Begin, SkillStage::Check);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let started = game.registered_skill(instance)?.lifecycle().started_at_ms();
    if now_milliseconds() < started.wrapping_add(delay) { return None; }
    game.update_registered_skill_visual(instance, 1);
    let target = monster_sufferer(game, instance);
    attack_by_monster(game, instance, source, target, runtime);
    Some(1)
}

/// Публикуется исходный регион целиком: visual, приём удара и вложенный End
/// работают с тем же экземпляром. FIFO и выбор следующего навыка остаются у AI.
pub fn execute_owned_monster_base_attack<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: BaseAttackGame + BaseAttackContact<Runtime>,
{
    let Some(holder) = owner.as_ref().and_then(|owner| game.base_attack_monster_holder(owner, monster_id))
    else { return false; };
    game.with_published_region(owner, |game| {
        let Some(instance) = game.registered_move_shape_skill(holder.0, holder.1, BASE_ATTACK_SKILL_ID) else { return false; };
        if let Some(argument) = execute_monster_stage(game, instance, runtime, now_milliseconds)
            && game.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended())
        {
            let termination = if argument != 0 { SkillTermination::Completed } else { SkillTermination::Rejected };
            game.end_registered_instance(instance, argument, termination, runtime);
        }
        true
    }).unwrap_or(false)
}

pub fn cancel_player_base_attack<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    nonzero_end: bool,
    runtime: &mut Runtime,
) -> bool
where
    Game: BaseAttackGame,
{
    let Some(instance) = game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID) else {
        return false;
    };
    let Some(dispatch) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    if game.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended()) {
        game.with_published_player_ai(player_id, player_ai, |game| {
            game.end_registered_instance(instance, i32::from(nonzero_end), SkillTermination::Cancelled, runtime);
        });
    }
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub fn abort_player_base_attack_on_region_change<Game: BaseAttackGame>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    let instance = game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    if let Some(instance) = instance {
        game.end_registered_instance_without_after_use(instance, SkillTermination::Cancelled);
    }
    game.finish_registered_player_command(instance, player_ai, dispatch, SkillTermination::Cancelled)
}

// ============================================================================
// FUNCTION: CBaseAttack::Restart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:110
// RVA: 0x00113E00
// ADDRESS: 00513e00
// PROTOTYPE: void __thiscall Restart(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
