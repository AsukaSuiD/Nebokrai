//! Ядро диспетчера боевого расписания обычного монстра `CMonsterAI` и
//! приручённого `CPet`: типизированные решения OnSchedule/OnIdle/OnChangeSkill,
//! отдельный timestamp интервала атаки и hub-оркестрация преследования,
//! стояния, снятия цели и stiffen-перехода над владельцами старого пакета.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходные владельцы PDB:
//! `server/gameserver/appserver/ai/monsterai.cpp` (ядро расписания), соседние
//! вызовы `ai/baseai.cpp`, `ai/pet.cpp` и координатор
//! `appserver/skills/monsterbaseattack.cpp` остаются у своих порций.
//!
//! Машинная база кластера A1 (VERIFIED по этой паре, до переноса свидетельства
//! зафиксированы в старом файле `appserver/ai/monsterai.rs` и его соседях):
//!
//! - `CMonsterAI::OnSchedule` (`1:0x1dbf80` → RVA `0x1DCF80`): пустой базовый
//!   hook; owner `[+0x68]` обязателен; `HasTarget == 1`; очереди пусты
//!   `[+0x14] == 0 ∧ [+0x28] == 0` (INFERRED active/backstage); `can_fight` —
//!   `[owner+0x170] != 0` (INFERRED name), иначе только virtual `OnLoseTarget`;
//!   цель жива и `IsAttackAble` (vtable `+0x134`); `GetCurrentSkill()` null →
//!   `OnChangeSkill` → повторный null → `OnLoseTarget`; Tracing держит
//!   дистанцию в `[min..max]` навыка (vt `+0x74` `GetAffectRangeMax`,
//!   vt `+0x70` — RET1-заглушка минимума), иначе ветвь преследования
//!   (backoff — UNKNOWN); интервал: `timeGetTime (IAT) >= [this+0x78] +
//!   (word)GetAtcInterval`, затем `[+0x78] = now`, и только после этого
//!   `ok = skill->vt[+0x08](owner, target)`; ok → `AddAIEvent(2)`,
//!   отказ → `OnLoseTarget + AddAIEvent(5)`. Очередь `AI_EVENT`
//!   `{+0 action, +4 param, +8 время, +0xC состояние}`; `state == 1` ждёт
//!   `timeGetTime >= ev.time + prev.param`; `2 → OnFighting`,
//!   `5 → OnSearchEnemy` (у CPet собственный, у монстра пустой RET1),
//!   `1 → OnMoving` у CPet.
//! - `Run` (thunk `0x1DCBB0` → `CBaseAI::Run` RVA `0x0C7D10`): guard
//!   owner/hibernate (→ 2) → OnSchedule → ProcessBackStageAction → != 2 →
//!   ProcessPassiveAction → ProcessActiveAction → все 0 && !HasTarget → OnIdle
//!   → vt+0x44 hook → ProcessActiveActionWarSoul; возврат max-состояния.
//! - `OnChangeSkill` (RVA `0x1DCBC0`): `SelectAttackSkill` → уже выбранный
//!   объект + `CSkill::IsRestored` (vt `+0x80`: `QueryProperty(10005) +
//!   [+0x40] < now`; null-props → 1) → готово; иначе
//!   `SetCurrentSkill(GetDefaultAttackSkillID())`; возврат всегда 1.
//! - `SelectAttackSkill` (RVA `0x1DD0B0`): `dynamic_cast CMonster`, один
//!   `random(10000)`, обход `std::list` (`[CMonster+0x210]`; узел
//!   `word[+8] = id`, `word[+0xC] = odds`), первый с префикс-суммой ≥ r;
//!   иначе default. `Attack(id, shape)` (RVA `0x1DCEC0`) игнорирует ID —
//!   только `SetTarget`.
//! - Контракт результата Begin: отказ — это `OnLoseTarget + AddAIEvent(5)`
//!   (в Rust — `BeginRejected` → `release_owned_monster_target` +
//!   `begin_active_ai_search_enemy` в `finish_monster_skill_call`).
//! - `CBaseAI::MoveTo` (`baseai.cpp` RVA `0x0C8020`): один Slip для ходьбы,
//!   два для бега с исходным направлением; после Move timestamp берётся заново.
//!
//! Честные UNKNOWN этой порции (не достраиваются догадкой): сайт вызова `Run`
//! и каденсия AI глобальным циклом; точная форма backoff-шага ветви
//! преследования `MoveTo`; семантика поля `owner+0x170`, принятого здесь как
//! `can_fight` по INFERRED name; маскировка вызова vt `+0x12C` вокруг Tracing;
//! массивы default-ID навыков по категориям (только порядок
//! `GetDefaultAttackSkillID` 0x004CE240) и поле `tdI[2]`; type шаблона списка
//! навыков монстра (удерживается setup-срезом, а не выводом шаблона).
//!
//! Объявленные швы переноса (не расхождения):
//!
//! - Трейты ниже — переходные фасады прежних владельцев `CGame`, `CPlayer`,
//!   `CServerRegion`, `ServerRegionOwner`, `CMoveShape` и `CMonster`
//!   (state-машина AI, скилл-реестр через фабрику, region publish, RNG,
//!   пространственный рантайм). Реализации и делегации прежних сигнатур — в
//!   файлах-делегатах старого пакета `appserver/ai/monsterai.rs` и
//!   `appserver/skills/monsterbaseattack.rs`; потребители не меняются.
//!   Потребление статическое (generic), dyn-совместимость и `Send`-контракт не
//!   вводятся (ADR-0013).
//! - Порядковые предикаты по `ai_type` записаны числовыми наборами вместо
//!   матчинга по `MonsterAiKind` (классификатор `CAIFactory::CreateAI` остаётся
//!   владением `appserver/ai/aifactory.rs` до порции фабрики). Эквивалентность
//!   по его таблице: стационарное расписание = FixedPositionArcher(5),
//!   GuardWithBow(8), CityGuardWithBow(11), VillageCountyGuardWithBow(13),
//!   GuardCountry(17|100), GuardCountry2(101), GodsBattleGuardWithSword(103);
//!   свой интервал атаки убирают также SmartGladiator(2), JiuMai(20),
//!   BossBlue(21), BossFiend(23); generic = всякий тип вне именованного
//!   набора `{0..=21, 23, 24, 100, 101, 103, 104}` (22 и 102 включены, как у
//!   default-ветви `from_ai_type`).
//! - Активный AI приходит проекцией `MonsterActiveAiView`: различение
//!   Pet/Carriage/guard-station/JiuMai/PuninessCreature вычисляет hub-владелец
//!   по своей таблице virtual-семей (guard post — CGuardWithSword и наследники
//!   9/10/12/16).
//! - Часы каждого события читаются отдельным вызовом `now_milliseconds`
//!   (fn-параметр от делегата старого main loop, как в
//!   `skills/baseattackruntime.rs`); это точное значение blanket
//!   `GameClockContext::now_milliseconds = game_tick_milliseconds`.
//! - `resolve_owned_monster_attack_target` hub-владельца сжат в один шов
//!   `monster_attack_target_view`: перенесённому коду нужна только `ShapeView`
//!   недохожей цели (filter `!dead`), выбор разрешения сохранён у владельца
//!   (`monsterattack.rs`).
//! - `AI_EVENT`/`AiShapeAction` и passive-реакции `CBaseAI` уже зонские
//!   (`ai/events.rs`, `ai/reactions.rs`); сами FIFO-очереди монстра остаются
//!   hub-владением и переставляются только через фасады трейта.

