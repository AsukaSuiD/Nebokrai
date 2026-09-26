//! Владелец трупного яда `CCorpsePtomaine` (`0x19F`): player и monster/pet
//! ветви, после задержки полный квадрат 3×3 вокруг caster-а, живым целям без
//! `CureState` централизованно заменяется канонический `CSpiderPoisonState`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/corpseptomaine.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/corpseptomaine.rs`; тела execute_owned,
//! player-путь и `add_corpse_poison_state` перенесены буквально (кластер D
//! полосы Monster 0x19x, карта — запись аудита «Zone skills: карта полосы
//! Monster 0x19x — 5 кластеров волн», 26 сентября 2026).
//!
//! Машинная разведка порции по этой паре (запись `.local/recon-de/notes/
//! D3-corpseptomaine.md`, тела `.local/recon-de/disasm/CCorpsePtomaine.txt`)
//! — MATCH по всем пунктам, кроме FIX F2 ниже:
//!
//! - vtable `0x257F5C`: End `0x546090` — ICF-фолд общего End
//!   (`CAgility::End` — нули `[+0x50]/[+0x4C]`, GetUser → SetMoveable(1) →
//!   `CAttackSkill::End`); Check `0x539A00`: null S/props → 0; reuse 10005 →
//!   visual(13); только U.type==400: RTTI player, `Query(2) == 0` → **тихий
//!   ret 0** (jbe), `MP − loss < 0` → visual(7); non-400 → **ret 1 без
//!   SetMoveable(0)** (0x539A71→0x539AB9). Begin-скелет
//!   `0x539840`/`0x539910`/`0x539AE0` MATCH.
//! - AI `0x53A230`: `[+0x4C]==0` → out; props null → End(0); U null →
//!   End(0); Begin-фаза только player: `MP([U+0x284]) − Query(2)` со
//!   **знаковым js** → visual(7) → End(0), иначе `SetMP(MP − loss)` и
//!   `vcall CPlayer+0x164` (имя цели — hub-шов); `Query(10006)` → `[+0x3C]`;
//!   visual(0); delay `Query(10001)+[+0x2C]` → out; visual(1); регион —
//!   RTTI `[U+0x40]` → null → **End(1)** (0x53A4E8), нормальное завершение
//!   также End(1).
//! - Обход: `g_bScope` — девять «1» (полный 3×3, центр включён), length =
//!   height = 3, X-внешний/Y-внутренний, `x + 3·y`, центр
//!   `GetTileX/Y(U) − 1`; клетка → RTTI CShape→CMoveShape → `IsDied` →
//!   `IsAttackAble(U)` `vcall+0x134` → `DoesStateExist(0x131)` → AddState;
//!   god-фильтра нет.
//! - AddState `0x539FF0`: MasterInfo local — type/id U; player: +0xB20 →
//!   mi+0xC, +0xB28 → mi+0x8, +0xB78 → mi+0x10, байты +0x278..+0x27B →
//!   mi+0x18..+0x1B, **country остаётся 0**; ctor `CSpiderPoisonState`
//!   (`0x5E90C0`) с порядком Query **const(20010) → freq(6001) → keep
//!   (10002) до замены**; скан первого `0x191` в `S+0x11C` → End
//!   `vcall+0x1C` → deleting-dtor `vcall+0x10(1)` → slot = 0 →
//!   `Begin(U, S)` `vcall+0x08` → тот же слот, иначе append; провал Begin —
//!   deleting-dtor нового. Player и monster ветви используют абсолютный
//!   срок `CSkill::IsRestored`, задержку — отдельной elapsed-проверкой,
//!   MP-списание — wrapping-sub фазы Begin.
//!
//! **FIX F2 (предикат, player-путь; основание — тело AI `0x53A230`):**
//! прежняя реконструкция ограничивала цели allowlist типов
//! `{400, 500, 600, 1100, 1200}`; нативный scan принимает любой живой
//! RTTI-CMoveShape по виртуальному `IsDied`/`IsAttackAble` (+Cure-гейт) без
//! allowlist. Ограничение снято. Узкая достижимость после правки: домен
//! скана задают живые разрешители owner-а — `base_magic_target_dead` и
//! региональный `IsAttackAble` знают ровно 400/600 (и 1100/1200 через
//! build-гейт); тип 500 (CNpc) машинно мёртв по `CMoveShape::IsDied`
//! (нулевой combat HP), поэтому его удалённое членство в allowlist было
//! недостижимым и там, и там; тип 1000 призванных фаланг вне домена
//! `resolve_state_move_shape` (не CMoveShape по модели арены). Для
//! гипотетических MoveShape-типов вне пятёрки нативный scan дал бы решение
//! виртуальным вратам, а разрешители Rust отвечают «мёртв/неатакуем» —
//! зафиксированы как неснимаемый остаток модели арены, машинная форма
//! scan-ветки player-пути сохранена. Monster-путь фильтра не имел и не
//! имеет: кандидаты читаются общим hub-хелпером `monsterattack` кластера A2.
//!
//! Объявленные швы переноса (не расхождения): hub-трейты `monsterattack`
//! (кластер A2; `published region` публикует настоящий derived region без
//! копии base/состояния, публикация настоящего `CPlayerAI` — тот же контракт
//! коллебека удара), арена состояний — общий hub `spiderpoison`
//! (`SpiderPoisonStateArena`: Cure-факт и замена первого 0x191 с порядком
//! прежнего `states/state.rs`), региональная форма `IsAttackAble` и
//! `finish_summon_skill` прежнего hub `states/summonskill` — фасады
//! `CorpsePtomaineGame`/`CorpsePtomaineContact`. Потребление статическое
//! (generic), dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::baseattackruntime::{
    BaseAttackContact, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::cure::CURE_SKILL_ID;
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::monsterattack::{
    MonsterCombatContact, MonsterCombatGame, MonsterCombatPlayer, monster_attack_cell_candidates,
    resolve_owned_monster_attack_target,
};
use super::spiderpoison::SpiderPoisonStateArena;
use super::spidermist::{SKILL_USAGE_STATE_PERSIST_TIME, SKILL_USAGE_TARGET_AFFECT_FREQUENCY};
use super::statefactory::SpiderPoisonState;
use crate::ai::monsterai::schedule_attack_interval;

