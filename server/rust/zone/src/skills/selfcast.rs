//! Hub-швы и общие типы self/zone-кастов порции №6c: ослепление CBlind и
//! общий lifecycle 8-байт lock-состояний (CBlindState...CBoaLockState),
//! накопление энергии CEnergyHolding, боевой клич CRoar, стойка CPillar,
//! закалки CCallosity/CCallosity2 и область зеркала душ CSoulMirror.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match). Прежний переходный владелец тел —
//! `src/gameserver/appserver/skills/{blind,blindstate,energyholding,
//! energyholdingstate,roar,roarstate,pillar,pillarstate,callosity,
//! callosity2,callositystate,soulmirror}.rs`; тела перенесены буквально
//! порцией №6c «self/zone-касты» (разведка — запись аудита «Zone skills:
//! машинная разведка battlefairy-навыков (порция №6)», 26 сентября 2026).
//!
//! Машинные якоря семьи (RVA той же точной пары): CBlind Begin `0x16DA30`,
//! AddBlindState `0x16E500` → `new 0x3C` + ctor CRushState2 `0x5F12E0`
//! (VERIFIED разведкой); codec семейства CBlindState/CBoaLockState/CCureState
//! 8-байтный (`effects/blind.rs` ✓). CEnergyHolding Begin `0x149BF0`,
//! Check `0x14A190`, AI `0x14A4A0`, state ctor `0x1EC410`, AddEnergy
//! `0x1EC490`, GetRemainedTime `0x201200`, skill End(H) 3-fold `0x1502F0`.
//! CRoar Begin `0x14A7D0`, AI `0x14B060` (окно `roar_bounds` VA
//! `0x54B1CC..0x54B23E` подтверждено ранее); CRoarState Serialize 5-fold
//! `0x1F65F0` (с heal-квартетом). CPillar Begin `0x16FA00`, AI `0x170110`,
//! state Begin `0x1F4B60`. CCallosity/CCallosity2 — собственные Check/AI,
//! методы состояний почти полностью попарно folded (9 методов, Restart-fold
//! с CPromotionState `0x1FD450`). CSoulMirror GetScope/Length/Height
//! `0x1A40D0/0x1A4120/0x1A4150`, CalculateAttackPower `0x1A4A30`, Attack
//! `0x1A4BF0`; End(H) 13-fold `0x146090` — общий CStateSkill tail нескольких
//! навыков: здесь не дублируется, порядок clear+End исполняет прежний
//! kernel/вход, как и раньше.
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — переходные
//! фасады прежнего владельца `CGame`/`CPlayer`/`CMoveShape`/`CPlayerAI`,
//! реализация остаётся у него в файле-делегате
//! `appserver/skills/selfcast.rs`; имена членов сохраняют исходную
//! операцию. Швы потребляются статически (generic), dyn-совместимость и
//! `Send`-контракт не вводятся (прецедент hub-паттерна порции №6a).
//! Общие хелперы старого пакета переносятся не как тела, а объявляются
//! швами: обвязка арены `states/state.rs` (`begin_base_applied_state`,
//! `begin_applied_state_visual`, `update_applied_state_visual_base`,
//! `update_property_state_visual`, `update_applied_state_end_visual`,
//! `resolve_applied_state_sufferer`, `remove_applied_state_from`,
//! `end_and_destroy_state_at`), машина накопления `accumulatedstate`
//! (`add_accumulated_state` и `update_accumulated_visual` — владелец
//! поделён с SoulCollect и перенесётся его порцией), прямой элементный
//! контакт `directelementattack::apply_direct_element_attack`, мастер
//! источника `weaponattack::source_master`, PK-вход
//! `player_on_first_skill_at_position` и lifecycle призванного существа из
//! `runtimespawn`/регион-владельца. Часы прежнего main loop приходят
//! указателем `now_milliseconds` (делегат передаёт
//! `game_tick_milliseconds`).
//!
//! Региональные чтения свёрнуты в одноразовые швы с сохранением порядка
//! старого тела: одиночный снимок клетки (тот же resolver и гейт
//! `find_region`, что у старого тела), SAFE-гейт (`find_region` →
//! `get_security == SAFE`, обе ветки старого тела завершали клетку молча),
//! проходимость пустой клетки SoulMirror (границы + `skill_cell_block & 7
//! == 0`) и подавление атаки монстра Roar (lookup property по origin name →
//! `RoarState::monster_losses` → wrapping-вычитание из модификаторов того
//! же монстра). Живой вызов `update_player_current_state(..., MoveShapeAi)`
//! назван `update_player_current_state_move_shape_ai` — параметр фазы у
//! семьи один.

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::{MasterInfo, PlayerCombatProperties};
use crate::content::CSkillBaseProperties;
use crate::effects::{EnergyHoldingState, RoarState};
use crate::regions::ShapeIdentity;
use crate::regions::shape::{CShape, ShapeView};

