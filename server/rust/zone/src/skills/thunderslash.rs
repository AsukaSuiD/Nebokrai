//! Громовое рассечение `CThunderSlash` (0x72) и слитый сюда visual
//! `CThunderSlashEffect` — один исходный `appserver/skills/thunderslash.cpp`.
//! Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match); исходные владельцы
//! `appserver/skills/thunderslash.cpp/.h` (тело и visual — тот же cpp).
//! Прежние переходные владельцы — `src/gameserver/appserver/skills/
//! {thunderslash,thunderslashvisual}.rs`; тела перенесены буквально порцией
//! T4 по прецеденту `skills/thunderblow2.rs` (visual слит в файл навыка).
//!
//! Машинные якоря (VA = RVA + 0x400000): ctor `0x57A240` (0x54 байта,
//! фабричный индекс `0x42`), vtable `0x65A974`; Begin триада
//! `0x57A3A0`/`0x57A2D0`/`0x57A490`; CheckCastCondition `0x57AAB0`;
//! AI `0x57ADD0`; Summon `0x57B2E0`; dtor `0x57A470`; End — общий хвост
//! `0x5AE7A0` (vcall vtable+0x68; `[+0x4C]/[+0x50]` в 0, свежему U
//! возвращается движение `0x4CCEE0(1)`, база `CSummonSkill::End` `0x5E0F40`);
//! `CThunderSlashEffect` vtable `0x65AA0C`, UpdateVisualEffect `0x57A550`
//! (jump-таблица 0..=15: modes 0/1 — around-пакеты, `[0, mode]` только
//! игроку для 2/4/7/10/11/13/14/15, mode 8/12 без пакета).
//!
//! Check (`0x57AAB0`): RTTI-гейт игрока; reuse `0x2715` (visual13 +
//! GS0278 при повторе раньше срока); непользовательский U допускается без
//! Move0/оружия/ресурсов; игроку нужны топор категории 1 слота 2
//! (`GetGoods(2)` → `GetAddonPropertyValues(5, 1) == 1` `0x4CB1E0`) →
//! visual14 + GS0287; signed-дефицит MP `query(2)` → visual7 + GS0288 и RP
//! `query(3)` (WORD-разность) → visual8 + GS0289 с суммами; успех запрещает
//! движение U (`0x4CCEE0(0)`).
//!
//! AI (`0x57ADD0`) двухфазный. Фаза `[+0x50] == 0` (без проверки смерти U
//! и без предварительного отсечения региона): повторная проверка оружия →
//! расход первого state `[+4] == 0x6E` БЕЗ RTTI/ended-проверок (End
//! vcall+0x1C + deleting dtor vcall+0x10 + обнуление слота), иначе visual4
//! + GS0304 + End(0) → расход MP (`0x4300D0`) и RP (`0x4300F0`) с
//! частичными эффектами при отказе (visual7/8 + GS + End(0)) →
//! OnChangeStates vcall+0x164 → CAN `0x2716` → `[+0x3C]` → назначение из
//! живой S (vcall+0x18) либо сохранённой точки `[+0x24]/[+0x28]` →
//! направление `0x41D080` → SetDirection vcall+0x60 → visual0.
//! Фаза `[+0x50] != 0`: задержка `0x2711` как unsigned `start + delay <= now`
//! → visual1 → лицевая клетка (GetTile, GetDirectionPos vcall+0x5C,
//! `0x45B330`) → RTTI-регион из `[U+0x40]`; без регионального владельца
//! End(1) без Summon. Мёртвый GetShapes/RTTI-скан не воспроизводится:
//! найденный объект в `0x57B225`–`0x57B28C` не использовался и игровых
//! callback-ов там нет. Summon (vcall+0x8C) получает отдельную свежую
//! таблицу (`0x46C390(0x72, level)`), lifetime `query(0x7531)`, снимок
//! боевых getter-ов Dex([+0x3B8])→SOUL(vcall+0x11C)→CCH(+0x114)→ELEMENT
//! (+0x118)→MIN(+0xE4)→MAX(+0xE8), country=0, PK-байты [+0x278..+0x27B]
//! только при RTTI CPlayer; безопасная замена ограничивает создание формы
//! игроком (у native Summon после необязательного RTTI есть разыменование
//! NULL), не меняя общий Check/AI.
//!
//! Объявленные швы переноса (не расхождения): hub-трейты
//! `ThunderSlashGame`/`ThunderSlashPlayer`/`ThunderSlashContact` —
//! переходные фасады прежнего `CGame`/`CPlayer` поверх общего
//! `statecast::StateCastGame` (реализация у делегата старого пакета);
//! consume расхода состояния — `skills/ragebreakstate.rs`. Часы AI —
//! `now_milliseconds` делегата прежнего main-loop runtime (прецедент
//! `skills/thunderblow.rs`). Результат AI — локальный исход, End выполняет
//! общий registered-вход делегата (`finish_outcome`), как раньше.
//!
//! UNKNOWN/объявленная неполнота: запись CAN `0x2716` → `[+0x3C]` в
//! исполнение player-kernel отдельно не материализуется — реконструкция
//! только читает свойство; потребители поля +0x3C этой стороны в волне не
//! устанавливались (конвенция summon-полосы).

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::{CShape, ShapeAreaCoordinates};

