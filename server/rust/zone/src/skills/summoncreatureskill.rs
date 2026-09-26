//! Семья призыва `CSummonSkill` — дериваты `CSummonCorpseCandle` (0x19A),
//! `CSummonSkeleton` (0x19B), `CSummonSpore` (0x19C) и `CBossFiendSummon`
//! (0x1F9). Одноимённого класса `CSummonCreatureSkill` в PDB не существует:
//! Rust-файл — общий путь дериватов `CSummonSkill`.
//!
//! Машинные quirks: Check — только reuse и `SetMoveable(0)` (без пути и
//! дальности); `lifetime` = Query(30001) и picture-id читаются ВНУТРИ цикла
//! создания (у BossFiend `random(3)` до нулевой проверки количества);
//! reuse-clock читается после создания и публикации всех существ; JJ-вариант
//! Summon — stub у всей семьи (живой JJ только у `CSpiderMist`); поворота в
//! пакете визуала нет — внешний поворот монстру задаёт движение подхода,
//! игрока — клиент.
//!
//! Швы: hub-трейты — фасады прежнего владельца `CGame`/`CPlayer`/
//! `CServerRegion` (делегат `appserver/skills/summoncreatureskill.rs`);
//! state Begin — statefactory Zone.
//!
//! UNKNOWN: имя слота `+0x2C` тела Summon; GUID `0xEF3D9C` — INFERRED; ветви
//! 2..0xF jump-таблицы visual вне режимов 0/1.
//!
//! Исходные владельцы PDB: `appserver/skills/{summoncreatureskill,
//! summoncorpsecandle,summonskeleton,summonspore,bossfiendsummon}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#summoncreatureskill--семья-csummonskill-0x19a0x19b0x19c0x1f9

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::{CShape, ShapeView};

use super::baseattackruntime::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::dispatch::PlayerSkillDispatch;
use super::execution::{
    PlayerSkillExecution, PlayerSpiderMistExecutionState, PlayerSummonCreatureExecutionState,
    SpiderMistProgress,
};
use super::lifecycle::{SkillStage, SkillTermination, skill_is_restored};
use super::spidermist::SpiderMistPhalanx;

pub const SUMMON_CORPSE_CANDLE_SKILL_ID: u32 = 0x19a;
pub const SUMMON_SKELETON_SKILL_ID: u32 = 0x19b;
pub const SUMMON_SPORE_SKILL_ID: u32 = 0x19c;
pub const BOSS_FIEND_SUMMON_SKILL_ID: u32 = 0x1f9;

pub const SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME: u32 = 30_001;
pub const SKILL_USAGE_SUMMONED_CREATURE_ID: u32 = 30_003;
pub const SKILL_USAGE_CONST: u32 = 20_010;
pub const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

const SUMMON_VISUAL_MESSAGE: i32 = 0x000b_fe01;

/// `random(3)` до нулевой проверки количества: 1 → 30004, 2 → 30005,
/// ноль и любой иной исход — свойство по умолчанию 30003.
pub const fn boss_fiend_summoned_creature_usage(random_value: i32) -> u32 {
    match random_value {
        1 => 30_004,
        2 => 30_005,
        _ => SKILL_USAGE_SUMMONED_CREATURE_ID,
    }
}

/// Диспетчерская форма player-cast призыва: четыре ID семьи по любой форме.
pub const fn is_player_summon_creature_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        player_skill_id(dispatch),
        SUMMON_CORPSE_CANDLE_SKILL_ID
            | SUMMON_SKELETON_SKILL_ID
            | SUMMON_SPORE_SKILL_ID
            | BOSS_FIEND_SUMMON_SKILL_ID
    )
}

pub const fn player_skill_id(dispatch: PlayerSkillDispatch) -> u32 {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    }
}

/// Стадии результата одного тика исполнения призыва; обёртка очереди с
/// полем `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SummonSkillOutcome {
    Begun,
    Pending,
    Completed,
    Rejected,
}

