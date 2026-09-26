//! Hub-швы и буквальные тела общего скелета Begin/Check/AI/visual/End
//! областных призывов: Weak (0x12E), PoisonFog (0xC9), SnowStorm (0x193),
//! YinYang/YinYang2 (0x139/0x146), GodThunder/GodThunder2 (0x140/0x143),
//! FireWall (0x134), ChaosSphere (0x137) и SoulMirror (0x13C).
//! Источник: gameserver.exe + GameServer.pdb (точная пара, ниже; RSDS match),
//! `appserver/skills/weak.cpp`, `poisonfog.cpp`, `snowstorm.cpp`,
//! `yinyang.cpp/yinyang2.cpp`, `godthunder.cpp/godthunder2.cpp`,
//! `firewall.cpp`, `chaossphere.cpp` и `soulmirror.cpp`. Прежний переходный
//! владелец — `src/gameserver/appserver/skills/zonalcast.rs` и клей
//! владельцев (`weak.rs`, `poisonfog.rs`, `snowstorm.rs`, `chaossphere.rs`,
//! `yinyang.rs`, `godthunder{,2}.rs`), тела перенесены буквально порцией
//! T5 «zonalcast-хаб»; реализация фасадов швов остаётся у прежнего
//! владельца в том же файле-делегате.
//!
//! Точная пара: `original/server/Miracle_server/GameServer/gameserver.exe`
//! (SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`)
//! + `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2; совпадение подтверждено `.local/evidence/symbols.py identity`).
//! Конвенция адресов: пабы PDB записаны как `seg:off` сегмента `.text`;
//! истинный RVA = off + 0x1000, VA = RVA + ImageBase 0x400000.
//!
//! Машинная проверка порции T5 (дизассемблер точной пары; статусы ниже
//! относятся только к перечисленным ветвям):
//!
//! - `VERIFIED` скелет `CWeak::CheckCastCondition` (RVA `0x1AF050`): свежая
//!   таблица `QuerySkillBaseProperties` → reuse `10005` + `timeGetTime` →
//!   visual13 + `GS0278` → длина пути `5003` (`jbe` при нуле) → visual11 +
//!   `GS0290` → MP-контракт (MP0 — тихий отказ, иначе signed-разность) →
//!   visual7 + `GS0288` с ценой → `SetMoveable(0)`. Блок-проверки пути у
//!   Weak нет — это `CastPathBlock::Ignore` прежнего хаба.
//! - `VERIFIED` скелет `CWeak::AI` (RVA `0x1AED40`): активный гейт фазы,
//!   одна таблица на всё тело, GetUser/GetSufferer, смерть S → visual10 +
//!   `GS0285` + `End(0)`; иначе два чтения X/Y S — в локальные регистры и в
//!   сохранённые поля — с очисткой identity S; NULL U → `End(0)`; Begin-доля:
//!   MP → `OnChangeStates` → CAN `10006` → `GetLineDir` → `SetDir` →
//!   visual0 → фаза=1 → проход в тот же тик без второго активного гейта;
//!   unsigned `start + delay(10001)` → visual1 → `Summon` (вирт. +0x8C) →
//!   `End(1)` независимо от результата.
//! - `VERIFIED` дистинктивные ветви Check: `CPoisonFog::CheckCastCondition`
//!   (RVA `0x193D50`) превращает S в точку и чистит её identity ещё до
//!   таблицы и reuse; требует арбалет — `GetAddonProperty` категории 4,
//!   отказ даёт visual14 + `GS0293`; `CFireWall::CheckCastCondition`
//!   (RVA `0x1AB8B0`) сканирует путь и запрещает blocker-клетки 1 и 2
//!   (ветка `GroundAndFly`) с visual15 + `GS0282`; у `CChaosSphere`
//!   (RVA `0x1A6DC0`) чтения пути нет вовсе; у `CSnowStorm` (RVA `0x183E20`
//!   и AI RVA `0x183C40`) ошибки reuse/пути/MP — только visual-режимы,
//!   без GS-форматирования, а недостаток MP и в Check, и в AI даёт
//!   visual7 без `GS0288`; `CChaosSphere::AI` (RVA `0x1A7FA0`) после
//!   проверки смерти читает X/Y S один раз (без повторного чтения Weak)
//!   и очищает identity S.
//! - `VERIFIED` wire-кадры visual `0x000BFE01` всех десяти тел
//!   `*Effect::UpdateVisualEffect`: switch по 16 режимам с таблицей переходов;
//!   личная ветка отказов кадром `[u8=0, u8=mode]` только игроку; mode 0 →
//!   кадр `action=1` (навык, уровень, источник, direction); mode 1 → кадр
//!   `action=2` с нулевой парой цели и X/Y. Наборы режимов: у девяти владельцев
//!   `0/1/2/7/10/11/13/15`, у PoisonFog добавлен `14` (таблицы RVA CWeak
//!   `0x1AE8C0`, CPoisonFog `0x1935E0`, CSnowStorm `0x1837C0`, CYinYang
//!   `0x1A5730`, CYinYang2 `0x1677A0`, CGodThunder `0x172D00`, CGodThunder2
//!   `0x152F40`, CFireWall `0x1AB140`, CChaosSphere `0x1A7940`, CSoulMirror
//!   `0x1A4400`). Источники X/Y mode 1: живой S с откатом к сохранённой точке
//!   (проверено для CWeak/CFireWall/CSnowStorm/CGodThunder/CYinYang/CYinYang2/
//!   CChaosSphere; у CGodThunder2 тот же шаблон компилятора — `MATCH` без
//!   отдельного досмотра); PoisonFog всегда пишет сохранённую точку;
//!   SoulMirror — текущий центр U.
//! - Входной кадр области `0x000BF502` (5-полевый префикс + снимок) сверен
//!   прежними порциями: таблица RVA уникальных тел AddToByteArray — в шапке
//!   `skills/summonshape.rs`; per-пhalanx encoder-ы — в шапках
//!   `skills/{weak,poisonfog,snowstorm,godthunder,masked_area,chaossphere}.rs`
//!   и в `docs/gameplay/skills.md`.
//!
//! Честные неизвестные (не повышаются этой порцией): тела
//! `CheckCastCondition`/`AI` YinYang/YinYang2, GodThunder/GodThunder2 и
//! SoulMirror (у последнего иная арность Check — `UAEHPAVCMoveShape@@0@Z`)
//! индивидуально не досматривались; их ветви перенесены по прежней
//! реконструкции и сверены с `git show HEAD` (`MATCH` контента, статус
//! `PARTIAL` без постатейной машинной выписки). Клиентское чтение
//! кадров — вне серверной базы (`UNKNOWN`, как в соседних волнах).
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — переходные
//! фасады прежнего владельца `CGame`/`CPlayer`/`CMoveShape`, реализация
//! остаётся у него в файле-делегате `appserver/skills/zonalcast.rs`; имена
//! членов сохраняют исходную операцию. Швы потребляются статически
//! (generic), dyn-совместимость и `Send`-контракт не вводятся (прецедент
//! ADR-0013 семейства statecast/selfcast). Общие хелперы старого пакета
//! переносятся не как тела, а объявляются швами: MP/путь/оружие
//! `rangedweaponcast` (`check_cast_mana*`, `spend_cast_mana*`,
//! `check_skill_path`, арбалетные проверки), мастер и живые CCH/элемент
//! `weaponattack::source_*`, обвязка арены `states/state.rs`, регистрация
//! областей и рассылка BF502 (`CGame::add_*_phalanx`/`send_*_entry`),
//! оружейный множитель/критическая ставка `globe_setup` + goods factory.
//! Тело `CFireWall::Summon` остаётся у прежнего владельца `appserver/
//! skills/firewall.rs` (файл вне порции) и вызывается швом `summon_fire_wall`.
//! `CSoulMirror` граница: его обход области и призыв клеток живут в
//! `skills/soulmirror.rs` (порция №6c, швы `SelfCastGame`), отсюда вызов
//! идёт швом `apply_soul_mirror_area`; маска клетки (`soul_mirror_scope_*`)
//! тоже принадлежит ему; входной снимок порождённых зеркалом существ —
//! wire-конверт `skills/summonshape.rs` (сноска по визуалам: hub visual
//! переносит только кадр `0xBFE01`, BF502 — дело summonshape).
//! Часы прежнего main loop приходят указателем `now_milliseconds`
//! (делегат передаёт перечитывание через `runtime.now_milliseconds()`).

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::combat::{AttackInformation, MasterInfo, PlayerCombatProperties, truncate_original};
use crate::content::CSkillBaseProperties;
use crate::effects::WeakState;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;