use super::dispatch::PlayerSkillDispatch;
use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::lifecycle::{SkillExecutionKernel, SkillLifecycle, SkillTermination};
use super::state::{AppliedState, StateData, StateKey};
use super::visualeffect::SkillVisualEffect;

/// Переходный фасад игрока-заклинателя и цели семьи: только операции,
/// которые выполняют перенесённые тела (MP/RP, уровень, запрет движения
/// команды и пересчёт боевых свойств состояний Roar/Callosity).
pub trait SelfCastPlayer {
    fn shape(&self) -> &CShape;

    fn movement_shape_mut(&mut self) -> &mut CShape;

    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);

    fn rp(&self) -> u16;

    fn set_rp(&mut self, rp: u16);

    fn level(&self) -> u8;

    fn is_dead(&self) -> bool;

    fn set_skill_moveable(&mut self, moveable: bool);

    /// Живой `update_state_combat_properties` прежнего `CPlayer`:
    /// замена снимка и синхронизация wire, без обхода остальных состояний.
    fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    );
}

/// Переходный фасад `CMoveShape`-арены состояний: операции записи и
/// typed-доступа, которыми пользуются Begin/restart/AI/End состояний семьи.
/// Имена сохраняют методы прежнего владельца; сериализуемый учёт записей
/// (`append/insert_replacement_state_record`) остаётся его логикой.
pub trait SelfCastMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);

    fn set_fightable(&mut self, fightable: bool);

    fn find_state_position(
        &self,
        matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)>;

    fn applied_state<T: AppliedState>(&self, key: StateKey) -> Option<&T>;

    fn applied_state_mut<T: AppliedState>(&mut self, key: StateKey) -> Option<&mut T>;

    /// Доступ к enum-записи арены (`applied_state_data` прежнего CMoveShape).
    fn applied_state_data(&self, key: StateKey) -> Option<&StateData>;

    fn applied_state_replacement_location(&self, key: StateKey) -> Option<(usize, usize)>;

    fn append_applied_state_record<T: AppliedState>(&mut self, state: T, record: &[u8]) -> StateKey;

    fn insert_replacement_state_record<T: AppliedState>(
        &mut self,
        state: T,
        record: &[u8],
        location: (usize, usize),
    ) -> Option<StateKey>;

    fn begin_applied_state_visual(&mut self, key: StateKey, loop_value: i32) -> bool;

    fn update_applied_state_visual_base(&mut self, key: StateKey) -> bool;

    fn mark_applied_state_begun(&mut self, key: StateKey) -> bool;

    fn mark_applied_state_ended(&mut self, key: StateKey) -> bool;

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
}

/// Адресат стандартных property-visual адресов (`StatePropertyTarget`
/// прежнего `states/state.rs`): у семьи все пути используют Sufferer,
/// форма параметра сохранена для тождества шва.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelfCastPropertyTarget {
    User,
    Sufferer,
}