use super::execution::RegisteredSkillRecord;
use super::fury::RageCastPlayer;
use super::lifecycle::SkillStage;
use super::lifecycle::skill_is_restored;
use super::skillfactory::SkillOwner;
use super::statecast::{StateCastGame, StateCastMoveShape, StateCastPlayer};
use super::thunderslashphalanx::CThunderSlashPhalanx;
use super::visualeffect::SkillVisualEffectKind;

pub const THUNDER_SLASH_SKILL_ID: u32 = 0x72;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

/// Локальный исход одного тика AI: End не выполняется здесь — общий
/// registered-вход делегата завершает экземпляр по исходу, как прежний
/// `terminal` + `finish_outcome`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderSlashOutcome {
    Pending,
    Rejected,
    Completed,
}

/// PK-допуски источника для MasterInfo снимка Summon (та же четвёрка, что
/// у dash/базовой атаки; country у этого владельца всегда ноль).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThunderSlashPkPermissions {
    pub player: bool,
    pub teammate: bool,
    pub guild_member: bool,
    pub criminal: bool,
}

/// Игрок-источник рассечения: переходный фасад прежнего `CPlayer`.
/// RP-пара — общий шов яростных навыков (`fury::RageCastPlayer`).
pub trait ThunderSlashPlayer: StateCastPlayer + RageCastPlayer {
    fn shape(&self) -> &CShape;

    fn set_skill_moveable(&mut self, moveable: bool);

    fn player_id(&self) -> i32;

    fn faction_id(&self) -> i32;

    fn team_id(&self) -> i32;

    fn union_id(&self) -> i32;

    fn thunder_slash_pk_permissions(&self) -> ThunderSlashPkPermissions;
}

/// Переходные фасады прежнего владельца `CGame`, открывающие исполнению
/// `CThunderSlash` только прежние обращения; имена сохраняют исходную операцию.
pub trait ThunderSlashGame: StateCastGame {
    /// Оружие slot2 категории 1: `GetGoods(2)` и
    /// `GetAddonPropertyValues(5, 1) == 1`; фабрика предметов — у владельца.
    fn thunder_slash_player_weapon_valid(&self, player: &Self::Player) -> bool;

    /// RTTI-гейт региона AI (`dyn_cast CServerRegion` из `[U+0x40]`);
    /// региональное владение — у владельца.
    fn thunder_slash_region_present(&self, region_id: i32) -> bool;

    /// Новый ID призванной формы (`g_lID` живого `CGame`).
    fn allocate_summon_shape_id(&mut self) -> i32;

    // Доставка visual `0x000BFE01`: кадр строится здесь, маршруты — у владельца.
    fn send_thunder_slash_visual_to_player(&self, player_id: i32, message: &CMessage);

    fn send_thunder_slash_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);
}

/// Контактная стадия AI: регистрация формы у владельца региона и входной
/// снимок `0xBF502` — тип хода принадлежит старому main loop, а не навыку.
pub trait ThunderSlashContact<Runtime>: ThunderSlashGame {
    /// Прежний `spawn_thunder_slash_phalanx`: Add/SetCenter в регионе,
    /// serialized snapshot и отправка окружающим; при отказе Add объект
    /// сериализуется и безопасно освобождается.
    fn spawn_thunder_slash_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CThunderSlashPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()>;
}

