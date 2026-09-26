//! Направленный громовой удар `CThunderBlow2` (`0x14D`) и его визуальный
//! ресурс (слиты: исходный `thunderblow2.cpp` — единственный владелец обеих
//! частей).
//!
//! Машинные quirks: при выпуске identity-цель превращается в точку, но
//! отбрасывание и удар используют S, захваченную в начале AI; отсутствие
//! региона источника отменяет только отбрасывание; Pillar-гейт — цель не
//! отбрасывается при живом состоянии `0x74` (`PILLAR_SKILL_ID`); не-
//! используемое исходное поле missile всегда ноль и не материализуется.
//! Visual `0xBFE01` modes 0/1/3 с заново разрешённой S у mode 1 — после
//! отбрасывания получатель эффекта может отличаться от S текущего AI.
//!
//! Швы: hub-трейты `ThunderBlow2Game`/`ThunderBlow2Player`/
//! `ThunderBlow2MoveShape`/`ThunderBlow2Contact` (делегат
//! `appserver/skills/thunderblow2.rs` старого пакета); формула и raw-контакт
//! без RP — владелец `appserver/skills/impactattack.rs`; запись исполнения —
//! `skills/execution/payload.rs` `ThunderBlow2Execution`.
//!
//! Исходный владелец PDB: `appserver/skills/thunderblow2.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#thunderblow2--cthunderblow2-0x14d

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;

use super::baseattackruntime::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord, ThunderBlow2Execution};
use super::lifecycle::{SkillLifecycle, SkillStage, skill_is_restored};
use super::pillar::PILLAR_SKILL_ID;
use super::skillfactory::SkillOwner;
use super::visualeffect::SkillVisualEffectKind;

pub const THUNDER_BLOW_2_SKILL_ID: u32 = 0x14d;
const THUNDER_BLOW_2_VISUAL_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

/// Игрок-источник удара: переходный фасад старого `CPlayer`.
pub trait ThunderBlow2Player {
    /// Форма игрока (identity для сравнения с S при Check).
    fn shape(&self) -> &CShape;

    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);
}

/// Живая фигура стороны удара: переходный фасад старого `CMoveShape`.
pub trait ThunderBlow2MoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    /// Живой `HasStateBySkillId` фигуры (Pillar-гейт отбрасывания, `0x74`).
    fn has_state_by_skill_id(&self, skill_id: u32) -> bool;
}

/// Стадии результата одного тика AI; обёртка очереди с полем
/// `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderBlow2Outcome {
    Pending,
    Rejected,
    Completed,
}

/// Переходные фасады прежнего владельца `CGame`, открывающие исполнению
/// `CThunderBlow2` только прежние обращения; имена сохраняют исходную операцию.
pub trait ThunderBlow2Game {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для исполнения.
    type SkillAddress: Copy;

    type Player: ThunderBlow2Player;

    type MoveShape: ThunderBlow2MoveShape;

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

    fn move_shape_health(&self, region_id: i32, holder: ShapeIdentity) -> Option<u32>;

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8>;

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        user: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    /// Гейт существования региона перед отбрасыванием (прежний find_region).
    fn thunder_blow_2_region_present(&self, region_id: i32) -> bool;

    // Частичные эффекты и текстовые хвосты (фасады `CGame`).
    fn publish_player_states(&self, player_id: i32);

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    // Доставка visual `0x000BFE01`: кадр строится здесь, маршруты — у владельца.
    /// Personal-ветвь по numeric identity игрока.
    fn send_thunder_blow_2_visual_to_player(&self, player_id: i32, message: &CMessage);

    /// Around-ветвь региона формы; отсутствующий регион пропускает отправку.
    fn send_thunder_blow_2_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);

    /// Общий шаг отбрасывания impactattack (владелец старого пакета).
    fn knock_back_impact_target(
        &mut self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        region_id: i32,
        properties: &CSkillBaseProperties,
    );
}

/// Контактная стадия с runtime игрового хода: raw-удар impactattack без RP.
/// Отделена, потому что тип хода принадлежит старому main loop.
pub trait ThunderBlow2Contact<Runtime>: ThunderBlow2Game {
    fn apply_thunder_blow_2_attack(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    );
}

fn failure<Game: ThunderBlow2Game>(game: &mut Game, instance: Game::SkillAddress, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code {
        10 => b"GS0286",
        11 => b"GS0290",
        13 => b"GS0278",
        _ => return,
    };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure<Game: ThunderBlow2Game>(game: &mut Game, instance: Game::SkillAddress, player_id: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

pub fn check_cast<Game: ThunderBlow2Game>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    target: Option<(i32, ShapeIdentity)>, now_milliseconds: fn() -> u32,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    if target.is_none_or(|(_, target)| target == player.shape().identity()) {
        failure(game, instance, player_id, 10);
        return false;
    }
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        if maximum < path.len() as u32 {
            failure(game, instance, player_id, 11);
            return false;
        }
    }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let mana = player.mana();
    let loss = properties.query_property(USER_MP_LOSE);
    if (mana.wrapping_sub(loss) as i32) < 0 {
        mana_failure(game, instance, player_id, &properties);
        return false;
    }
    true
}

