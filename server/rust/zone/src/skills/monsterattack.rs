//! Общая доставка удара боевых навыков монстров и допуск целей: живой
//! клеточный resolver 400/500/600/1100/1200, снимок цели для конкретного
//! удара и применение попадания через опубликованный производный регион.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходные владельцы PDB:
//! `appserver/skills/monsterattack.cpp` (общий контакт) и
//! `appserver/monster.cpp` (`CMonster::IsAttackAble`). Прежний переходный
//! владелец — `src/gameserver/appserver/skills/monsterattack.rs`; тела
//! перенесены буквально поверх hub-трейтов ниже (кластер A2 полосы Monster,
//! карта — запись аудита «Zone skills: карта полосы Monster 0x19x — 5
//! кластеров волн», 26 сентября 2026).
//!
//! Статусы по этой паре:
//!
//! - `CMonster::IsAttackAble` (RVA `0x0E7230`, VERIFIED): первый класс-гейт
//!   типа цели — ветвь `[edi+4] == 0x190` (player) и `0x258` (monster);
//!   остальные типы отклоняются до PK/tame-ветвей.
//! - `end_owned_monster_skill_without_reuse` — отображение на owner
//!   `End(0)` живого cast без reuse-штампа (VERIFIED косвенно: owner
//!   `CMonsterThorn::AI` 0x142180 выполняет End(0) для мёртвой/длинной цели,
//!   `CMonsterBaseAttack::End` 0x1B3010 не восстанавливает движение и не
//!   перечитывает свойства).
//! - `monster_attack_cell_candidates`: список допустимых типов
//!   400/500/600/1100/1200 и исключение источника — PARTIAL (свидетельство
//!   прежнего владельца импортировано; resolver полного региона и фильтр
//!   исчезнувших форм — шов старого хоста).
//! - Во время попадания настоящий производный регион публикуется целиком:
//!   общий получатель видит живые защиты, HP/MP, источник и выбранный AI
//!   цели; возврат из callback требует заново получить регион и объекты, а
//!   не применять сохранённые до защиты снимки (контракт прежнего владельца,
//!   сохранён).
//! - У NPC нулевой combat HP: наследуемый `CMoveShape::IsDied` (GetHP == 0)
//!   истинен независимо от action — снимок цели держит этот факт здесь, а не
//!   в навыках (контракт прежнего владельца, не изменён переносом).
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — переходные
//! фасады прежнего владельца `CGame`/`CPlayer`/`CServerRegion`/
//! `ServerRegionOwner`; реализация остаётся у делегата старого пакета
//! (`appserver/skills/monsterattack.rs`). Потребление статическое
//! (generic), dyn-совместимость и `Send`-контракт не вводятся (ADR-0013).
//! `QuerySkillBaseProperties` (skillfactory), `GetShapes`/региональный
//! resolver, чтение живой цели (player/monster/npc/build) и around-доставка —
//! швы трейта. Применение попадания использует общий `BaseAttackContact`
//! контактной стадии. Часы приходят fn-параметром `now_milliseconds` от
//! делегата старого main loop (точное значение blanket
//! `GameClockContext::now_milliseconds = game_tick_milliseconds`).

use nebokrai_shared::resources::{GlobeSetupSnapshot, MonsterProperties};

use crate::app::game_message::CMessage;
use crate::combat::{AttackInformation, MasterInfo, PlayerCombatProperties};
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::{CShape, ShapeView};
use crate::skills::execution::PlayerMonsterThornExecutionState;

use super::baseattackruntime::BaseAttackContact;
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillExecutionKernel, SkillStage, SkillTermination};

/// Стадии результата одного тика исполнения боевого навыка семьи; обёртка
/// очереди с полем `first_contact` остаётся у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonsterCombatOutcome {
    Begun,
    Pending,
    Completed,
    Rejected,
    /// Отказ после пройденного срока удара: owner выполняет живой `End(1)`.
    RejectedAfterUse,
}

