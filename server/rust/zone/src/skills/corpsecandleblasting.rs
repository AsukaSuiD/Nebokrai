//! Взрыв трупной свечи `CCorpseCandleBlasting` (`0x194`): монстр после
//! задержки взрывает сам себя, обходит восемь клеток маски 3×3 без центра,
//! бьёт каждую живую цель и уходит на удаление по кадру смерти.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `appserver/skills/corpsecandleblasting.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/corpsecandleblasting.rs`; тела
//! execute_owned, wire-кадры и формула перенесены буквально (кластер D полосы
//! Monster 0x19x, карта — запись аудита «Zone skills: карта полосы Monster
//! 0x19x — 5 кластеров волн», 26 сентября 2026).
//!
//! Машинная разведка порции по этой паре (запись `.local/recon-de/notes/
//! D2-corpsecandleblasting.md`, тела `.local/recon-de/disasm/
//! CCorpseCandleBlasting.txt`) — MATCH по всем пунктам, кроме FIX F1 ниже:
//!
//! - vtable `0x25AEF4`: Check `0x582600` — ICF-фолд со `CSporeBlasting`
//!   (null S → 0; reuse 10005 → visual(13); срок — SetMoveable(0), ret 1);
//!   Begin-скелет `0x582670`/`0x582740`/`0x582870`; End `0x582810` — ICF со
//!   SporeBlasting (`[+0x50]=0`,`[+0x4C]=0` → GetUser → SetMoveable(1) →
//!   `CAttackSkill::End`). Общая очистка `CMonster` вызывает его после
//!   сообщения смерти либо при отмене/Stiffen без взрыва; скрипт, урон и
//!   пометка удаления не являются побочными эффектами End.
//! - AI `0x5830E0`: `[+0x4C]==0` → out; props null → End(0); U null → End(0);
//!   первая фаза — updateVE(0); delay `Query(10001)+[+0x2C]` unsigned → out;
//!   updateVE(1); регион — RTTI `[U+0x40]` → null → End(0); центр
//!   `GetTileX/Y(U) − length/2 / height/2` (обе 3 → −1).
//! - Обход: внешний X (esi), внутренний Y (ebp), маска `g_bScope[x + 3·y]`
//!   — девять байт `[1,1,1,1,0,1,1,1,1];` GetShape клетки → RTTI
//!   CShape→CMoveShape → `IsAttackAble(U)` `vcall+0x134` → `Attack(U, S)`.
//! - Attack `0x582FC0`: отдельный отказ 600→600 внутри Attack (scan его не
//!   делает); info ctor-дефолт; U==400 → MasterInfo-поля игрока; Calculate →
//!   `vcall+0x15C` приёмника.
//! - После обхода: `[U+0x80]=1` (stage-for-delete); RTTI CMonster + GetSufferer
//!   RTTI→CPlayer: оба живы и `script_file[0] != 0x30` → `RunScript`
//!   (stRunScript с регионом/игроком/файлом, NULL point); затем кадр смерти
//!   `0xBF60B`: `long 0, long 0, long 600, long id, long 0, byte 2`;
//!   SendToAround(U, 0); End(1) `vcall+0x68`.
//! - Wire `0xBFE01` (updateVE `0x582930`, живы плечи 0/1): кадры старта и
//!   исполнения байт-в-байт совпадают со сборщиками ниже.
//!
//! **FIX F1 (формула, основание — тело Calculate `0x582EA0`):** прежняя
//! реконструкция читала MIN/MAX урона ключами `20_001/20_002` и hit-
//! модификатор ключом `3`; машина кверит hit-модификатор `push 0x4E21`
//! (`20_001` → `[info+0x18]`, 0x582EE8) и диапазон `push 0x4E29`/`push
//! 0x4E28` (`20_009` max, `20_008` min, 0x582F33/0x582F45/0x582F68). Ключи
//! исправлены на машинные; остальная формула — MATCH: `|max − min| + 1`
//! (cdq-abs), один RNG, `GetAddElementAtk` `vcall+0x118` монстра ≡ 0
//! (свёрнут), jns-clamp, kind 3 (Element), mp_damage = 0. Узкая
//! достижимость: значения строки навыка — вне машинной базы (строки БД —
//! проектный UNKNOWN плана D/E): отсутствующий ключ даёт
//! `query_property == 0`, машинная форма чтений не меняется.
//!
//! Объявленные швы переноса (не расхождения): hub-трейты `monsterattack`
//! кластера A2 (`monster_combat_facts`, `monster_attack_cell_candidates`,
//! снимок цели, применение попадания, зарегистрированный цикл каста),
//! подход/расписание — `ai::monsterai` через контакт; `ServerRegionOwner`
//! переиспользуется до синхронной смерти, продолжение заново получает
//! оставшегося владельца, не создавая замену исчезнувшему региону; дамп
//! `script_file` и stage-for-delete — фасады `CorpseCandleGame`, сам
//! `RunScript` прежнего скриптового owner-а — фасад `CorpseCandleContact`.
//! Потребление статическое (generic), dyn-совместимость и `Send`-контракт
//! не вводятся (ADR-0013).