pub fn run_ai<Game, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, runtime: &mut Runtime, now_milliseconds: fn() -> u32,
) -> ThunderBlow2Outcome
where
    Game: ThunderBlow2Game + ThunderBlow2Contact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return ThunderBlow2Outcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return ThunderBlow2Outcome::Pending;
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return ThunderBlow2Outcome::Rejected; };
    let (region, identity) = skill.lifecycle().user();
    let source = game.resolve_state_move_shape(region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let target = game.resolve_skill_sufferer(skill.lifecycle());
    let (Some(source), Some(target)) = (source, target) else { return ThunderBlow2Outcome::Rejected; };
    if stage == SkillStage::Begin {
        if source.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(source.1.id) else { return ThunderBlow2Outcome::Rejected; };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                mana_failure(game, instance, source.1.id, &properties);
                return ThunderBlow2Outcome::Rejected;
            }
            if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
            game.publish_player_states(source.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return ThunderBlow2Outcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return ThunderBlow2Outcome::Rejected; };
        let destination = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
            .map_or_else(|| skill.lifecycle().destination(), |target| {
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            });
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return ThunderBlow2Outcome::Rejected; };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) { user.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<ThunderBlow2Execution>()).map(|state| state.attacking_started)
    else { return ThunderBlow2Outcome::Rejected; };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(skill) = game.registered_skill(instance) else { return ThunderBlow2Outcome::Rejected; };
        let started = skill.lifecycle().started_at_ms();
        if now_milliseconds() < started.wrapping_add(delay) { return ThunderBlow2Outcome::Pending; }
        let saved_target = skill.lifecycle().sufferer().1;
        if saved_target.object_type != 0 && saved_target.id != 0 {
            let fresh_target = game.resolve_skill_sufferer(skill.lifecycle());
            let Some((region, identity)) = fresh_target
                .filter(|(region, identity)| game.move_shape_health(*region, *identity).is_some_and(|hp| hp != 0))
            else {
                game.update_registered_skill_visual(instance, 10);
                return ThunderBlow2Outcome::Rejected;
            };
            let Some(target_shape) = game.resolve_state_move_shape(region, identity) else { return ThunderBlow2Outcome::Rejected; };
            let x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
            let y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
            let Some(skill) = game.registered_skill_mut(instance) else { return ThunderBlow2Outcome::Rejected; };
            skill.lifecycle_mut().set_point_target((x, y));
        }
        let Some(skill) = game.registered_skill(instance) else { return ThunderBlow2Outcome::Rejected; };
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
            if maximum < path.len() as u32 {
                if source.1.object_type == PLAYER_TYPE { failure(game, instance, source.1.id, 11); }
                else { game.update_registered_skill_visual(instance, 11); }
                return ThunderBlow2Outcome::Rejected;
            }
        }
        if let Some(region_id) = game.resolve_state_move_shape(source.0, source.1)
            .filter(|source| source.shape().is_assigned_to_server_region())
            .map(|source| source.shape().get_region_id())
            .filter(|region| game.thunder_blow_2_region_present(*region))
        {
            let target_level = game.move_shape_level(target.0, target.1);
            let source_level = game.move_shape_level(source.0, source.1);
            if target_level.zip(source_level).is_some_and(|(target, source)| target <= source)
                && game.resolve_state_move_shape(target.0, target.1)
                    .is_some_and(|target| !target.has_state_by_skill_id(PILLAR_SKILL_ID))
            {
                game.knock_back_impact_target(source, target, region_id, &properties);
            }
        }
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<ThunderBlow2Execution>()) {
            state.attacking_started = true;
        }
        drop(path);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return ThunderBlow2Outcome::Rejected; };
    if now_milliseconds() < started.wrapping_add(delay) { return ThunderBlow2Outcome::Pending; }
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0)
        || game.resolve_state_move_shape(target.0, target.1)
            .map(|target| target.shape().get_region_id())
            .is_none_or(|region| !game.live_skill_target_attackable(region, source.1, target.1))
    {
        game.update_registered_skill_visual(instance, 3);
        return ThunderBlow2Outcome::Rejected;
    }
    game.apply_thunder_blow_2_attack(instance, source, target, runtime);
    ThunderBlow2Outcome::Completed
}

/// Visual-ресурс `0x000BFE01`: modes 0/3 передают направление живого U,
/// mode1 заново разрешает S либо использует точку базы.
pub fn publish_thunder_blow_2_visual<Game: ThunderBlow2Game>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if skill.owner() != SkillOwner::CThunderBlow2
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::ThunderBlow2 || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let shape = user.shape();
    let mut message = CMessage::new(THUNDER_BLOW_2_VISUAL_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if shape.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_thunder_blow_2_visual_to_player(shape.identity().id, &message);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    let target = if mode == 1 {
        game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
            .map(|target| target.shape())
    } else { None };
    let destination = target.map_or_else(|| skill.lifecycle().destination(), |target| {
        (target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN))
    });
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(shape.identity().object_type);
    message.add_long(shape.identity().id);
    if mode == 1 {
        message.add_long(target.map_or(0, |target| target.identity().object_type));
        message.add_long(target.map_or(0, |target| target.identity().id));
        message.add_long(destination.0);
        message.add_long(destination.1);
    } else {
        message.add_long(shape.get_direction());
    }
    if shape.is_assigned_to_server_region() {
        game.send_thunder_blow_2_visual_around(shape.get_region_id(), shape, &message);
    }
}