use super::baseattackruntime::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::chaossphere::{CChaosSpherePhalanx, CHAOS_SPHERE_SKILL_ID};
use super::elementphalanx::ElementSummonLiveField;
use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::firewall::FIRE_WALL_SKILL_ID;
use super::godthunder::{CGodThunderPhalanx, GOD_THUNDER_2_SKILL_ID, GOD_THUNDER_SKILL_ID};
use super::lifecycle::{SkillLifecycle, SkillStage, skill_is_restored};
use super::masked_area::MaskedElementPhalanx;
use super::poisonfog::CPoisonFogPhalanx;
use super::snowstorm::{CSnowStormPhalanx, SNOW_STORM_SKILL_ID};
use super::soulmirror::SOUL_MIRROR_SKILL_ID;
use super::state::{AppliedState, StateKey};
use super::visualeffect::SkillVisualEffectKind;
use super::weak::{CWeakPhalanx, WEAK_SKILL_ID};
use super::yinyang::{YIN_YANG_2_SKILL_ID, YIN_YANG_SKILL_ID};

use crate::effects::POISON_FOG_STATE_ID as POISON_FOG_SKILL_ID;

const ELEMENT_SCALE_PROPERTY: u32 = 20_015;
const ZONAL_CAST_VISUAL_MESSAGE: i32 = 0x000b_fe01;

