//! Поклеточный арбалетный выстрел `CPoisonMoth` (`0xCF`): собственные
//! CheckCastCondition и AI (фаза направления, подготовка пути, полёт по
//! одной клетке за тик). Поклеточный удар и формула принадлежат семейным
//! hub `crossbowattack`/`rangedweaponcast` прежнего пакета (объявленные швы,
//! здесь не дублируются); оркестрация player-cast и общий End — hub
//! `playercast`/`states/skill.rs`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/poisonmoth.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/poisonmoth.rs`; тела Check/AI перенесены
//! буквально (кластер D полосы «трупная/ядовая state-линия», порция D6,
//! карта — запись аудита «Zone skills: карта полосы Monster 0x19x — 5
//! кластеров волн», 26 сентября 2026).
//!
//! Машинная разведка порции по этой паре (запись `.local/recon-de/notes/
//! D8-poisonmoth.md`, тела `.local/recon-de/disasm/CPoisonMoth.txt`);
//! сопоставление с перенесённым кодом — MATCH по всем пунктам:
//!
//! - Check `0x58C4C0`: null U → 0; self-target (arg2==U) → visual(10) +
//!   GS0286 (клей самонаведения здесь); затем семейный скелет
//!   `check_ranged_weapon_cast` с правилом пути DistanceAndBlocks и
//!   арбалетом категории 4 — шов `rangedweaponcast`: reuse → visual(13) +
//!   GS0278; dist 5003 (jbe) → visual(11) + GS0290; клетка third==2
//!   (BLOCK_UNFLY) → visual(15) + GS0282; арбалет иначе visual(14) + GS0293;
//!   Query(2)==0 → тихий ret 0; signed MP → visual(7) + GS0288; иначе
//!   SetMoveable(0), ret 1; non-player — без Move0/MP/арбалета.
//! - AI `0x58CED0`, фаза 0 (player): списание MP → `vcall+0x164`
//!   (OnChangeStates) → повторная проверка арбалета (шов
//!   `prepare_ranged_weapon_player`); CAN(10006) → `[+0x3C]`; GetSufferer:
//!   свежие X/Y S, иначе сохранённая точка команды; GetLineDir → SetDir на
//!   U; visual(0); `[+0x50]=1`.
//! - Подготовка пути ([+0x54]==0): Query(10001)+started unsigned → out;
//!   **SetMoveable(1)** → GetTargetPath в payload навыка; дальность: отказ
//!   iff `size > max + 1` (jae проход) — **оригинальная квазнота MAX+1
//!   против MAX в Check** → visual(11) + GS0290 → End(0); первая клетка
//!   third==2 задаёт endpoint (заблокированная), иначе последняя;
//!   flight = Query(10008) × count до неё; visual(1); запуск фазы атаки.
//! - Полёт ([+0x54]==1): deadline = Query(10008) × position + Query(10001) +
//!   started (signed/unsigned переполнение — wrapping); текущий регион U
//!   проверяется **после часов**, даже для завершённого пути (нет региона —
//!   Pending без End); index ≥ size → visual(3) + End(1); текущая клетка
//!   сохраняется в payload; block клетки: 3 (BLOCK_SHAPE) → поклеточный
//!   удар (шов), попадание → visual(3) + position = size + 1; 2
//!   (BLOCK_UNFLY) → visual(3) + стоп; иначе шаг вперёд; **одна клетка за
//!   AI-тик**, дедупликации целей нет; **двойной visual(3)** — при
//!   остановке и повторно на тике завершения перед End(1).
//! - Attack(JJ) `0x58CD80` — семейный `run_poison_moth_cell` (шов): (0,0)-
//!   гейт; GetShapes клетки без типового фильтра (RTTI CMoveShape), skip
//!   null/self/IsDied; IsAttackAble(U) `vcall+0x134`; **visual-target пишется
//!   перед ударом**; ret — был ли хотя бы один удар. Attack(U,S) `0x58CC70`:
//!   MasterInfo-канва игрока + Calculate + OnBeenAttacked(&info,0) — семейный
//!   контакт (шов). Calc `0x58C9E0` — семейный `calculate_crossbow_attack`
//!   (`kind 5` записей общего хвоста; hit = −Query(20001) только у 0xCF,
//!   element-добавка 0 вместо Query(20013), jns/jge-clamp, второй RNG крита
//!   обязателен; f64-модель произведения факторов — принятая модель
//!   combat.md). Эти семейные тела сверены ранее и здесь не переоткрываются.
//! - End `0x58C3A0`: сброс фазы/счётчиков/цели, освобождение пути до
//!   свежего U Move1 и общего Attack End с исходным аргументом — payload
//!   `PoisonMothExecutionState::clear_end_paths` (Zone `skills/execution/
//!   payload.rs`, перенесён раньше) + hub-кадр End зарегистрированного
//!   экземпляра прежнего пакета.
//!
//! Объявленные швы переноса (не расхождения): трейты `PoisonMothGame`/
//! `PoisonMothMoveShape` и `PoisonMothContact<Runtime>` ниже — фасады
//! прежних `CGame`/`CMoveShape` и вызовы семей `rangedweaponcast`
//! (`check_ranged_weapon_cast`, `prepare_ranged_weapon_player`,
//! `ranged_weapon_failure` с арбалетом) и `crossbowattack`
//! (`run_poison_moth_cell`) в прежних точках; оркестрация
//! `execute_registered_player_cast` (Begin-запись, visual-ресурс, End по
//! исходу) остаётся hub `playercast`. Потребление статическое (generic),
//! dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).