/// Живой снимок цели конкретного удара монстра: форма и вид для геометрии,
/// боевой контекст игрока (когда цель player) и смерть/god-флаги вычислены
/// до контакта; решение IsAttackAble сам снимок не принимает.
#[derive(Clone, Debug)]
pub struct OwnedMonsterAttackTarget {
    pub shape: CShape,
    pub view: ShapeView,
    pub player_properties: Option<PlayerCombatProperties>,
    pub dead: bool,
    pub god: bool,
    pub city_dead: bool,
    pub monster_property: Option<MonsterProperties>,
}

/// Снимок живого монстра на входе исполнения удара: форма, setup-строка,
/// attack interval расписания, живой cast (цель/id/уровень/начало) и
/// reuse-момент; резолв строки и приручённого интервала остаётся в шве.
#[derive(Clone, Debug)]
pub struct MonsterCombatFacts {
    pub source: CShape,
    pub property: MonsterProperties,
    pub attack_interval_ms: u32,
    pub ai_kind: u32,
    pub cast: Option<MonsterCombatCast>,
    pub last_used_ms: u32,
}

/// Живой cast удара монстра на момент чтения (dispatch записан новым Begin
/// расписанием и дальше читается только через эту пару).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonsterCombatCast {
    pub target: ShapeIdentity,
    pub skill_id: u32,
    pub skill_level: u16,
    pub started_at_ms: u32,
    pub stage: SkillStage,
}

/// Снимок монстра-цели удара: форма, setup-строка, живой вид и HP/god.
#[derive(Clone, Debug)]
pub struct MonsterShapeFacts {
    pub shape: CShape,
    pub property: MonsterProperties,
    pub view: ShapeView,
    pub hit_points: u32,
    pub god: bool,
}

/// Снимок цели приручения: setup-строка, живая позиция/здоровье и допуск
/// `CMonster::IsTamable` по этой строке (порядок чтения прежнего владельца).
#[derive(Clone, Debug)]
pub struct MonsterTamingTarget {
    pub property: MonsterProperties,
    pub tile_x: i32,
    pub tile_y: i32,
    pub display_name: Vec<u8>,
    pub hit_points: u32,
    pub tamable: bool,
}

/// Игрок боевого входа семьи: переходный фасад прежнего `CPlayer`.
pub trait MonsterCombatPlayer {
    /// Форма игрока (identity, клетки, direction).
    fn shape(&self) -> &CShape;

    fn shape_view(&self) -> Option<ShapeView>;

    /// Поворот источника к цели выпуска (AI-стадия навыка).
    fn face_cast_direction(&mut self, direction: i32);

    /// Боевой снимок для формул урона и допусков.
    fn combat_properties(&self) -> PlayerCombatProperties;

    fn server_region_id(&self) -> Option<i32>;

    fn is_dead(&self) -> bool;

    fn is_god_mode(&self) -> bool;

    fn city_war_died_state(&self) -> bool;

    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);

    fn set_skill_moveable(&mut self, moveable: bool);

    fn set_current_skill_id(&mut self, skill_id: Option<u32>);

    /// Полный MasterInfo источника (PK-допуски относятся к player-ветви).
    fn master_info(&self) -> MasterInfo;

    // Поля приручения.
    fn level(&self) -> u8;

    fn player_name(&self) -> &[u8];

    fn current_pets_mode(&self) -> i32;

    fn active_pets_count(&self) -> u32;

    /// `CMoveShape::AddPet`: push-back живой записи питомца с прочитанной
    /// владельцем фигурой (call-site передаёт байт figure).
    fn add_active_pet(&mut self, object_type: i32, id: i32, figure: i32);
}

/// Переходные фасады прежнего владельца `CGame`, открывающие боевому
/// семейству только прежние обращения; имена сохраняют исходную операцию.
pub trait MonsterCombatGame {
    type Player: MonsterCombatPlayer;

