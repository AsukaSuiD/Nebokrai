//! Координатор семейства навыков боевого духа: общий зарегистрированный
//! вход, общий End-контракт и wire visual девятнадцати тел
//! `*Effect::UpdateVisualEffect`.
//!
//! End-контракт живёт в hub `states/skill.rs` старого пакета (здесь не
//! дублируется): AfterUse при arg≠0 → общий `CSkill::End` с занулением
//! девяти DWORD, reuse-штамп только при arg≠0, прямой delete effect → ended.
//! Failure-кадр — `[4, mode]` точечно игроку; доставка `0xBF918` расхода MP
//! — точечная `SendToPlayer` (машинная форма).
//!
//! Quirk: CFatalBlow и CLeiming2 в visual пишут живой тип юзера `[user+4]`,
//! CThunder — литерал 700 (`BattleFairySourceType`); Po-семейство идёт в
//! fire без проверки sufferer (достижимость — UNKNOWN).
//!
//! Швы: трейты `BattleFairyPlayer`/`BattleFairyMoveShape`/`BattleFairyGame` —
//! фасады старого пакета (файл-делегат `appserver/skills/battlefairyskill.rs`);
//! hub-lifecycle арены состояний и end-оркестрация — у того же владельца.
//!
//! Исходные владельцы PDB: семейство `appserver/skills/*`, базовый
//! `appserver/states/skill.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#battlefairyskill--координатор-и-wire-visual

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::{MasterInfo, PlayerCombatProperties};
use crate::content::CSkillBaseProperties;
use crate::effects::{BattleFairyAttributeState, LifeShieldState};
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, ShapeFigure};

use super::dispatch::BattleFairySkillDispatch;
use super::execution::{
    BattleFairyExecution, MonsterSkillExecutionAccess, RegisteredSkillDispatch,
    RegisteredSkillRecord,
};
use super::lifecycle::{
    SkillExecutionKernel, SkillLifecycle, SkillTermination,
};
use super::skillfactory::{SkillEndEffect, SkillOwner};
use super::state::{StateData, StateKey};
use super::visualeffect::SkillVisualEffectKind;

const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;
const BATTLE_FAIRY_EFFECT_MESSAGE: i32 = 0x000b_fe01;
const BATTLE_FAIRY_GOODS_UPDATE_MESSAGE: i32 = 0x0b_f918;
const BATTLE_FAIRY_STATE_CONFLICT_MESSAGE: i32 = 0x0b_f807;

#[derive(Clone, Copy)]
enum BattleFairyFireTarget {
    Optional,
    Required,
    Point,
    Source,
    LifeShield,
}

/// Источник wire-типа source в кадрах begin/fire (решение A шапки файла).
#[derive(Clone, Copy)]
enum BattleFairySourceType {
    /// Литерал 700 во всех ветках (CThunder и пять остальных атакующих).
    LiteralWarSoul,
    /// Живой `[user+4]` только в fire (12 кастеров Po/Yu/transfer/Wangsheng).
    LiveOnFire,
    /// Живой `[user+4]` в case0 и fire: CFatalBlow `0x51E538`/`0x51FB3B`/
    /// `0x51E5C8` и CLeiming2. Динамика war-soul — INFERRED (шапка, пункт A).
    LiveAlways,
}

/// Только различия wire конкретных классов. База ресурса и её lifecycle
/// общие; здесь нет копий source, target, уровня или исполнения навыка.
struct BattleFairyVisualContract {
    target: BattleFairyFireTarget,
    failure_modes: &'static [u32],
    fire_time: bool,
    source_type: BattleFairySourceType,
    fire_id: Option<u32>,
    zero_prefix_mode: Option<u32>,
    wide_failure_eight: bool,
    boolean_end_effect: Option<SkillEndEffect>,
}