use nebokrai_shared::resources::{GlobeSetupSnapshot, MonsterProperties, MonsterSkill};
use nebokrai_shared::runtime::get_line_direction;

use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::region::CRegion;
use crate::regions::shape::{
    CShape, ShapeAreaCoordinates, ShapeFigure, ShapeView, real_distance_between_points,
};
use crate::skills::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use crate::skills::skillfactory::CSkillFactory;
use crate::skills::{SkillExecutionKernel, SkillTermination, is_immediate_state_skill};
use crate::skills::littlestar::LITTLE_STAR_SKILL_ID;

use super::reactions::PassiveStiffenAction;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

/// Исход одного диспетчерского вызова навыка расписанием. `BeginRejected`
/// существует только у подключённых владельцев: BeginRejected →
/// `OnLoseTarget + AddAIEvent(5)`, как в native-хвосте OnSchedule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonsterSkillCallOutcome {
    NotHandled,
    Handled,
    BeginRejected,
}

/// Dispatch активного cast-а монстра: цель, ID и WORD-уровень записываются
/// при Begin расписанием и дальше читаются только через эту пару. Раньше тип
/// жил в hub-владельце `CMonster` (`appserver/monster.rs`); данные чистые,
/// поэтому дом — рядом с диспетчером, а alias
/// `MonsterBaseAttackCast = SkillExecutionKernel<MonsterBaseAttackDispatch>`
/// остаётся в старом пакете ради прежних сигнатур hub-методов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonsterBaseAttackDispatch {
    pub target: ShapeIdentity,
    pub skill_id: u32,
    pub skill_level: u16,
}

/// Отдельный timestamp расписания `CMonsterAI` (`[this+0x78]`); он не является
/// cooldown конкретного `CSkill` и обновляется до его `CheckCast`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MonsterAiScheduleState {
    last_attack_attempt_ms: u32,
}

impl MonsterAiScheduleState {
    pub fn begin_attack_attempt_with_clock(
        &mut self,
        interval_ms: u32,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        if self.last_attack_attempt_ms.wrapping_add(interval_ms) > now() {
            return false;
        }
        self.last_attack_attempt_ms = now();
        true
    }

    pub const fn begin_attack_attempt(&mut self, now_ms: u32, interval_ms: u32) -> bool {
        // Exact `m_dwTimeStamp + GetAttackSpeed() <= timeGetTime()` сохраняет
        // wrapped absolute deadline, а не устойчивый elapsed-интервал.
        if self.last_attack_attempt_ms.wrapping_add(interval_ms) > now_ms {
            return false;
        }
        self.last_attack_attempt_ms = now_ms;
        true
    }
}

/// Generic-ветви `CAIFactory::CreateAI` (`Monster`, в т.ч. типы 22 и 102+,
/// попадающие в default-раскладку таблицы). Числовая форма — объявленный шов,
/// см. шапку.
pub const fn is_generic_ai_type(ai_type: u32) -> bool {
    !matches!(ai_type, 0..=21 | 23 | 24 | 100 | 101 | 103 | 104)
}

