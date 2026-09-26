//! Hub-швы и общий wire-кадр visual state-кастов пятёрки
//! (CCure, CHearten, CPromotion, CGodBless/CGodBless2) и heal-квартета
//! (CHeal/CHeal2/CSuperHeal/CSuperHeal2). Источник:
//! gameserver.exe + GameServer.pdb, `appserver/states/stateskill.cpp`
//! и совместимые `appserver/skills/{cure,hearten,promotion,godbless{,2},
//! heal{,2},superheal{,2}}.cpp/.h`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/{cure,hearten,promotion,godbless,
//! heal}.rs` и старшие state-файлы; тела перенесены буквально порцией
//! №6a «state-касты пятёрки + heal-квартет» (разведка — запись аудита
//! «Zone skills: машинная разведка battlefairy-навыков (порция №6)»,
//! 26 сентября 2026, по точной паре `gameserver.exe` `4F5C98E0…` +
//! `GameServer.pdb` RSDS match).
//!
//! Общий кадр `publish_state_cast_visual` — перенос тела
//! `publish_state_skill_visual` переходного `stateskill.rs`: wire `0xBFE01`,
//! личная ветвь отказов BYTE-парой `[0, mode]` только игроку, around-кадр
//! с S (fallback U) для mode 1. У пятёрки нет flight-хвоста; DWORD-форма
//! отказов `[dword 0][byte mode]` принадлежит Fury/RageBreak и переносится
//! порцией T4 как `RageCastVisualContract` + `publish_rage_cast_visual`
//! (якорь `UpdateVisualEffect@CRageBreakEffect` VA `0x59FB90`).
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — переходные
//! фасады прежнего владельца `CGame`/`CPlayer`/`CMoveShape`, реализация
//! остаётся у него в файле-делегате `appserver/skills/statecast.rs`;
//! имена членов сохраняют исходную операцию. Швы потребляются статически
//! (generic), dyn-совместимость и `Send`-контракт не вводятся (прецедент
//! ADR-0013 семейства dash). Общие хелперы старого пакета — MP/путь
//! `rangedweaponcast` (`check_cast_mana`/`spend_cast_mana`/`check_skill_path`)
//! и обвязка арены `states/state.rs` (`end_and_destroy_state_at`,
//! `end_move_shape_state`, `remove_applied_state_from`,
//! `resolve_applied_state_sufferer`, `update_property_state_visual`,
//! `update_applied_state_end_visual`, `update_player_state_properties`) —
//! переносятся не как тела, а объявляются швами `{check,spend}_cast_mana`,
//! `check_skill_path` и одноимёнными методами трейта; их машинное поведение
//! уже сверено соседними волнами и здесь не переоткрывается.
//! Аргумент `now` — часы прежнего main loop (делегат передаёт
//! `runtime.now_milliseconds()`), как в `skills/flash.rs` порции №5.

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::combat::PlayerCombatProperties;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::CShape;

use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::lifecycle::SkillLifecycle;
use super::state::{AppliedState, StateData, StateKey};
use super::visualeffect::SkillVisualEffectKind;
use crate::effects::CureState;

/// Переходный фасад игрока-заклинателя пятёрки: только операции,
/// которые выполняют перенесённые тела (MP и пересчёт боевых свойств).
pub trait StateCastPlayer {
    fn mana(&self) -> u32;

    fn set_mana(&mut self, mana: u32);

    /// Живой `combat_properties()` прежнего `CPlayer` (снимок Zone-типом).
    fn combat_properties(&self) -> PlayerCombatProperties;

    /// Живой `update_state_combat_properties` прежнего `CPlayer`:
    /// замена снимка и синхронизация wire, без обхода остальных состояний.
    fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    );
}

/// Переходный фасад `CMoveShape`-арены состояний: операции записи и
/// typed-доступа, которыми пользуются Begin/restart/AI/End состояний пятёрки.
/// Имена сохраняют методы прежнего владельца; сериализуемый учёт записей
/// (`append/insert_replacement_state_record`) остаётся его логикой.
pub trait StateCastMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);

    fn state_slot_count(&self) -> usize;

    fn state_at(&self, position: usize) -> Option<(StateKey, &StateData)>;

    fn find_state_position(
        &self,
        matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)>;

    fn applied_state<T: AppliedState>(&self, key: StateKey) -> Option<&T>;

    fn applied_state_mut<T: AppliedState>(&mut self, key: StateKey) -> Option<&mut T>;

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

    fn defense_shield_key(&self, skill_id: u32) -> Option<StateKey>;

    fn cure_state_key(&self) -> Option<StateKey>;

    fn cure_state_by_key(&self, key: StateKey) -> Option<CureState>;
}

/// Адресат стандартных property-visual адресов (`StatePropertyTarget`
/// прежнего `states/state.rs`): у пятёрки все пути используют Sufferer,
/// форма параметра сохранена для тождества шва.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateCastPropertyTarget {
    User,
    Sufferer,
}