impl BattleFairyVisualContract {
    fn for_owner(owner: SkillOwner) -> Option<Self> {
        use BattleFairyFireTarget as Target;
        use BattleFairySourceType as SourceType;
        use SkillOwner::*;
        let mut result = Self {
            target: Target::Required,
            failure_modes: &[2, 7, 10, 11, 13, 15],
            fire_time: false,
            source_type: SourceType::LiteralWarSoul,
            fire_id: None,
            zero_prefix_mode: None,
            wide_failure_eight: false,
            boolean_end_effect: None,
        };
        match owner {
            BFBaseAttack => { result.target = Target::Optional; result.fire_time = true; }
            CFatalBlow => {
                result.fire_time = true; result.failure_modes = &[2, 7, 10, 11, 13, 14, 15];
                result.source_type = SourceType::LiveAlways;
            }
            CThunder | CLeiming2 => {
                result.target = Target::Point;
                if owner == CLeiming2 {
                    result.zero_prefix_mode = Some(7);
                    result.source_type = SourceType::LiveAlways;
                }
            }
            CTianhuo => { result.target = Target::Optional; result.fire_id = Some(0x13a); }
            CBloodLoss | CPoisonArrow => {
                result.failure_modes = &[2, 7, 8, 10, 11, 13, 14, 15];
                if owner == CBloodLoss { result.zero_prefix_mode = Some(13); }
            }
            CLifeShield => { result.target = Target::LifeShield; result.failure_modes = &[2, 7, 8, 13, 14]; }
            CPojia | CPobing | CPomo | CPofa | CYujia | CYubing | CYumo | CYufa
            | CHuoxieshu | CLingzhishu | CWangsheng => {
                if !matches!(owner, CPojia | CPobing | CPomo | CPofa) { result.target = Target::Source; }
                result.source_type = SourceType::LiveOnFire; result.wide_failure_eight = true;
                result.failure_modes = if owner == CHuoxieshu { &[2, 6, 8, 13] } else { &[2, 7, 8, 13] };
                result.boolean_end_effect = Some(if matches!(owner, CHuoxieshu | CLingzhishu | CWangsheng) {
                    SkillEndEffect::BattleFairySummon
                } else {
                    SkillEndEffect::BattleFairyState
                });
            }
            _ => return None,
        }
        Some(result)
    }
}

/// PK-допуски источника для полей `permitted_to_kill_*` `tagMasterInfo`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyPkPermissions {
    pub player: bool,
    pub teammate: bool,
    pub guild_member: bool,
    pub criminal: bool,
}

/// Игрок-заклинатель боевого духа: фасад `CPlayer` старого пакета.
pub trait BattleFairyPlayer {
    fn player_id(&self) -> i32;

    /// Форма игрока (identity, клетки, direction, region-link).
    fn shape(&self) -> &CShape;

    /// Живая фигура для проекции `ShapeView` выстрела базовой атаки феи.
    fn figure(&self) -> ShapeFigure;

    fn health(&self) -> u32;

    fn set_health(&mut self, health: u32);

    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);

    /// POINT боевого духа для временного SetTileXY выстрела BFBaseAttack.
    fn war_soul_point(&self) -> (i32, i32);

    fn team_id(&self) -> i32;

    fn faction_id(&self) -> i32;

    fn union_id(&self) -> i32;

    fn country(&self) -> u8;

    /// Живой снимок боевых свойств (cch, add_element и формулы снарядов).
    fn combat_properties(&self) -> PlayerCombatProperties;

    fn occupation(&self) -> u8;

    fn level(&self) -> u8;

    fn battle_fairy_pk_permissions(&self) -> BattleFairyPkPermissions;

    /// `m_tgCurrentWarSoulSkill` выбранного зарегистрированного навыка феи.
    fn selected_battle_fairy_skill_id(&self) -> u32;
}

/// Живая фигура-участник BF-исполнения: фасад `CMoveShape` старого пакета.
pub trait BattleFairyMoveShape {
    fn shape(&self) -> &CShape;

    /// Первый живой слот исходного m_vStates; предикат задаёт выбор caller-а.
    fn find_state_position(
        &self,
        matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)>;

    fn state_at(&self, position: usize) -> Option<(StateKey, &StateData)>;
}