/// Общий OnSchedule 0x0060B890 и его наследник CGBGuardWithSward:
/// AI 5/8/11/13/17/100/101/103.
pub const fn uses_stationary_attack_schedule(ai_type: u32) -> bool {
    matches!(ai_type, 5 | 8 | 11 | 13 | 17 | 100 | 101 | 103)
}

/// `CMonsterAI::OnSchedule` (RVA `0x1DCF80`) расширяет только word из
/// `GetAtcInterval`, затем складывает его с DWORD timestamp. Обычные
/// наследники сохраняют это усечение, включая значения setup больше 65535.
/// Стационарное OnSchedule 0x0060B890, SmartGladiator 0x006106E0, JiuMai
/// 0x0060AB10 и оба босса переходят от дальности/Tracing/CheckCast прямо к
/// Begin без дополнительного timestamp владельца.
pub const fn schedule_attack_interval(
    ai_type: u32,
    ordinary_interval_ms: u32,
) -> Option<u32> {
    if uses_stationary_attack_schedule(ai_type) || matches!(ai_type, 2 | 20 | 21 | 23) {
        None
    } else {
        Some(ordinary_interval_ms & 0xffff)
    }
}

/// Определяет достигнутые `OnIdle`, которые при отсутствии игроков переводят
/// владельца в sleeping-индекс области. Умный гладиатор сначала обязан
/// исчерпать сохранённые шаги отхода; приручение, цель, cast и фактическую
/// пустоту соседних областей проверяет непосредственный runtime caller.
pub const fn hibernates_without_nearby_players(
    ai_type: u32,
    smart_gladiator_ready_to_idle: bool,
) -> bool {
    is_generic_ai_type(ai_type)
        || matches!(
            ai_type,
            0 | 3
            | 4
            | 5
            | 6
            | 8
            | 9
            | 10
            | 11
            | 12
            | 13
            | 16
            | 17
            | 20
            | 21
            | 23
            | 100
            | 101
            | 103
        )
        || (ai_type == 2 && smart_gladiator_ready_to_idle)
}

pub const fn has_owned_search_enemy(ai_type: u32, pet_ai: bool) -> bool {
    pet_ai
        || is_generic_ai_type(ai_type)
        || matches!(
            ai_type,
            0..=21 | 23 | 100 | 101 | 103 | 104
        )
}

/// Сохраняет точный порядок и границу сравнения `SelectAttackSkill` (RVA
/// `0x1DD0B0`): один бросок на 10000, итоговая сумма `odds` по исходному
/// порядку списка, первый ID с накоплением не меньше броска.
/// `roll` получает вызывающая сторона из исходного генератора случайных чисел,
/// а стандартный навык вычисляет владелец формы по категориям установленных
/// навыков.
pub fn select_attack_skill(
    skills: &[MonsterSkill],
    roll: i32,
    default_skill_id: u16,
) -> u16 {
    let mut cumulative_odds = 0_i32;
    for skill in skills {
        cumulative_odds = cumulative_odds.wrapping_add(i32::from(skill.odds));
        if roll <= cumulative_odds {
            return skill.id;
        }
    }
    default_skill_id
}

/// Сохраняет целевую часть `CMonsterAI::WhenBeenHurted`: существующая цель
/// не заменяется, игрок допустим непосредственно, а монстр требует успешного
/// `DoesCreatureBeenTamed` у отдельного владельца атакующего.
pub const fn accepts_hurt_target(
    current_target: Option<ShapeIdentity>,
    attacker: ShapeIdentity,
    attacker_is_tamed: bool,
) -> bool {
    current_target.is_none()
        && (attacker.object_type == PLAYER_TYPE
            || (attacker.object_type == MONSTER_TYPE && attacker_is_tamed))
}

/// Живая форма сохраняет геометрию цели; точка нужна только для уже начатого
/// навыка, когда объект исчез, а подтверждённый прогресс ещё хранит назначение.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonsterTraceTarget {
    Shape(ShapeView),
    Point(ShapeAreaCoordinates),
}

impl MonsterTraceTarget {
    pub const fn point(x: i32, y: i32) -> Self {
        Self::Point(ShapeAreaCoordinates { x, y })
    }

    const fn coordinates(self) -> ShapeAreaCoordinates {
        match self {
            Self::Shape(view) => ShapeAreaCoordinates {
                x: view.tile_x,
                y: view.tile_y,
            },
            Self::Point(point) => point,
        }
    }
}

/// Проекция активного AI владельца, достаточная диспетчеру: раскладку
/// virtual-семей вычисляет hub по своей таблице (`CAIFactory`), Zone снова её
/// не угадывает. `Carriage` покрывает обе формы повозки (auxiliary и primary
/// AI24), `OtherPrimary` — всех прочих первичных владельцев, включая обычный
/// `CMonsterAI` default-ветви фабрики.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonsterActiveAiView {
    Pet,
    Carriage,
    GuardStation,
    JiuMai,
    PuninessCreature,
    OtherPrimary,
}

/// Подвижная форма монстра: переходный фасад прежнего `CMoveShape`
/// (current_skill через фабрику, доступ к клеткам и запрету движения).
pub trait MonsterDispatcherMoveShape {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type Execution: MonsterSkillExecutionAccess;

    fn is_moveable(&self) -> bool;

    fn shape(&self) -> &CShape;

    /// Проекция `GetCurrentSkill` в реестр; execution и End остаются у owner.
    fn current_skill(
        &self,
        factory: &CSkillFactory,
    ) -> Option<&RegisteredSkillRecord<Self::Execution>>;