/// Переходные фасады прежнего владельца `CGame`, открывающие self/zone-кастам
/// семьи только прежние обращения; имена сохраняют исходную операцию.
pub trait SelfCastGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ);
    /// непрозрачен для исполнения.
    type SkillAddress: Copy;

    type Player: SelfCastPlayer;

    type MoveShape: SelfCastMoveShape;

    /// AI игрока kernel-входа CBlind (`CPlayerAI` прежнего пакета);
    /// непрозрачен для перенесённых тел.
    type PlayerAi;

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

    /// Слот навыка игрока → адрес записи (`registered_player_skill`
    /// прежнего `states/skill.rs`).
    fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<Self::SkillAddress>;

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

    // Kernel-исполнение игрока (вход CBlind вне зарегистрированного playercast).
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

    fn player_skill_lifecycle(&self, player_id: i32, skill_id: u32) -> Option<&SkillLifecycle>;

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32;

    fn begin_player_skill_execution(
        &mut self,
        player_id: i32,
        kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool;

    fn replace_player_skill_visual_effect(
        &mut self,
        player_id: i32,
        skill_id: u32,
        effect: SkillVisualEffect,
    );

    fn update_player_skill_visual(&mut self, player_id: i32, skill_id: u32, mode: u32);

    /// Прежний `update_player_current_state(player_id, MoveShapeAi)`:
    /// единственная фаза, которую использует семья (первый AI CBlind).
    fn update_player_current_state_move_shape_ai(&mut self, player_id: i32);

    /// Хвост команды игрока прежнего `CPlayerAI` после owner-ского End.
    fn finish_player_skill(
        &mut self,
        player_id: i32,
        ai: &mut Self::PlayerAi,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool;

    /// Публикация захваченного AI обратно игроку на время callback
    /// (`with_published_player_ai` прежнего `CGame`).
    fn with_published_player_ai<R>(
        &mut self,
        player_id: i32,
        ai: &mut Self::PlayerAi,
        body: impl FnOnce(&mut Self) -> R,
    ) -> R;

    // Диагностика игрока (GS-тексты и OnChangeStates).
    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    fn send_skill_system_info_with_text(&self, player_id: i32, text: &[u8], name: &[u8]);

    fn publish_player_states(&self, player_id: i32);

    // Живой мир: здоровье, уровни, допуски и имена целей.
    fn move_shape_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32>;

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8>;

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        user: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool;

    fn live_skill_target_attackable_between(
        &self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
    ) -> bool;

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    /// Имя цели для именованной ошибки препятствия (GS0291); отсутствие
    /// имени — пустой вектор, как `unwrap_or_default()` прежнего результата.
    fn base_magic_target_name(&self, region_id: i32, target: ShapeIdentity) -> Vec<u8>;

    fn skill_target_controller(&self, region_id: i32, target: ShapeIdentity) -> Option<i32>;

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    /// Одиночный GetShape клетки (снимок фигуры области; тот же resolver и
    /// гейт региона, что у старого тела).
    fn area_shape_view_at(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeView>;

    /// Снимок фигур клетки обхода SoulMirror (прежний `cell_views` семьи
    /// Flash, тот же resolver и гейт региона).
    fn area_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView>;

    /// Размеры региона для окна обхода CRoar (`find_region` + width/height).
    fn region_dimensions(&self, region_id: i32) -> Option<(i32, i32)>;

    /// SAFE-гейт клетки обхода CRoar: `None` — регион исчез до проверки
    /// (старый `let Some(region) = find_region... else return`),
    /// `Some(true)` — `get_security == SAFE` и клетка пропускается.
    fn region_cell_safe(&self, region_id: i32, x: i32, y: i32) -> Option<bool>;

    /// Пустая проходимая клетка SoulMirror: границы региона и
    /// `skill_cell_block(x, y) & 7 == 0` без второго снимка мира.
    fn region_cell_walkable(&self, region_id: i32, x: i32, y: i32) -> bool;

    fn update_move_shape_properties(&mut self, region_id: i32, target: ShapeIdentity);

    fn send_move_shape_around(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
        message: &CMessage,
    );

    // Доставка visual `0x000BFE01`: кадр строит владелец навыка,
    // маршруты — у прежнего владельца (гейт региона вокруг сохранён).
    fn send_cast_visual_to_player(&self, player_id: i32, message: &CMessage);

    fn send_cast_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);

    /// Категория оружия слота 2 по живому addon (`GAP_WEAPON_CATEGORY`, 1);
    /// резолв фабрики и отсутствия — у владельца шва.
    fn player_weapon_addon_category(&self, player: &Self::Player) -> Option<i32>;

    // Обвязка арены состояний прежнего `states/state.rs` (объявленные швы).
    fn resolve_applied_state_sufferer(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> Option<(i32, ShapeIdentity)>;

    fn begin_base_applied_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> bool;

    fn begin_applied_state_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        loop_value: i32,
    ) -> bool;

    fn update_applied_state_visual_base(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> bool;

    fn update_applied_state_end_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: SelfCastPropertyTarget,
    ) -> bool;

    fn update_property_state_visual<S: AppliedState>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: SelfCastPropertyTarget,
        now: &mut dyn FnMut() -> u32,
        client_time: impl FnOnce(&S, &mut dyn FnMut() -> u32) -> u32,
    ) -> bool;

    fn remove_applied_state_from(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: (i32, ShapeIdentity),
        bytes: usize,
    ) -> bool;

    /// Полный End + destructor по найденной позиции (`end_and_destroy_state_at`);
    /// `false` — отсутствие слота/ключа, как `None` прежнего результата.
    fn end_and_destroy_state_at(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        index: usize,
    ) -> bool;

    // Machine накопления `accumulatedstate` прежнего пакета (объявленный шов;
    /// владелец разделяется с SoulCollect): поиск первого слота ID89,
    /// typed-инкремент либо создание с локальным visual и append.
    fn add_energy_holding_state(
        &mut self,
        source: (i32, ShapeIdentity),
        create: impl FnOnce(&Self) -> Option<EnergyHoldingState>,
        now: &mut dyn FnMut() -> u32,
    ) -> bool;

    /// Visual End(2) машины накопления (`update_accumulated_visual` прежнего
    /// пакета) конкретного ключа держателя.
    fn update_energy_holding_accumulated_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        mode: u32,
    );

    /// Подавление атаки живого монстра Roar (lookup property по origin name,
    /// `RoarState::monster_losses` от свежих границ, wrapping-вычитание из
    /// `property_modifiers_mut` того же монстра).
    fn roar_monster_apply_losses(&mut self, region_id: i32, monster_id: i32, state: RoarState);

    /// Мастер источника SoulMirror (`source_master` прежнего
    /// `weaponattack`): игрок — полный `master_info`, прочий CMoveShape —
    /// identity-формула; `master_country_id = 0` задаёт тело навыка.
    fn soul_mirror_source_master(&self, source: (i32, ShapeIdentity)) -> Option<MasterInfo>;

    /// Lifecycle призванного существа пустой клетки SoulMirror: property по
    /// picture id, owner региона, `add_summoned_creature_owned`, возврат
    /// owner — в порядке прежнего тела, одной операцией владельца шва.
    fn add_soul_mirror_summoned_creature(
        &mut self,
        region_id: i32,
        picture_id: u32,
        master: MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    );
}