/// Фасады `CGame` старого пакета, открывающие BF-ядру только
/// прежние обращения; имена сохраняют исходную операцию.
pub trait BattleFairyGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ старого
    /// пакета); непрозрачен для BF-ядра.
    type SkillAddress: Copy;

    type Player: BattleFairyPlayer;

    type MoveShape: BattleFairyMoveShape;

    // Реестр и исполнение экземпляра (фасады `CGame`).
    fn registered_skill(
        &self,
        address: Self::SkillAddress,
    ) -> Option<&RegisteredSkillRecord<Self::MonsterExecution>>;

    fn registered_skill_mut(
        &mut self,
        address: Self::SkillAddress,
    ) -> Option<&mut RegisteredSkillRecord<Self::MonsterExecution>>;

    fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<Self::SkillAddress>;

    // Игроки.
    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut Self::Player>;

    // Разрешение сторон и пространственные допуски.
    /// Живой `GetUser` по сохранённым region/type/id.
    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&Self::MoveShape>;

    /// Точный общий `GetSufferer`: сохранённая identity, затем клетка региона.
    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)>;

    /// Живая фигура для проекции `ShapeView`: ветка 400/500/600/1100/1200
    /// остаётся у делегата (players/monster properties/stationary build).
    fn battle_fairy_shape_figure(&self, region_id: i32, identity: ShapeIdentity) -> Option<ShapeFigure>;

    /// Существование живого региона (допуск `summon_user_region`).
    fn battle_fairy_region_exists(&self, region_id: i32) -> bool;

    /// Временный/возвратный SetTileXY игрока выстрела BFBaseAttack.
    fn set_player_tile_position(&mut self, player_id: i32, tile_x: i32, tile_y: i32);

    // Определения уровня и случайность.
    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    fn skill_random_below(&mut self, maximum: i32) -> i32;

    /// Пять нижних границ blast/full-miss из globe-установок.
    fn base_combat_scales(&self) -> [f32; 5];

    // Цель и путь.
    /// Общий факт `GetSufferer` для объектной цели (HP/god у движущихся).
    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    fn base_magic_target_name(&self, region_id: i32, target: ShapeIdentity) -> Option<&[u8]>;

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    fn skill_target_path_with_length(
        &self,
        lifecycle: &SkillLifecycle,
        length: u32,
    ) -> Vec<(i32, i32, u8)>;

    // Visual, диагностика и wire.
    fn update_registered_skill_visual(&mut self, address: Self::SkillAddress, mode: u32);

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    fn send_skill_system_info_with_text(&self, player_id: i32, text: &[u8], name: &[u8]);

    /// Цветной notice `0xBF806` (ZHGS0011 у октета Po/Yu).
    fn send_battle_fairy_notice(&self, player_id: i32, first_color: u32, second_color: u32, text: &[u8]);

    /// Точечный SendToPlayer подготовленного кадра (failure visual, `0xBF807`,
    /// `0xBF918` решения C).
    fn send_battle_fairy_message_to_player(&self, player_id: i32, message: &CMessage);

    /// Around-доставка visual-кадра с гейтом существующего региона прежнего
    /// caller-а (`send_game_shape_around` без исключений).
    fn send_battle_fairy_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);

    /// Строка сообщений исходной таблицы (ZHGS-тексты).
    fn battle_fairy_string(&self, id: &[u8]) -> &[u8];

    fn publish_player_states(&self, player_id: i32) -> bool;

    fn update_move_shape_properties(&mut self, region_id: i32, holder: ShapeIdentity) -> bool;

    // Предмет боевого духа (слот equipment[10]; war-soul — с marker-допуском).
    /// Наличие живого предмета в слоте 10 без чтения свойств (AI-гейт transfer).
    fn battle_fairy_equipment_present(&self, player_id: i32) -> bool;

    /// Чтение addon-свойства живого предмета в слоте 10 без marker-допуска.
    fn battle_fairy_equipment_addon(&self, player_id: i32, property: i32) -> Option<i32>;

    /// Тот же слот 10: identity GUID и addon-значение одним захватом.
    fn battle_fairy_equipment_identity_addon(
        &self,
        player_id: i32,
        property: i32,
    ) -> Option<(CGuid, i32)>;

    /// `GetWarSoulGoods` + addon-свойство (свойство читается после допуска).
    fn battle_fairy_war_soul_addon(&self, player_id: i32, property: i32) -> Option<i32>;

    /// Только маркер `GetWarSoulGoods` без чтения свойства.
    fn battle_fairy_war_soul_goods_present(&self, player_id: i32) -> bool;

    /// Общий setter equipment-свойства (перезагружает fairy-проекции внутри
    /// делегата), слот 10, offset 1.
    fn set_battle_fairy_equipment_addon(
        &mut self,
        player_id: i32,
        property: i32,
        value: i32,
    ) -> Option<()>;

    /// `SerializeForOldClient` живого предмета слота 10 и его GUID; отказ
    /// сериализации не подавляет отправку (решение C).
    fn battle_fairy_equipment_payload(&self, player_id: i32) -> Option<(CGuid, Vec<u8>)>;

    /// Та же сериализация предмета, возвращённого `GetWarSoulGoods`.
    fn battle_fairy_war_soul_payload(&self, player_id: i32) -> Option<(CGuid, Vec<u8>)>;

    /// Списание MP боевого духа с сериализацией обновлённого предмета
    /// (`spend_war_soul_mana_record` прежнего владельца).
    fn spend_battle_fairy_mana(&mut self, player_id: i32, amount: u32) -> Option<(CGuid, Vec<u8>)>;

    // Арена состояний (hub-lifecycle остаётся прежнему владельцу).
    /// Первый живой слот с нужным ID проходит End и destructor свежего
    /// остатка позиции (`end_and_destroy_state_at` прежнего hub-а).
    fn battle_fairy_end_first_state(&mut self, region_id: i32, holder: ShapeIdentity, state_id: u32) -> bool;

    /// Primary Begin состояния Po/Yu с собственными часами и silent loop1.
    fn begin_battle_fairy_attribute_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        sufferer: ShapeIdentity,
        state: BattleFairyAttributeState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool;

    /// Primary Begin щита жизни через прежний hub самозащитных состояний.
    fn begin_life_shield_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        user: Option<(i32, ShapeIdentity)>,
        sufferer: Option<(i32, ShapeIdentity)>,
        state: LifeShieldState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool;

    // Завершение зарегистрированного BF-экземпляра (общий End-контракт
    // `0x5DFBD0`; orchestration остаётся прежнему hub `states/skill.rs`).
    fn prepare_registered_skill_end_effect(
        &mut self,
        address: Self::SkillAddress,
        effect: SkillEndEffect,
        argument: i32,
    ) -> bool;

    fn end_registered_battle_fairy(
        &mut self,
        address: Self::SkillAddress,
        argument: i32,
        termination: SkillTermination,
        now: &mut dyn FnMut() -> u32,
    ) -> bool;

    /// End(0) без AfterUse: не требует ни часов, ни контекста износа.
    fn end_registered_battle_fairy_without_after_use(
        &mut self,
        address: Self::SkillAddress,
        termination: SkillTermination,
    ) -> bool;

    // Призванные формы.
    fn allocate_summon_shape_id(&mut self) -> i32;
}

