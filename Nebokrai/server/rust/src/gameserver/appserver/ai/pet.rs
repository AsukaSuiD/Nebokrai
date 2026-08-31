//! Владелец жизненного цикла прирученного монстра `CPet`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/pet.cpp`. Контроллер сохраняет секундный поиск хозяина,
//! шестичасовой счётчик жизни, переходы режима/действия, возврат и одичание.
//! `OnMoving` живого питомца без текущего навыка ставит отдельный
//! `ASA_SEARCH_ENEMY` в общую FIFO-очередь.
//! `OnIdle` сохраняет `ChangeSkill? → Stand → SearchEnemy`, а `OnLoseTarget`
//! для атакующего питомца ставит эту очередь до внешнего повторного поиска и
//! переводит действие в `FOLLOWING`.
//! Поиск живых владельцев, пространственное перемещение, пакеты и удаление
//! остаются у `CGame`; состояние хранится ровно один раз внутри `CMonster`.
//!
//! Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
//! Декомпилятор: Ghidra 12.1.2
//! Встречный сброс цели при отказе от атаки сопоставлен с `GetAI` цели и
//! виртуальным `OnLoseTarget`; сохранены остальные ещё не сопоставленные
//! боевые и событийные ветви `CPet`.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

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
            mode: 0,
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
    }

    pub(crate) const fn action(self) -> i32 {
        self.action
    }

    pub(crate) const fn set_action(&mut self, action: i32) -> bool {
        self.action = action;
        action != 0
    }

    pub(crate) const fn begin_target(&mut self) {
        if self.action == 1 {
            self.action = 0;
        }
    }

    pub(crate) fn retarget_passive(&mut self, tamed: bool, has_target: bool) -> bool {
        if !tamed || self.mode != 1 || has_target {
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
        let replace = self.mode != 0
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
                outcome.clear_target = facts.safe_cell || current_mode == 2;
                outcome.mode = Some(if facts.safe_cell { 0 } else { 1 });
                self.seek_master_ms = 0;
                self.invalid_master_ms = facts.now_ms;
            }
        } else {
            if self.master_logout && facts.reclaimable {
                outcome.clear_target = current_mode == 2 || has_target;
                self.invalid_master_ms = 0;
                self.seek_master_ms = 0;
                outcome.mode = Some(1);
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
    from_fifo: bool,
) -> bool {
    let Some(master) = region.find_monster_by_id(monster_id).and_then(|pet| {
        (pet.is_tamed()
            && pet.pet_mode() == 2
            && pet.ai_target().is_none()
            && (from_fifo
                || (pet.pet_action() == 1 && pet.primary_ai_queues_idle())))
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
        let distance = real_distance(
            master_view.tile_x,
            master_view.tile_y,
            candidate.tile_x,
            candidate.tile_y,
        );
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
    runtime: &mut Runtime,
) -> bool {
    let Some((alive, has_skill)) = region.find_monster_by_id(monster_id).map(|pet| {
        (
            !CMoveShape::is_died(pet.hit_points()),
            pet.move_shape().current_skill_id().is_some(),
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

/// Выполняет `CPet::OnLoseTarget` и следующий `SearchEnemy` окружающего
/// schedule-owner-а. Только действие `ATTACKING` вызывает промежуточный
/// `OnIdle`; `clear_ai_target` канонически переводит его в `FOLLOWING`.
pub(crate) fn lose_pet_target_and_search<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    stop_frame: u32,
    runtime: &mut Runtime,
) {
    let was_attacking = region
        .find_monster_by_id(monster_id)
        .is_some_and(|pet| pet.is_tamed() && pet.pet_action() == 0);
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.clear_ai_target();
    }
    if was_attacking {
        let _ = queue_pet_idle(region, monster_id, stop_frame, runtime);
    }
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.begin_active_ai_search_enemy(runtime.now_milliseconds());
    }
}

/// Чистый virtual `CPet::OnLoseTarget` из `OnBeenKilled`: сохранённый Move
/// остаётся в FIFO, атакующий питомец выполняет свой `OnIdle`, а внешний
/// schedule-`SearchEnemy` сюда не добавляется.
pub(crate) fn release_pet_target_for_death<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    stop_frame: u32,
    runtime: &mut Runtime,
) {
    let was_attacking = region
        .find_monster_by_id(monster_id)
        .is_some_and(|pet| pet.is_tamed() && pet.pet_action() == 0);
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.release_ai_target_for_death();
    }
    if was_attacking {
        let _ = queue_pet_idle(region, monster_id, stop_frame, runtime);
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
            if !monster.is_tamed() || monster.pet_action() != 1 {
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
        let _ = game.move_owned_pet_step(
            region,
            monster_id,
            destination.x,
            destination.y,
            figure,
        );
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
// FUNCTION: CPet::CPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:19
// RVA: 0x000E9400
// ADDRESS: 004e9400
// PROTOTYPE: undefined __thiscall CPet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPet::~CPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\pet.cpp:23
// RVA: 0x000E9450
// ADDRESS: 004e9450
// PROTOTYPE: void __thiscall ~CPet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