/// Снимок читаемого исполнения монстра семьи (фасад старого `CMonster`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SummonMonsterCast {
    pub skill_id: u32,
    pub started_at_ms: u32,
}

/// Снимок монстра на входе исполнения: форма, attack-speed расписание,
/// живой cast и reuse-момент; резолв таблицы свойств остаётся в шве.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummonMonsterFacts {
    pub source: CShape,
    pub attack_interval_ms: u32,
    pub ai_kind: u32,
    pub cast: Option<SummonMonsterCast>,
    pub last_used_ms: u32,
}

/// Игрок-заклинатель семьи: переходный фасад прежнего `CPlayer`.
pub trait SummonSkillPlayer {
    /// Форма игрока (identity, клетки, direction «как есть»).
    fn shape(&self) -> &CShape;

    /// Поворот к сохранённой точке эффекта; машинно он существует только
    /// в AI `0x5409B0` паучьего тумана перед выпуском.
    fn cast_face_direction(&mut self, direction: i32);

    fn server_region_id(&self) -> Option<i32>;

    fn set_skill_moveable(&mut self, moveable: bool);

    fn set_current_skill_id(&mut self, skill_id: Option<u32>);

    /// Полный MasterInfo игрока (PK-допуски); у монстра — только type/id.
    fn master_info(&self) -> MasterInfo;
}

/// Переходные фасады прежнего владельца `CGame`, открывающие семье призыва
/// только прежние обращения; имена сохраняют исходную операцию.
pub trait SummonSkillGame {
    type Player: SummonSkillPlayer;

    /// Живой регион старого пакета (`CServerRegion`); непрозрачен для исполнения.
    type Region;

    /// Извлечённый владелец региона монстровой ветви паучьего тумана
    /// (`ServerRegionOwner` прежнего пакета); непрозрачен для исполнения.
    type RegionOwner;

    /// AI игрока для `finish_player_skill`; непрозрачен для исполнения.
    type PlayerAi;

    /// Доступ к базовому региону извлечённого владельца (`base_mut`).
    fn summon_region_base_mut(owner: &mut Self::RegionOwner) -> &mut Self::Region;

    // Игрок, определения и reuse.
    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut Self::Player>;

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32;

    /// Уровень выученного навыка игрока (чтение через фабрику — у владельца).
    fn summon_player_skill_level(&self, player_id: i32, skill_id: u32) -> Option<i32>;

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView>;