/// Плоский итог одного тика BF-исполнения; обёртка очереди с полем
/// `first_contact` и строковые соответствия остаются у делегата старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySkillOutcome {
    Begun,
    Pending,
    Completed,
    Rejected,
    /// Применение отклонено, но владелец требует успешный хвост End(1).
    RejectedAfterUse,
}

pub fn publish_battle_fairy_visual<Game: BattleFairyGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    let Some(contract) = BattleFairyVisualContract::for_owner(skill.owner()) else { return };
    if skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::BattleFairy || effect.is_ended()) {
        return;
    }
    let (region_id, source_identity) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(region_id, source_identity) else { return };
    let source = source.shape();
    let identity = source.identity();
    let mut message = CMessage::new(BATTLE_FAIRY_EFFECT_MESSAGE);
    if contract.failure_modes.contains(&mode) {
        if identity.object_type != 400 { return; }
        if mode == 8 && contract.wide_failure_eight { message.add_long(4); }
        else { message.add_byte(if contract.zero_prefix_mode == Some(mode) { 0 } else { 4 }); }
        message.add_byte(mode as u8);
        game.send_battle_fairy_message_to_player(identity.id, &message);
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    message.add_byte(action);
    message.add_long(if action == 2 { contract.fire_id.unwrap_or(skill.id()) } else { skill.id() } as i32);
    message.add_short(skill.level() as i16);
    let live_source_type = match contract.source_type {
        BattleFairySourceType::LiveAlways => matches!(action, 1 | 2),
        BattleFairySourceType::LiveOnFire => action == 2,
        BattleFairySourceType::LiteralWarSoul => false,
    };
    message.add_long(if live_source_type { identity.object_type } else { BATTLE_FAIRY_VISUAL_OBJECT_TYPE });
    message.add_long(identity.id);
    if action == 2 {
        let sufferer = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, target)| game.resolve_state_move_shape(region, target))
            .map(|target| target.shape());
        if matches!(contract.target, BattleFairyFireTarget::Required | BattleFairyFireTarget::LifeShield)
            && sufferer.is_none() { return; }
        let target = if matches!(contract.target, BattleFairyFireTarget::Source) { Some(source) } else { sufferer };
        let target_identity = target.map(|target| target.identity());
        let zero_identity = matches!(contract.target, BattleFairyFireTarget::Point | BattleFairyFireTarget::LifeShield);
        message.add_long(if zero_identity { 0 } else { target_identity.map_or(0, |identity| identity.object_type) });
        message.add_long(if zero_identity { 0 } else { target_identity.map_or(0, |identity| identity.id) });
        if !matches!(contract.target, BattleFairyFireTarget::LifeShield) {
            let (x, y) = match target {
                Some(target) => {
                    let (Ok(x), Ok(y)) = (target.get_tile_x(), target.get_tile_y()) else { return };
                    (x, y)
                }
                None => skill.lifecycle().destination(),
            };
            message.add_long(x); message.add_long(y);
        }
        if contract.fire_time {
            let time = match skill.battle_fairy_execution_state() {
                Some(BattleFairyExecution::BaseMagic(state)) => state.attack_time(),
                Some(BattleFairyExecution::FatalBlow(state)) => state.missile_flying_time(),
                _ => 0,
            };
            message.add_ulong(time);
        }
    } else { message.add_long(source.get_direction()); }
    game.send_battle_fairy_visual_around(source.get_region_id(), source, &message);
}

