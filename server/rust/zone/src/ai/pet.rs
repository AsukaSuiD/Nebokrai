//! Жизненный цикл и боевые расписания приручённого монстра `CPet`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match). Исходный
//! владелец PDB: `server/gameserver/appserver/ai/pet.cpp`.
//!
//! Машинная база кластера A1 (VERIFIED по этой паре):
//!
//! - `CPet::OnSchedule` (RVA `0x0E9DC0`): выбирает собственные ветви по режиму
//!   `[+0x80]` — 0 = Attack, 1 = Follow (`OnFallowingSchedule` RVA `0x0E9BB0`),
//!   2 = Stay (`OnStayingSchedule` RVA `0x0E9650`); watchdog мастера —
//!   `Distance(master) > 32` клеток вместе с глобалом `0xEF44E8` → `GS0012` и
//!   `Evanish` (vt `+0x188`); авто-отзыв при полных слотах
//!   (`CheckSkill(0xD4)`/`AddPet`, `GS0011`); шестичасовые
//!   (`0x1499700` мс) уведомления о сроке жизни со счётчиком `>= 4`;
//!   master разрешается `dynamic_cast<CPlayer*>`, region —
//!   `dynamic_cast<CServerRegion*>`.
//! - `CPet::OnAttackingSchedule` (RVA `0x0E9A20`): дистанционный гейт
//!   `CShape::Distance <` BSS `0xEF44EC` от мастера (при его отсутствии — от
//!   питомца); отказ `IsAttackAble` живой цели: если AI цели целится в самого
//!   питомца — встречный virtual `OnLoseTarget` цели, затем свой
//!   `OnLoseTarget + SearchEnemy`; БЕЗ GetAtcInterval-гейта — Tracing/диапазон
//!   и сразу Begin.
//! - `CPet::OnStayingSchedule` (RVA `0x0E9650`) не вызывает Tracing:
//!   включительный min/max диапазон текущего навыка проверяет общий dispatcher
//!   перед Begin, при выходе — `OnLoseTarget → SearchEnemy`.
//! - Lifecycle-хвост OnSchedule (RVA `0x0E9E4E`) выполняется после ветви
//!   действия и до background/passive даже при занятых FIFO; секундный поиск
//!   мастера (`0x3E8` мс) и одичание не останавливаются ожиданием Move/атаки.
//!
//! Честные UNKNOWN этой порции: точная семантика BSS-глобалов питомца
//! (`0xEF44E8` watchdog-флаг/условие слежения и `0xEF44EC` предел
//! преследования — значения записываются внешним lifecycle `CGame`, здесь
//! приходят как факты `PetLifecycleFacts` и distance-гейт hub-владельца);
//! конкретная нить вызова `Evanish` и отзыва из watchdog (выполняет hub по
//! результату `tick`).
//!
//! Объявленные швы переноса (не расхождения): hub-фасады семейства
//! `MonsterDispatcher*` (`ai/monsterai.rs`) покрывают пространство региона,
//! слот следования игрока, перенос и часы. Часы каждого события читаются
//! отдельным вызовом `now_milliseconds` (fn-параметр делегата).
//! Случайная свободная клетка 7x7 читается общим `CRegion::get_random_pos_in_range`
//! с исходным `RegionRandomContext` старого main loop; ближний шаг остаётся
//! общим `MoveTo(run = 0)`.
//!
//! Сохранённые доказанные контракты старого хозяина (комментарии перенесены
//! вместе с телами): `OnMoving` живого питомца без текущего навыка ставит
//! отдельный `ASA_SEARCH_ENEMY` в общую FIFO; `OnIdle` сохраняет
//! `ChangeSkill? → Stand → SearchEnemy`; `OnLoseTarget` (RVA `0x0E95F0`)
//! вызывает базовую очистку цели, затем для ATTACKING проверяет HasTarget
//! (+0x50), при необходимости повторяет OnLoseTarget (+0x2C) и пишет
//! FOLLOWING. `SetTarget` (RVA `0x0E9630`) меняет FOLLOWING на ATTACKING и
//! передаёт пару цели в `CBaseAI::SetTarget` (0x004C7C60); текущий cast и
//! Move не отменяются. `SetPetCurrentAction` (RVA `0x0E94B0`) вызывает
//! OnLoseTarget только для нового FOLLOWING при HasTarget; STAYING цель не
//! очищает. Числовой PET_MODE: 0 — активный (OnSearchEnemy RVA `0x0E98CD`),
//! 1 — защитный, 2 — пассивный (OnBeenHurted RVA `0x0E94F9`).
//! `SetPetCurrentAIMode` (RVA `0x0E9460`) перед записью режима вызывает
//! OnLoseTarget, если новый режим пассивный либо прежний активный, затем
//! обнуляет invalid_master_ms (+0x84) и seek_master_ms (+0x88), не затрагивая
//! шестичасовой счётчик и master_logout. Конструктор (RVA `0x0E9400`, записи
//! 0x0E942E/0x0E9435) задаёт mode = 2 и action = 1 при нулевых
//! lifecycle-таймерах. Runtime-входы выбираются по `CMonster::GetAI`
//! (0x004E6D80), а не по знаку приручения.

use nebokrai_shared::values::CGuid;

use crate::combat::MasterInfo;
use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::region::RegionRandomContext;
use crate::regions::shape::{CShape, ShapeAreaCoordinates};
use crate::skills::skillfactory::CSkillFactory;