fn master_info<Player: ThunderSlashPlayer>(player: &Player) -> MasterInfo {
    let permissions = player.thunder_slash_pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn failure<Game: ThunderSlashGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: Option<i32>,
    mode: u32,
) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id {
        let text = match mode { 4 => &b"GS0304"[..], 13 => b"GS0278", 14 => b"GS0287", _ => return };
        game.send_skill_system_info(player_id, text);
    }
}

fn resource_failure<Game: ThunderSlashGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    properties: &CSkillBaseProperties,
    usage: u32,
) {
    let (mode, text) = if usage == USER_MP_LOSE { (7, &b"GS0288"[..]) } else { (8, &b"GS0289"[..]) };
    game.update_registered_skill_visual(instance, mode);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

/// MP/RP с signed-дефицитом DWORD-разности; отказ сохраняет уже
/// выполненные частичные эффекты прохода (visual + GS, без Set).
fn handle_resources<Game>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    properties: &CSkillBaseProperties,
    spend: bool,
) -> bool
where
    Game: ThunderSlashGame,
    Game::Player: ThunderSlashPlayer,
{
    if properties.query_property(USER_MP_LOSE) != 0 {
        let Some(mana) = game.find_player(player_id).map(|player| player.mana()) else { return false; };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, player_id, properties, USER_MP_LOSE);
            return false;
        }
        if spend {
            let Some(player) = game.find_player_mut(player_id) else { return false; };
            player.set_mana(remaining);
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let Some(rp) = game.find_player(player_id).map(|player| player.rp()) else { return false; };
        let remaining = u32::from(rp).wrapping_sub(properties.query_property(USER_RP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, player_id, properties, USER_RP_LOSE);
            return false;
        }
        if spend {
            let Some(player) = game.find_player_mut(player_id) else { return false; };
            player.set_rp(remaining as u16);
        }
    }
    true
}

/// CheckCastCondition `0x57AAB0` с исходным U, ранним отсчётом reuse и
/// loop1 visual общего входа; отказ Check вызывает End(0) без
/// дополнительного visual2 (его даёт обвязка делегата при отказе Begin).
pub fn check_thunder_slash_cast<Game>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: (i32, ShapeIdentity),
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: ThunderSlashGame,
    Game::Player: ThunderSlashPlayer,
{
    let Some(source) = game.resolve_state_move_shape(original_user.0, original_user.1) else { return false; };
    let identity = source.shape().identity();
    let player_id = (identity.object_type == PLAYER_TYPE).then_some(identity.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    let Some(player_id) = player_id else { return true; };
    let weapon_valid = game.find_player(player_id)
        .is_some_and(|player| game.thunder_slash_player_weapon_valid(player));
    if !weapon_valid {
        failure(game, instance, Some(player_id), 14);
        return false;
    }
    if !handle_resources(game, instance, player_id, &properties, false) { return false; };
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

/// Summon `0x57B2E0`: свежая таблица и снимок боевых getter-ов после
/// visual1 и лицевой клетки; форма, её wire и runtime — отдельные владельцы.
#[allow(clippy::too_many_lines, reason = "порядок ctor-аргументов EXE сохранён буквально")]
fn summon_thunder_slash<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    x: i32,
    y: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) where
    Game: ThunderSlashContact<Runtime>,
    Game::Player: ThunderSlashPlayer,
{
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    let identity = user.shape().identity();
    let mut master = if identity.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(identity.id) else { return; };
        master_info(player)
    } else {
        MasterInfo {
            master_type: identity.object_type,
            master_id: identity.id,
            ..Default::default()
        }
    };
    master.master_country_id = 0;
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    // Безопасная замена native-разыменования: форма создаётся только игроком.
    if identity.object_type != PLAYER_TYPE { return; }
    let Some(player) = game.find_player(identity.id) else { return; };
    let dexterity = player.combat_properties().dexterity as i32;
    let soul = i32::from(player.combat_properties().add_soul_attack);
    let cch = i32::from(player.combat_properties().cch);
    let element = player.combat_properties().add_element_attack as i32;
    let minimum = player.combat_properties().minimum_attack as i32;
    let maximum = player.combat_properties().maximum_attack as i32;
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let started = now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CThunderSlashPhalanx::new(
        id, master, started, lifetime, level, frequency, maximum, minimum,
        element, dexterity, cch, soul, x, y,
    );
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.spawn_thunder_slash_phalanx(region, phalanx, started, runtime);
}

