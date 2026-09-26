//! Громовой удар `CThunderBlow` (`0x13F`) и его стационарная форма
//! `CThunderBlowPhalanx`. Источник: точная пара `gameserver.exe`
//! (SHA-256 `4F5C98E0…`) + `GameServer.pdb` (RSDS match), исходные владельцы
//! `appserver/skills/thunderblow.cpp` и `appserver/skills/thunderblowphalanx.cpp`.
//! Адресная конвенция факт-листа волны: истинный RVA (pub off + 0x1000;
//! VA = off + 0x401000, т.е. VA = RVA + 0x400000). Прежние переходные
//! владельцы — `src/gameserver/appserver/skills/thunderblow.rs` и
//! `thunderblowphalanx.rs`; тела перенесены буквально порцией T3
//! «ThunderBlow-пара». Решение волны: пара навыка и его области слита в один
//! файл по прецеденту `skills/spidermist.rs` — конструктор области
//! вызывается только здесь, а существующего zone-владельца полосы 0x13F нет.
//!
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка, расход
//! ресурсов и эффекты AI выполняются после постановки Attack в том же Run;
//! исходный отсчёт Begin сохраняется общим kernel. Владелец сохраняет две
//! проверки MP и дальности, время восстановления, задержку, направление и
//! визуальные пакеты. После задержки он создаёт в целевой проходимой клетке
//! принадлежащую региону форму; её срок жизни, поиск цели и формула остаются
//! у владельца формы. Собственный `End` не меняет движение, но вызывает
//! оружейный `CAttackSkill::End`; успех, отказ после `Begin` и клиентская
//! отмена используют один хвост с `AfterUseSkill` и временем восстановления.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; задержка
//! формы остаётся elapsed.
//!
//! Форма до истечения срока жизни проверяет собственную клетку в порядке
//! регионального индекса, атакует все допустимые цели текущего прохода и
//! после него завершается. Формула сохраняет два вызова генератора MSVCRT:
//! диапазон урона, затем критический удар. Критический множитель применяется
//! в расширенной точности x87 и усекается к нулю при записи результата в `i32`.
//!
//! Машинный факт (MATCH по снятой доказательной базе порции T3):
//!
//! - Check/AI (якорь `0x17A520`, VA `0x57A520`): AI в Begin — RTTI player →
//!   MP [+0x284] → signed-дефицит (visual7 + GS0288 + End0) / **SetMP
//!   сразу** → state-update (+0x164) → CAN=10006 → [+0x3C] → назначение из
//!   S/снимка → SetDirection → **повторная дальность** (visual 0x0B +
//!   GS0290 + End0 при частичных эффектах) → visual0 → advance.
//!   FIX порции T3: прежняя реконструкция проверяла повторную дальность ДО
//!   списания MP и поворота (отказ не терял ни того, ни другого); порядок
//!   приведён к машинному — списание MP, обновление состояния и поворот идут
//!   до повторной проверки, её отказ теряет уже списанные MP и поворот,
//!   общий хвост AfterUse выполняется. Расхождение достижимо только при
//!   смене цели/пути между начальным Check и этим AI (начальный Check уже
//!   проверил ту же дальность и дефицит MP).
//! - срок unsigned → IsDied S (visual 0x0A + GS0285 + End0) → S → точка
//!   (+0x24/+0x28) → visual1 → Summon (+0x8C) → End(1);
//! - Summon: region-RTTI → block (vcall +0x44) == 2 → abort → мёртвое 20015 →
//!   ctor (min/max/elem/level/lifetime) → SetCenter → same-cell свёртка →
//!   Add → 0xBF502. Свёртка, регистрация области и безусловный входной
//!   `0xBF502` исполняет прежний фасад `add_thunder_blow_phalanx` владельца
//!   региона (contact-шов ниже, здесь он не переоткрывается);
//! - `CThunderBlowPhalanx` ctor `0x1F5430` (VA `0x5F5430`): 6 аргументов,
//!   0xd8 байт; живое тело с 2 RNG и x87-критом — формуле ниже;
//! - `End` базы `CSummonSkill` `0x1E0F40` (VA `0x5E0F40`).
//!
//! Объявленные швы переноса (не расхождения): hub-трейты `ThunderBlowGame`/
//! `ThunderBlowPlayer`/`ThunderBlowContact` — переходные фасады прежнего
//! владельца `CGame`/`CPlayer` (реализация у делегата старого пакета
//! `appserver/skills/thunderblow.rs`); имена членов сохраняют исходную
//! операцию. `update_player_fight_state_move_shape` — тот же согласованный
//! фасад state-update (+0x164), что объявлен у семьи summoncreatureskill.
//! Отсутствующий регион области трактуется отказом Add, который молча
//! возвращает `None` (внешний эффект прежнего гейта caller-а сохранён).
//! Результат Add свёрнут до `bool`: прежний код использовал вариант ошибки
//! только как `is_ok()`. Часы `now` — шов делегата прежнего main-loop
//! runtime (`game_tick_milliseconds` у caller-а), как в
//! `skills/baseattackruntime.rs`; `time_reached` — локальная копия общего
//! адаптера прежнего baseattack (тело сохранено буквально). Потребление швов
//! статическое (generic), dyn-совместимость и `Send`-контракт не вводятся
//! (прецедент ADR-0013).
//!
//! UNKNOWN/объявленная неполнота: машинная запись CAN=10006 → [+0x3C] в
//! исполнение player-kernel отдельно не материализуется — прежняя
//! реконструкция (и семейная конвенция summon-полосы) только читает свойство
//! (`_can_be_breaked`); потребители поля +0x3C этой стороны в волне не
//! устанавливались.

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::runtime::get_line_direction;
use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::{
    AttackInformation, AttackPower, AttackPowerType, MasterInfo, PlayerCombatProperties,
    truncate_original,
};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeView};