use super::monsterai::{
    MonsterActiveAiView, MonsterDispatcherGame, MonsterDispatcherMonster,
    MonsterDispatcherMoveShape, MonsterDispatcherPlayer, MonsterDispatcherRegion,
    move_owned_monster_to,
};

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
pub struct PetLifecycleFacts {
    pub now_ms: u32,
    pub wild_time_ms: u32,
    pub master_present: bool,
    pub master_close: bool,
    pub safe_cell: bool,
    pub reclaimable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PetLifecycleNotice {
    AgeWarning,
    AgeExpired,
    BecameWild,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PetLifecycleOutcome {
    pub notice: Option<PetLifecycleNotice>,
    pub reclaim: bool,
    pub vanish: bool,
    pub clear_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PetMasterRef {
    Player(i32),
    Region(ShapeIdentity),
}

/// Безопасный эквивалент `CPet::GetPetMaster`: нулевой master не разрешается,
/// игрок ищется глобально, остальные типы остаются привязаны к текущему
/// `CServerRegion`.
pub const fn pet_master_ref(master: MasterInfo) -> Option<PetMasterRef> {
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
pub struct PetBehaviorState {
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
    pub const fn mode(self) -> i32 {
        self.mode
    }

    pub const fn set_mode(&mut self, mode: i32) {
        self.mode = mode;
        self.lifecycle.invalid_master_ms = 0;
        self.lifecycle.seek_master_ms = 0;
    }

    pub const fn mode_change_releases_target(&self, mode: i32) -> bool {
        mode == PET_MODE_PASSIVE || self.mode == PET_MODE_ACTIVE
    }

    pub const fn action(self) -> i32 {
        self.action
    }

    pub const fn set_action(&mut self, action: i32) {
        self.action = action;
    }

    pub const fn begin_target(&mut self) {
        if self.action == 1 {
            self.action = 0;
        }
    }

    pub fn retarget_passive(&mut self, tamed: bool, has_target: bool) -> bool {
        if !tamed || self.mode != PET_MODE_DEFENSIVE || has_target {
            return false;
        }
        if self.action == 1 {
            self.action = 0;
        }
        true
    }

    pub fn on_hurt(
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

    pub fn begin_ai_target(&mut self, tamed: bool) {
        if tamed && self.action == 1 {
            self.action = 0;
        }
    }

    pub fn target_cleared(&mut self, tamed: bool) {
        if tamed && self.action == 0 {
            self.action = 1;
        }
    }

    pub fn tick(
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
    fn tick(
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
pub fn execute_owned_pet_active_search<Game, Region>(
    game: &Game,
    region: &mut Region,
    region_id: i32,
    monster_id: i32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
{
    let Some(master) = region.find_monster_by_id(monster_id).and_then(|pet| {
        (matches!(pet.active_ai_view(), Some(MonsterActiveAiView::Pet))
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
                !candidate.is_tamed() && !is_died(candidate.hit_points())
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
pub fn queue_pet_idle<Region>(
    region: &mut Region,
    monster_id: i32,
    stop_frame: u32,
    factory: &CSkillFactory,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Region: MonsterDispatcherRegion,
{
    let Some((alive, has_skill)) = region.find_monster_by_id(monster_id)
        .filter(|pet| matches!(pet.active_ai_view(), Some(MonsterActiveAiView::Pet)))
        .map(|pet| {
            (
                !is_died(pet.hit_points()),
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
        pet.begin_active_ai_change_skill(now_milliseconds());
    }
    pet.begin_active_ai_stand(stop_frame, now_milliseconds());
    pet.begin_active_ai_search_enemy(now_milliseconds());
    true
}

/// Выполняет `CPet::OnLoseTarget` и отдельный SearchEnemy его schedule-caller-а.
pub fn lose_pet_target_and_search<Region>(
    region: &mut Region,
    monster_id: i32,
    now_milliseconds: fn() -> u32,
)
where
    Region: MonsterDispatcherRegion,
{
    release_pet_target(region, monster_id);
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.begin_active_ai_search_enemy(now_milliseconds());
    }
}

/// Чистый virtual CPet::OnLoseTarget, общий для расписания и death FIFO.
pub fn release_pet_target<Region>(
    region: &mut Region,
    monster_id: i32,
)
where
    Region: MonsterDispatcherRegion,
{
    if let Some(pet) = region.find_monster_by_id_mut(monster_id) {
        pet.release_pet_ai_target();
    }
}

/// Исполняет достигнутое следование `CPet`: слот питомца задаёт позицию позади
/// хозяина, близкая цель достигается обычным шагом, а далёкая — переносом в
/// свободную клетку `7x7` с тем же порядком пространственной доставки.
pub fn execute_owned_pet_follow<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    region_id: i32,
    monster_id: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: super::monsterai::MonsterDispatcherOwner<Region = Region>,
    Runtime: RegionRandomContext,
{
    let Some((pet_shape, pet_health, master, moveable, property)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            if !matches!(monster.active_ai_view(), Some(MonsterActiveAiView::Pet))
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
    if is_died(pet_health) || !moveable {
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
    let figure = <Region::Monster as MonsterDispatcherMonster>::figure_for(&property);
    if (crate::regions::shape::real_distance_between_points(pet_x, pet_y, master_x, master_y) as f32)
        <= game.globe_setup().pet_translate_distance()
    {
        move_owned_monster_to(game, region, monster_id, destination, 0, now_milliseconds);
        return true;
    }
    let Ok(position) = region.base_region().get_random_pos_in_range(
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