use nebokrai_shared::runtime::get_line_direction;

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;

use super::baseattackruntime::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME};
use super::execution::{MonsterSkillExecutionAccess, PoisonMothExecutionState, RegisteredSkillRecord};
use super::lifecycle::{SkillLifecycle, SkillStage};

pub const POISON_MOTH_SKILL_ID: u32 = 0xCF;

const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;

/// Стадии исхода одного тика AI `CPoisonMoth`; обёртка очереди с полем
/// `first_contact` и End по исходу остаются у делегата/hub `playercast`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoisonMothAiOutcome {
    Pending,
    Rejected,
    Completed,
}

/// Живая фигура-источник выстрела: переходный фасад прежнего `CMoveShape`.
pub trait PoisonMothMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);
}

/// Переходные фасады прежнего владельца `CGame`, открывающие `CPoisonMoth`
/// только прежние обращения; имена членов сохраняют исходную операцию.
pub trait PoisonMothGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ);
    /// непрозрачен для исполнения.
    type SkillAddress: Copy;

    type MoveShape: PoisonMothMoveShape;

    // Реестр и исполнение экземпляра (фасады `CGame`).
    fn registered_skill(
        &self,
        address: Self::SkillAddress,
    ) -> Option<&RegisteredSkillRecord<Self::MonsterExecution>>;

    fn registered_skill_mut(
        &mut self,
        address: Self::SkillAddress,
    ) -> Option<&mut RegisteredSkillRecord<Self::MonsterExecution>>;

    fn update_registered_skill_visual(&mut self, address: Self::SkillAddress, mode: u32);

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

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

    /// Точный общий `GetSufferer`: сохранённая identity, затем клетка региона.
    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)>;

    /// Общий `GetTargetPath` записи навыка (`vcall+0x58` живого экземпляра).
    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    /// Существование живого региона после часов полёта (`find_region`).
    fn poison_moth_region_present(&self, region_id: i32) -> bool;

    /// `CRegion::GetBlock` клетки навыка; `None` — регион отсутствует
    /// (caller трактует как BLOCK_UNFLY, `map_or(2, …)` прежнего тела).
    fn poison_moth_skill_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8>;

    // Швы `rangedweaponcast` (оружейный Check-скелет и MP-контракт).
    /// `ranged_weapon_failure` с арбалетом: visual + GS-строка режима.
    fn poison_moth_weapon_failure(
        &mut self,
        address: Self::SkillAddress,
        player: Option<i32>,
        mode: u32,
    );

    /// `prepare_ranged_weapon_player` с арбалетом: списание MP →
    /// OnChangeStates → повторная проверка оружия фазы 0.
    fn prepare_poison_moth_weapon_player(
        &mut self,
        address: Self::SkillAddress,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool;
}

/// Контакт с runtime игрового хода: семейный оружейный Check и поклеточный
/// удар `crossbowattack`.
pub trait PoisonMothContact<Runtime>: PoisonMothGame {
    /// `check_ranged_weapon_cast` с правилом пути DistanceAndBlocks и
    /// арбалетом категории 4 (reuse, путь 5003/BLOCK_UNFLY, оружие, MP).
    fn check_poison_moth_ranged_weapon_cast(
        &mut self,
        address: Self::SkillAddress,
        original_user: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) -> bool;

    /// Поклеточный удар Attack(JJ) `0x58CD80` (`run_poison_moth_cell`
    /// семьи `crossbowattack`): (0,0)-гейт, живые цели клетки без типового
    /// фильтра, IsAttackAble, visual-target перед каждым ударом; `true` —
    /// был хотя бы один удар (stop-flag).
    fn run_poison_moth_cell(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        cell: (i32, i32),
        runtime: &mut Runtime,
    ) -> bool;
}

/// `CheckCastCondition` `0x58C4C0`: клей самонаведения (arg2==U → visual(10)
/// + GS0286) затем семейный оружейный Check; S координатной команды уже
/// разрешён общим входом и приходит аргументом.
pub fn check_poison_moth_cast<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
    target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool
where
    Game: PoisonMothContact<Runtime>,
{
    let Some(user) = original_user else { return false; };
    let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { return false; };
    let targets_self = target
        .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
        .is_some_and(|target| std::ptr::eq(source, target));
    if targets_self {
        game.poison_moth_weapon_failure(
            instance, (user.1.object_type == PLAYER_TYPE).then_some(user.1.id), 10,
        );
        return false;
    }
    game.check_poison_moth_ranged_weapon_cast(instance, user, runtime)
}