/// Конфликт выбирается по первой позиции исходного массива состояний, а не
/// по приоритету ID. Цвет SystemInfo здесь белый, без дополнительного visual.
pub fn check_battle_fairy_target_states<Game: BattleFairyGame>(
    game: &Game,
    player_id: i32,
    target: (i32, ShapeIdentity),
) -> bool {
    let Some(holder) = game.resolve_state_move_shape(target.0, target.1) else { return false; };
    let conflict = holder.find_state_position(|state| matches!(state.state_id(), 0x192 | 0xd2 | 0x67))
        .and_then(|(position, _)| holder.state_at(position))
        .map(|(_, state)| state.state_id());
    let Some(state_id) = conflict else { return true; };
    let text = game.battle_fairy_string(if state_id == 0xd2 { b"ZHGS0047" } else { b"ZHGS0046" });
    let mut message = CMessage::new(BATTLE_FAIRY_STATE_CONFLICT_MESSAGE);
    message.add_ulong(0xffff_ffff);
    let length = text.iter().position(|byte| *byte == 0).unwrap_or(text.len());
    message.base_mut().add(&text[..length]);
    message.base_mut().add_byte(0);
    game.send_battle_fairy_message_to_player(player_id, &message);
    false
}

/// Навыки без собственных скалярных полей используют тот же общий вход.
pub fn execute_registered_battle_fairy_state<Game: BattleFairyGame, Runtime: ?Sized, Outcome>(
    game: &mut Game,
    player_id: i32,
    instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    begin_failure_visual: Option<u32>,
    rejected: impl Fn() -> Outcome,
    installed: impl Fn() -> Outcome,
    check: impl FnOnce(&mut Game, Game::SkillAddress, i32, &mut Runtime) -> bool,
    run_ai: impl FnOnce(&mut Game, Game::SkillAddress, &mut Runtime) -> Outcome,
) -> Outcome {
    execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, begin_failure_visual, rejected, installed,
        check,
        |dispatch, started| BattleFairyExecution::State(SkillExecutionKernel::begin(dispatch, started)),
        run_ai,
    )
}