use crate::app::game_message::CMessage;
use crate::combat::{AttackInformation, AttackPower, AttackPowerType};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::baseattackruntime::{
    BaseAttackContact, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
};
use super::lifecycle::{SkillStage, skill_is_restored};
use super::monsterattack::{
    MonsterCombatContact, MonsterCombatGame, apply_owned_monster_attack_hit,
    monster_attack_cell_candidates, resolve_owned_monster_attack_target,
};
use crate::ai::monsterai::schedule_attack_interval;

pub const CORPSE_CANDLE_BLASTING_SKILL_ID: u32 = 0x194;

// FIX F1: машинные ключи Calculate 0x582EA0 (push 0x4E28/0x4E29/0x4E21).
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;

const CORPSE_CANDLE_VISUAL_MESSAGE: i32 = 0x000b_fe01;
const CORPSE_CANDLE_DEATH_MESSAGE: i32 = 0x000b_f60b;

/// Маска области взрыва `g_bScope` при length = height = 3: кольцо вокруг
/// центра (`x + 3·y`, дырка в центре).
const SCOPE: [u8; 9] = [1, 1, 1, 1, 0, 1, 1, 1, 1];

/// Швы владельца `CMonster`, недоступные общему боевому hub A2: файл
/// death-скрипта живого монстра и пометка удаления после взрыва.
pub trait CorpseCandleGame: MonsterCombatGame {
    /// `CMonster::GetScriptFile` живого монстра (байты имени без копии).
    fn monster_script_file(&self, region: &Self::Region, monster_id: i32) -> Option<Vec<u8>>;

    /// `[U+0x80] = 1` — пометка удаления монстра-источника после взрыва.
    fn monster_stage_for_delete(&mut self, region: &mut Self::Region, monster_id: i32);
}

/// Контакт с runtime игрового хода: запуск death-скрипта прежнего
/// скриптового owner-а (`RunScript` с живым игроком-целью и регионом).
pub trait CorpseCandleContact<Runtime>: CorpseCandleGame + MonsterCombatContact<Runtime> {
    /// `RunScript(stRunScript{регион, игрок, файл})`; результат ворота —
    /// `false` только протоколируется, маршрут взрыва не меняется.
    fn run_corpse_candle_script(
        &mut self,
        script_file: &[u8],
        player_id: Option<i32>,
        region_id: i32,
        runtime: &mut Runtime,
    ) -> bool;
}