/// Runtime-связанные контакты семьи: PK-вход первого умения и прямой
/// элементный удар CSoulMirror. Реализация остаётся у прежнего владельца;
/// `runtime` пробрасывается без собственных чтений часов.
pub trait SelfCastContact<Runtime>: SelfCastGame {
    /// Прежний `player_on_first_skill_at_position` (PK-контроллер пары
    /// игроков у CRoar и CBlind).
    fn skill_first_attack_at_position(
        &mut self,
        attacker_id: i32,
        victim_id: i32,
        region_id: i32,
        position: (i32, i32),
        runtime: &mut Runtime,
    );

    /// Прежний `apply_direct_element_attack` (`directelementattack` старого
    /// пакета; профиль SoulMirror, PK и расчёт — владение этого шва).
    fn apply_direct_element_contact(
        &mut self,
        instance: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    );
}

/// Исход одного прохода Check/AI семьи. Значения соответствуют
/// `QueuedSkillExecutionState` прежнего планировщика один к одному;
/// обработку End выполняет прежний общий вход (playercast/kernel).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelfCastExecutionOutcome {
    Pending,
    Rejected,
    RejectedAfterUse,
    Begun,
    Completed,
}

/// Живой participant прежнего тела навыка: повторное разрешение фигуры и
/// снятие фактического (region, identity) — перенос локальных `participant`
/// переходных файлов.
pub fn selfcast_participant<Game: SelfCastGame>(
    game: &Game,
    value: (i32, ShapeIdentity),
) -> Option<(i32, ShapeIdentity)> {
    let shape = game.resolve_state_move_shape(value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

/// Participant записи арены: регион/identity с обнулённым `ex_id`
/// (`CGuid::GUID_INVALID`), как в прежних Begin-экземпляров состояний.
pub fn selfcast_storage_participant<Game: SelfCastGame>(
    game: &Game,
    value: (i32, ShapeIdentity),
) -> Option<(i32, ShapeIdentity)> {
    let shape = game.resolve_state_move_shape(value.0, value.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID,
        ..shape.identity()
    }))
}