pub const fn is_zonal_cast_skill(id: u32) -> bool {
    matches!(id, WEAK_SKILL_ID | POISON_FOG_SKILL_ID | SNOW_STORM_SKILL_ID
        | YIN_YANG_SKILL_ID | YIN_YANG_2_SKILL_ID | GOD_THUNDER_SKILL_ID | GOD_THUNDER_2_SKILL_ID
        | FIRE_WALL_SKILL_ID | CHAOS_SPHERE_SKILL_ID | SOUL_MIRROR_SKILL_ID)
}

/// Политика блок-клеток пути Check (выбор семьи по владельцу; тело проверки
/// остаётся прежним хелпером `rangedweaponcast` за объявленным швом).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZonalCastPathBlock {
    Ignore,
    Generic,
    GroundAndFly,
}

/// Адресат стандартных property-visual операций арены состояний (`Sufferer`
/// у обеих перенесённых семей; форма параметра сохранена для тождества шва).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZonalCastPropertyTarget {
    User,
    Sufferer,
}

/// Исход одного тика AI областного призыва. `Pending`/`Rejected`/`Completed`
/// отображаются делегатами: игрок — исходами внешней очереди, монстр —
/// полным `End(0)`/`End(1)` прежнего stateskill.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZonalCastAiOutcome {
    Pending,
    Rejected,
    Completed,
}