    /// `GetDefaultAttackSkillID` (0x004CE240): порядок категорий важнее ID.
    fn default_attack_skill_id(&self) -> u32;

    /// Typed boundary записи выбранного ID; полное действие `SetCurrentSkill`
    /// не подменяется и остаётся у соответствующего owner-а.
    fn set_current_skill_id(&mut self, skill_id: Option<u32>);
}

/// Монстр региона с его AI state-машиной: переходный фасад прежнего
/// `CMonster`. Имена сохраняют исходные операции; очереди FIFO и их данные
/// остаются hub-владением.
pub trait MonsterDispatcherMonster {
    type MoveShape: MonsterDispatcherMoveShape;

    fn move_shape(&self) -> &Self::MoveShape;
    fn move_shape_mut(&mut self) -> &mut Self::MoveShape;

    fn base_property_key(&self) -> Option<&[u8]>;

    /// `CMonster::figure(properties)`: footprint из setup-строки монстра.
    fn figure_for(properties: &MonsterProperties) -> ShapeFigure;

    fn shape_view(
        &self,
        properties: &MonsterProperties,
    ) -> Option<ShapeView>;

    fn stop_frame(&self, properties: &MonsterProperties) -> u32;

    fn speed(&self) -> f32;

    fn hit_points(&self) -> u32;

    fn is_tamed(&self) -> bool;

    fn master_info(&self) -> MasterInfo;

    fn pet_mode(&self) -> i32;

    fn pet_action(&self) -> i32;

    fn active_primary_ai_type(&self) -> Option<u32>;

    /// Проекция активного AI (`CMonster::GetAI` выбирает owner-а, а не знак
    /// приручения); раскладка семей остаётся у hub-реализации.
    fn active_ai_view(&self) -> Option<MonsterActiveAiView>;

    fn primary_ai_queues_idle(&self) -> bool;

    fn ai_target(&self) -> Option<ShapeIdentity>;

    fn set_ai_target(&mut self, target: ShapeIdentity);

    fn clear_ai_target(&mut self, factory: &CSkillFactory);

    fn lose_ai_target_and_search(&mut self, now_ms: u32, factory: &CSkillFactory);

    fn release_ai_target_for_death(&mut self);

    fn release_pet_ai_target(&mut self);

    fn begin_active_ai_move(&mut self, delay_ms: u32, now_ms: u32);

    fn begin_active_ai_stand(&mut self, delay_ms: u32, now_ms: u32);

    fn begin_active_ai_search_enemy(&mut self, now_ms: u32);

    fn begin_active_ai_change_skill(&mut self, now_ms: u32);

    /// Проекция исполнения для `OnFighting`, не общий доступ к ресурсу CSkill.
    fn current_active_attack_cast(
        &self,
        factory: &CSkillFactory,
    ) -> Option<SkillExecutionKernel<MonsterBaseAttackDispatch>>;

    fn skill_last_used_ms(&self, skill_id: u32, factory: &CSkillFactory) -> u32;

    // Stiffen-переход `ProcessPassiveAction`: материализация, прерывание
    // атаки и завершение по deadline (порядковая механика — `ai/reactions.rs`).
    fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction;

    fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction;

    /// Ответ concrete owner-а на прерванный Attack: `None`, если AI-owner
    /// отсутствует (исходный `selected_base_ai()?`).
    fn stiffen_attack_needs_end(&self) -> Option<bool>;

    fn prepare_stiffen_attack(
        &mut self,
        factory: &CSkillFactory,
    ) -> Option<(bool, Option<u32>)>;

    fn finish_stiffen_attack(
        &mut self,
        ended_skill: Option<u32>,
        factory: &CSkillFactory,
        now: impl FnOnce() -> u32,
    );

    fn resume_stiffen_after_target_release(&mut self, released: bool);
}

/// Регион владельца: переходный фасад прежнего `CServerRegion`. Членство,
/// ячейки и идентификатор региона остаются hub-владением.
pub trait MonsterDispatcherRegion {
    type Monster: MonsterDispatcherMonster;

    fn find_monster_by_id(&self, id: i32) -> Option<&Self::Monster>;

    fn find_monster_by_id_mut(&mut self, id: i32) -> Option<&mut Self::Monster>;

    fn monster_ids_around_area(&self, area_index: usize) -> Vec<i32>;

    fn straight_skill_path(
        &self,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
        forced_length: Option<u32>,
    ) -> Vec<(i32, i32, u8)>;

    fn base_region(&self) -> &CRegion;

    fn region_id(&self) -> i32;
}

/// Извлекаемый владелец региона старого хоста: переходный фасад прежнего
/// `ServerRegionOwner`. Base-проекция сохраняет исходное членство шести форм.
pub trait MonsterDispatcherOwner {
    type Region: MonsterDispatcherRegion;

    fn base(&self) -> &Self::Region;

    fn base_mut(&mut self) -> &mut Self::Region;

    fn region_id(&self) -> i32;
}

/// Игрок-хозяин питомца: переходный фасад прежнего `CPlayer`. Нужен только
/// боевому расписанию `CPet` (anchor преследования и слот следования).
pub trait MonsterDispatcherPlayer {
    fn shape(&self) -> &CShape;

    fn shape_view(&self) -> Option<ShapeView>;

