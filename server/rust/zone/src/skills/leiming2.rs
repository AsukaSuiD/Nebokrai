//! CLeiming2 (0x21B, ID 32-бит) — однократный гром боевого духа: общие с
//! CThunder Check/AI (`skills/thunder.rs`) и собственный Summon.
//!
//! Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match), `appserver/skills/thunder2.cpp`. Адресная
//! конвенция факт-листа волны: истинный RVA (pub off + 0x1000; VA = RVA +
//! 0x400000). Прежний переходный владелец —
//! `src/gameserver/appserver/skills/thunder2.rs`; тело перенесено буквально
//! порцией T1 «thunder/leiming2/tianhuo — BF-облака призыва».
//!
//! Машинный факт (MATCH по снятой доказательной базе): Check и AI общие с
//! CThunder (`0x121330`/`0x121940`, см. шапку `skills/thunder.rs`); доставка
//! BF918 — `SendToAround(U-шейп, player)` в `CLeiming2::AI` `0x12080B`
//! (VA `0x52080B`, fix №2 порции T1). Собственный Summon сохраняет порядок
//! CCH → AddElementAtk → max → min → текущий level → lifetime: к вычисленному
//! стихийному коэффициенту прибавляется AddElementAtk, частота и число целей
//! не читаются. Clock конструктора предшествует ID; центр устанавливается до
//! допуска региона (initialize-прохода RNG у Leiming2 нет). Отказ регистрации
//! обрабатывает общий publisher области; AI в любом случае заканчивает попытку
//! внешним End(1), без повторного visual или Begin. RVA тела Summon факт-лист
//! волны не называет — новых утверждений нет, перенос прежнего тела буквально.
//!
//! Объявленные швы переноса (не расхождения): hub `thunder::SummonCloudGame`;
//! конструктор `CLeimingPhalanx2`, допуск региона и входное сообщение
//! `0xBF502` выполняются прежним владельцем через callback `complete_summon`
//! (`Leiming2Summon`), т.к. тип фаланги ещё у старого пакета. UNKNOWN
//! списком: второй аргумент `SendToAround`; RVA тела Summon CLeiming2.

use crate::combat::MasterInfo;
use crate::regions::ShapeIdentity;

use super::battlefairyskill::{BattleFairySkillOutcome, summon_user_add_element, summon_user_cch};
use super::dispatch::BattleFairySkillDispatch;
use super::thunder::{
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_SUMMONED_LIFETIME,
    SummonCloudGame, execute_thunder_family, thunder_summon_properties,
};

pub const LEIMING2_SKILL_ID: u32 = 0x21b;
pub const LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;

/// Параметры призыва для делегата: конструктор `CLeimingPhalanx2` ещё у
/// старого владельца, как и допуск региона с входным `0xBF502`.
pub struct Leiming2Summon {
    pub source: (i32, ShapeIdentity),
    pub id: i32,
    pub master: MasterInfo,
    pub started_at_ms: u32,
    pub lifetime_ms: u32,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
    pub cch: i32,
    pub center_x: i32,
    pub center_y: i32,
}

pub fn execute_battle_fairy_leiming2<Game: SummonCloudGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    complete_summon: impl FnMut(&mut Game, &mut Runtime, Leiming2Summon),
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != LEIMING2_SKILL_ID { return BattleFairySkillOutcome::Rejected; }
    execute_thunder_family(game, player_id, instance, dispatch, begin_target, runtime, now,
        |game, instance, source, position, runtime| summon_leiming2(game, instance, source, position, runtime, now),
        complete_summon)
}

fn summon_leiming2<Game: SummonCloudGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
    position: (i32, i32), runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> Option<Leiming2Summon> {
    let (master, properties, element) = thunder_summon_properties(game, instance, source)?;
    let cch = summon_user_cch(game, source.1);
    let element = summon_user_add_element(game, source.1).wrapping_add(element);
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let skill_level = game.registered_skill(instance).map(|skill| skill.level())?;
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started_at_ms = now(runtime);
    let id = game.allocate_summon_shape_id();
    Some(Leiming2Summon {
        source, id, master, started_at_ms, lifetime_ms, skill_level,
        minimum_attack: minimum, maximum_attack: maximum,
        element_modifier: element, cch,
        center_x: position.0, center_y: position.1,
    })
}