/// Переходный фасад игрока семьи: операции MP и боевых свойств, которые
/// выполняют перенесённые тела (прежний `CPlayer`).
pub trait ZonalCastPlayer {
    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);

    fn combat_properties(&self) -> PlayerCombatProperties;

    /// Живой `update_state_combat_properties`: замена снимка и wire без
    /// обхода остальных состояний.
    fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    );

    /// Живой уровень игрока (для расчёта потерь PoisonFogState).
    fn level(&self) -> u8;
}

/// Живая фигура стороны каста и арена состояний (прежний `CMoveShape`).
pub trait ZonalCastMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);

    fn applied_state<T: AppliedState>(&self, key: StateKey) -> Option<&T>;

    fn append_applied_state_record<T: AppliedState>(&mut self, state: T, record: &[u8]) -> StateKey;

    fn mark_applied_state_begun(&mut self, key: StateKey) -> bool;

    fn set_applied_state_user(
        &mut self,
        key: StateKey,
        user: Option<(i32, ShapeIdentity)>,
    ) -> bool;

    fn set_applied_state_sufferer(
        &mut self,
        key: StateKey,
        sufferer: Option<(i32, ShapeIdentity)>,
    ) -> bool;

    fn set_applied_state_sufferer_region(&mut self, key: StateKey, region_id: i32) -> bool;
}

