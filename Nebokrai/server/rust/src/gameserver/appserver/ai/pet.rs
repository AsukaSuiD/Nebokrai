//! Владелец жизненного цикла прирученного монстра `CPet`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/pet.cpp`. Контроллер сохраняет секундный поиск хозяина,
//! шестичасовой счётчик жизни, переходы режима/действия, возврат и одичание.
//! `OnMoving` живого питомца без текущего навыка ставит отдельный
//! `ASA_SEARCH_ENEMY` в общую FIFO-очередь.
//! Ближнее следование (0x004E9DA2) использует общий MoveTo(run=0): Slip,
//! OnMove, затем ASA_MOVE даже при отказе spatial-вызова. Задержка берёт
//! CMonster::GetSpeed/GetStopFrame с pet factors, а не сырую скорость CShape.
//! Общий OnMove использует runtime AREA_WIDTH/AREA_HEIGHT; отдельного
//! pet wire-пути нет. Дальний перенос остаётся самостоятельной ветвью.
//! `OnIdle` сохраняет `ChangeSkill? → Stand → SearchEnemy`. OnLoseTarget
//! 0x004E95F0 вызывает базовую очистку цели, затем для ATTACKING проверяет
//! HasTarget (+0x50), при необходимости повторяет OnLoseTarget (+0x2C),
//! и пишет FOLLOWING. Это не OnIdle (+0x48): новых событий здесь нет.
//! Базовый переход обнуляет цель, поэтому повторный вызов не требуется.
//! Исполнение навыка и Move сохраняются; внешний SearchEnemy ставит caller.
//! SetTarget (0x004E9630) меняет FOLLOWING на ATTACKING и передаёт пару цели
//! в CBaseAI::SetTarget (0x004C7C60). Текущий cast и Move не отменяются:
//! цель уже начатого навыка остаётся в его dispatch, новая цель принадлежит
//! расписанию AI. Команда атаки не является вызовом End.
//! SetPetCurrentAction (0x004E94B0) вызывает OnLoseTarget только для нового
//! FOLLOWING при HasTarget, затем записывает действие. STAYING не очищает
//! цель; ни одна из этих команд не отменяет cast или активное движение.
//! Числовой PET_MODE: 0 — активный (OnSearchEnemy 0x004E98CD), 1 —
//! защитный, 2 — пассивный (OnBeenHurted 0x004E94F9). OnSchedule при потере
//! хозяина в безопасной клетке выбирает 2, вне неё — 1; прежний режим 0
//! вызывает OnLoseTarget (0x004EA134). Возврат хозяина также проверяет 0
//! (0x004E9FB2). Эти переходы сохраняют текущие cast и Move.
//! SetPetCurrentAIMode (0x004E9460) перед записью режима вызывает OnLoseTarget,
//! если новый режим пассивный либо прежний активный; HasTarget не проверяет.
//! Затем обнуляет invalid_master_ms (+0x84) и seek_master_ms (+0x88),
//! не затрагивая шестичасовой счётчик и master_logout.
//! OnIdle (RVA 0x000E9580) проверяет GetCurrentSkill, а не один выбранный ID:
//! если зарегистрированного навыка нет, ChangeSkill сохраняется в начале FIFO.
//! OnFallowingSchedule (0x004E9BB0) требует пустые active/passive очереди и
//! вызывается до background/passive. Ближняя ветвь +0x58 (0x004E9DA2) — общий
//! MoveTo: один Slip-шаг с ASA_MOVE, а не прямой перенос к слоту хозяина.
//! Дальний перенос остаётся отдельной ветвью; последующий OnIdle не повторяет
//! следование и ставит собственные Stand/SearchEnemy.
//! Секундный/lifecycle-хвост OnSchedule (0x004E9E4E) выполняется после ветви
//! действия и до background/passive даже при занятых FIFO. Ожидание Move или
//! атаки не останавливает проверку хозяина, одичание и срок жизни питомца.
//! OnSearchEnemy вызывается только достигнутым active-событием: OnIdle
//! ставит SearchEnemy после Stand, но не выполняет поиск немедленно. Отдельной
//! idle-ветви поиска в CPet::OnSchedule (0x004E9DC0) нет; Rust не обходит FIFO.
//! Поиск живых владельцев, пространственное перемещение, пакеты и удаление
//! остаются у `CGame`; состояние хранится ровно один раз внутри `CMonster`.
//! Runtime-входы выбираются по GetAI == CPet, а не по tamed sign:
//! CMonster::GetAI (0x004E6D80) проверяет identity хозяина и auxiliary owner.
//! Прямые команды сохранённого pet-list по-прежнему адресуют m_pPetAI.
//! `GetPetMaster` разрешает игрока глобально, а остальные типы — только через
//! реестр текущего региона; эта же typed-развилка используется унаследованной
//! повозкой и боевым ограничением преследования.
//! Конструктор (0x004E9400, записи 0x004E942E/0x004E9435) задаёт числовые
//! mode=2 и action=1 при нулевых lifecycle-таймерах. Символьное имя PSEM_DEFENSE
//! в RAW не подменяет подтверждённое значение режима. Прямые m_pPetAI-команды
//! читают свою цель/FIFO даже тогда, когда GetAI выбирает первичный контроллер.
//!
//! Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
//! Декомпилятор: Ghidra 12.1.2
//! Встречный сброс цели при отказе от атаки сопоставлен с `GetAI` цели и
//! виртуальным `OnLoseTarget`; сохранены остальные ещё не сопоставленные
//! боевые и событийные ветви `CPet`.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::ai::aifactory::ActiveMonsterAi;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const PET_MODE_ACTIVE: i32 = 0;
const PET_MODE_DEFENSIVE: i32 = 1;
const PET_MODE_PASSIVE: i32 = 2;