    fn server_region_id(&self) -> Option<i32>;

    fn is_badman(&self, pk_count_per_kill: u32) -> bool;

    fn active_pets(&self) -> &[crate::regions::moveshape::MoveShapePet];
}

/// Переходные фасады прежнего владельца `CGame`: таблицы свойств, RNG, часы
/// не приходят сюда (fn-параметр у функций), пространственный slip/runtime,
/// публикация региона и узкие маршруты к ещё не перенесённым соседним
/// владельцам AI. Имена членов сохраняют исходную операцию.
pub trait MonsterDispatcherGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для диспетчера.
    type SkillAddress: Copy;

    type Player: MonsterDispatcherPlayer;

    /// Извлекаемый владелец региона старого хоста.
    type RegionOwner: MonsterDispatcherOwner;

    fn skill_factory(&self) -> &CSkillFactory;

    fn find_monster_property_by_origin_name(
        &self,
        origin_name: &[u8],
    ) -> Option<&MonsterProperties>;

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    /// Общий legacy RNG навыков (`skill_random_below` прежнего `CGame`).
    fn skill_random_below(&mut self, maximum: i32) -> i32;

    fn globe_setup(&self) -> &GlobeSetupSnapshot;

    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    /// `GetSufferer`-проекция живой формы внутри извлечённого владельца.
    fn shape_view_in_owner(
        &self,
        owner: &Self::RegionOwner,
        identity: ShapeIdentity,
    ) -> Option<ShapeView>;

    /// Шов разрешения боевой цели: hub-владелец раскрывает живую форму цели;
    /// сюда возвращается только view недохожей цели (см. шапку про свёртку).
    fn monster_attack_target_view(
        &self,
        owner: &Self::RegionOwner,
        identity: ShapeIdentity,
    ) -> Option<ShapeView>;

    // Пространственный runtime `CBaseAI::MoveTo` (slip-шаги и задержка хода;
    // сами FindFreeCell-порядки остаются у прежнего `baseai` до его порции).
    fn find_slip_step_in_direction(
        &self,
        region: &<Self::RegionOwner as MonsterDispatcherOwner>::Region,
        origin: ShapeAreaCoordinates,
        desired_direction: i32,
        figure_index: usize,
    ) -> Option<(i32, ShapeAreaCoordinates)>;

    fn one_step_move_delay_ms(direction: i32, speed: f32, stop_frame: u32) -> u32;

    /// Применяет один шаг движения монстра с сетевой доставкой вокруг.
    fn move_owned_monster_step_with_run(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        x: i32,
        y: i32,
        figure: ShapeFigure,
        run: i32,
    ) -> bool;

    /// Готовность spatial runtime к мгновенному переносу питомца.
    fn spatial_delivery_ready(&self) -> bool;

    fn set_owned_pet_position(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        x: i32,
        y: i32,
        figure: ShapeFigure,
    ) -> bool;

    // Узкие маршруты к ещё hub-владельцам производных AI (их перенос — свои
    // порции; здесь только прежний контракт вызова).
    /// derived `OnLoseTarget` близнеца Цзюмай; признак отказа отбрасывается
    /// вызывающим диспетчером, как и раньше.
    fn release_jiumai_target(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
    ) -> bool;

    /// ID-классификатор семьи NonFun (полоса `nonfun.rs`): прерывание
    /// stiffen-ом завершает их зарегистрированный экземпляр, как именованных.
    fn is_non_fun_skill_id(skill_id: u32) -> bool;

    /// Кадр завершения CLittleStar (action 3) вокруг источника.
    fn send_little_star_end(
        &self,
        region: &<Self::RegionOwner as MonsterDispatcherOwner>::Region,
        source: &CShape,
        skill_level: u16,
    );

    // Реестр и публикация региона для stiffen-перехода.
    fn with_published_region<Output>(
        &mut self,
        owner: &mut Option<Self::RegionOwner>,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Option<Output>;

    fn registered_move_shape_skill(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        skill_id: u32,
    ) -> Option<Self::SkillAddress>;

    fn registered_skill(
        &self,
        address: Self::SkillAddress,
    ) -> Option<&RegisteredSkillRecord<Self::MonsterExecution>>;

    /// Общий зарегистрированный `CSkill::End` текущего пакета; признак
    /// завершения у caller-ов отбрасывается, как и раньше.
    fn end_registered_instance<Runtime>(
        &mut self,
        address: Self::SkillAddress,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
    );
}

/// Маршруты диспетчера, которым нужен сам runtime игрового хода (ведение
/// соседних AI-владельцев с их прежними generic-границами). Отделён, потому
/// что тип хода принадлежит старому main loop, а не самому расписанию
/// (прецедент `skills/baseattackruntime.rs`, `skills/dash.rs`).
pub trait MonsterDispatcherRuntime<Runtime>: MonsterDispatcherGame {
    /// derived `OnLoseTarget` мечевых охранников (guard station family).
    fn release_guard_sword_target(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        runtime: &mut Runtime,
    );

    /// Собственное расписание слабого существа вместо боевого Begin.
    fn execute_puniness_creature(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        runtime: &mut Runtime,
    ) -> bool;

