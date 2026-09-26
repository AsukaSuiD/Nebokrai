//! Общая геометрия, визуальный формат и контактная атака рывков
//! Flash/LittleFlash, плюс hub-швы семейства melee-рывков
//! (dash/flash/littleflash/rush). Источник: точная пара
//! `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! appserver/skills/flash.cpp, littleflash.cpp и littleflash2.cpp;
//! тела перенесены буквально.
//!
//! Путь и список поражённых целей принадлежат concrete владельцам и не
//! копируются через callback атаки. Общая обработка пути сохраняет блоки
//! GetTargetPath, проверяет первую фигуру клетки и выбирает выход сначала
//! среди восьми соседей, затем через существующий CRegion random-поиск.
//! LittleFlash2 не требует занятую клетку и очищает одиночный закрытый выход;
//! поклеточный удар и live IsAttackAble остаются у вызывающего AI.
//! Единый формат visual передаёт последнюю клетку подготовленного пути;
//! безусловный базовый callback остаётся у зарегистрированного ресурса.
//!
//! Контакт использует общий оружейный расчёт с коэффициентом TARGET_DAMAGE_FACTOR.
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — фасады
//! `CGame`/`CPlayer`/`CMoveShape` старого пакета, реализация
//! остаётся у него в файле-делегате `appserver/skills/dash.rs`; имена членов
//! сохраняют исходную операцию. Швы потребляются статически (generic),
//! dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).
//! Региональные швы `dash_*` перечитывают owner-а региона на каждый вызов:
//! native держит один borrow региона на весь цикл AI, в пределах одного тика
//! регион не покидает map; первичный гейт существования региона сохранён,
//! а недостижимое исчезновение посередине трактуется закрытой клеткой
//! (`unwrap_or(1)`) — безопасный отказ без выдуманной свободной клетки.
//! Сравнение `std::ptr::eq` живых фигур сохранено над прежним `CMoveShape`
//! (assoc-тип `MoveShape`), а не над разрешённой `CShape`.
//! Часы `now_milliseconds` приходят от делегата старого main loop
//! (fn-параметр, `game_tick_milliseconds` у caller-а), как в
//! `skills/baseattackruntime.rs`.
//!
//! Сознательные отклонения от native-UB (сохранены со старого файла):
//! `check_dash_path` обращается к `back()` пути даже после подрезки
//! `truncate(maximum)` до нуля — пустой путь остаётся безопасным отказом без
//! выдуманной клетки назначения.
//!
//! Оставшиеся UNKNOWN/PARTIAL: порядок геттеров X/Y
//! (сквозная согласованность подтверждена), место push в Attack-списке
//! CFlash, потребители raw CAN `available`. Машинные статусы сверенных
//! пунктов — в шапках `flash.rs`, `littleflash.rs` и `rush.rs`.

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::citygate::CITY_GATE_OBJECT_TYPE;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, ShapeView};
use crate::skills::state::StateKey;

use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::lifecycle::SkillLifecycle;
use super::rush::{Rush2State, RushState};
use super::visualeffect::SkillVisualEffectKind;

const TARGET_DAMAGE_FACTOR: u32 = 20_003;

/// PK-допуски источника рывков для полей `permitted_to_kill_*` `tagMasterInfo`
/// (та же четвёрка, что и у базовой атаки, без отдельной страны).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DashSkillPkPermissions {
    pub player: bool,
    pub teammate: bool,
    pub guild_member: bool,
    pub criminal: bool,
}

/// Игрок-источник рывка: фасад старого `CPlayer`.
pub trait DashSkillPlayer {
    /// Форма игрока (identity, клетки, direction).
    fn shape(&self) -> &CShape;

    fn mana(&self) -> u32;

    fn rp(&self) -> u16;

    fn set_mana(&mut self, mana: u32);

    fn set_rp(&mut self, rp: u16);

    fn level(&self) -> u8;

    fn player_id(&self) -> i32;

    fn faction_id(&self) -> i32;

    fn team_id(&self) -> i32;

    fn union_id(&self) -> i32;

    fn country(&self) -> u8;

    fn pk_permissions(&self) -> DashSkillPkPermissions;

    /// Живой `HasStateBySkillId` игрока (Pillar-гейт семейства).
    fn has_state_by_skill_id(&self, skill_id: u32) -> bool;

    fn set_skill_moveable(&mut self, moveable: bool);
}

/// Живая фигура стороны рывка: фасад старого `CMoveShape`.
pub trait DashSkillMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);
}

/// Стадии результата одного тика ИИ рывка; обёртка очереди с полем
/// `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DashSkillExecutionOutcome {
    Pending,
    Rejected,
    Completed,
}