/// Общая материализация сохраняет единственную базу зарегистрированного
/// экземпляра. Координатор уже выполнил base Begin и опубликовал AI; здесь
/// нет вторых часов или End. Фабрика создаёт лишь собственные данные owner-а.
#[allow(clippy::too_many_arguments, reason = "форма повторяет исходный общий вход семейства")]
pub fn execute_registered_battle_fairy_skill<Game: BattleFairyGame, Runtime: ?Sized, Outcome>(
    game: &mut Game,
    player_id: i32,
    instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    begin_failure_visual: Option<u32>,
    rejected: impl Fn() -> Outcome,
    installed: impl Fn() -> Outcome,
    check: impl FnOnce(&mut Game, Game::SkillAddress, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(BattleFairySkillDispatch, u32) -> BattleFairyExecution,
    run_ai: impl FnOnce(&mut Game, Game::SkillAddress, &mut Runtime) -> Outcome,
) -> Outcome {
    let Some(skill) = game.registered_skill(instance) else { return rejected() };
    if skill.id() != dispatch.skill_id() { return rejected() }
    if let Some(previous) = skill.battle_fairy_dispatch() {
        if previous != dispatch { return rejected() }
        return run_ai(game, instance, runtime);
    }
    if !check(game, instance, player_id, runtime) {
        if let Some(mode) = begin_failure_visual {
            game.update_registered_skill_visual(instance, mode);
        }
        return rejected();
    }
    let Some(skill) = game.registered_skill_mut(instance) else { return rejected() };
    let execution = materialize(dispatch, skill.lifecycle().started_at_ms());
    if !skill.install_battle_fairy_execution(execution) { return rejected() }
    installed()
}

/// SetWarSoulXY (0x0042DF50) и DelWarSoul (0x0042E0A0) вызывают End(int,0)
/// выбранного зарегистрированного навыка, если его база ещё не ended.
/// Payload и текущая команда не являются gates; очередь и target не снимаются.
/// Собственный End(bool) Po/Yu/transfer сюда не подставляется.
pub fn cancel_active_battle_fairy_skill<Game: BattleFairyGame>(
    game: &mut Game,
    player_id: i32,
) -> bool {
    let Some(skill_id) = game.find_player(player_id).map(|player| player.selected_battle_fairy_skill_id()) else { return false };
    let Some(instance) = game.registered_player_skill(player_id, skill_id) else { return false };
    let Some(skill) = game.registered_skill(instance) else { return false };
    if skill.lifecycle().is_ended() { return false; }
    let dispatch = skill.battle_fairy_dispatch();
    let _ = game.end_registered_battle_fairy_without_after_use(instance, SkillTermination::Cancelled);
    if let Some(dispatch) = dispatch && let Some(skill) = game.registered_skill_mut(instance) {
        skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
    }
    true
}

/// Терминальная граница после concrete owner и contacts. Выполняет точный
/// concrete End(bool/int); общий End сохраняет источник до AfterUse и
/// только затем сбрасывает базу. Callback не может перенести reuse/cleanup
/// на новую регистрацию того же ID. Неуспешный Begin не требует payload.
pub fn finish_registered_battle_fairy_skill<Game: BattleFairyGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    argument: i32,
    termination: SkillTermination,
    begin_attempted: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false };
    let materialized = skill.battle_fairy_dispatch() == Some(dispatch);
    let failed_begin = begin_attempted && !skill.lifecycle().is_ended()
        && skill.is_execution_inactive();
    if !materialized && !failed_begin {
        return false;
    }
    let argument = if materialized { argument } else { 0 };
    prepare_battle_fairy_boolean_end(game, instance);
    let _ = game.end_registered_battle_fairy(instance, argument, termination, now);
    if let Some(skill) = game.registered_skill_mut(instance) {
        skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
    }
    true
}