    /// Унаследованный мечевыми охранниками virtual Tracing; готовность —
    /// `CitySwordTraceOutcome::Ready` прежнего владельца.
    fn trace_guard_sword_target_ready(
        &mut self,
        region: &mut <Self::RegionOwner as MonsterDispatcherOwner>::Region,
        monster_id: i32,
        owner: ShapeView,
        target: ShapeView,
        minimum_distance: i32,
        maximum_distance: i32,
        chase_range: i32,
        runtime: &mut Runtime,
    ) -> bool;
}

/// Общий приёмник результата диспетчерского вызова: отказ Begin — отдельный
/// результат owner-а и не ожидание, и не End AI. Расписание после него
/// вызывает virtual OnLoseTarget, затем ставит SearchEnemy с новым timestamp.
pub fn finish_monster_skill_call<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    outcome: MonsterSkillCallOutcome,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherRuntime<Runtime>,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    match outcome {
        MonsterSkillCallOutcome::NotHandled => false,
        MonsterSkillCallOutcome::Handled => true,
        MonsterSkillCallOutcome::BeginRejected => {
            release_owned_monster_target(game, region, monster_id, runtime);
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.begin_active_ai_search_enemy(now_milliseconds());
            }
            true
        }
    }
}

/// Подтверждённая политика End(4) находится у CMonster: она сохраняет выбранный
/// навык до callback, обновляет reuse и выполняет конкретную очистку, не
/// снимая наложенные состояния. Прицельные Strike/Yaksha завершаются через
/// точный ключ зарегистрированного навыка при опубликованном регионе. End(4) и
/// его callbacks видят живую очередь; только IsEnded того же экземпляра
/// разрешает снять Attack. Исчезнувший экземпляр не заменяется новым
/// одноимённым, reuse не повторяется.
pub fn process_owned_monster_stiffen<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    monster_id: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> PassiveStiffenAction
where
    Game: MonsterDispatcherRuntime<Runtime>,
{
    let Some(monster) = owner.as_mut()
        .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
    else {
        return PassiveStiffenAction::None;
    };
    let action = monster.begin_reached_stiffen_action();
    if action.interrupts_attack() {
        loop {
            let registered = owner.as_ref().and_then(|region| {
                let monster = region.base().find_monster_by_id(monster_id)?;
                if !monster.stiffen_attack_needs_end()? { return None; }
                let skill = monster.move_shape().current_skill(game.skill_factory())?;
                (matches!(skill.owner(),
                    crate::skills::skillfactory::SkillOwner::CStrike
                    | crate::skills::skillfactory::SkillOwner::CYakshaSlash
                    | crate::skills::skillfactory::SkillOwner::CSeal
                    | crate::skills::skillfactory::SkillOwner::CHeal
                    | crate::skills::skillfactory::SkillOwner::CHeal2
                    | crate::skills::skillfactory::SkillOwner::CSuperHeal
                    | crate::skills::skillfactory::SkillOwner::CSuperHeal2
                    | crate::skills::skillfactory::SkillOwner::CGodBless
                    | crate::skills::skillfactory::SkillOwner::CGodBless2
                    | crate::skills::skillfactory::SkillOwner::CSoulCollect
                    | crate::skills::skillfactory::SkillOwner::CEnergyBolt
                    | crate::skills::skillfactory::SkillOwner::CSnakeBolt
                    | crate::skills::skillfactory::SkillOwner::CZombieClaw
                    | crate::skills::skillfactory::SkillOwner::CChuckStone
                    | crate::skills::skillfactory::SkillOwner::CSkeletonArchery
                    | crate::skills::skillfactory::SkillOwner::CWeak
                    | crate::skills::skillfactory::SkillOwner::CPoisonFog
                    | crate::skills::skillfactory::SkillOwner::CSnowStorm
                    | crate::skills::skillfactory::SkillOwner::CYinYang
                    | crate::skills::skillfactory::SkillOwner::CYinYang2
                    | crate::skills::skillfactory::SkillOwner::CGodThunder
                    | crate::skills::skillfactory::SkillOwner::CGodThunder2
                    | crate::skills::skillfactory::SkillOwner::CFireWall
                    | crate::skills::skillfactory::SkillOwner::CChaosSphere
                    | crate::skills::skillfactory::SkillOwner::CSoulMirror
                    | crate::skills::skillfactory::SkillOwner::CLightning
                    | crate::skills::skillfactory::SkillOwner::CChainLightning
                    | crate::skills::skillfactory::SkillOwner::CInfernol)
                    || is_immediate_state_skill(skill.id())
                    || Game::is_non_fun_skill_id(skill.id()))
                    .then_some((region.region_id(), monster.move_shape().shape().identity(), skill.id()))
            });
            let (release_target, ended_skill) = if let Some((region, source, skill_id)) = registered {
                let ended = game.with_published_region(owner, |game| {
                    let Some(instance) = game.registered_move_shape_skill(region, source, skill_id) else { return false; };
                    game.end_registered_instance(instance, 4, SkillTermination::Cancelled, runtime);
                    game.registered_skill(instance).is_none_or(|skill| skill.lifecycle().is_ended())
                });
                if ended != Some(true) { break; }
                (true, None)
            } else {
                let Some(prepared) = owner.as_mut()
                    .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
                    .and_then(|monster| monster.prepare_stiffen_attack(game.skill_factory()))
                else { break; };
                prepared
            };
            let Some(region) = owner.as_mut().map(MonsterDispatcherOwner::base_mut) else { break; };
            if ended_skill == Some(LITTLE_STAR_SKILL_ID)
                && let Some(monster) = region.find_monster_by_id(monster_id)
                && let Some(skill) = monster.move_shape().current_skill(game.skill_factory())
            {
                game.send_little_star_end(
                    region, monster.move_shape().shape(), skill.level() as u16,
                );
            }
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.finish_stiffen_attack(ended_skill, game.skill_factory(), now_milliseconds);
            }
            if release_target {
                release_owned_monster_target(game, region, monster_id, runtime);
            }
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.resume_stiffen_after_target_release(release_target);
            }
        }
    }
    owner.as_mut().and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        .map_or(PassiveStiffenAction::None, |monster| {
        monster.finish_reached_stiffen_action(action, now_milliseconds)
    })
}