/// Фасады `CGame` старого пакета, открывающие семейству рывков
/// только прежние обращения; имена сохраняют исходную операцию.
pub trait DashSkillGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для исполнения.
    type SkillAddress: Copy;

    type Player: DashSkillPlayer;

    type MoveShape: DashSkillMoveShape;

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

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    fn skill_target_path_with_length(
        &self,
        lifecycle: &SkillLifecycle,
        maximum: u32,
    ) -> Vec<(i32, i32, u8)>;

    // Ячейки региона dash-семейства: owner перечитывается на каждый вызов
    // (см. шапку о шве); первичный гейт существования региона сохранён.
    fn dash_skill_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8>;

    fn dash_region_size(&self, region_id: i32) -> Option<(i32, i32)>;

    /// Первая фигура клетки области региона (GetShape области), живой доступ.
    fn dash_shape_view_at(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeView>;

    /// Все фигуры клетки области региона (GetShapes), пустой список при отказе.
    fn dash_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView>;

    /// Уровень монстра региона: monster → base_property_key → реестр свойств.
    fn dash_monster_level(&self, region_id: i32, monster_id: i32) -> Option<u8>;

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    fn publish_player_states(&self, player_id: i32);

    /// Живой addon категории оружия (`GAP_WEAPON_CATEGORY`, index 1) в слоте 2;
    /// резолв фабрики предметов и отсутствующего оружия остаётся у владельца.
    fn player_weapon_addon_category(&self, player: &Self::Player) -> Option<i32>;

    fn set_player_tile_position(&mut self, player_id: i32, tile_x: i32, tile_y: i32);

    fn relocate_region_shape(&mut self, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32);

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        user: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8>;

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    fn skill_target_controller(&self, region_id: i32, target: ShapeIdentity) -> Option<i32>;

    /// Позиция состояния `state_id` в живом списке фигуры (find_state_position).
    fn move_shape_state_position(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        state_id: u32,
    ) -> Option<usize>;

    /// Полный End + destructor по найденной позиции (end_and_destroy_state_at);
    /// признак завершения у caller-ов отбрасывается, как и раньше.
    fn end_move_shape_state_at(&mut self, region_id: i32, identity: ShapeIdentity, index: usize);

    // Доставка visual `0x000BFE01`: кадр строится здесь, маршруты — у владельца.
    /// Personal-ветвь по numeric identity игрока.
    fn send_dash_visual_to_player(&self, player_id: i32, message: &CMessage);

    /// Around-ветвь региона формы; отсутствующий регион пропускает отправку.
    fn send_dash_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);

    /// Общий Begin блокировки движения/боя состояния рывка (0x73) —
    /// прежний `begin_primary_blind_state` семейства Blind.
    fn begin_rush_state(
        &mut self,
        target: (i32, ShapeIdentity),
        user: Option<(i32, ShapeIdentity)>,
        state: RushState,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<StateKey>;

    /// То же для состояния второго рывка (0x7C).
    fn begin_rush_2_state(
        &mut self,
        target: (i32, ShapeIdentity),
        user: Option<(i32, ShapeIdentity)>,
        state: Rush2State,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<StateKey>;

    /// ForceMove игрока/монстра/постройки (BF604 → SetTileXY → AI-ожидание);
    /// развёрнутый результат у caller-ов всегда отбрасывался через `let _`.
    fn force_move_skill_target(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        tile_x: i32,
        tile_y: i32,
        duration_ms: u32,
    );
}

/// Контактные операции с runtime игрового хода: общий удар оружейного расчёта,
/// контроллер первого удара и прямой virtual `OnBeenAttacked(..., false)`;
/// random-поиск свободной клетки расходует тот же поток случайных чисел, что и
/// native GetRandomPosInRange от runtime. Отделены, потому что тип хода
/// принадлежит старому main loop, а не самому семейству рывков.
pub trait DashSkillContact<Runtime>: DashSkillGame {
    /// Общий удар семейства: fresh Calculate, raw OnBeenAttacked и RP послесловие
    /// (`apply_player_weapon_attack` старого пакета с переданным коэффициентом).
    fn apply_dash_weapon_attack(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        damage_factor_usage: u32,
        runtime: &mut Runtime,
    );

    /// PK-контроллер первого удара игрока против текущего владельца цели рывка.
    fn rush_first_attack_at_position(
        &mut self,
        player_id: i32,
        controller: i32,
        region_id: Option<i32>,
        position: (i32, i32),
        runtime: &mut Runtime,
    );

    /// Прямой virtual `OnBeenAttacked(..., false)`, без IsAttackAble.
    fn apply_owned_skill_contact(
        &mut self,
        master: crate::combat::MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: crate::combat::AttackInformation,
        runtime: &mut Runtime,
    );

    /// `CRegion::GetRandomPosInRange` региона с RNG текущего runtime.
    fn dash_random_pos_in_range(
        &mut self,
        region_id: i32,
        left: i32,
        top: i32,
        range_width: i32,
        range_height: i32,
        runtime: &mut Runtime,
    ) -> Option<(i32, i32)>;
}

pub fn publish_dash_visual<Game: DashSkillGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
    kind: SkillVisualEffectKind,
    destination: Option<(i32, i32)>,
) {
    if skill.visual_effect().is_none_or(|effect| effect.kind() != kind || effect.is_ended()) {
        return;
    }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 4 | 7 | 8 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_dash_visual_to_player(source.identity().id, &message);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if mode == 1 {
        let Some((x, y)) = destination else { return; };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(source.get_direction());
    }
    game.send_dash_visual_around(source.get_region_id(), source, &message);
}