    /// Живой регион старого пакета (`CServerRegion`); непрозрачен для исполнения.
    type Region;

    /// Извлечённый владелец региона (`ServerRegionOwner` прежнего пакета).
    type RegionOwner;

    /// AI игрока для `finish_player_skill`; непрозрачен для исполнения.
    type PlayerAi;

    // Владелец региона и базовые проекции.
    fn owner_base(owner: &Self::RegionOwner) -> &Self::Region;

    fn owner_base_mut(owner: &mut Self::RegionOwner) -> &mut Self::Region;

    fn owner_region_id(owner: &Self::RegionOwner) -> i32;

    // Игрок, определения, RNG и часы.
    fn find_player(&self, player_id: i32) -> Option<&Self::Player>;

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut Self::Player>;

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32;

    /// Уровень выученного навыка игрока (чтение через фабрику — у владельца).
    fn monster_combat_player_skill_level(&self, player_id: i32, skill_id: u32) -> Option<i32>;

    fn globe_setup(&self) -> &GlobeSetupSnapshot;

    fn skill_random_below(&mut self, maximum: i32) -> i32;

    // Kernel-исполнение игрока (без typed payload) и typed thorn-состояние.
    fn player_kernel(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>>;

    fn player_kernel_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>>;

    fn begin_player_kernel(
        &mut self,
        player_id: i32,
        kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool;

    /// Боевое начало команды игрока (`begin_player_skill_with_combat`).
    fn begin_player_combat_command(
        &mut self,
        player_id: i32,
        dispatch: PlayerSkillDispatch,
        started_at_ms: u32,
    );

    fn player_monster_thorn_state(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&PlayerMonsterThornExecutionState>;

    fn player_monster_thorn_state_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut PlayerMonsterThornExecutionState>;

    /// Регистрация typed thorn-состояния игрока (payload с точкой эффекта).
    fn begin_player_monster_thorn_execution(
        &mut self,
        player_id: i32,
        state: PlayerMonsterThornExecutionState,
    ) -> bool;

    fn update_player_fight_state_move_shape(&mut self, player_id: i32) -> bool;

    fn publish_player_states(&self, player_id: i32);

    /// Кадр отказа `{0x000BFE01, 0, reason}` самому игроку (message-owner
    /// семьи навыков; mode-кадры формируются так же).
    fn send_cast_failure(&self, player_id: i32, reason: u8);

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    fn send_skill_system_info_text(&self, player_id: i32, text: &[u8], argument: &[u8]);

    /// Around-доставка кадра от формы игрока.
    fn send_player_visual(&mut self, player_id: i32, message: &CMessage);

    /// Around-доставка кадра от формы источника региона.
    fn send_visual_around(&self, region: &Self::Region, origin: &CShape, message: &CMessage);

    fn with_published_player_ai<Output>(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Output;

    /// Публикует извлечённого владельца региона на время callback и
    /// возвращает его обратно; отсутствующий владелец пропускает callback.
    fn with_published_region<Output>(
        &mut self,
        owner: &mut Option<Self::RegionOwner>,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Option<Output>;

    fn finish_monster_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool;

    // Разрешение сторон и целей player-ветви.
    /// Точный общий `CState::GetSufferer` объектной формы.
    fn resolve_identity_sufferer(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeIdentity>;

    /// Точный общий `CState::GetSufferer` координатной формы.
    fn resolve_coordinate_sufferer(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeIdentity>;

    /// `resolve_owned_skill_begin_object` прежнего владельца (identity цели
    /// с владельцем региона для записи Begin).
    fn resolve_owned_skill_begin_object(
        &self,
        region: &Self::Region,
        target: ShapeIdentity,
    ) -> Option<(i32, ShapeIdentity)>;

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView>;

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    fn base_magic_path(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)>;

    /// Точка footprint цели для пути навыка (`GetBeAttackedPoint` владельца).
    fn base_magic_target_point_in(
        &self,
        owner: &Self::RegionOwner,
        source_x: i32,
        source_y: i32,
        identity: ShapeIdentity,
    ) -> Option<(i32, i32)>;

    /// Точка footprint цели player-ветви по идентификатору региона
    /// (`base_magic_target_point` прежнего владельца).
    fn base_magic_target_point(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        identity: ShapeIdentity,
    ) -> Option<(i32, i32)>;

    /// Живой IsAttackAble источника против цели внутри извлечённого владельца.
    fn live_skill_target_attackable_in(
        &self,
        owner: &Self::RegionOwner,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    /// War/camp-допуск построек и ворот игроком (до расчёта).
    fn stationary_build_attackable_by_player(
        &self,
        player_id: i32,
        region_id: i32,
        target: ShapeIdentity,
    ) -> bool;

    /// Общий допуск цели player-навыка по MasterInfo региона.
    fn owned_player_skill_target_attackable(
        &self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
    ) -> bool;

    /// Уровень живой фигуры для оружейного фактора (постройки/ворота = 1).
    fn monster_combat_target_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8>;

    /// Исходный упорядоченный снимок клетки (`flash::cell_views` прежнего
    /// владельца): следующая клетка читается после предыдущих ударов.
    fn monster_combat_cell_views(&self, region_id: i32, tile_x: i32, tile_y: i32) -> Vec<ShapeView>;

    /// Регион живой фигуры (`resolve_state_move_shape` прежнего владельца).
    fn state_move_shape_region_id(&self, region_id: i32, target: ShapeIdentity) -> Option<i32>;

    /// Оружейный фактор игрока против уровня цели; чтение реестра и фабрики
    /// предметов остаётся у владельца (`weapon_modifier` с divisor/floor).
    fn monster_combat_weapon_modifier(&self, player_id: i32, target_level: i32) -> f32;

    /// Общий расчёт player-удара базовой/быстрой атаки монстров
    /// (`lordfastattack::calculate_attack` прежнего пакета, включая личный
    /// критический множитель); split владельца — порция lord-полосы.
    fn monster_combat_calculate_attack(
        &mut self,
        player_id: i32,
        skill_id: u32,
        level: i32,
        hit_modifier: i32,
    ) -> Option<(MasterInfo, AttackInformation)>;

    /// Уровень оружия игрока для порога приручения (`GetWeaponDamageLevel`).
    fn monster_combat_weapon_damage_level(&self, player_id: i32) -> Option<i32>;

    // Регион и монстр: факты, cast, движение и пути.
    /// Входной срез монстра исполнения (форма, строка, interval, живой cast,
    /// reuse момент); резолв строки и приручённого интервала — в шве.
    fn monster_combat_facts(
        &self,
        region: &Self::Region,
        monster_id: i32,
        skill_id: u32,
    ) -> Option<MonsterCombatFacts>;

    /// Снимок монстра-цели удара (форма, строка, вид, живые HP/god).
    fn monster_shape_facts(&self, region: &Self::Region, monster_id: i32) -> Option<MonsterShapeFacts>;

    fn monster_shape(&self, region: &Self::Region, monster_id: i32) -> Option<CShape>;

    fn monster_begin_attack_attempt(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool;

    fn monster_set_moveable(&mut self, region: &mut Self::Region, monster_id: i32, moveable: bool);

    fn monster_set_direction(&mut self, region: &mut Self::Region, monster_id: i32, direction: i32);

    fn monster_set_action(&mut self, region: &mut Self::Region, monster_id: i32, action: i32);

    /// Живой экземпляр cast навыка существует (`base_attack_cast`). End без
    /// reuse — отдельный шов; полное действие остаётся у владельца.
    fn monster_cast_present(&self, region: &Self::Region, monster_id: i32, skill_id: u32) -> bool;

    /// Owner `End(0)` живого cast: без reuse-штампа; движение восстанавливает
    /// только owner по своей таблице производных End.
    fn monster_finish_cast_without_reuse(&mut self, region: &mut Self::Region, monster_id: i32, skill_id: u32);

    fn monster_advance_cast(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        skill_id: u32,
        from: SkillStage,
        to: SkillStage,
    );

    fn monster_install_cast(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        target: ShapeIdentity,
        skill_id: u32,
        skill_level: u16,
        now_ms: u32,
        target_object: Option<(i32, ShapeIdentity)>,
    );

    fn monster_clear_ai_target(&mut self, region: &mut Self::Region, monster_id: i32);

    /// Текущие границы атаки монстра после состояний (`state_attack_bounds`
    /// владельца); отсутствующий монстр — `None`, как прежний ранний return.
    fn monster_state_attack_bounds(
        &self,
        region: &Self::Region,
        monster_id: i32,
        minimum: u32,
        maximum: u32,
    ) -> Option<(u32, u32)>;

    /// `GetShapes` клетки полного региона с resolver старого хоста; типы и
    /// само-исключение фильтруются здесь, отказ чтения — пустой вектор в шве.
    fn monster_region_cell_identities(
        &self,
        owner: &Self::RegionOwner,
        tile_x: i32,
        tile_y: i32,
    ) -> Vec<ShapeIdentity>;

    /// NPC клетки: форма и god; нулевой combat HP (`IsDied` всегда) читается
    /// здесь постоянным, как у прежнего владельца.
    fn npc_move_shape_facts(&self, region: &Self::Region, npc_id: i32) -> Option<(CShape, bool)>;

    /// Постройка/ворота извлечённого владельца: форма, god и HP (`hp == 0`
    /// игровых построек — их IsDied владельца).
    fn stationary_build_facts(
        &self,
        owner: &Self::RegionOwner,
        identity: ShapeIdentity,
    ) -> Option<(CShape, bool, u32)>;

    fn shape_view_in_owner(&self, owner: &Self::RegionOwner, identity: ShapeIdentity) -> Option<ShapeView>;

    fn monster_straight_skill_path(
        &self,
        region: &Self::Region,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)>;

    // Приручение: снимок цели и жизненный цикл питомца.
    /// Снимок цели приручения (setup-строка, live позиция/HP/имя, IsTamable).
    fn monster_taming_target_snapshot(&self, region_id: i32, monster_id: i32) -> Option<MonsterTamingTarget>;

    /// Блокировка клетки пути (`region.get_block(x, y) == 2`, отсутствие → 2).
    fn monster_combat_cell_blocked(&self, region_id: i32, tile_x: i32, tile_y: i32) -> bool;

    /// Извлечённый владелец региона на время применения успеха приручения;
    /// отсутствующий регион пропускает callback, как и прежний путь.
    fn with_monster_combat_region<Output>(
        &mut self,
        region_id: i32,
        callback: impl FnOnce(&mut Self, &mut Self::Region) -> Output,
    ) -> Option<Output>;

    /// `StopAllSkills` живого объекта региона перед назначением master.
    fn monster_combat_stop_all_skills(&mut self, region_id: i32, holder: ShapeIdentity);

    /// `IncreaseTameAttemptCount` живого монстра (странная граница счётчика
    /// попыток остаётся у владельца).
    fn monster_increase_tame_attempt(&mut self, region: &mut Self::Region, monster_id: i32) -> bool;

    /// `SetTamedSign(1) + SetMasterInfo` с auxiliary-эффектами питомца;
    /// отказ — уже приручённое существо (`DoesCreatureBeenTamed` владельца).
    fn monster_try_become_tamed(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        master: MasterInfo,
        pet_mode: i32,
    ) -> bool;

    /// Форма живого питомца после назначения (пакет 0xC0201 читает живую
    /// shape без копирования владельца).
    fn monster_region_shape(&self, region: &Self::Region, monster_id: i32) -> Option<CShape>;

    /// Текущие level/experience питомца (`GetPetLevel/GetPetExperience`).
    fn monster_pet_progress(&self, region: &Self::Region, monster_id: i32) -> Option<(u32, u32)>;

    /// `UpgradePetLevel` прежнего владельца поверх progression-факторов
    /// глобальных настроек (чтение таблицы — `globe_setup` шва).
    fn monster_upgrade_pet_level(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        property: &MonsterProperties,
        experience_factor: f32,
        current_factors: Option<[f32; 10]>,
        next_factors: Option<[f32; 10]>,
    );

    /// HP и максимум живого монстра по его строке (`GetHP/GetMaxHP`).
    fn monster_hp_snapshot(
        &self,
        region: &Self::Region,
        monster_id: i32,
        property: &MonsterProperties,
    ) -> Option<(u32, u32)>;

    /// Refresh-record decrement региона после успешного приручения
    /// (`GetMonsterRefeash(...)[+0x38] -= 1`, ноль не уходит в минус).
    fn monster_taming_decrease_refresh(&mut self, region: &mut Self::Region, monster_id: i32);
}

/// Контактные операции с runtime игрового хода боевого семейства. Отделены,
/// потому что тип хода принадлежит старому main loop, а не самому исполнению
/// (прецедент `SummonSkillContact`/`BaseAttackContact`).
pub trait MonsterCombatContact<Runtime>: MonsterCombatGame {
    /// Подход монстра к дистанции цели (`approach_attack_range` общего
    /// `ai::monsterai` через прежний адаптер владельца).
    fn monster_combat_approach_attack_range(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        target_view: ShapeView,
        maximum_distance: u32,
        runtime: &mut Runtime,
    ) -> bool;

    /// Живой `End` cast с reuse-часами владельца (`CSkill::End` читает reuse
    /// после побочных эффектов derived End).
    fn monster_finish_cast_clock(
        &mut self,
        region: &mut Self::Region,
        monster_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    );

    /// `AfterUseSkill` игрока: износ оружия и reuse-штамп успешного исхода.
    fn monster_combat_after_use_player_skill(
        &mut self,
        player_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    );

    /// `IncreaseRp(1, 0)` источника после возврата приёмника удара.
    fn monster_combat_increase_rp(&mut self, player_id: i32, attacking: bool, damage: u16);

    /// Общий virtual +15C (..., false) игрока-цели круговой атаки.
    fn monster_combat_apply_attack_to_player(
        &mut self,
        master: MasterInfo,
        player_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );

    /// Общий virtual +15C (..., false) монстра-цели круговой атаки.
    fn monster_combat_apply_attack_to_monster(
        &mut self,
        master: MasterInfo,
        monster_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );

    /// Контакт постройки/ворот круговой атаки по их owner.
    fn monster_combat_apply_attack_to_build(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        attack: AttackInformation,
        runtime: &mut Runtime,
    );
}

/// Owner `End(0)` живого cast монстра: подтверждённый экземпляр завершается
/// без reuse-штампа. Отсутствующий монстр или cast — `false`, как прежний
/// ранний return владельца.
pub fn end_owned_monster_skill_without_reuse<Game: MonsterCombatGame>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
    skill_id: u32,
) -> bool {
    if !game.monster_cast_present(region, monster_id, skill_id) {
        return false;
    }
    game.monster_finish_cast_without_reuse(region, monster_id, skill_id);
    true
}

/// Общий хвост удара монстра после Apply: стадии Calculate→Attack→Apply,
/// action 1 и живой `End` с reuse-часами (движение восстанавливает только
/// owner по своей таблице производных End).
pub fn finish_owned_monster_attack_impact<Game, Runtime>(
    game: &mut Game,
    region: &mut Game::Region,
    monster_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) where
    Game: MonsterCombatContact<Runtime>,
{
    game.monster_advance_cast(region, monster_id, skill_id, SkillStage::Calculate, SkillStage::Attack);
    game.monster_advance_cast(region, monster_id, skill_id, SkillStage::Attack, SkillStage::Apply);
    game.monster_set_action(region, monster_id, 1);
    game.monster_finish_cast_clock(region, monster_id, skill_id, runtime);
}

/// Живой упорядоченный снимок одной клетки для конкретного удара монстра:
/// полный список читает региональный resolver старого хоста; допуск типов
/// (player/npc/monster/постройки/ворота) и исключение источника остаются
/// здесь, как у прежнего владельца.
pub fn monster_attack_cell_candidates<Game: MonsterCombatGame>(
    game: &Game,
    owner: &Game::RegionOwner,
    source_monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    game.monster_region_cell_identities(owner, tile_x, tile_y)
        .into_iter()
        .filter(|identity| {
            matches!(identity.object_type, 400 | 500 | 600 | 1_100 | 1_200)
                && !(identity.object_type == MONSTER_TYPE && identity.id == source_monster_id)
        })
        .collect()
}

/// Живое разрешение цели удара монстра для конкретного тика. Игрок читается
/// глобальной таблицей; монстр — через извлечённый регион и его setup-строку;
/// NPC — постоянный IsDied (нулевой combat HP); постройки и ворота — их HP.
pub fn resolve_owned_monster_attack_target<Game: MonsterCombatGame>(
    game: &Game,
    owner: &Game::RegionOwner,
    identity: ShapeIdentity,
) -> Option<OwnedMonsterAttackTarget> {
    let region = Game::owner_base(owner);
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            let view = player.shape_view()?;
            Some(OwnedMonsterAttackTarget {
                shape: player.shape().clone(),
                view,
                player_properties: Some(player.combat_properties()),
                dead: player.is_dead(),
                god: player.is_god_mode(),
                city_dead: player.city_war_died_state(),
                monster_property: None,
            })
        }
        MONSTER_TYPE => {
            let facts = game.monster_shape_facts(region, identity.id)?;
            Some(OwnedMonsterAttackTarget {
                shape: facts.shape,
                view: facts.view,
                player_properties: None,
                // `CMoveShape::IsDied`: GetHP == 0 (0x004CCF20) без action-ветви.
                dead: facts.hit_points == 0,
                god: facts.god,
                city_dead: false,
                monster_property: Some(facts.property),
            })
        }
        500 => {
            // CNpc наследует GetHP == 0: его IsDied истинен независимо от action.
            let (shape, god) = game.npc_move_shape_facts(region, identity.id)?;
            let view = game.shape_view_in_owner(owner, identity)?;
            Some(OwnedMonsterAttackTarget {
                shape,
                view,
                player_properties: None,
                dead: true,
                god,
                city_dead: false,
                monster_property: None,
            })
        }
        1_100 | 1_200 => {
            let (shape, god, hit_points) = game.stationary_build_facts(owner, identity)?;
            let view = game.shape_view_in_owner(owner, identity)?;
            Some(OwnedMonsterAttackTarget {
                shape,
                view,
                player_properties: None,
                dead: hit_points == 0,
                god,
                city_dead: false,
                monster_property: None,
            })
        }
        _ => None,
    }
}

/// Применение рассчитанного удара монстра к цели: владелец региона
/// публикуется целиком на время контакта; общий получатель читает живые
/// стороны, а не снимки до защиты. Отсутствующий владелец пропускает удар.
pub fn apply_owned_monster_attack_hit<Game, Runtime>(
    game: &mut Game,
    owner: &mut Option<Game::RegionOwner>,
    runtime: &mut Runtime,
    target: ShapeIdentity,
    attack: AttackInformation,
) where
    Game: MonsterCombatGame + BaseAttackContact<Runtime>,
{
    let Some(region_id) = owner.as_ref().map(Game::owner_region_id) else { return; };
    let master = MasterInfo {
        master_type: attack.attacker_type,
        master_id: attack.attacker_id,
        ..MasterInfo::default()
    };
    let _ = game.with_published_region(owner, |game| {
        game.apply_owned_skill_contact(master, target, region_id, attack, runtime);
    });
}