/// Обычный virtual OnLoseTarget (0x005DCC30 → 0x004C7DA0) очищает только цель,
/// не вызывает End и не снимает Move. Разрешение идёт через `CMonster::GetAI`,
/// а не повторный поиск MonsterProperties: CPet, мечевые охранники и JiuMai
/// сохраняют свои побочные эффекты поверх базовой очистки CMonsterAI.
pub fn release_owned_monster_target<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    runtime: &mut Runtime,
)
where
    Game: MonsterDispatcherRuntime<Runtime>,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some(ai) = region.find_monster_by_id(monster_id).and_then(|monster| monster.active_ai_view()) else {
        return;
    };
    match ai {
        MonsterActiveAiView::Pet => super::pet::release_pet_target(region, monster_id),
        MonsterActiveAiView::GuardStation => {
            game.release_guard_sword_target(region, monster_id, runtime)
        }
        MonsterActiveAiView::JiuMai => {
            let _ = game.release_jiumai_target(region, monster_id);
        }
        _ => {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.release_ai_target_for_death();
            }
        }
    }
}

/// MoveTo (RVA `0x0C8020`): один Slip для ходьбы, два для ненулевого run,
/// затем Move и FIFO. Второй Slip сохраняет исходное желаемое направление;
/// его отказ не публикует даже первый шаг. Задержка зависит от направления
/// между исходной и окончательной клетками, без удвоения при беге.
/// Отказ вызывает пустой CMoveShape::OnCannotMove (+0xA4) и не меняет очередь.
pub fn move_owned_monster_to<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    target: ShapeAreaCoordinates,
    run: i32,
    now: impl FnOnce() -> u32,
) where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((origin, figure, speed, stop_frame)) = region.find_monster_by_id(monster_id)
        .and_then(|monster| {
            if !monster.move_shape().is_moveable() { return None; }
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let shape = monster.move_shape().shape();
            Some((ShapeAreaCoordinates { x: shape.get_tile_x().ok()?, y: shape.get_tile_y().ok()? },
                <Region::Monster as MonsterDispatcherMonster>::figure_for(property),
                monster.speed(),
                monster.stop_frame(property)))
        })
    else { return; };
    let desired_direction = get_line_direction(origin.x, origin.y, target.x, target.y);
    let figure_index = usize::from(figure.get(0).min(2));
    let Some((_, mut destination)) = game.find_slip_step_in_direction(
        region, origin, desired_direction, figure_index,
    )
    else { return; };
    if run != 0 {
        let Some((_, second)) = game.find_slip_step_in_direction(
            region, destination, desired_direction, figure_index,
        ) else { return; };
        destination = second;
    }
    let direction = get_line_direction(origin.x, origin.y, destination.x, destination.y);
    let delay = Game::one_step_move_delay_ms(direction, speed, stop_frame);
    let _ = game.move_owned_monster_step_with_run(
        region, monster_id, destination.x, destination.y, figure, run,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_move(delay, now());
    }
}

/// Ставит достигнутый общий `CMonsterAI::OnIdle`: при необходимости отдельный
/// `ChangeSkill`, затем ровно один выбор `Move/Stand` и завершающий
/// `SearchEnemy`. Каждый `AddAIEvent` получает собственный замер часов.
pub fn queue_monster_idle<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((origin, stop_frame, has_skill)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let shape = monster.move_shape().shape();
            Some((
                ShapeAreaCoordinates {
                    x: shape.get_tile_x().ok()?,
                    y: shape.get_tile_y().ok()?,
                },
                monster.stop_frame(property),
                monster.move_shape().current_skill(game.skill_factory()).is_some(),
            ))
        })
    else {
        return false;
    };
    if !has_skill
        && let Some(monster) = region.find_monster_by_id_mut(monster_id)
    {
        monster.begin_active_ai_change_skill(now_milliseconds());
    }
    if (game.skill_random_below(10_000) as u32) < property.move_timer {
        let direction = game.skill_random_below(8);
        if let Ok(destination) = CShape::get_direction_position(direction, origin) {
            move_owned_monster_to(game, region, monster_id, destination, 0, now_milliseconds);
        }
    } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_stand(stop_frame, now_milliseconds());
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_search_enemy(now_milliseconds());
    }
    true
}