use super::baseattackruntime::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::summonshape::SUMMON_SHAPE_TYPE;

pub const THUNDER_BLOW_SKILL_ID: u32 = 0x13f;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;
pub const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

/// PK-допуски источника удара для полей `permitted_to_kill_*` `tagMasterInfo`
/// (та же четвёрка, что у dash/базовой атаки, страна у этого владельца — ноль).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThunderBlowPkPermissions {
    pub player: bool,
    pub teammate: bool,
    pub guild_member: bool,
    pub criminal: bool,
}

/// Игрок-источник громового удара: переходный фасад старого `CPlayer`.
pub trait ThunderBlowPlayer {
    /// Форма игрока (identity, клетки, direction).
    fn shape(&self) -> &CShape;

    /// Изменяемая форма для поворота AI Begin.
    fn movement_shape_mut(&mut self) -> &mut CShape;

    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);

    fn player_id(&self) -> i32;

    fn faction_id(&self) -> i32;

    fn team_id(&self) -> i32;

    fn union_id(&self) -> i32;

    fn server_region_id(&self) -> Option<i32>;

    fn is_dead(&self) -> bool;

    fn set_current_skill_id(&mut self, skill_id: Option<u32>);

    fn combat_properties(&self) -> PlayerCombatProperties;

    fn occupation(&self) -> u8;

    fn level(&self) -> u8;

    fn pk_permissions(&self) -> ThunderBlowPkPermissions;
}

/// Переходные фасады прежнего владельца `CGame`, открывающие исполнению
/// `CThunderBlow` только прежние обращения; имена сохраняют исходную операцию.
pub trait ThunderBlowGame {
    type Player: ThunderBlowPlayer;

    /// AI игрока, передаваемый хвостам очереди; непрозрачен для исполнения.
    type PlayerAi;

    // Kernel исполнения навыка игрока (фасады `CGame`).
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