/// Собственный AI `0x58CED0` одного тика: фаза направления fазы 0, разовая
/// подготовка пути с квазнотой MAX+1, затем одна клетка полёта за тик.
pub fn execute_poison_moth_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> PoisonMothAiOutcome
where
    Game: PoisonMothContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return PoisonMothAiOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return PoisonMothAiOutcome::Pending;
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return PoisonMothAiOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return PoisonMothAiOutcome::Rejected; };
    let player = (user.1.object_type == PLAYER_TYPE).then_some(user.1.id);
    if stage == SkillStage::Begin {
        if !game.prepare_poison_moth_weapon_player(instance, player, &properties) {
            return PoisonMothAiOutcome::Rejected;
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return PoisonMothAiOutcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return PoisonMothAiOutcome::Rejected; };
        let destination = match game.resolve_skill_sufferer(skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = game.resolve_state_move_shape(region, identity) else {
                    return PoisonMothAiOutcome::Rejected;
                };
                (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                )
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = game.resolve_state_move_shape(user.0, user.1) else {
            return PoisonMothAiOutcome::Rejected;
        };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(source) = game.resolve_state_move_shape_mut(user.0, user.1) {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<PoisonMothExecutionState>())
        .map(|state| state.attacking_started)
    else { return PoisonMothAiOutcome::Rejected; };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return PoisonMothAiOutcome::Rejected;
        };
        if now(runtime) < started.wrapping_add(delay) {
            return PoisonMothAiOutcome::Pending;
        }
        if let Some(source) = game.resolve_state_move_shape_mut(user.0, user.1) {
            source.set_moveable(true);
        }
        let Some(skill) = game.registered_skill(instance) else { return PoisonMothAiOutcome::Rejected; };
        let path = game.skill_target_path(skill.lifecycle());
        let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        else { return PoisonMothAiOutcome::Rejected; };
        state.path = path;
        if properties.query_property(TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(TARGET_MAX_DISTANCE);
            // Оригинальная квазнота: отказ iff size > max + 1 (jae проход).
            if maximum.wrapping_add(1) < state.path.len() as u32 {
                game.poison_moth_weapon_failure(instance, player, 11);
                return PoisonMothAiOutcome::Rejected;
            }
        }
        let stop = state.path.iter().position(|cell| cell.2 == 2).unwrap_or(state.path.len());
        let blocked_endpoint = state.path.get(stop).map(|cell| (cell.0, cell.1));
        let last_endpoint = state.path.last().map(|cell| (cell.0, cell.1));
        if let Some(endpoint) = blocked_endpoint {
            if let Some(skill) = game.registered_skill_mut(instance) {
                skill.lifecycle_mut().set_destination(endpoint);
            }
        }
        let flight = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(stop as u32);
        let Some(skill) = game.registered_skill_mut(instance) else { return PoisonMothAiOutcome::Rejected; };
        if let Some(state) = skill.player_state_mut::<PoisonMothExecutionState>() {
            state.missile_flying_time = flight;
        }
        if blocked_endpoint.is_none() {
            if let Some(endpoint) = last_endpoint {
                skill.lifecycle_mut().set_destination(endpoint);
            }
        }
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        {
            state.attacking_started = true;
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    let step = properties.query_property(MISSILE_FLYING_TIME);
    let Some(skill) = game.registered_skill(instance) else { return PoisonMothAiOutcome::Rejected; };
    let Some(state) = skill.player_state::<PoisonMothExecutionState>() else {
        return PoisonMothAiOutcome::Rejected;
    };
    if !state.attacking_started { return PoisonMothAiOutcome::Pending; }
    let position = state.current_position;
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let deadline = step.wrapping_mul(position).wrapping_add(delay).wrapping_add(skill.lifecycle().started_at_ms());
    if now(runtime) < deadline { return PoisonMothAiOutcome::Pending; }
    // Текущий регион U проверяется после часов, даже для завершённого пути.
    let Some(region_id) = game.resolve_state_move_shape(user.0, user.1)
        .filter(|source| source.shape().is_assigned_to_server_region())
        .map(|source| source.shape().get_region_id())
        .filter(|region_id| game.poison_moth_region_present(*region_id))
    else { return PoisonMothAiOutcome::Pending; };
    let Some(cell) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<PoisonMothExecutionState>())
        .and_then(|state| state.path.get(state.current_position as usize).copied())
    else {
        game.update_registered_skill_visual(instance, 3);
        return PoisonMothAiOutcome::Completed;
    };
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
    { state.end_tile = (cell.0, cell.1); }
    let block = game.poison_moth_skill_cell_block(region_id, cell.0, cell.1).unwrap_or(2);
    let stop = match block {
        3 => game.run_poison_moth_cell(instance, user, (cell.0, cell.1), runtime),
        2 => true,
        _ => false,
    };
    if stop {
        game.update_registered_skill_visual(instance, 3);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        { state.current_position = state.path.len() as u32; }
    }
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
    { state.current_position = state.current_position.wrapping_add(1); }
    PoisonMothAiOutcome::Pending
}