/// Переходные фасады прежнего владельца `CGame`, открывающие скелету
/// областного призыва и состояниям Weak/PoisonFog только прежние обращения;
/// имена сохраняют исходную операцию.
pub trait ZonalCastGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ);
    /// непрозрачен для исполнения.
    type SkillAddress: Copy;

    type Player: ZonalCastPlayer;

    type MoveShape: ZonalCastMoveShape;

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
        length: u32,
    ) -> Vec<(i32, i32, u8)>;

    fn move_shape_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32>;

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8>;

    // Диагностика игрока (GS-тексты).
    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    // Захват мастера и живые значения источника Summon (`weaponattack`).
    /// Мастер источника прежнего `source_master`; без обнуления country —
    /// обнуление выполняет перенесённое тело, как и прежний caller.
    fn zonal_source_master(&self, source: (i32, ShapeIdentity)) -> Option<MasterInfo>;

    /// Живые CCH и AddElementAttack источника (`source_property` прежнего
    /// пакета; поле выбирает вызывающий Summon в прежней точке).
    fn zonal_source_property(
        &self,
        source: (i32, ShapeIdentity),
        property: ElementSummonLiveField,
    ) -> Option<u32>;

    /// Живой уровень оружия игрока (`weapon_damage_level(goods_factory())`);
    /// отсутствие оружия читается нулём у владельца.
    fn player_weapon_damage_level(&self, player: &Self::Player) -> u32;

    /// SAFE-гейт клетки для Weak: `None` — регион не найден (прежний ранний
    /// возврат), иначе `get_security == SAFE` (прежние ветки завершали
    /// Summon молча).
    fn zonal_area_safe(&self, region_id: i32, x: i32, y: i32) -> Option<bool>;

    // Часы/RNG/ID прежнего `CGame`.
    fn allocate_summon_shape_id(&mut self) -> i32;

    fn skill_random_below(&mut self, maximum: i32) -> i32;

    // Check-скелет `rangedweaponcast` прежнего пакета (объявленные швы):
    // дальность/препятствия с выбранной веткой, MP-контракт и списание MP,
    // проверка арбалета PoisonFog до и после расхода MP.
    fn check_zonal_skill_path(
        &mut self,
        address: Self::SkillAddress,
        properties: &CSkillBaseProperties,
        path: &[(i32, i32, u8)],
        player: Option<i32>,
        block: ZonalCastPathBlock,
    ) -> bool;

    /// `check_ranged_weapon_and_mana` с арбалетом и обязательной ценой
    /// (отказ оружия даёт visual14 + GS0293).
    fn check_crossbow_cast(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool;

    fn check_cast_mana(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool;

    /// MP-контракт SnowStorm: те же проверки, но без повторного запроса
    /// цены для GS0288 — только visual7.
    fn check_cast_mana_without_text(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool;

    fn spend_cast_mana(
        &mut self,
        address: Self::SkillAddress,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool;

    fn spend_cast_mana_without_text(
        &mut self,
        address: Self::SkillAddress,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool;

    /// Повторная проверка арбалета после расхода MP, без возврата расхода
    /// (`prepare_ranged_weapon_player` с `RangedWeaponKind::Crossbow`).
    fn prepare_crossbow_player(
        &mut self,
        address: Self::SkillAddress,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool;

    // Применение элементальных областей (расчётные данные —
    // `skills/elementphalanx.rs`; здесь живые разрешения цепочки).
    /// Оружейный множитель живого Player-владельца против уровня цели
    /// (оружейные факторы `globe_setup` и фабрика предметов — у владельца).
    fn element_phalanx_weapon_modifier(&self, attacker_id: i32, target_level: i32) -> Option<f32>;

    fn element_phalanx_critical_rate(&self) -> f32;

    /// PK-допуск боевого духа: глобальные Player цели и источника, не self,
    /// не action==6 и не мёртвый, затем `live_skill_target_attackable_between`.
    /// Возвращает допустимую пару (region, identity) цели.
    fn element_phalanx_war_soul_target(
        &self,
        master: MasterInfo,
        target_id: i32,
    ) -> Option<(i32, ShapeIdentity)>;

    /// Уровень монстра по имени исходной таблицы (monster→base_property_key
    /// → property реестра → level, младший байт).
    fn monster_property_level(&self, region_id: i32, monster_id: i32) -> Option<u8>;

    /// Запись `state.apply_to_monster_attacks` в модификаторы живого монстра
    /// региона; `false`, если монстр не найден (прежний caller молчал).
    fn apply_weak_monster_attacks(
        &mut self,
        region_id: i32,
        monster_id: i32,
        state: &WeakState,
    ) -> bool;

    /// Wrapping-вычитание потерь тумана из модификаторов живой фигуры
    /// (`resolve_state_move_shape_mut` → `property_modifiers_mut`).
    fn subtract_poison_fog_monster_losses(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        defense_loss: u32,
        element_resistance_loss: u32,
    ) -> bool;

    // Обвязка арены состояний прежнего `states/state.rs` (объявленные швы,
    // одноимённые операции).
    fn begin_base_applied_state(&mut self, region_id: i32, holder: ShapeIdentity, key: StateKey) -> bool;

    fn begin_applied_state_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        loop_value: i32,
    ) -> bool;

    fn resolve_applied_state_sufferer(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> Option<(i32, ShapeIdentity)>;

    fn update_property_state_visual<S: AppliedState>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: ZonalCastPropertyTarget,
        now: &mut dyn FnMut() -> u32,
        client_time: impl FnOnce(&S, &mut dyn FnMut() -> u32) -> u32,
    ) -> bool;

    fn update_applied_state_end_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: ZonalCastPropertyTarget,
    ) -> bool;

    fn remove_applied_state_from(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: (i32, ShapeIdentity),
        bytes: usize,
    ) -> bool;

    // Доставка visual `0x000BFE01`: кадр строится в `publish_zonal_cast_visual`,
    // маршруты — у владельца.
    fn send_zonal_cast_visual_to_player(&self, player_id: i32, message: &CMessage);

    /// Around-ветвь по региону формы; уже проверена назначенность источника,
    /// отсутствующий регион пропускает отправку, как и раньше.
    fn send_zonal_cast_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);
}

/// Ветки суммона и контакта с runtime игрового хода: регистрация областей,
/// рассылка BF502, применение ударов и чужие тела владельцев.
/// Отделена, потому что тип хода принадлежит старому main loop, а не
/// скелету призыва (прецедент `BaseAttackContact<Runtime>`).
pub trait ZonalCastContact<Runtime>: ZonalCastGame {
    /// Регистрация области Weak в живом регионе (`take_region_owner` →
    /// `AddShape` → encode/BF502 даже при отказе). `Some(Err(()))` — отказ
    /// Add (блок не различался и раньше); `None` — регион не найден.
    fn add_weak_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CWeakPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, ()>>;

    fn send_weak_phalanx_entry(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()>;

    fn add_poison_fog_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CPoisonFogPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, ()>>;

    fn send_poison_fog_phalanx_entry(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()>;

    fn add_snow_storm_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CSnowStormPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()>;

    fn add_god_thunder_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CGodThunderPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()>;

    /// Add маскированной области YinYang — без прохода замены соседних
    /// масок (её у YinYang не было и в прежнем теле).
    fn add_masked_element_phalanx(
        &mut self,
        region_id: i32,
        phalanx: MaskedElementPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()>;

    fn add_chaos_sphere_phalanx(
        &mut self,
        region_id: i32,
        phalanx: CChaosSpherePhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()>;

    /// Общий боевой контакт (прежний virtual +15C `apply_owned_skill_contact`).
    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );

    fn apply_owned_skill_attack_to_war_soul(
        &mut self,
        master: MasterInfo,
        target_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );

    /// Тело `CFireWall::Summon` прежнего владельца `appserver/skills/firewall.rs`
    /// (файл вне порции T5).
    fn summon_fire_wall(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        destination: (i32, i32),
        runtime: &mut Runtime,
    );

    /// Обход области SoulMirror `skills/soulmirror.rs` порции №6c
    /// (через прежнего делегата `appserver/skills/soulmirror.rs`).
    fn apply_soul_mirror_area(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
        runtime: &mut Runtime,
    );
}

/// Свежий пользователь по сохранённым region/type/id записи (`GetUser`).
pub fn zonal_cast_resolved_user<Game: ZonalCastGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = game.resolve_state_move_shape(region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

/// Общий префикс Summon YinYang/GodThunder/FireWall: Master(country0) и
/// Player EM, затем свежая таблица. Unsigned usage20015 умножается на
/// расширенный literal 0.01f и signed EM; FISTP с усечением выполняется
/// до CCH и остальных запросов конструктора, без промежуточного округления.
pub fn prepare_element_summon<Game: ZonalCastGame>(
    game: &Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
) -> Option<(MasterInfo, CSkillBaseProperties, i32)> {
    let mut master = game.zonal_source_master(source)?;
    master.master_country_id = 0;
    let element = if source.1.object_type == PLAYER_TYPE {
        game.find_player(source.1.id)?.combat_properties().element_modify
    } else { 0 };
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let modifier = properties.query_property(ELEMENT_SCALE_PROPERTY);
    let scaled = truncate_original(f64::from(modifier) * f64::from(0.01_f32) * f64::from(element));
    Some((master, properties, scaled))
}

/// Общий Check семьи. PoisonFog превращает найденную S в точку ещё до
/// таблицы и reuse; абсолютный reuse `10005`, путь читают все, кроме
/// ChaosSphere и SoulMirror; Weak — только дальность, FireWall запрещает
/// BLOCK1|2; PoisonFog требует арбалет; SnowStorm сообщает только visual.
pub fn check_zonal_cast<Game: ZonalCastGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
    now_ms: u32,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return false; };
    let source = (user.shape().get_region_id(), user.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let id = skill.id();
    if id == POISON_FOG_SKILL_ID {
        let destination = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ));
        if let Some(destination) = destination
            && let Some(skill) = game.registered_skill_mut(instance)
        { skill.lifecycle_mut().set_point_target(destination); }
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(id, skill.level()).cloned() else { return false; };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let text_player = player.filter(|_| id != SNOW_STORM_SKILL_ID);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_ms) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = text_player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    if !matches!(id, CHAOS_SPHERE_SKILL_ID | SOUL_MIRROR_SKILL_ID) {
        let path = game.skill_target_path(skill.lifecycle());
        let block = match id {
            WEAK_SKILL_ID => ZonalCastPathBlock::Ignore,
            FIRE_WALL_SKILL_ID => ZonalCastPathBlock::GroundAndFly,
            _ => ZonalCastPathBlock::Generic,
        };
        if !game.check_zonal_skill_path(instance, &properties, &path, text_player, block) { return false; }
    }
    match id {
        POISON_FOG_SKILL_ID => game.check_crossbow_cast(instance, source, &properties),
        SNOW_STORM_SKILL_ID => game.check_cast_mana_without_text(instance, source, &properties),
        SOUL_MIRROR_SKILL_ID => {
            // Машинный Check `0x1A4850`: MP-контракт по arg1 (U), но цель
            // `SetMoveable(_, 0)` — arg2 (sufferer); для практических кастов
            // self-цели они совпадают. Разрешаем sufferer с откатом к source
            // (patch-контур verify-t5); разрешение — вложенным неизменяемым
            // блоком, чтобы не продлевать заём `skill` до &mut-вызова.
            let target = game
                .registered_skill(instance)
                .and_then(|skill| game.resolve_skill_sufferer(skill.lifecycle()))
                .and_then(|(region, identity)| {
                    game.resolve_state_move_shape(region, identity)
                        .map(|_| (region, identity))
                })
                .map_or(source, |resolved| resolved);
            game.check_cast_mana(instance, target, &properties)
        }
        _ => game.check_cast_mana(instance, source, &properties),
    }
}

/// Общий AI семьи: одна таблица переживает MP→OnChangeStates→CAN→направление
/// →visual0; unsigned `start+delay` без нового активного гейта в том же тик;
/// затем visual1 и ветка Summon владельца; любая попытка Summon завершается
/// общим Completed (делегаты переводят его в End(1)).
pub fn run_zonal_cast_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: &mut dyn FnMut(&mut Runtime) -> u32,
) -> ZonalCastAiOutcome
where
    Game: ZonalCastContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return ZonalCastAiOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return ZonalCastAiOutcome::Pending;
    };
    let id = skill.id();
    let Some(properties) = game.skill_base_properties(id, skill.level()).cloned() else {
        return ZonalCastAiOutcome::Rejected;
    };
    let source = zonal_cast_resolved_user(game, skill);
    let destination = if id == POISON_FOG_SKILL_ID {
        skill.lifecycle().destination()
    } else {
        let target = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity));
        if let Some(target) = target {
            let target_identity = (target.shape().get_region_id(), target.shape().identity());
            if game.move_shape_health(target_identity.0, target_identity.1) == Some(0) {
                game.update_registered_skill_visual(instance, 10);
                if id != SNOW_STORM_SKILL_ID
                    && let Some((_, user)) = source.filter(|(_, user)| user.object_type == PLAYER_TYPE)
                { game.send_skill_system_info(user.id, b"GS0285"); }
                return ZonalCastAiOutcome::Rejected;
            }
            let destination = (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            );
            if matches!(id, WEAK_SKILL_ID | CHAOS_SPHERE_SKILL_ID) {
                let point = if id == WEAK_SKILL_ID { (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                ) } else { destination };
                if let Some(skill) = game.registered_skill_mut(instance) {
                    skill.lifecycle_mut().set_point_target(point);
                }
            }
            destination
        } else { skill.lifecycle().destination() }
    };
    let Some(source) = source else { return ZonalCastAiOutcome::Rejected; };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    if stage == SkillStage::Begin {
        let paid = match id {
            POISON_FOG_SKILL_ID => game.prepare_crossbow_player(instance, player, &properties),
            SNOW_STORM_SKILL_ID => game.spend_cast_mana_without_text(instance, player, &properties),
            _ => game.spend_cast_mana(instance, player, &properties),
        };
        if !paid { return ZonalCastAiOutcome::Rejected; }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let destination = if id == POISON_FOG_SKILL_ID {
            let Some(skill) = game.registered_skill(instance) else { return ZonalCastAiOutcome::Rejected; };
            skill.lifecycle().destination()
        } else { destination };
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else {
            return ZonalCastAiOutcome::Rejected;
        };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return ZonalCastAiOutcome::Rejected;
    };
    if now_milliseconds(runtime) < started.wrapping_add(delay) {
        return ZonalCastAiOutcome::Pending;
    }
    if id == POISON_FOG_SKILL_ID
        && let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1)
    { user.set_moveable(true); }
    game.update_registered_skill_visual(instance, 1);
    match id {
        WEAK_SKILL_ID => super::weak::summon_weak(game, instance, source, destination, runtime, now_milliseconds),
        POISON_FOG_SKILL_ID => {
            if let Some(destination) = game.registered_skill(instance).map(|skill| skill.lifecycle().destination()) {
                super::poisonfog::summon_poison_fog(game, instance, source, destination, runtime, now_milliseconds);
            }
        }
        SNOW_STORM_SKILL_ID => super::snowstorm::summon_snow_storm(game, instance, source, destination, runtime, now_milliseconds),
        YIN_YANG_SKILL_ID | YIN_YANG_2_SKILL_ID => super::yinyang::summon_yin_yang(game, instance, source, destination, runtime, now_milliseconds),
        GOD_THUNDER_SKILL_ID | GOD_THUNDER_2_SKILL_ID => super::godthunder::summon_god_thunder(game, instance, id, source, destination, runtime, now_milliseconds),
        FIRE_WALL_SKILL_ID => game.summon_fire_wall(instance, source, destination, runtime),
        CHAOS_SPHERE_SKILL_ID => super::chaossphere::summon_chaos_sphere(game, instance, source, runtime, now_milliseconds),
        SOUL_MIRROR_SKILL_ID => game.apply_soul_mirror_area(instance, source, &properties, runtime),
        _ => {}
    }
    ZonalCastAiOutcome::Completed
}