pub const CORPSE_PTOMAINE_SKILL_ID: u32 = 0x19f;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_CONST: u32 = 20_010;
const CORPSE_PTOMAINE_VISUAL_MESSAGE: i32 = 0x000b_fe01;

/// Стадии результата одного тика player-cast трупного яда; обёртка очереди
/// с полем `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpsePtomaineOutcome {
    Begun,
    Pending,
    Completed,
    Rejected,
}

/// Общая диспетчерская форма player-cast трупного яда: любая из трёх форм
/// с этим ID.
pub const fn is_player_corpse_ptomaine_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: CORPSE_PTOMAINE_SKILL_ID, .. }
            | PlayerSkillDispatch::Point { skill_id: CORPSE_PTOMAINE_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: CORPSE_PTOMAINE_SKILL_ID, .. }
    )
}

/// Региональная форма живого `IsAttackAble` источника против цели
/// (`live_skill_target_attackable` прежнего owner-а: find региона + допуск).
pub trait CorpsePtomaineGame: MonsterCombatGame {
    fn ptomaine_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;
}

/// Контакт с runtime игрового хода: общий `finish_summon_skill` игрока
/// (износ и reuse-штамп успешного исхода) у прежнего hub
/// `states/summonskill`.
pub trait CorpsePtomaineContact<Runtime>: CorpsePtomaineGame + MonsterCombatContact<Runtime> {
    fn finish_corpse_ptomaine_summon_skill(
        &mut self,
        player_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    );
}