/// AI `0x57ADD0`: двухфазный проход с расходом состояния и ресурсов на
/// первом тике и Summon после задержки. Смерть U и живая смена региона не
/// проверяются; мёртвый GetShapes-скан не воспроизводится (см. шапку).
pub fn run_thunder_slash_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> ThunderSlashOutcome
where
    Game: ThunderSlashContact<Runtime>,
    Game::Player: ThunderSlashPlayer,
{
    let Some(skill) = game.registered_skill(instance) else { return ThunderSlashOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return ThunderSlashOutcome::Pending;
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return ThunderSlashOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(region, identity) else {
        return ThunderSlashOutcome::Rejected;
    };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let weapon_valid = game.find_player(user.1.id)
                .is_some_and(|player| game.thunder_slash_player_weapon_valid(player));
            if !weapon_valid {
                failure(game, instance, Some(user.1.id), 14);
                return ThunderSlashOutcome::Rejected;
            }
            if !super::ragebreakstate::consume_rage_break_state(game, user) {
                failure(game, instance, Some(user.1.id), 4);
                return ThunderSlashOutcome::Rejected;
            }
            if !handle_resources(game, instance, user.1.id, &properties, true) {
                return ThunderSlashOutcome::Rejected;
            }
            game.publish_player_states(user.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return ThunderSlashOutcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return ThunderSlashOutcome::Rejected; };
        let destination = match game.resolve_skill_sufferer(skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = game.resolve_state_move_shape(region, identity) else {
                    return ThunderSlashOutcome::Rejected;
                };
                (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                )
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = game.resolve_state_move_shape(user.0, user.1) else {
            return ThunderSlashOutcome::Rejected;
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
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return ThunderSlashOutcome::Rejected;
    };
    if now_milliseconds() < started.wrapping_add(delay) {
        return ThunderSlashOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    if let Some(source) = game.resolve_state_move_shape(user.0, user.1) {
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let direction = source.shape().get_direction();
        if let Ok(front) = CShape::get_direction_position(direction, ShapeAreaCoordinates { x, y })
            && source.shape().is_assigned_to_server_region()
            && game.thunder_slash_region_present(source.shape().get_region_id())
        {
            summon_thunder_slash(game, instance, user, front.x, front.y, runtime, now_milliseconds);
        }
    }
    ThunderSlashOutcome::Completed
}

/// UpdateVisualEffect `0x57A550` (`0x000BFE01`): подготовка передаёт живое
/// направление U; выпуск заново разрешает S и её X/Y либо базовую точку с
/// нулевыми type/id (это не клетка будущей формы). Ошибки
/// 2/4/7/10/11/13/14/15 отправляются только игроку; mode 8/12 не имеют
/// производного пакета. Общий владелец сохраняет базовый visual-tail.
pub fn publish_thunder_slash_visual<Game: ThunderSlashGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if skill.owner() != SkillOwner::CThunderSlash
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::ThunderSlash || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 4 | 7 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            let mut message = CMessage::new(EFFECT_MESSAGE);
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_thunder_slash_visual_to_player(source.identity().id, &message);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if mode == 1 {
        Some(match game.resolve_skill_sufferer(skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = game.resolve_state_move_shape(region, identity) else { return; };
                let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
                let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
                (target.shape().identity().object_type, target.shape().identity().id, x, y)
            }
            None => {
                let (x, y) = skill.lifecycle().destination();
                (0, 0, x, y)
            }
        })
    } else { None };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some((kind, id, x, y)) = target {
        message.add_long(kind);
        message.add_long(id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region() {
        game.send_thunder_slash_visual_around(source.get_region_id(), source, &message);
    }
}