    fn begin_player_skill_execution(
        &mut self,
        player_id: i32,
        kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool;

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32;

    fn finish_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool;

    // Игрок, определения уровня и разрешение сторон.
    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut Self::Player>;

    /// `learned_skill_level` с фабрикой навыков владельца.
    fn thunder_blow_skill_level(&self, player_id: i32) -> Option<i32>;

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView>;

    /// HP монстра региона (`find_monster_by_id` → `hit_points`).
    fn thunder_blow_monster_hit_points(&self, region_id: i32, monster_id: i32) -> Option<u32>;

    /// Общий прямой путь локального каста (`base_magic_path`).
    fn base_magic_path(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
        forced_length: Option<u32>,
    ) -> Vec<(i32, i32, u8)>;

    /// GetBlock клетки области региона (vcall +0x44); отсутствующий регион
    /// отдаёт `None`, что проваливается в молчаливый отказ Add ниже.
    fn thunder_blow_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8>;

    // Пакеты и частичные эффекты (фасады `CGame`).
    fn send_self_state_skill_failure(&self, message_type: i32, player_id: i32, action: u8);

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    /// Общий фасад state-update (+0x164) семьи summoncreatureskill.
    fn update_player_fight_state_move_shape(&mut self, player_id: i32) -> bool;

    fn send_player_shape_around(
        &mut self,
        player_id: i32,
        excluded_player_id: Option<i32>,
        message: &CMessage,
    );

    fn allocate_summon_shape_id(&mut self) -> i32;

    // Мир формулы области.
    fn weapon_damage_factors(&self) -> (f32, f32);

    /// `weapon_modifier` уже найденного игрока против уровня цели; чтение
    /// фабрики предметов остаётся у владельца.
    fn thunder_blow_weapon_modifier(
        &self,
        player: &Self::Player,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> f32;

    fn critical_rate(&self) -> f32;

    fn base_combat_scales(&self) -> [f32; 5];

    fn skill_random_below(&mut self, maximum: i32) -> i32;
}

/// Стадии, зависящие от runtime игрового хода: общий хвост AfterUse,
/// регистрация области и её входной снимок. Отделены, потому что тип хода
/// принадлежит старому main loop, а не самому удару.
pub trait ThunderBlowContact<Runtime>: ThunderBlowGame {
    /// Износ оружия и reuse без восстановления движения (прежний
    /// `finish_immediate_base_attack`).
    fn finish_immediate_base_attack(&mut self, player_id: i32, skill_id: u32, runtime: &mut Runtime);

    /// Регистрация области у владельца региона (прежний фасад
    /// `add_thunder_blow_phalanx`: SetCenter, same-cell свёртка, Add).
    /// `None` — owner региона не извлечён; `Some(false)` — отказ membership.
    fn add_thunder_blow_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CThunderBlowPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<bool>;

    /// Входной снимок области `0xBF502` (прежний фасад владельца региона).
    fn send_thunder_blow_phalanx_entry(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()>;
}

/// Стадии результата одного тика исполнения; обёртка очереди с полем
/// `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderBlowOutcome {
    Begun,
    Pending,
    Completed,
    Rejected,
}

/// Общий адаптер прежнего `baseattack`: перенесён буквально вместе с
/// callers этого навыка.
const fn time_reached(now_ms: u32, started_at_ms: u32, delay_ms: u32) -> bool {
    now_ms.wrapping_sub(started_at_ms) >= delay_ms
}

fn finish_player_thunder_blow<Game, Runtime>(game: &mut Game, player_id: i32, runtime: &mut Runtime)
where
    Game: ThunderBlowContact<Runtime>,
{
    game.finish_immediate_base_attack(player_id, THUNDER_BLOW_SKILL_ID, runtime);
}

pub fn cancel_player_thunder_blow<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
) -> bool
where
    Game: ThunderBlowContact<Runtime>,
{
    let Some(dispatch) = game
        .player_skill_execution(player_id, THUNDER_BLOW_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else {
        return false;
    };
    finish_player_thunder_blow(game, player_id, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn master_info(player: &impl ThunderBlowPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(),
        master_guild_id: player.faction_id(), master_team_id: player.team_id(),
        master_union_id: player.union_id(), master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn destination<Game: ThunderBlowGame>(game: &Game, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
            let view = game.base_magic_target_view(region_id, target)?;
            Some((view.tile_x, view.tile_y, Some(target)))
        }
        _ => None,
    }
}

fn target_dead<Game: ThunderBlowGame>(game: &Game, region_id: i32, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_none_or(ThunderBlowPlayer::is_dead),
        MONSTER_TYPE => game.thunder_blow_monster_hit_points(region_id, target.id)
            .is_none_or(|hit_points| hit_points == 0),
        _ => true,
    }
}

fn send_failure<Game: ThunderBlowGame>(game: &Game, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_visual<Game: ThunderBlowGame>(game: &mut Game, player_id: i32, level: i32, destination: Option<(ShapeIdentity, i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if destination.is_some() { 2 } else { 1 });
    message.add_long(THUNDER_BLOW_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if let Some((target, x, y)) = destination {
        message.add_long(target.object_type); message.add_long(target.id);
        message.add_long(x); message.add_long(y);
    } else { message.add_long(player.shape().get_direction()); }
    game.send_player_shape_around(player_id, None, &message);
}

pub const fn is_thunder_blow_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: THUNDER_BLOW_SKILL_ID, .. }
        | PlayerSkillDispatch::Object {
            skill_id: THUNDER_BLOW_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
        }
    )
}

pub fn execute_player_thunder_blow<Game, Runtime>(
    game: &mut Game, player_id: i32, dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime, now_milliseconds: fn() -> u32,
) -> ThunderBlowOutcome
where
    Game: ThunderBlowContact<Runtime>,
{
    if !is_thunder_blow_dispatch(dispatch) { return ThunderBlowOutcome::Rejected; }
    let Some((region_id, level, initial_mana, source_x, source_y)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, game.thunder_blow_skill_level(player_id)?, player.mana(),
        player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
    ))) else { return ThunderBlowOutcome::Rejected };
    let Some(properties) = game.skill_base_properties(THUNDER_BLOW_SKILL_ID, level) else {
        return ThunderBlowOutcome::Rejected;
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_execution(player_id, THUNDER_BLOW_SKILL_ID).is_none() {
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, THUNDER_BLOW_SKILL_ID),
            cooldown_ms,
            now_milliseconds(),
        ) { send_failure(&*game, player_id, 0x0d, mp_loss); return ThunderBlowOutcome::Rejected; }
        let Some((target_x, target_y, _)) = destination(game, region_id, dispatch) else {
            return ThunderBlowOutcome::Rejected;
        };
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(&*game, player_id, 0x0b, mp_loss);
            return ThunderBlowOutcome::Rejected;
        }
        if mp_loss == 0 || (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            if mp_loss != 0 { send_failure(game, player_id, 7, mp_loss); }
            return ThunderBlowOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_current_skill_id(Some(THUNDER_BLOW_SKILL_ID)); }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now_milliseconds()));
        return ThunderBlowOutcome::Begun;
    } else if game.player_skill_execution(player_id, THUNDER_BLOW_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) {
        return ThunderBlowOutcome::Rejected;
    }

    let Some((target_x, target_y, target)) = destination(game, region_id, dispatch) else {
        finish_player_thunder_blow(game, player_id, runtime);
        return ThunderBlowOutcome::Rejected;
    };
    if game.player_skill_execution(player_id, THUNDER_BLOW_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, ThunderBlowPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            finish_player_thunder_blow(game, player_id, runtime);
            return ThunderBlowOutcome::Rejected;
        }
        // AI Begin по машинному порядку (якорь 0x17A520): SetMP сразу, затем
        // state-update (+0x164) и поворот ДО повторной проверки дальности;
        // отказ проверки идёт уже после частичных эффектов — MP потерян,
        // поворот сохранён. Достижимо только при смене цели/пути между
        // начальным Check и этим AI.
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_fight_state_move_shape(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b, mp_loss);
            finish_player_thunder_blow(game, player_id, runtime);
            return ThunderBlowOutcome::Rejected;
        }
        send_visual(game, player_id, level, None);
        if let Some(state) = game.player_skill_execution_mut(player_id, THUNDER_BLOW_SKILL_ID) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = game.player_skill_execution(player_id, THUNDER_BLOW_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение громового удара создано");
    if !time_reached(now_milliseconds(), started, delay_ms) { return ThunderBlowOutcome::Pending; }
    if target.is_some_and(|identity| target_dead(game, region_id, identity)) {
        send_failure(game, player_id, 10, mp_loss);
        finish_player_thunder_blow(game, player_id, runtime);
        return ThunderBlowOutcome::Rejected;
    }
    send_visual(game, player_id, level, Some((target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: Default::default() }), target_x, target_y)));
    let summon_id = game.allocate_summon_shape_id();
    let started_at_ms = now_milliseconds();
    let master = game.find_player(player_id).map(master_info).unwrap_or_default();
    let mut phalanx = CThunderBlowPhalanx::new(
        summon_id, master, started_at_ms, lifetime_ms, level, minimum, maximum, element_modifier,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let result = if game.thunder_blow_cell_block(region_id, target_x, target_y) != Some(2) {
        game.add_thunder_blow_phalanx(region_id, phalanx, target_x, target_y, started_at_ms, runtime)
    } else { None };
    if result.as_ref().is_some_and(|result| *result) {
        let _ = game.send_thunder_blow_phalanx_entry(region_id, summon_id, runtime);
    }
    tracing::trace!(region_id, player_id, summon_id, ?result, "создана форма громового удара");
    if let Some(state) = game.player_skill_execution_mut(player_id, THUNDER_BLOW_SKILL_ID) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_thunder_blow(game, player_id, runtime);
    ThunderBlowOutcome::Completed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderBlowPhalanxTick {
    Scan { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CThunderBlowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
}


impl CThunderBlowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, skill_level, minimum_attack, maximum_attack, element_modifier }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.master }
    pub fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }

    pub fn tick(&mut self, now_ms: u32) -> ThunderBlowPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.finish();
            ThunderBlowPhalanxTick::Expired
        } else {
            ThunderBlowPhalanxTick::Scan { sampled_at_ms: now_ms }
        }
    }

    pub fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 } else {
            self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(THUNDER_BLOW_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            // Машинное тело 0x1F54D0 пишет master type/id ([esi+0x84]/[esi+0x88],
            // копия tagMasterInfo из базового ctor), не собственную идентичность
            // формы — как и все sibling-конверты этой ICF-группы (досверка хвоста).
            writer.write_i32(self.master.master_type);
            writer.write_i32(self.master.master_id);
            writer.write_u32(remained);
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub fn calculate_owned_thunder_blow_attack<Game: ThunderBlowGame>(
    game: &mut Game,
    phalanx: &CThunderBlowPhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master.master_id)?;
    let mut combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let (divisor, floor) = game.weapon_damage_factors();
    let damage_factor = game.thunder_blow_weapon_modifier(player, i32::from(target_level), divisor, floor);
    let width_delta = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let damage = phalanx.element_modifier
        .wrapping_mul(combat.element_modify).wrapping_div(100)
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(phalanx.minimum_attack).max(0);
    let mut attack = AttackInformation {
        skill_id: THUNDER_BLOW_SKILL_ID,
        skill_level: phalanx.skill_level as u8,
        attacker_type: phalanx.master.master_type,
        attacker_id: phalanx.master.master_id,
        attacker_team_id: phalanx.master.master_team_id,
        attacker_faction_id: phalanx.master.master_guild_id,
        attacker_union_id: phalanx.master.master_union_id,
        hit_modifier: 100,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.critical_rate();
        for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); }
    }
    let [blast_attack, blast_defense, element_blast_attack, element_blast_defense, full_miss] = game.base_combat_scales();
    if combat.blast_attack_scale() < 1.0 { combat.blast_attack_scale_bits = blast_attack.max(1.0).to_bits(); }
    if combat.blast_defense_scale() < 0.01 { combat.blast_defense_scale_bits = blast_defense.max(0.01).to_bits(); }
    if combat.element_blast_attack_scale() < 1.0 { combat.element_blast_attack_scale_bits = element_blast_attack.max(1.0).to_bits(); }
    if combat.element_blast_defense_scale() < 0.01 { combat.element_blast_defense_scale_bits = element_blast_defense.max(0.01).to_bits(); }
    if combat.full_miss_scale() < 0.01 { combat.full_miss_scale_bits = full_miss.max(0.01).to_bits(); }
    if combat.critical_rate() < 1.0 { combat.critical_rate_bits = game.critical_rate().max(1.0).to_bits(); }
    Some((attack, combat, occupation, attacker_level))
}