pub fn check_dash_path<Game, Runtime>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    mut path: Vec<(i32, i32, u8)>,
    maximum: u32,
    require_shape_block: bool,
    clear_single_blocked: bool,
    runtime: &mut Runtime,
) -> Vec<(i32, i32, u8)>
where
    Game: DashSkillGame + DashSkillContact<Runtime>,
{
    if path.is_empty() { return path; }
    let Some(shape) = game.resolve_state_move_shape(source.0, source.1).map(|shape| shape.shape()) else {
        return Vec::new();
    };
    if !shape.is_assigned_to_server_region() { return Vec::new(); }
    let region_id = shape.get_region_id();
    let Some((region_width, region_height)) = game.dash_region_size(region_id) else {
        return Vec::new();
    };
    let (source_x, source_y) = (shape.get_tile_x().unwrap_or(i32::MIN), shape.get_tile_y().unwrap_or(i32::MIN));
    if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
    path.truncate(maximum as usize);
    // Native обращается к back() даже после подрезки до нуля. Пустой путь
    // остаётся безопасным отказом без выдуманной клетки назначения.
    let Some(last) = path.last().copied() else { return path; };
    if let Ok(next) = CShape::get_direction_position(
        get_line_direction(source_x, source_y, last.0, last.1),
        ShapeAreaCoordinates { x: last.0, y: last.1 },
    ) {
        let block = game.dash_skill_cell_block(region_id, next.x, next.y).unwrap_or(1);
        path.push((next.x, next.y, block));
    }
    let mut saw_shape_block = false;
    let mut trim_index = path.len();
    for (index, cell) in path.iter().enumerate() {
        let city_gate = game.dash_shape_view_at(region_id, cell.0, cell.1)
            .is_some_and(|shape| shape.identity.object_type == CITY_GATE_OBJECT_TYPE as i32);
        if city_gate || matches!(cell.2, 1 | 2) {
            trim_index = index.saturating_sub(1);
            break;
        }
        if !saw_shape_block { saw_shape_block = cell.2 == 3; }
        else if cell.2 != 3 { trim_index = index; break; }
    }
    if require_shape_block && !saw_shape_block { return Vec::new(); }
    if trim_index == path.len() { trim_index = path.len().saturating_sub(1); }
    path.truncate(trim_index.saturating_add(1));
    let Some(anchor) = path.last().copied() else { return path; };
    if anchor.2 != 0 {
        if clear_single_blocked && path.len() == 1 {
            path.clear();
            return path;
        }
        for direction in 0..8 {
            let Ok(candidate) = CShape::get_direction_position(
                direction, ShapeAreaCoordinates { x: anchor.0, y: anchor.1 },
            ) else { continue; };
            if candidate.x >= 0 && candidate.y >= 0 && candidate.x < region_width
                && candidate.y < region_height
                && game.dash_skill_cell_block(region_id, candidate.x, candidate.y).unwrap_or(1) & 7 == 0
            {
                path.push((candidate.x, candidate.y, 0));
                return path;
            }
        }
        if let Some(candidate) = game.dash_random_pos_in_range(
            region_id, anchor.0.wrapping_sub(2), anchor.1.wrapping_sub(2), 5, 5, runtime,
        ) {
            path.push((candidate.0, candidate.1, 0));
        }
    }
    path
}

pub fn apply_dash_attack<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) where
    Game: DashSkillGame + DashSkillContact<Runtime>,
{
    game.apply_dash_weapon_attack(instance, source, target, TARGET_DAMAGE_FACTOR, runtime);
}