/// Кадр начала `0x000BFE01` (updateVE mode 0): action 1, навык, уровень,
/// сторона источника и его direction.
pub fn corpse_ptomaine_start_message(
    skill_level: i32,
    actor_type: i32,
    actor_id: i32,
    direction: i32,
) -> CMessage {
    let mut message = CMessage::new(CORPSE_PTOMAINE_VISUAL_MESSAGE);
    message.add_byte(1);
    message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    message.add_long(direction);
    message
}

/// Кадр исполнения `0x000BFE01` (updateVE mode 1): action 2, два нулевых
/// long и клетка caster-а.
pub fn corpse_ptomaine_fire_message(
    skill_level: i32,
    actor_type: i32,
    actor_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(CORPSE_PTOMAINE_VISUAL_MESSAGE);
    message.add_byte(2);
    message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(actor_type);
    message.add_long(actor_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

/// AddState `0x539FF0` одной цели: MasterInfo живого источника (country = 0),
/// ctor channel-записи до замены и семейная замена первого `0x191` либо
/// append через общую арену ядовой линии.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет живые стороны и такт наслоения")]
fn add_corpse_poison_state<Game: CorpsePtomaineGame + SpiderPoisonStateArena>(
    game: &mut Game,
    region_id: i32,
    source: ShapeIdentity,
    target: ShapeIdentity,
    properties: &CSkillBaseProperties,
    now: &mut dyn FnMut() -> u32,
) {
    let Some(source_region) = game.state_move_shape_region_id(region_id, source) else { return };
    let Some(target_region) = game.state_move_shape_region_id(region_id, target) else { return };
    let mut master = if source.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.id) else { return };
        player.master_info()
    } else {
        MasterInfo { master_type: source.object_type, master_id: source.id, ..MasterInfo::default() }
    };
    master.master_country_id = 0;
    // Конструктор предшествует замене; порядок запросов — const → freq → keep.
    let state = SpiderPoisonState::new(
        master,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
        properties.query_property(SKILL_USAGE_CONST),
    );
    let _ = game.replace_or_begin_spider_poison_state(
        region_id,
        target,
        Some((source_region, source)),
        Some((target_region, target)),
        state,
        now,
    );
}