    // Typed-исполнение игрока семьи (payload в `skills/execution`).
    fn player_summon_creature_state(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&PlayerSummonCreatureExecutionState>;

    fn player_summon_creature_state_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut PlayerSummonCreatureExecutionState>;

    fn player_spider_mist_state(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&PlayerSpiderMistExecutionState>;

    fn player_spider_mist_state_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut PlayerSpiderMistExecutionState>;

    fn begin_player_execution(&mut self, player_id: i32, execution: PlayerSkillExecution) -> bool;

    // Публикации visual и завершение команды.
    /// Общий `update_player_current_state(..., MoveShapeAi)` прежнего пакета.
    fn update_player_fight_state_move_shape(&mut self, player_id: i32) -> bool;

    /// `send_self_state_skill_failure` по wire `0x000BFE01`.
    fn send_summon_cast_failure(&self, player_id: i32, action: u8);

    fn send_player_summon_visual(&mut self, player_id: i32, message: &CMessage);

    /// Around-доставка кадра от формы источника; отсутствующий владелец
    /// пропускает отправку внутри шва, как и раньше.
    fn send_summon_visual_around(&self, region: &Self::Region, origin: &CShape, message: &CMessage);

    fn finish_summon_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool;

    // Региональная арена и создание существ.
    /// Извлечённый владелец региона на время создания; гейт его
    /// существования и публикация остаются в шве старого пакета.
    fn with_summon_region<Output>(
        &mut self,
        region_id: i32,
        callback: impl FnOnce(&mut Self, &mut Self::Region) -> Output,
    ) -> Option<Output>;

    /// `GetRandomPosInRange`; не найденная клетка отсутствует, как
    /// `position.found == false` прежнего тела.
    fn summon_random_region_position(
        &mut self,
        region: &Self::Region,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Option<(i32, i32)>;

    /// `AddSummonedCreature(info, id, x, y, -1, lifetime)`: резолв свойства
    /// по picture-id выполняет шов; отсутствие свойства пропускает создание.
    fn summon_add_creature_by_picture(
        &mut self,
        region: &mut Self::Region,
        picture_id: u32,
        master: MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    ) -> bool;

    fn summon_allocate_shape_id(&mut self) -> i32;

    /// Входной срез монстра: форма, attack interval, живой cast и reuse
    /// заданного навыка; резолв базовой таблицы и приручённых свойств — внутри шва.
    fn summon_monster_facts(
        &self,
        region: &Self::Region,
        monster_id: i32,
        skill_id: u32,
    ) -> Option<SummonMonsterFacts>;

    fn summon_monster_shape(&self, region: &Self::Region, monster_id: i32) -> Option<CShape>;

    /// Снимок позиции объектной цели монстра (`target_view` прежнего тела).
    fn summon_monster_target_view(
        &self,
        region: &Self::Region,
        target: ShapeIdentity,
    ) -> Option<ShapeView>;

    /// `resolve_owned_monster_attack_target` семьи: форма и вид живой цели;
    /// разрешение построек/дочерних форм остаётся у владельца региона.
    fn summon_monster_attack_target(
        &self,
        owner: &Self::RegionOwner,
        target: ShapeIdentity,
    ) -> Option<(CShape, ShapeView)>;

    /// Независимое attack-speed расписание (`begin_ai_attack_attempt`).
    fn summon_monster_begin_attack_attempt(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool;

    /// `schedule_attack_interval` монстрового AI-владельца.
    fn summon_schedule_attack_interval(&self, ai_kind: u32, interval_ms: u32) -> Option<u32>;

    fn summon_skill_begin_object(
        &self,
        region: &Self::Region,
        target: ShapeIdentity,
    ) -> Option<(i32, ShapeIdentity)>;

    fn summon_monster_begin_cast(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        target: ShapeIdentity,
        skill_id: u32,
        skill_level: u16,
        now_ms: u32,
        target_object: Option<(i32, ShapeIdentity)>,
    );

    fn summon_monster_advance_cast(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        skill_id: u32,
        from: SkillStage,
        to: SkillStage,
    );

    fn summon_monster_cancel_cast(&mut self, region: &mut Self::Region, monster_id: i32);

    fn summon_monster_clear_ai_target(&mut self, region: &mut Self::Region, monster_id: i32);

    fn summon_monster_set_moveable(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        moveable: bool,
    );

    /// Поворот к сохранённой клетке паучьего тумана (машинно — только в AI).
    fn summon_monster_face_direction(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        direction: i32,
    );

    fn summon_straight_skill_path(
        &self,
        region: &Self::Region,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)>;

    fn summon_base_magic_path(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)>;

    fn summon_spider_mist_progress(
        &self,
        region: &Self::Region,
        monster_id: i32,
    ) -> Option<SpiderMistProgress>;

    fn summon_set_spider_mist_progress(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        progress: SpiderMistProgress,
    );

    // Клетки и цели живой области паучьего тумана.
    /// `GetShapes` клетки региона; отказ чтения прерывает обход (прежний break).
    fn summon_region_cell_identities(
        &self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<Vec<ShapeIdentity>>;

    fn summon_shape_present_in_region(&self, region_id: i32, identity: ShapeIdentity) -> bool;

    fn summon_shape_region_of(&self, region_id: i32, identity: ShapeIdentity) -> Option<i32>;

    fn summon_move_shape_health(&self, region_id: i32, identity: ShapeIdentity) -> Option<u32>;

    /// Наличие состояния навыка у живой фигуры; неразрешённая фигура —
    /// пропуск кандидата, как `None` прежнего результата разрешения.
    fn summon_shape_has_state(&self, region_id: i32, identity: ShapeIdentity, skill_id: u32) -> Option<bool>;

    fn summon_skill_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    /// Begin основного состояния паучьего яда (`begin_primary_poison_state`
    /// прежнего пакета; statefactory-тип состояния уже Zone).
    fn summon_begin_spider_poison_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        user: (i32, ShapeIdentity),
        state: super::statefactory::SpiderPoisonState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool;

    fn skill_random_below(&mut self, maximum: i32) -> i32;
}

/// Контактные операции с runtime игрового хода семьи. Отделены, потому что
/// тип хода принадлежит старому main loop, а не самому исполнению.
pub trait SummonSkillContact<Runtime>: SummonSkillGame {
    /// Подход монстра к дистанции цели (`approach_attack_range` владельца AI).
    fn summon_approach_attack_range(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        target_view: ShapeView,
        maximum_distance: u32,
        runtime: &mut Runtime,
    ) -> bool;

    /// Общий End монстрового каста с часами завершения (`CSkill::End` читает
    /// reuse после создания существ).
    fn summon_monster_finish_cast_clock(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    );

    /// Регистрация живой области паучьего тумана: адаптер к CShape и входной
    /// снимок `0xBF502` — внутри шва прежнего владельца.
    fn summon_add_spider_mist_phalanx(
        &mut self,
        region: &mut Self::Region,
        phalanx_id: i32,
        rule: &SpiderMistPhalanx,
        destination_x: i32,
        destination_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<i32>;

    /// `AfterUseSkill` игрока: износ оружия и reuse-штамп (`states/summonskill.rs`
    /// прежнего пакета).
    fn summon_after_use_player_skill(
        &mut self,
        player_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    );
}

/// Кадр начала `0x000BFE01`: action 1, навык, уровень, сторона и GetDir
/// источника «как есть» (машинно — без SetDir в телах семьи).
pub fn summon_visual_start_message(
    skill_id: u32,
    skill_level: i32,
    actor_type: i32,
    actor_id: i32,
    direction: i32,
) -> CMessage {
    let mut message = CMessage::new(SUMMON_VISUAL_MESSAGE);
    message.add_byte(1);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    message.add_long(direction);
    message
}

/// Кадр исполнения `0x000BFE01`: action 2, навык, уровень, сторона, два
/// нулевых long и клетка эффекта (fallback (0, 0) — у ветвей семьи).
pub fn summon_visual_fire_message(
    skill_id: u32,
    skill_level: i32,
    actor_type: i32,
    actor_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(SUMMON_VISUAL_MESSAGE);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

fn player_destination<Game: SummonSkillGame>(
    game: &Game,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
    source: (i32, i32),
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => Some(source),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game
            .base_magic_target_view(region_id, target)
            .map(|view| (view.tile_x, view.tile_y)),
    }
}

fn restore_player_movement<Game: SummonSkillGame>(game: &mut Game, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

pub fn cancel_player_summon_creature<Game: SummonSkillGame>(
    game: &mut Game,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut Game::PlayerAi,
) -> bool {
    let Some(dispatch) = game
        .player_summon_creature_state(player_id, execution_skill_id)
        .map(|state| state.kernel().dispatch())
    else { return false };
    restore_player_movement(game, player_id);
    game.finish_summon_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub fn execute_player_summon_creature<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> SummonSkillOutcome
where
    Game: SummonSkillContact<Runtime>,
{
    let skill_id = player_skill_id(dispatch);
    if !is_player_summon_creature_dispatch(dispatch) {
        return SummonSkillOutcome::Rejected;
    }
    let Some((region_id, source_x, source_y, skill_level, master)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                game.summon_player_skill_level(player_id, skill_id)?,
                player.master_info(),
            ))
        })
    else {
        return SummonSkillOutcome::Rejected;
    };
    let Some(properties) = game.skill_base_properties(skill_id, skill_level).cloned() else {
        restore_player_movement(game, player_id);
        return SummonSkillOutcome::Rejected;
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let amount = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = now_milliseconds();
    if game.player_summon_creature_state(player_id, skill_id).is_none() {
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, skill_id),
            reuse_delay_ms,
            now_ms,
        ) {
            game.send_summon_cast_failure(player_id, 0x0d);
            restore_player_movement(game, player_id);
            return SummonSkillOutcome::Rejected;
        }
        let Some(destination) = player_destination(game, region_id, dispatch, (source_x, source_y))
        else {
            return SummonSkillOutcome::Rejected;
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_execution(
            player_id,
            PlayerSummonCreatureExecutionState::begin(dispatch, destination, now_ms).into(),
        );
        return SummonSkillOutcome::Begun;
    }
    if game
        .player_summon_creature_state(player_id, skill_id)
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return SummonSkillOutcome::Rejected;
    }
    // Сохранённая точка эффекта читалась только поворотом Begin, которого
    // в оригинале нет; запись в payload хранится для wire-паритета.
    let _ = game
        .player_summon_creature_state(player_id, skill_id)
        .map(|state| state.destination)
        .expect("выполнение призыва хранит координаты эффекта");
    if game
        .player_summon_creature_state(player_id, skill_id)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        // Поворота к точке в оригинале нет: пакет ниже читает GetDir «как есть».
        let _ = game.update_player_fight_state_move_shape(player_id);
        if let Some(player) = game.find_player(player_id) {
            let direction = player.shape().get_direction();
            game.send_player_summon_visual(
                player_id,
                &summon_visual_start_message(skill_id, skill_level, PLAYER_TYPE, player_id, direction),
            );
        }
        if let Some(state) = game.player_summon_creature_state_mut(player_id, skill_id) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = game
        .player_summon_creature_state(player_id, skill_id)
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение призыва хранит время начала");
    if !skill_is_restored(started_at_ms, delay_ms, now_milliseconds()) {
        return SummonSkillOutcome::Pending;
    }
    let fire_destination = player_destination(game, region_id, dispatch, (source_x, source_y))
        .unwrap_or((0, 0));
    if game.find_player(player_id).is_some() {
        game.send_player_summon_visual(
            player_id,
            &summon_visual_fire_message(
                skill_id, skill_level, PLAYER_TYPE, player_id,
                fire_destination.0, fire_destination.1,
            ),
        );
    }
    let _ = game.with_summon_region(region_id, |game, region| {
        let creature_usage = if skill_id == BOSS_FIEND_SUMMON_SKILL_ID {
            boss_fiend_summoned_creature_usage(game.skill_random_below(3))
        } else {
            SKILL_USAGE_SUMMONED_CREATURE_ID
        };
        for _ in 0..amount {
            let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
            let picture_id = properties.query_property(creature_usage);
            let (tile_x, tile_y) = game
                .summon_random_region_position(
                    region,
                    source_x.wrapping_sub(4),
                    source_y.wrapping_sub(4),
                    8,
                    8,
                )
                .unwrap_or((0, 0));
            let _ = game.summon_add_creature_by_picture(
                region, picture_id, master, tile_x, tile_y, -1, lifetime_ms,
            );
        }
    });
    if let Some(state) = game.player_summon_creature_state_mut(player_id, skill_id) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    restore_player_movement(game, player_id);
    game.summon_after_use_player_skill(player_id, skill_id, runtime);
    SummonSkillOutcome::Completed
}

/// Общий объектный путь монстра/питомца семьи: подход к дистанции, reuse,
/// независимое attack-speed расписание, выпуск и создание K существ.
pub fn execute_owned_summon_creature<Game, Runtime>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
    target: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool
where
    Game: SummonSkillContact<Runtime>,
{
    let Some(facts) = game.summon_monster_facts(region, monster_id, skill_id) else {
        return false;
    };

    if let Some(cast) = facts.cast {
        if cast.skill_id != skill_id {
            return false;
        }
        if !skill_is_restored(
            cast.started_at_ms,
            properties.query_property(SKILL_USAGE_DELAY_TIME),
            now_ms,
        ) {
            return true;
        }
        let (target_x, target_y) = game
            .summon_monster_target_view(region, target)
            .map(|view| (view.tile_x, view.tile_y))
            .unwrap_or((0, 0));
        game.summon_monster_advance_cast(
            region, monster_id, skill_id, SkillStage::Check, SkillStage::Calculate,
        );
        let message = summon_visual_fire_message(
            skill_id,
            i32::from(skill_level),
            MONSTER_TYPE,
            monster_id,
            target_x,
            target_y,
        );
        game.send_summon_visual_around(region, &facts.source, &message);

        let amount = properties.query_property(SKILL_USAGE_CONST);
        let creature_usage = if skill_id == BOSS_FIEND_SUMMON_SKILL_ID {
            boss_fiend_summoned_creature_usage(game.skill_random_below(3))
        } else {
            SKILL_USAGE_SUMMONED_CREATURE_ID
        };
        let source_x = facts.source.get_tile_x().unwrap_or_default();
        let source_y = facts.source.get_tile_y().unwrap_or_default();
        let master = MasterInfo {
            master_type: MONSTER_TYPE,
            master_id: monster_id,
            ..MasterInfo::default()
        };
        for _ in 0..amount {
            let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
            let picture_id = properties.query_property(creature_usage);
            let (tile_x, tile_y) = game
                .summon_random_region_position(
                    region,
                    source_x.wrapping_sub(4),
                    source_y.wrapping_sub(4),
                    8,
                    8,
                )
                .unwrap_or((0, 0));
            let _ = game.summon_add_creature_by_picture(
                region, picture_id, master, tile_x, tile_y, -1, lifetime_ms,
            );
        }
        game.summon_monster_advance_cast(
            region, monster_id, skill_id, SkillStage::Calculate, SkillStage::Attack,
        );
        game.summon_monster_advance_cast(
            region, monster_id, skill_id, SkillStage::Attack, SkillStage::Apply,
        );
        game.summon_monster_finish_cast_clock(region, monster_id, skill_id, runtime);
        return true;
    }

    let Some(target_view) = game.summon_monster_target_view(region, target) else {
        game.summon_monster_clear_ai_target(region, monster_id);
        return true;
    };
    if !game.summon_approach_attack_range(
        region,
        monster_id,
        target_view,
        properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
        runtime,
    ) {
        return true;
    }
    if let Some(attack_interval_ms) =
        game.summon_schedule_attack_interval(facts.ai_kind, facts.attack_interval_ms)
        && !game.summon_monster_begin_attack_attempt(
            region, monster_id, now_ms, attack_interval_ms,
        )
    {
        return true;
    }
    if !skill_is_restored(
        facts.last_used_ms,
        properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
        now_ms,
    ) {
        return true;
    }
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let target_object = game.summon_skill_begin_object(region, target);
    // Поворота в Begin семьи нет: visual ниже читает GetDir «как есть».
    game.summon_monster_set_moveable(region, monster_id, false);
    game.summon_monster_begin_cast(
        region, monster_id, target, skill_id, skill_level, now_ms, target_object,
    );
    let source = game.summon_monster_shape(region, monster_id);
    let source = source.as_ref().unwrap_or(&facts.source);
    let message = summon_visual_start_message(
        skill_id,
        i32::from(skill_level),
        MONSTER_TYPE,
        monster_id,
        source.get_direction(),
    );
    game.send_summon_visual_around(region, source, &message);
    true
}

/// Точка назначения для поворота AI паучьего тумана; единственный машинный
/// потребитель `get_line_direction` семьи.
pub fn summon_face_direction(
    source_x: i32,
    source_y: i32,
    destination_x: i32,
    destination_y: i32,
) -> i32 {
    get_line_direction(source_x, source_y, destination_x, destination_y)
}