/// Переходные фасады прежнего владельца `CGame`, открывающие state-кастам
/// пятёрки и heal-квартета только прежние обращения; имена сохраняют
/// исходную операцию.
pub trait StateCastGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ);
    /// непрозрачен для исполнения.
    type SkillAddress: Copy;

    type Player: StateCastPlayer;

    type MoveShape: StateCastMoveShape;

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

    /// Ключ первого Cure-слота живого игрока (`move_shape().cure_state_key()`).
    fn player_cure_state_key(&self, player_id: i32) -> Option<StateKey>;

    /// Фактические регион и identity живого игрока (`shape()` его MoveShape).
    fn player_shape_participant(&self, player_id: i32) -> Option<(i32, ShapeIdentity)>;

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    // Диагностика игрока (GS-тексты и OnChangeStates).
    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    fn send_skill_system_info_with_text(&self, player_id: i32, text: &[u8], name: &[u8]);

    fn publish_player_states(&self, player_id: i32);

    // Обвязка арены состояний прежнего `states/state.rs` (объявленные швы).
    fn resolve_applied_state_sufferer(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> Option<(i32, ShapeIdentity)>;

    fn update_player_state_properties<S: AppliedState + Copy>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        update: impl FnOnce(S, &mut Self::Player),
    ) -> bool;

    fn update_property_state_visual<S: AppliedState>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: StateCastPropertyTarget,
        now: &mut dyn FnMut() -> u32,
        client_time: impl FnOnce(&S, &mut dyn FnMut() -> u32) -> u32,
    ) -> bool;

    fn update_applied_state_end_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: StateCastPropertyTarget,
    ) -> bool;

    fn remove_applied_state_from(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: (i32, ShapeIdentity),
        bytes: usize,
    ) -> bool;

    fn end_move_shape_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> bool;

    /// Полный End + destructor по найденной позиции (`end_and_destroy_state_at`);
    /// `false` — отсутствие слота/ключа, как `None` прежнего результата.
    fn end_and_destroy_state_at(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        index: usize,
    ) -> bool;

    fn send_move_shape_around(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
        message: &CMessage,
    );

    // Check-скелет `rangedweaponcast` прежнего пакета (объявленные швы):
    // дальность/препятствия с веткой Ignore, MP-контракт и списание MP.
    fn check_skill_path(
        &mut self,
        instance: Self::SkillAddress,
        properties: &CSkillBaseProperties,
        path: &[(i32, i32, u8)],
        player: Option<i32>,
    ) -> bool;

    fn check_cast_mana(
        &mut self,
        instance: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool;

    fn spend_cast_mana(
        &mut self,
        instance: Self::SkillAddress,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool;

    // Живой мир: монстры, здоровье и имена целей.
    /// Живой монстр региона (`find_monster_by_id(...).is_some()`);
    /// требуется гейту CGodBless2, отличному от проверки дикости.
    fn monster_present(&self, region_id: i32, monster_id: i32) -> bool;

    /// Дикий нетранспортный монстр: неприручённый и без Carriage AI
    /// (`!is_tamed && active_ai() ∉ {Carriage, Primary(Carriage)}`).
    fn wild_untamed_non_carriage_monster(&self, region_id: i32, monster_id: i32) -> bool;

    /// Живой уровень оружия игрока (`weapon_damage_level(goods_factory())`);
    /// отсутствие оружия читается нулём у владельца.
    fn player_weapon_damage_level(&self, player: &Self::Player) -> u32;

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool;

    /// Имя цели для именованной ошибки препятствия (GS0295);
    /// отсутствие имени — пустой вектор, как `unwrap_or_default().to_vec()`.
    fn base_magic_target_name(&self, region_id: i32, target: ShapeIdentity) -> Vec<u8>;

    fn move_shape_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32>;

    fn move_shape_maximum_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32>;

    /// Живой setter HP; `false` соответствует `None` прежнего результата.
    fn set_move_shape_health(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        health: u32,
    ) -> bool;

    fn publish_move_shape_states(&mut self, region_id: i32, target: ShapeIdentity);

    fn update_move_shape_properties(&mut self, region_id: i32, target: ShapeIdentity);

    /// Прибавки `property_modifiers_mut` живого монстра цели GodBless
    /// (три wrapping-сложения minimum/maximum/element).
    fn god_bless_monster_gains(
        &mut self,
        region_id: i32,
        monster_id: i32,
        gains: (i32, i32, i32),
    );

    /// RNG прежнего `CGame` для вероятностного CastCure.
    fn skill_random_below(&mut self, maximum: i32) -> i32;

    // Доставка visual `0x000BFE01`: кадр строится в `publish_state_cast_visual`,
    // маршруты — у владельца.
    fn send_state_cast_visual_to_player(&self, player_id: i32, message: &CMessage);

    fn send_state_cast_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage);
}

/// Исход одного тика AI state-каста. `Pending`/`Rejected` возвращаются
/// внешней очереди без End; `EndRejected`/`EndCompleted` требуют от делегата
/// прежний полный End того же зарегистрированного экземпляра с аргументом
/// 0/1 (соответствует `end_state_skill(..., 0/1)` переходного тела).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateCastExecutionOutcome {
    Pending,
    Rejected,
    EndRejected,
    EndCompleted,
}