const SEEK_MASTER_INTERVAL_MS: u32 = 1_000;
const LIFE_CYCLE_INTERVAL_MS: u32 = 21_600_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PetLifecycleState {
    seek_master_ms: u32,
    life_cycle_ms: u32,
    life_cycle_counter: u32,
    invalid_master_ms: u32,
    master_logout: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleFacts {
    pub(crate) now_ms: u32,
    pub(crate) wild_time_ms: u32,
    pub(crate) master_present: bool,
    pub(crate) master_close: bool,
    pub(crate) safe_cell: bool,
    pub(crate) reclaimable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PetLifecycleNotice {
    AgeWarning,
    AgeExpired,
    BecameWild,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PetLifecycleOutcome {
    pub(crate) notice: Option<PetLifecycleNotice>,
    pub(crate) reclaim: bool,
    pub(crate) vanish: bool,
    pub(crate) clear_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PetMasterRef {
    Player(i32),
    Region(ShapeIdentity),
}

/// Безопасный эквивалент `CPet::GetPetMaster`: нулевой master не разрешается,
/// игрок ищется глобально, остальные типы остаются привязаны к текущему
/// `CServerRegion`.
pub(crate) const fn pet_master_ref(master: MasterInfo) -> Option<PetMasterRef> {
    if master.master_type == 0 || master.master_id == 0 {
        None
    } else if master.master_type == PLAYER_TYPE {
        Some(PetMasterRef::Player(master.master_id))
    } else {
        Some(PetMasterRef::Region(ShapeIdentity {
            object_type: master.master_type,
            id: master.master_id,
            ex_id: CGuid::GUID_INVALID,
        }))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PetLifecycleTransition {
    notice: Option<PetLifecycleNotice>,
    reclaim: bool,
    vanish: bool,
    action: Option<i32>,
    mode: Option<i32>,
    clear_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PetBehaviorState {
    mode: i32,
    action: i32,
    lifecycle: PetLifecycleState,
}

impl Default for PetBehaviorState {
    fn default() -> Self {
        Self {
            mode: PET_MODE_PASSIVE,
            action: 1,
            lifecycle: PetLifecycleState::default(),
        }
    }
}

impl PetBehaviorState {
    pub(crate) const fn mode(self) -> i32 {
        self.mode
    }

    pub(crate) const fn set_mode(&mut self, mode: i32) {
        self.mode = mode;
        self.lifecycle.invalid_master_ms = 0;
        self.lifecycle.seek_master_ms = 0;
    }

    pub(crate) const fn mode_change_releases_target(&self, mode: i32) -> bool {
        mode == PET_MODE_PASSIVE || self.mode == PET_MODE_ACTIVE
    }

    pub(crate) const fn action(self) -> i32 {
        self.action
    }

    pub(crate) const fn set_action(&mut self, action: i32) {
        self.action = action;
    }

    pub(crate) const fn begin_target(&mut self) {
        if self.action == 1 {
            self.action = 0;
        }
    }

    pub(crate) fn retarget_passive(&mut self, tamed: bool, has_target: bool) -> bool {
        if !tamed || self.mode != PET_MODE_DEFENSIVE || has_target {
            return false;
        }
        if self.action == 1 {
            self.action = 0;
        }
        true
    }

    pub(crate) fn on_hurt(
        &mut self,
        current_target: Option<ShapeIdentity>,
        attacker: ShapeIdentity,
    ) -> bool {
        let replace = self.mode != PET_MODE_PASSIVE
            && (current_target.is_none()
                || current_target.is_some_and(|target| {
                    target.object_type != PLAYER_TYPE && attacker.object_type == PLAYER_TYPE
                }));
        if replace && self.action == 1 {
            self.action = 0;
        }
        replace
    }

    pub(crate) fn begin_ai_target(&mut self, tamed: bool) {
        if tamed && self.action == 1 {
            self.action = 0;
        }
    }

    pub(crate) fn target_cleared(&mut self, tamed: bool) {
        if tamed && self.action == 0 {
            self.action = 1;
        }
    }

    pub(crate) fn tick(
        &mut self,
        facts: PetLifecycleFacts,
        has_target: bool,
    ) -> PetLifecycleOutcome {
        let transition = self.lifecycle.tick(facts, self.mode, has_target);
        if let Some(mode) = transition.mode {
            self.mode = mode;
        }
        if let Some(action) = transition.action {
            self.action = action;
        }
        PetLifecycleOutcome {
            notice: transition.notice,
            reclaim: transition.reclaim,
            vanish: transition.vanish,
            clear_target: transition.clear_target,
        }
    }
}

impl PetLifecycleState {
    pub(crate) fn tick(
        &mut self,
        facts: PetLifecycleFacts,
        current_mode: i32,
        has_target: bool,
    ) -> PetLifecycleTransition {
        let mut outcome = PetLifecycleTransition::default();
        if self.seek_master_ms != 0
            && facts.now_ms.wrapping_sub(self.seek_master_ms) < SEEK_MASTER_INTERVAL_MS
        {
            return outcome;
        }
        if self.life_cycle_ms == 0 {
            self.life_cycle_ms = facts.now_ms;
        }
        if facts.now_ms.wrapping_sub(self.life_cycle_ms) >= LIFE_CYCLE_INTERVAL_MS {
            self.life_cycle_counter = self.life_cycle_counter.wrapping_add(1);
            self.life_cycle_ms = facts.now_ms;
            if self.life_cycle_counter < 4 {
                if facts.master_present {
                    outcome.notice = Some(PetLifecycleNotice::AgeWarning);
                }
            } else {
                if facts.master_present {
                    outcome.notice = Some(PetLifecycleNotice::AgeExpired);
                }
                outcome.vanish = true;
                return outcome;
            }
        }
        self.seek_master_ms = facts.now_ms;
        if !facts.master_present {
            self.master_logout = true;
            if self.invalid_master_ms == 0 {
                outcome.action = Some(2);
                outcome.clear_target = facts.safe_cell || current_mode == PET_MODE_ACTIVE;
                outcome.mode = Some(if facts.safe_cell { PET_MODE_PASSIVE } else { PET_MODE_DEFENSIVE });
                self.seek_master_ms = 0;
                self.invalid_master_ms = facts.now_ms;
            }
        } else {
            if self.master_logout && facts.reclaimable {
                outcome.clear_target = current_mode == PET_MODE_ACTIVE || has_target;
                self.invalid_master_ms = 0;
                self.seek_master_ms = 0;
                outcome.mode = Some(PET_MODE_DEFENSIVE);
                outcome.action = Some(1);
                self.master_logout = false;
                outcome.reclaim = true;
            }
            if facts.master_close {
                self.invalid_master_ms = 0;
                return outcome;
            }
            if self.invalid_master_ms == 0 {
                self.invalid_master_ms = facts.now_ms;
            }
        }
        if self.invalid_master_ms != 0
            && facts.now_ms.wrapping_sub(self.invalid_master_ms) >= facts.wild_time_ms
        {
            if facts.master_present {
                outcome.notice = Some(PetLifecycleNotice::BecameWild);
            }
            outcome.vanish = true;
        }
        outcome
    }
}

/// Выбирает ближайшего дикого монстра для активного питомца. Равная дальность
/// сохраняет исходное правило: побеждает более поздняя запись обхода региона.
pub(crate) fn execute_owned_pet_active_search(
    game: &CGame,
    region: &mut CServerRegion,
    region_id: i32,
    monster_id: i32,
) -> bool {
    let Some(master) = region.find_monster_by_id(monster_id).and_then(|pet| {
        (matches!(pet.active_ai(), Some(ActiveMonsterAi::Pet))
            && pet.pet_mode() == PET_MODE_ACTIVE
            && pet.ai_target().is_none())
            .then_some(pet.master_info())
    }) else {
        return false;
    };
    if master.master_type != PLAYER_TYPE || master.master_id == 0 {
        return false;
    }
    let Some((master_view, master_badman, area_index)) = game
        .find_player(master.master_id)
        .filter(|player| player.server_region_id() == Some(region_id))
        .and_then(|player| {
            Some((
                player.shape_view()?,
                player.is_badman(game.globe_setup().pk_count_per_kill()),
                player.shape().area_index()?,
            ))
        })
    else {
        return false;
    };

    let mut selected = None;
    let mut selected_distance = i32::MAX;
    for candidate_id in region.monster_ids_around_area(area_index) {
        if candidate_id == monster_id {
            continue;
        }
        let Some(candidate) = region
            .find_monster_by_id(candidate_id)
            .filter(|candidate| {
                !candidate.is_tamed() && !CMoveShape::is_died(candidate.hit_points())
            })
            .and_then(|candidate| {
                let property =
                    game.find_monster_property_by_origin_name(candidate.base_property_key()?)?;
                (!(property.tamable == 1 && property.maximum_tame_attempt_count == 0)
                    && (property.kind != 5 || master_badman))
                    .then(|| candidate.shape_view(property))?
            })
        else {
            continue;
        };
        let distance = master_view.real_distance(Some(candidate));
        if distance <= 10 && distance <= selected_distance {
            selected = Some(candidate.identity);
            selected_distance = distance;
        }
    }
    if let Some(target) = selected
        && let Some(pet) = region.find_monster_by_id_mut(monster_id)
    {
        pet.set_ai_target(target);
        return true;
    }
    false
}

/// Ставит точный `CPet::OnIdle` без случайного движения. Каждый исходный
/// `AddAIEvent` получает отдельный замер часов.
pub(crate) fn queue_pet_idle<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    stop_frame: u32,
    factory: &CSkillFactory,
    runtime: &mut Runtime,
) -> bool {
    let Some((alive, has_skill)) = region.find_monster_by_id(monster_id)
        .filter(|pet| matches!(pet.active_ai(), Some(ActiveMonsterAi::Pet)))
        .map(|pet| {
            (
                !CMoveShape::is_died(pet.hit_points()),
                pet.move_shape().current_skill(factory).is_some(),
            )
        }) else {
        return false;
    };
    if !alive {
        return true;
    }
    let Some(pet) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    if !has_skill {
        pet.begin_active_ai_change_skill(runtime.now_milliseconds());
    }
    pet.begin_active_ai_stand(stop_frame, runtime.now_milliseconds());
    pet.begin_active_ai_search_enemy(runtime.now_milliseconds());
    true
}

/// Выполняет `CPet::OnLoseTarget` и отдельный SearchEnemy его schedule-caller-а.
pub(crate) fn lose_pet_target_and_search<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) {
    release_pet_target(region, monster_id);
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.begin_active_ai_search_enemy(runtime.now_milliseconds());
    }
}

/// Чистый virtual CPet::OnLoseTarget, общий для расписания и death FIFO.
pub(crate) fn release_pet_target(
    region: &mut CServerRegion,
    monster_id: i32,
) {
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.release_pet_ai_target();
    }
}

/// Исполняет достигнутое следование `CPet`: слот питомца задаёт позицию позади
/// хозяина, близкая цель достигается обычным шагом, а далёкая — переносом в
/// свободную клетку `7x7` с тем же порядком пространственной доставки.
pub(crate) fn execute_owned_pet_follow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    region_id: i32,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some((pet_shape, pet_health, master, moveable, property)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            if !matches!(monster.active_ai(), Some(ActiveMonsterAi::Pet))
                || monster.pet_action() != 1
                || !monster.primary_ai_queues_idle()
            {
                return None;
            }
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                monster.move_shape().shape().clone(),
                monster.hit_points(),
                monster.master_info(),
                monster.move_shape().is_moveable(),
                property,
            ))
        })
    else {
        return false;
    };
    if CMoveShape::is_died(pet_health) || !moveable {
        return true;
    }
    if master.master_type != PLAYER_TYPE || master.master_id == 0 {
        return true;
    }
    let Some((master_shape, pet_index)) = game.find_player(master.master_id).and_then(|player| {
        (player.server_region_id() == Some(region_id)).then(|| {
            let index = player
                .active_pets()
                .iter()
                .position(|pet| pet.object_type == MONSTER_TYPE && pet.id == monster_id)?;
            Some((player.shape().clone(), index))
        })?
    }) else {
        return true;
    };
    let (Ok(pet_x), Ok(pet_y), Ok(master_x), Ok(master_y), Ok(rear)) = (
        pet_shape.get_tile_x(),
        pet_shape.get_tile_y(),
        master_shape.get_tile_x(),
        master_shape.get_tile_y(),
        master_shape.get_rear_direction(),
    ) else {
        return true;
    };
    let mut destination = ShapeAreaCoordinates {
        x: master_x,
        y: master_y,
    };
    for _ in 0..(pet_index / 3 + 1) {
        let Ok(next) = CShape::get_direction_position(rear, destination) else {
            return true;
        };
        let Ok(next) = CShape::get_direction_position(rear, next) else {
            return true;
        };
        destination = next;
    }
    let side = match pet_index % 3 {
        1 => master_shape.get_left_direction().ok(),
        2 => master_shape.get_right_direction().ok(),
        _ => None,
    };
    if let Some(side) = side {
        let Ok(next) = CShape::get_direction_position(side, destination) else {
            return true;
        };
        let Ok(next) = CShape::get_direction_position(side, next) else {
            return true;
        };
        destination = next;
    }
    if (pet_x, pet_y) == (destination.x, destination.y) {
        return true;
    }
    if !game.spatial_delivery_ready() {
        return true;
    }
    let figure = CMonster::figure(&property);
    if (real_distance(pet_x, pet_y, master_x, master_y) as f32)
        <= game.globe_setup().pet_translate_distance()
    {
        super::monsterai::move_owned_monster_to(game, region, monster_id, destination, 0,
            || runtime.now_milliseconds());
        return true;
    }
    let Ok(position) = region.region.get_random_pos_in_range(
        master_x.wrapping_sub(3),
        master_y.wrapping_sub(3),
        7,
        7,
        runtime,
    ) else {
        return true;
    };
    if position.found {
        let _ = game.set_owned_pet_position(
            region,
            monster_id,
            position.x,
            position.y,
            figure,
        );
    }
    true
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp

// ============================================================================
// FUNCTION: CPet::OnStayingSchedule
// STATUS: IMPLEMENTED
// IMPLEMENTED: target-loss, выбор навыка, точный диапазон без движения,
// `Begin`, FIFO `ATTACK/SEARCH_ENEMY` и встречный `GetAI → OnLoseTarget`
// проходят через monsterbaseattack.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:399
// RVA: 0x000E9650
// ADDRESS: 004e9650
// PROTOTYPE: void __thiscall OnStayingSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::OnAttackingSchedule
// STATUS: IMPLEMENTED
// IMPLEMENTED: master-centered tracing limit, target-loss, attackability,
// выбор навыка, `Begin` и FIFO `ATTACK/SEARCH_ENEMY` проходят через
// monsterbaseattack; встречный `GetAI → OnLoseTarget` сохраняет derived
// переход цели.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:235
// RVA: 0x000E9A20
// ADDRESS: 004e9a20
// PROTOTYPE: void __thiscall OnAttackingSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