/// Общий wire-каркас `0x000BFE01` семьи. Личная ветка отказов кадром
/// `[0, mode]` только игроку; mode 0 — action 1 с direction; mode 1 —
/// action 2 с нулевой парой цели и точкой: PoisonFog — сохранённая,
/// SoulMirror — текущий центр U, остальные — живой S с откатом к точке.
/// Базовый хвост visual исполняет внешний dispatcher, как и раньше.
pub fn publish_zonal_cast_visual<Game: ZonalCastGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if !is_zonal_cast_skill(skill.id()) || skill.visual_effect().is_none_or(|effect|
        effect.kind() != SkillVisualEffectKind::ZonalCast || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) || (skill.id() == POISON_FOG_SKILL_ID && mode == 14) {
        if source.identity().object_type == PLAYER_TYPE {
            let mut message = CMessage::new(ZONAL_CAST_VISUAL_MESSAGE);
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_zonal_cast_visual_to_player(source.identity().id, &message);
        }
        return;
    }
    let destination = match mode {
        0 => None,
        1 => Some(if skill.id() == SOUL_MIRROR_SKILL_ID {
            (
                source.get_tile_x().unwrap_or(i32::MIN),
                source.get_tile_y().unwrap_or(i32::MIN),
            )
        } else if skill.id() == POISON_FOG_SKILL_ID { skill.lifecycle().destination() } else {
            game.resolve_skill_sufferer(skill.lifecycle())
                .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
                .map_or_else(|| skill.lifecycle().destination(), |target| (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                ))
        }),
        _ => return,
    };
    let mut message = CMessage::new(ZONAL_CAST_VISUAL_MESSAGE);
    message.add_byte(if mode == 0 { 1 } else { 2 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some((x, y)) = destination {
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region() {
        game.send_zonal_cast_visual_around(source.get_region_id(), source, &message);
    }
}