fn prepare_battle_fairy_boolean_end<Game: BattleFairyGame>(game: &mut Game, instance: Game::SkillAddress) {
    if let Some(effect) = game.registered_skill(instance)
        .and_then(|skill| BattleFairyVisualContract::for_owner(skill.owner()))
        .and_then(|contract| contract.boolean_end_effect)
    {
        let _ = game.prepare_registered_skill_end_effect(instance, effect, 0);
    }
}

pub fn send_battle_fairy_skill_failure<Game: BattleFairyGame>(game: &Game, player_id: i32, action: u8) {
    let mut message = CMessage::new(BATTLE_FAIRY_EFFECT_MESSAGE);
    message.add_byte(4);
    message.add_byte(action);
    game.send_battle_fairy_message_to_player(player_id, &message);
}

/// Обновление предмета боевого духа wire `0xBF918` (решение C шапки файла):
/// оригинал доставляет `SendToPlayer(player_id)` точечно — машинные якоря
/// `0x501BDE..0x501C61` и `0x51F249`. Отказ Serialize не подавляет пакет:
/// payload приходит от делегата уже без статуса. Форма кадра: id, GUID,
/// len, blob `SerializeForOldClient`.
pub fn send_battle_fairy_goods_update<Game: BattleFairyGame>(
    game: &Game,
    player_id: i32,
    ex_id: CGuid,
    old_client_payload: &[u8],
) {
    let mut message = CMessage::new(BATTLE_FAIRY_GOODS_UPDATE_MESSAGE);
    message.add_long(player_id);
    message.base_mut().add_guid(ex_id);
    message.add_ulong(old_client_payload.len() as u32);
    message.base_mut().add(old_client_payload);
    game.send_battle_fairy_message_to_player(player_id, &message);
}

// Read-проекции BF-summon (здесь единственная копия; старый пакет
// пользуется ими через публичную поверхность модуля).

pub fn summon_user_cch<Game: BattleFairyGame>(game: &Game, source: ShapeIdentity) -> i32 {
    if source.object_type == 400 {
        game.find_player(source.id).map_or(0, |player| i32::from(player.combat_properties().cch))
    } else { 0 }
}

pub fn summon_user_add_element<Game: BattleFairyGame>(game: &Game, source: ShapeIdentity) -> i32 {
    if source.object_type == 400 {
        game.find_player(source.id).map_or(0, |player| player.combat_properties().add_element_attack as i32)
    } else { 0 }
}

pub fn summon_user_region<Game: BattleFairyGame>(game: &Game, source: (i32, ShapeIdentity)) -> Option<i32> {
    let shape = game.resolve_state_move_shape(source.0, source.1)?.shape();
    if !shape.is_assigned_to_server_region() { return None; }
    let region = shape.get_region_id();
    if !game.battle_fairy_region_exists(region) { return None; }
    Some(region)
}

/// `MasterInfo` заклинателя боевого духа по живым полям игрока (общее тело
/// BF-summon семьи, собрано здесь без дублирования чтений).
pub fn battle_fairy_master_info<Player: BattleFairyPlayer>(player: &Player) -> MasterInfo {
    let permissions = player.battle_fairy_pk_permissions();
    MasterInfo {
        master_type: 400, master_id: player.player_id(),
        master_guild_id: player.faction_id(), master_team_id: player.team_id(),
        master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}