/// Поле цели клиентского кадра применения (перенос `StateSkillVisualTarget`
/// переходного `stateskill.rs`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateCastVisualTarget {
    Sufferer,
    SuffererOrUser,
    User,
}

/// Контракт visual владельца кадра: ID, ресурс, допустимые режимы отказов
/// и выбор адресата mode 1.
pub struct StateCastVisualContract {
    pub skill_id: u32,
    pub kind: SkillVisualEffectKind,
    pub failures: &'static [u32],
    pub target: StateCastVisualTarget,
}

/// Контракт visual Fury/RageBreak: поверх пятёрочного один RP-отказ несёт
/// DWORD-префикс (`add_long(0)` перед BYTE mode) — форма `[dword 0][byte 8]`,
/// отложенная порцией №6a и перенесённая порцией T4.
pub struct RageCastVisualContract {
    pub skill_id: u32,
    pub kind: SkillVisualEffectKind,
    pub failures: &'static [u32],
    pub dword_failures: &'static [u32],
    pub target: StateCastVisualTarget,
}

/// Общий wire-каркас `0xBFE01` state-кастов пятёрки и heal-квартета —
/// перенос `publish_state_skill_visual` переходного `stateskill.rs`.
/// Перечень отказов, fallback и адресат — реальные игровые различия
/// владельца; базовый хвост visual исполняет внешний dispatcher, как раньше.
pub fn publish_state_cast_visual<Game: StateCastGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
    contract: &StateCastVisualContract,
) {
    publish_state_cast_visual_impl(
        game, skill, mode, contract.skill_id, contract.kind,
        contract.failures, &[], contract.target,
    );
}

/// Каркас `0xBFE01` Fury/RageBreak: отказы из `dword_failures` пишут
/// `add_long(0)` перед BYTE mode (внешний вид `[dword 0][byte 8]` вместо
/// BYTE-пары пятёрки); остальные режимы идут общим путём.
pub fn publish_rage_cast_visual<Game: StateCastGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
    contract: &RageCastVisualContract,
) {
    publish_state_cast_visual_impl(
        game, skill, mode, contract.skill_id, contract.kind,
        contract.failures, contract.dword_failures, contract.target,
    );
}

#[allow(clippy::too_many_arguments, reason = "части контракта независимы, как в исходном владельце")]
fn publish_state_cast_visual_impl<Game: StateCastGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
    skill_id: u32,
    kind: SkillVisualEffectKind,
    failures: &[u32],
    dword_failures: &[u32],
    target_kind: StateCastVisualTarget,
) {
    if skill.id() != skill_id || skill.visual_effect().is_none_or(|effect| {
        effect.kind() != kind || effect.is_ended()
    }) { return; }
    let (user_region, user) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(user_region, user).map(|shape| shape.shape()) else { return; };
    let mut message = CMessage::new(0x000b_fe01);
    if failures.contains(&mode) {
        if source.identity().object_type == PLAYER_TYPE {
            // RP-отказ Fury/RageBreak имеет DWORD-префикс; остальные
            // ошибки семейства используют BYTE даже в том же owner-е.
            if dword_failures.contains(&mode) { message.add_long(0); }
            else { message.add_byte(0); }
            message.add_byte(mode as u8);
            game.send_state_cast_visual_to_player(source.identity().id, &message);
        }
        return;
    }
    let target = match mode {
        0 => None,
        1 if matches!(target_kind, StateCastVisualTarget::User) => Some(source),
        1 => {
            let target = game.resolve_skill_sufferer(skill.lifecycle())
                .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
                .or_else(|| {
                    if matches!(target_kind, StateCastVisualTarget::SuffererOrUser) {
                        game.resolve_state_move_shape(user_region, user)
                    } else { None }
                });
            let Some(target) = target else { return; };
            Some(target.shape())
        }
        _ => return,
    };
    message.add_byte(if mode == 0 { 1 } else { 2 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
    } else {
        message.add_long(source.get_direction());
    }
    game.send_state_cast_visual_around(source.get_region_id(), source, &message);
}

/// Живой participant прежнего тела навыка: повторное разрешение фигуры и
/// снятие фактического (region, identity) — перенос локальных `participant`
/// переходных файлов.
pub fn state_cast_participant<Game: StateCastGame>(
    game: &Game,
    value: (i32, ShapeIdentity),
) -> Option<(i32, ShapeIdentity)> {
    let shape = game.resolve_state_move_shape(value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

/// Participant записи арены: регион/identity с обнулённым `ex_id`
/// (`CGuid::GUID_INVALID`), как в прежних Begin-экземпляров состояний.
pub fn state_cast_storage_participant<Game: StateCastGame>(
    game: &Game,
    value: (i32, ShapeIdentity),
) -> Option<(i32, ShapeIdentity)> {
    let shape = game.resolve_state_move_shape(value.0, value.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
}
