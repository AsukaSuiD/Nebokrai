//! Печать `CSeal` (`0x138`) для Player и Monster. Источник: `gameserver.exe`
//! (SHA-256 `4F5C98E0…`) + `GameServer.pdb` (RSDS match), исходный владелец
//! `appserver/skills/seal.cpp`. Полёт прицельного снаряда — общий
//! TargetedProjectile (`targetedprojectile.cpp`, payload
//! `TargetedProjectileProgress` в `skills/execution`), он остаётся
//! hub-владением и сюда не переносится.
//!
//! Общий TargetedProjectile хранит зарегистрированные U/S, первую проверку,
//! unsigned delay и flight, prepared-background и хвост Attack End. Этот
//! владелец оставляет только подтверждённые различия Seal: Check принимает
//! лишь S типа CMonster (визуальный отказ 10 с GS0317), затем общие
//! reuse/путь (именованная преграда GS0295) и только для Player MP/Move0 —
//! через швы общего `rangedweaponcast`. Общий путь берёт точку фигуры S по
//! сохранённому user_region Begin.
//!
//! Impact пропускается при Cure. Иначе общий DirectElement-профиль сохраняет
//! generic MasterInfo, Player-only EM, единственный RNG и расширенную
//! EM/FISTP-прибавку с f32-константой (hub-шов `directelementattack`). После
//! Attack уровни читаются S→U даже после фатального попадания; новый SealState
//! создаётся до полного End/destructor прежнего ID и занимает его прежнее
//! место. State Begin устанавливает timestamp и visual, поэтому создание
//! получает только keep-time.
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — переходные
//! фасады прежнего владельца `CGame`/`CMoveShape`, реализация остаётся у
//! делегата старого пакета (`appserver/skills/seal.rs`) и общих helper-ов
//! `rangedweaponcast`/`directelementattack`/`blindstate`; имена членов
//! сохраняют исходную операцию, швы потребляются статически (generic),
//! dyn-совместимость и `Send`-контракт не вводятся (ADR-0013). Часы приходят
//! от делегата (fn-параметр), как в `skills/flash.rs`.

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::CShape;

use super::baseattackruntime::SKILL_USAGE_REUSE_DELAY_TIME;
use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::lifecycle::SkillLifecycle;
use super::skill_is_restored;

pub const SEAL_SKILL_ID: u32 = 0x138;
pub const CURE_SKILL_ID: u32 = 0x131;
pub const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub const SKILL_USAGE_CONST: u32 = 20_010;

/// Живая фигура стороны печати: переходный фасад старого `CMoveShape`.
pub trait SealMoveShape {
    fn shape(&self) -> &CShape;

    /// Живой `HasStateBySkillId` цели (гейт Cure).
    fn has_state_by_skill_id(&self, skill_id: u32) -> bool;
}

/// Переходные фасады прежнего владельца `CGame`, открывающие печати только
/// прежние обращения; имена сохраняют исходную операцию.
pub trait SealGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для исполнения.
    type SkillAddress: Copy;

    type MoveShape: SealMoveShape;

    fn registered_skill(
        &self,
        address: Self::SkillAddress,
    ) -> Option<&RegisteredSkillRecord<Self::MonsterExecution>>;

    fn update_registered_skill_visual(&mut self, address: Self::SkillAddress, mode: u32);

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    /// Живое разрешение сохранённых region/type/id.
    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&Self::MoveShape>;

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    /// Живой уровень фигуры (S, затем U — в порядке чтений native).
    fn move_shape_level(&self, region_id: i32, identity: ShapeIdentity) -> Option<u8>;

    /// Общая проверка пути с именованной преградой (`check_skill_path`
    /// `rangedweaponcast`, `CastPathBlock::Named` + GS0295); visual15 и имя
    /// захваченной цели остаются внутри шва.
    fn check_seal_path(
        &mut self,
        address: Self::SkillAddress,
        properties: &CSkillBaseProperties,
        path: &[(i32, i32, u8)],
        player: Option<i32>,
        target: (i32, ShapeIdentity),
    ) -> bool;

    /// Общий MP-контракт (`check_cast_mana` `rangedweaponcast`): только Player
    /// требует ненулевую цену, достаток и Move0.
    fn check_seal_cast_mana(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool;
}