/// Общий адаптер подхода оставшихся владельцев навыков. Его проверка прямого
/// пути и отсутствие минимальной дистанции не подменяют точный Tracing ниже.
pub fn approach_attack_range<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    target: MonsterTraceTarget,
    maximum_distance: u32,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((
        property,
        monster_view,
        monster_x,
        monster_y,
        pet_ai,
        pet_action,
        moveable,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let monster_view = monster.shape_view(&property)?;
        Some((
            property.clone(),
            monster_view,
            monster.move_shape().shape().get_tile_x().ok()?,
            monster.move_shape().shape().get_tile_y().ok()?,
            matches!(monster.active_ai_view()?, MonsterActiveAiView::Pet),
            monster.pet_action(),
            monster.move_shape().is_moveable(),
        ))
    }) else {
        return false;
    };

    if (pet_ai && pet_action == 2) || (!pet_ai && uses_stationary_attack_schedule(property.ai)) {
        return true;
    }
    let target_coordinates = target.coordinates();
    let (target_x, target_y) = (target_coordinates.x, target_coordinates.y);
    let distance = match target {
        MonsterTraceTarget::Shape(target) => monster_view.real_distance(Some(target)),
        MonsterTraceTarget::Point(_) => real_distance_between_points(monster_x, monster_y, target_x, target_y),
    };
    let path_blocked = region
        .straight_skill_path(monster_x, monster_y, target_x, target_y, None)
        .iter()
        .any(|cell| cell.2 == 2);
    if (maximum_distance == 0 || distance <= maximum_distance as i32) && !path_blocked {
        return true;
    }
    let chase_range = if pet_ai {
        game.globe_setup().maximum_pet_tracing_distance()
    } else {
        property.chase_range
    };
    if distance > chase_range as i32 {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if has_owned_search_enemy(property.ai, pet_ai) {
                monster.lose_ai_target_and_search(now_milliseconds(), game.skill_factory());
            } else {
                monster.clear_ai_target(game.skill_factory());
            }
        }
        return false;
    }
    if !moveable {
        return false;
    }

    move_owned_monster_to(game, region, monster_id,
        ShapeAreaCoordinates { x: target_x, y: target_y }, 0, now_milliseconds);
    false
}

/// Virtual Tracing перед новым Begin с диапазоном зарегистрированного навыка.
/// Стоящий питомец и стационарный OnSchedule проверяют свой диапазон снаружи
/// без вызова Tracing.
pub fn trace_owned_target_state_skill<Game, Runtime>(
    game: &mut Game,
    owner: &mut Game::RegionOwner,
    monster_id: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherRuntime<Runtime>,
{
    let Some(monster) = owner.base().find_monster_by_id(monster_id) else { return false; };
    let Some(active_ai) = monster.active_ai_view() else { return false; };
    if matches!(active_ai, MonsterActiveAiView::PuninessCreature) {
        let _ = game.execute_puniness_creature(
            owner.base_mut(), monster_id, runtime,
        );
        return false;
    }
    // OnSchedule повозки не вызывает Begin атакующего навыка.
    if matches!(active_ai, MonsterActiveAiView::Carriage) { return false; }
    let Some(skill) = monster.move_shape().current_skill(game.skill_factory()) else {
        release_owned_monster_target(game, owner.base_mut(), monster_id, runtime);
        return false;
    };
    let (skill_id, skill_level) = (skill.id(), skill.level());
    let minimum = skill.minimum_range(game.skill_factory()) as i32;
    let target = monster.ai_target()
        .and_then(|identity| game.monster_attack_target_view(owner, identity));
    let Some(target) = target else {
        release_owned_monster_target(game, owner.base_mut(), monster_id, runtime);
        if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.begin_active_ai_search_enemy(now_milliseconds());
        }
        return false;
    };
    let Some(source) = game.shape_view_in_owner(owner, monster.move_shape().shape().identity()) else {
        return false;
    };
    let maximum = |game: &Game| {
        game.skill_base_properties(skill_id, skill_level)
            .map(|properties| properties.query_property(5_003) as i32)
            .filter(|value| *value > 0)
            .unwrap_or(1)
    };
    let distance = source.real_distance(Some(target));
    if distance >= minimum && distance <= maximum(game) { return true; }
    let chase_range = if monster.is_tamed()
        && monster.master_info().master_type == 400 && monster.master_info().master_id != 0
    {
        game.globe_setup().maximum_pet_tracing_distance() as i32
    } else {
        let Some(property) = monster.base_property_key()
            .and_then(|key| game.find_monster_property_by_origin_name(key))
        else { return false; };
        property.chase_range as i32
    };
    if matches!(active_ai, MonsterActiveAiView::GuardStation) {
        let maximum_distance = maximum(game);
        return game.trace_guard_sword_target_ready(
            owner.base_mut(), monster_id, source, target, minimum, maximum_distance, chase_range, runtime,
        );
    }
    if distance > chase_range {
        release_owned_monster_target(game, owner.base_mut(), monster_id, runtime);
        if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.begin_active_ai_search_enemy(now_milliseconds());
        }
        return false;
    }
    let destination = if distance <= maximum(game) {
        let direction = get_line_direction(target.tile_x, target.tile_y, source.tile_x, source.tile_y);
        let Ok(point) = CShape::get_direction_position(
            direction, ShapeAreaCoordinates { x: source.tile_x, y: source.tile_y },
        ) else { return false; };
        point
    } else {
        ShapeAreaCoordinates { x: target.tile_x, y: target.tile_y }
    };
    if !owner.base().find_monster_by_id(monster_id)
        .is_some_and(|monster| monster.move_shape().is_moveable())
    { return false; }
    move_owned_monster_to(game, owner.base_mut(), monster_id, destination, 0, now_milliseconds);
    false
}