/// Объектный путь монстра/питомца `0x19F`: подход к дистанции, attack-speed
/// расписание, reuse, Begin каста без запрета движения (non-400 форма Check),
/// после задержки — полный квадрат 3×3 с живыми допуски и заменой яда.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub fn execute_owned_corpse_ptomaine<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: CorpsePtomaineContact<Runtime> + BaseAttackContact<Runtime> + SpiderPoisonStateArena,
{
    let Some(region_owner) = owner.as_mut() else { return false; };
    let region_id = Game::owner_region_id(region_owner);
    let Some(facts) = game.monster_combat_facts(
        Game::owner_base_mut(region_owner),
        monster_id,
        CORPSE_PTOMAINE_SKILL_ID,
    ) else {
        return false;
    };
    let (source, cast) = (facts.source, facts.cast);

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity)
        else {
            game.monster_clear_ai_target(Game::owner_base_mut(region_owner), monster_id);
            return true;
        };
        if !game.monster_combat_approach_attack_range(
            Game::owner_base_mut(region_owner),
            monster_id,
            target.view,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) =
            schedule_attack_interval(facts.ai_kind, facts.attack_interval_ms)
            && !game.monster_begin_attack_attempt(
                Game::owner_base_mut(region_owner),
                monster_id,
                now_ms,
                attack_interval_ms,
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
        let region = Game::owner_base_mut(region_owner);
        let target_object = game.resolve_owned_skill_begin_object(&*region, target_identity);
        // Non-400 форма Check возвращает 1 без SetMoveable(0).
        game.monster_install_cast(
            region,
            monster_id,
            target_identity,
            CORPSE_PTOMAINE_SKILL_ID,
            skill_level,
            now_ms,
            target_object,
        );
        let message = corpse_ptomaine_start_message(
            i32::from(skill_level),
            MONSTER_TYPE,
            monster_id,
            source.get_direction(),
        );
        game.send_visual_around(&*region, &source, &message);
        return true;
    }

    let cast = cast.expect("выполнение трупного яда проверено выше");
    if cast.skill_id != CORPSE_PTOMAINE_SKILL_ID {
        return false;
    }
    if !skill_is_restored(
        cast.started_at_ms,
        properties.query_property(SKILL_USAGE_DELAY_TIME),
        now_ms,
    ) {
        return true;
    }
    let (Ok(center_x), Ok(center_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    {
        let message = corpse_ptomaine_fire_message(
            i32::from(skill_level),
            MONSTER_TYPE,
            monster_id,
            center_x,
            center_y,
        );
        game.send_visual_around(Game::owner_base(&*region_owner), &source, &message);
    }
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let Some(region_owner) = owner.as_mut() else { return true; };
            let candidates = monster_attack_cell_candidates(
                game,
                region_owner,
                monster_id,
                center_x.wrapping_add(offset_x),
                center_y.wrapping_add(offset_y),
            );
            for identity in candidates {
                let Some(region_owner) = owner.as_mut() else { return true; };
                let Some(target) = resolve_owned_monster_attack_target(game, region_owner, identity)
                else {
                    continue;
                };
                if target.dead
                    || !game.live_skill_target_attackable_in(region_owner, source.identity(), identity)
                {
                    continue;
                }
                // Callback публикует настоящий derived region, без копии base.
                let _ = game.with_published_region(owner, |game| {
                    if game.poison_shape_has_state(region_id, identity, CURE_SKILL_ID) != Some(false)
                    {
                        return;
                    }
                    add_corpse_poison_state(
                        game,
                        region_id,
                        source.identity(),
                        identity,
                        properties,
                        &mut || now_milliseconds(),
                    );
                });
            }
        }
    }
    let Some(region_owner) = owner.as_mut() else { return true; };
    let region = Game::owner_base_mut(region_owner);
    game.monster_advance_cast(
        region, monster_id, CORPSE_PTOMAINE_SKILL_ID, SkillStage::Check, SkillStage::Calculate,
    );
    game.monster_advance_cast(
        region, monster_id, CORPSE_PTOMAINE_SKILL_ID, SkillStage::Calculate, SkillStage::Attack,
    );
    game.monster_advance_cast(
        region, monster_id, CORPSE_PTOMAINE_SKILL_ID, SkillStage::Attack, SkillStage::Apply,
    );
    game.monster_finish_cast_clock(region, monster_id, CORPSE_PTOMAINE_SKILL_ID, runtime);
    true
}