/// Контактные операции с runtime игрового хода: стихийный удар и замена
/// первичного состояния. Отделены, потому что тип хода принадлежит старому
/// main loop.
pub trait SealContact<Runtime>: SealGame {
    /// Прямой стихийный удар (`apply_direct_element_attack` владельца):
    /// generic MasterInfo, единственный RNG и EM-прибавка — hub-шов.
    fn apply_seal_direct_attack(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    );

    /// Новый SealState до полного End/destructor прежнего ID
    /// (`replace_primary_blind_state` `blindstate`); часы назначения читаются
    /// швом от runtime в момент замены.
    fn replace_seal_state(
        &mut self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        keep_time_ms: u32,
        runtime: &mut Runtime,
    );
}

fn resolved_object<Game: SealGame>(
    game: &Game,
    object: (i32, ShapeIdentity),
) -> Option<(i32, ShapeIdentity)> {
    let shape = game.resolve_state_move_shape(object.0, object.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

/// Формула срока печати: `(U.level − S.level + 20010).max(1) * 10002`.
/// Множитель не опускается ниже единицы; произведение переполняется беззнаково.
pub fn seal_keep_time(
    source_level: u8,
    target_level: u8,
    state_persist_time: u32,
    usage_const: u32,
) -> u32 {
    let multiplier = i32::from(source_level)
        .wrapping_sub(i32::from(target_level))
        .wrapping_add(usage_const as i32)
        .max(1);
    state_persist_time.wrapping_mul(multiplier as u32)
}

/// Конкретный Check печати. Аргумент Begin живёт только на стеке вызова:
/// U/S резолвит обвязка старого пакета (`StateSkillBeginTarget`), сюда
/// приходят живые снимки.
pub fn check_cast<Game: SealGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>,
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(source) = original_user.and_then(|source| resolved_object(game, source)) else {
        return false;
    };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some((skill_id, skill_level, last_used)) = game.registered_skill(instance).map(|skill| {
        (skill.id(), skill.level(), skill.last_used_ms())
    }) else {
        return false;
    };
    let Some(properties) = game.skill_base_properties(skill_id, skill_level).cloned() else {
        return false;
    };
    let target = original_target.and_then(|target| resolved_object(game, target));
    let Some(target) = target.filter(|target| target.1.object_type == MONSTER_TYPE) else {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player {
            game.send_skill_system_info(player, b"GS0317");
        }
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(last_used, reuse, now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player {
            game.send_skill_system_info(player, b"GS0278");
        }
        return false;
    }
    let Some(lifecycle) = game.registered_skill(instance).map(|skill| *skill.lifecycle()) else {
        return false;
    };
    let path = game.skill_target_path(&lifecycle);
    if !game.check_seal_path(instance, &properties, &path, player, target) {
        return false;
    }
    game.check_seal_cast_mana(instance, source, &properties)
}

/// Конкретный Attack печати: Cure-гейт, прямой стихийный контакт и замена
/// первичного состояния с новым keep-time.
pub fn apply_attack<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) where
    Game: SealGame + SealContact<Runtime>,
{
    let target_has_cure = game.resolve_state_move_shape(target.0, target.1)
        .is_some_and(|target| target.has_state_by_skill_id(CURE_SKILL_ID));
    if target_has_cure {
        return;
    }
    // Native Attack гасит только raw contact при U == S; state-tail остаётся.
    game.apply_seal_direct_attack(instance, source, target, runtime);

    // Native читает S, затем U после OnBeenAttacked; смерть S не отменяет SealState.
    let Some(target_level) = game.move_shape_level(target.0, target.1) else {
        return;
    };
    let Some(source_level) = game.move_shape_level(source.0, source.1) else {
        return;
    };
    let usage_const = properties.query_property(SKILL_USAGE_CONST);
    let keep = seal_keep_time(
        source_level,
        target_level,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        usage_const,
    );
    game.replace_seal_state(source, target, keep, runtime);
}