/// Кадр начала `0x000BFE01` (updateVE mode 0): action 1, навык, уровень,
/// сторона 600 и direction источника (GetDir, `vcall+0x5C`).
pub fn corpse_candle_start_message(skill_level: i32, monster_id: i32, direction: i32) -> CMessage {
    let mut message = CMessage::new(CORPSE_CANDLE_VISUAL_MESSAGE);
    message.add_byte(1);
    message.add_long(CORPSE_CANDLE_BLASTING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(direction);
    message
}

/// Кадр исполнения `0x000BFE01` (updateVE mode 1): action 2, два нулевых
/// long и клетка источника.
pub fn corpse_candle_fire_message(
    skill_level: i32,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(CORPSE_CANDLE_VISUAL_MESSAGE);
    message.add_byte(2);
    message.add_long(CORPSE_CANDLE_BLASTING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

/// Кадр смерти `0x000BF60B` после взрыва: пять Add(long) — 0, 0, тип 600,
/// id источника, 0 — и Add(byte) 2; SendToAround(U, NULL).
pub fn corpse_candle_death_message(monster_id: i32) -> CMessage {
    let mut message = CMessage::new(CORPSE_CANDLE_DEATH_MESSAGE);
    message.add_long(0);
    message.add_long(0);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_ulong(0);
    message.add_byte(2);
    message
}

/// Формула Calculate `0x582EA0` (FIX F1): MIN/MAX 20008/20009, `|max −
/// min| + 1` со знаковым abs, один RNG, элементная запись kind 3 с
/// jns-clamp; `GetAddElementAtk` монстра свёрнут в ноль заранее.
fn corpse_candle_blasting_attack<Game: MonsterCombatGame>(
    game: &mut Game,
    source_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
) -> AttackInformation {
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let damage = minimum.wrapping_add(game.skill_random_below(width)).max(0);
    AttackInformation {
        skill_id: CORPSE_CANDLE_BLASTING_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: source_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: damage,
            mp_damage: 0,
        }],
    }
}

/// Объектный путь монстра `0x194`: подход к дистанции, attack-speed
/// расписание, reuse, запрет движения и Begin каста; после задержки — обход
/// маски, удары, скрипт, кадр смерти, stage-for-delete и End(1) общего
/// зарегистрированного цикла.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub fn execute_owned_corpse_candle_blasting<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool
where
    Game: CorpseCandleContact<Runtime> + BaseAttackContact<Runtime>,
{
    let Some(region) = owner.as_mut().map(Game::owner_base_mut) else { return false; };
    let Some(facts) = game.monster_combat_facts(region, monster_id, CORPSE_CANDLE_BLASTING_SKILL_ID)
    else {
        return false;
    };
    let script_file = game.monster_script_file(region, monster_id).unwrap_or_default();
    let (source, cast) = (facts.source, facts.cast);

    let initial_target = if cast.is_none() {
        owner
            .as_ref()
            .and_then(|region_owner| resolve_owned_monster_attack_target(game, region_owner, target_identity))
    } else {
        None
    };
    let Some(region_owner) = owner.as_mut() else { return false; };
    if cast.is_none() {
        let Some(target) = initial_target else {
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
        game.monster_set_moveable(region, monster_id, false);
        game.monster_install_cast(
            region,
            monster_id,
            target_identity,
            CORPSE_CANDLE_BLASTING_SKILL_ID,
            skill_level,
            now_ms,
            target_object,
        );
        let region = Game::owner_base_mut(region_owner);
        let message = corpse_candle_start_message(
            i32::from(skill_level),
            monster_id,
            source.get_direction(),
        );
        game.send_visual_around(&*region, &source, &message);
        return true;
    }
    let cast = cast.expect("выполнение взрыва трупной свечи проверено выше");
    if cast.skill_id != CORPSE_CANDLE_BLASTING_SKILL_ID {
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
        let message = corpse_candle_fire_message(i32::from(skill_level), monster_id, center_x, center_y);
        game.send_visual_around(Game::owner_base(&*region_owner), &source, &message);
    }
    for x in 0_i32..3 {
        for y in 0_i32..3 {
            if SCOPE[(x + 3 * y) as usize] == 0 {
                continue;
            }
            let cell_x = center_x.wrapping_sub(1).wrapping_add(x);
            let cell_y = center_y.wrapping_sub(1).wrapping_add(y);
            let Some(region_owner) = owner.as_ref() else { return true; };
            let candidates =
                monster_attack_cell_candidates(game, region_owner, monster_id, cell_x, cell_y);
            for identity in candidates {
                let Some(region_owner) = owner.as_ref() else { return true; };
                if !game.live_skill_target_attackable_in(region_owner, source.identity(), identity) {
                    continue;
                }
                // Scan вызывал IsAttackAble; отдельный Attack отвергает 600 → 600.
                if identity.object_type == MONSTER_TYPE { continue; }
                let attack = corpse_candle_blasting_attack(game, monster_id, skill_level, properties);
                apply_owned_monster_attack_hit(game, owner, runtime, identity, attack);
            }
        }
    }
    let Some(region_owner) = owner.as_mut() else { return true; };
    game.monster_stage_for_delete(Game::owner_base_mut(region_owner), monster_id);
    let region_id = Game::owner_region_id(region_owner);
    if target_identity.object_type == PLAYER_TYPE
        && !script_file.is_empty()
        && script_file[0] != b'0'
    {
        let _ = game.run_corpse_candle_script(
            &script_file,
            Some(target_identity.id),
            region_id,
            runtime,
        );
    }
    {
        let died = corpse_candle_death_message(monster_id);
        game.send_visual_around(Game::owner_base(&*region_owner), &source, &died);
    }
    let region = Game::owner_base_mut(region_owner);
    game.monster_advance_cast(
        region, monster_id, CORPSE_CANDLE_BLASTING_SKILL_ID, SkillStage::Check, SkillStage::Calculate,
    );
    game.monster_advance_cast(
        region, monster_id, CORPSE_CANDLE_BLASTING_SKILL_ID, SkillStage::Calculate, SkillStage::Attack,
    );
    game.monster_advance_cast(
        region, monster_id, CORPSE_CANDLE_BLASTING_SKILL_ID, SkillStage::Attack, SkillStage::Apply,
    );
    game.monster_finish_cast_clock(region, monster_id, CORPSE_CANDLE_BLASTING_SKILL_ID, runtime);
    true
}