fn restore_player<Game: CorpsePtomaineGame>(game: &mut Game, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

/// Общий terminal обеих завершений player-исполнения: движение назад, затем
/// `finish_summon_skill` прежнего hub с его reuse/износом.
fn finish_player_ptomaine<Game, Runtime>(game: &mut Game, player_id: i32, runtime: &mut Runtime)
where
    Game: CorpsePtomaineContact<Runtime>,
{
    restore_player(game, player_id);
    game.finish_corpse_ptomaine_summon_skill(player_id, CORPSE_PTOMAINE_SKILL_ID, runtime);
}

pub fn cancel_player_corpse_ptomaine<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
) -> bool
where
    Game: CorpsePtomaineContact<Runtime>,
{
    let Some(dispatch) = game
        .player_kernel(player_id, CORPSE_PTOMAINE_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else {
        return false;
    };
    finish_player_ptomaine(game, player_id, runtime);
    game.finish_monster_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

/// Player-вход `0x19F`: cooldown, тихий отказ нулевой MP-цены, повторная
/// проверка и списание MP в фазе Begin, задержка и fire на клетке caster-а;
/// успешный Begin возвращает Begun до первого AI, координатор ставит Attack
/// и продолжает AI в том же Run.
pub fn execute_player_corpse_ptomaine<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> CorpsePtomaineOutcome
where
    Game: CorpsePtomaineContact<Runtime> + BaseAttackContact<Runtime> + SpiderPoisonStateArena,
{
    if !is_player_corpse_ptomaine_dispatch(dispatch) {
        return CorpsePtomaineOutcome::Rejected;
    }
    let Some((region_id, center_x, center_y, level, mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                game.monster_combat_player_skill_level(player_id, CORPSE_PTOMAINE_SKILL_ID)?,
                player.mana(),
            ))
        })
    else {
        return CorpsePtomaineOutcome::Rejected;
    };
    let Some(properties) = game.skill_base_properties(CORPSE_PTOMAINE_SKILL_ID, level).cloned()
    else {
        if game.player_kernel(player_id, CORPSE_PTOMAINE_SKILL_ID).is_some() {
            restore_player(game, player_id);
        }
        return CorpsePtomaineOutcome::Rejected;
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now = now_milliseconds();
    if game.player_kernel(player_id, CORPSE_PTOMAINE_SKILL_ID).is_none() {
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, CORPSE_PTOMAINE_SKILL_ID), reuse, now) {
            game.send_cast_failure(player_id, 0x0d);
            return CorpsePtomaineOutcome::Rejected;
        }
        // Нулевая MP-цена — исходный тихий отказ player-cast (jbe → ret 0).
        if mp_loss == 0 {
            return CorpsePtomaineOutcome::Rejected;
        }
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_cast_failure(player_id, 7);
            return CorpsePtomaineOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(CORPSE_PTOMAINE_SKILL_ID));
        }
        game.begin_player_kernel(player_id, SkillExecutionKernel::begin(dispatch, now));
        return CorpsePtomaineOutcome::Begun;
    }
    if game
        .player_kernel(player_id, CORPSE_PTOMAINE_SKILL_ID)
        .is_none_or(|kernel| kernel.dispatch() != dispatch)
    {
        return CorpsePtomaineOutcome::Rejected;
    }
    if game
        .player_kernel(player_id, CORPSE_PTOMAINE_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let current = game.find_player(player_id).map_or(0, |player| player.mana());
        if (current.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_cast_failure(player_id, 7);
            restore_player(game, player_id);
            return CorpsePtomaineOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_fight_state_move_shape(player_id);
        if let Some(player) = game.find_player(player_id) {
            let message = corpse_ptomaine_start_message(
                level,
                PLAYER_TYPE,
                player_id,
                player.shape().get_direction(),
            );
            game.send_player_visual(player_id, &message);
        }
        if let Some(kernel) = game.player_kernel_mut(player_id, CORPSE_PTOMAINE_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = game
        .player_kernel(player_id, CORPSE_PTOMAINE_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .unwrap_or_default();
    if !skill_is_restored(started, delay, now) {
        return CorpsePtomaineOutcome::Pending;
    }
    let fire = corpse_ptomaine_fire_message(level, PLAYER_TYPE, player_id, center_x, center_y);
    game.send_player_visual(player_id, &fire);
    let source = ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID };
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            for view in game.monster_combat_cell_views(
                region_id,
                center_x.wrapping_add(offset_x),
                center_y.wrapping_add(offset_y),
            ) {
                let identity = view.identity;
                // FIX F2: allowlist типов снят — нативный scan принимает любой
                // живой CMoveShape; предикаты IsDied/IsAttackAble решают ниже.
                if game.base_magic_target_dead(region_id, identity)
                    || !game.ptomaine_target_attackable(region_id, source, identity)
                {
                    continue;
                }
                if game.poison_shape_has_state(region_id, identity, CURE_SKILL_ID) != Some(false) {
                    continue;
                }
                game.with_published_player_ai(player_id, player_ai, |game| {
                    add_corpse_poison_state(
                        game,
                        region_id,
                        source,
                        identity,
                        &properties,
                        &mut || now_milliseconds(),
                    );
                });
            }
        }
    }
    if let Some(kernel) = game.player_kernel_mut(player_id, CORPSE_PTOMAINE_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_ptomaine(game, player_id, runtime);
    CorpsePtomaineOutcome::Completed
}
