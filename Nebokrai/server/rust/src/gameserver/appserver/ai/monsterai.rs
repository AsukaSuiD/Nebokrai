//! ИИ обычного монстра GameServer.
//!
//! Точная `CMonsterAI::SelectAttackSkill` из `gameserver.exe/.pdb` делает один
//! вызов `random(10000)`, проходит список навыков в исходном порядке и выбирает
//! первый ID, для которого бросок не больше накопленной суммы `odds`. Если
//! сумма не покрыла бросок, назначается стандартная атака владельца.
//! `OnChangeSkill` сохраняет выбранный concrete skill только после точной
//! проверки `CSkill::IsRestored`, иначе вызывает
//! `SetCurrentSkill(GetDefaultAttackSkillID())`; AI5/AI103 вместо отката ждут
//! полный restore delay в собственном FIFO.
//! Выбранный ID хранится каноническим `CMoveShape::current_skill_id`; конкретный
//! владелец навыка разрешает уровень и исполняет стадии. Общий достигнутый шаг
//! преследования сохраняет slip-порядок, задержку движения и ограничения
//! питомца без дополнительного RNG.
//! `CBaseAI::MoveTo` (baseai.cpp, 0x004C9020) обслуживает общий spatial/FIFO
//! путь: один Slip для ходьбы, два для бега с исходным направлением. После
//! Move timestamp берётся заново; старое время начала такта из caller-а
//! не используется. Параметры навыка и выбор назначения остаются у caller-а.
//! Реакция `WhenBeenHurted` назначает новую
//! цель только свободному ИИ: игрок принимается всегда, а монстр — только после
//! подтверждения приручения. `HasTarget` считает object-целью только пару со
//! строго положительными типом объекта и ID; отрицательные legacy-значения
//! могут храниться, но расписание их целью не считает. Timestamp
//! попытки атаки принадлежит расписанию ИИ и не подменяет отдельные reuse-таймеры
//! установленных навыков; его DWORD deadline сохраняет исходное раннее
//! срабатывание рядом с переполнением часов. OnSchedule 0x005DD044 берёт
//! только младшие 16 бит интервала; собственные расписания стационарных
//! лучников, SmartGladiator, JiuMai и боссов не используют этот таймер.
//! Общая политика допуска обслуживает все concrete навыки. Default-ветвь
//! `CAIFactory::CreateAI` также проходит
//! этот общий runtime как обычный `CMonsterAI`, сохраняя исходный `ai_type`.
//! Остальные AI-ветви ниже остаются RAW.
//! Внешний virtual `Attack(skill, target)` не запускает навык: он только
//! передаёт identity в `SetTarget`; client-команда и `CPetsControl` проводят
//! этот контракт через ordered список питомцев игрока.
//! CPet::OnStayingSchedule (0x004E9650) не вызывает Tracing: его диапазон
//! проверяет общий dispatcher перед Begin. Поэтому помощник преследования
//! в Stay ничего не делает и не добавляет проверку прямого пути вместо Begin.
//! Ветвь CPet в Tracing и наличии OnSearchEnemy выбирает текущий GetAI,
//! а не tamed sign; пустой GetAI не начинает новый шаг преследования.
//! Та же форма без Tracing у стационарного OnSchedule 0x0060B890:
//! общий признак владельца задаёт диапазон перед Begin, отсутствие движения
//! и дополнительного таймера; CheckCast конкретного навыка остаётся отдельным.
//! Virtual OnLoseTarget разрешается через CMonster::GetAI, а не повторный
//! поиск MonsterProperties. Бой и смерть используют одну диспетчеризацию:
//! CPet (0x004E95F0), мечевые охранники (0x0060D020), JiuMai (0x0060A990)
//! сохраняют свои побочные эффекты поверх очистки цели CMonsterAI
//! (0x005DCC30). Следующее событие FIFO и отмена исполнения не входят сюда.
//! OnStiffen (0x004C8770) вызывает этот же virtual после каждого снятого
//! Attack, до default и продолжения active FIFO; timestamp пассивного события
//! читается после всего обработчика. Подтверждённая политика End(4) находится
//! у CMonster: она сохраняет выбранный навык до callback, обновляет reuse и
//! выполняет конкретную очистку, не снимая наложенные состояния. Для
//! неподключённого concrete End(4) Attack и исполнение сохраняются;
//! расширенная отмена CMonster не подменяет отсутствующие End и IsEnded.
//! Эта ветвь требует восстановления owner-а, а не означает успешный End.
//! End разделён вокруг синхронной доставки: очистка concrete owner-а → его
//! эффект → reuse и снятие Attack → OnLoseTarget. LittleStar (0x005355F0)
//! посылает action 3 на этой границе; это не отложенная очередь эффектов.
//! Отказ Begin — отдельный результат owner-а, не ожидание и не End AI.
//! Расписание после него вызывает virtual OnLoseTarget, затем ставит
//! SearchEnemy с новым timestamp: CMonsterAI 0x005DD07F (и thunk
//! 0x0060AF50), стационарный 0x0060B979, SmartGladiator 0x006107BF,
//! JiuMai 0x0060ACDA, BossBlue 0x0060A0A4, BossFiend 0x00609643,
//! CPet Stay/Attack 0x004E979A/0x004E9B94. Общая граница не повторяет End,
//! не отменяет Move и не фильтрует SearchEnemy по наличию тела обработчика.
//! Явный результат подключён к MonsterThorn, MonsterRangeAttack и общей группе широких атак;
//! прежний bool остальных
//! owners ещё не отличает отказ Begin от ожидания расписания.
//! Начальный Tracing для KnockOut/SpiderWeb сохраняет CBaseAI из baseai.cpp:
//! включительный диапазон, отход от слишком близкой цели и обычный MoveTo
//! без проверки прямого пути. Мечевые стражи используют свой унаследованный
//! override из cityguardwithsword.cpp, включая ForceMove; прочие concrete навыки пока
//! остаются на прежнем адаптере подхода. Часы сравнения интервала и записи
//! timestamp CMonsterAI читаются раздельно перед Begin этих двух навыков.

use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::ai::baseai::{PassiveStiffenAction, one_step_move_delay_ms};
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView,
};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterSkill;

const SLIP_ORDER: [[usize; 8]; 8] = [
    [0, 7, 1, 6, 2, 5, 3, 4],
    [1, 0, 2, 7, 3, 6, 4, 5],
    [2, 1, 3, 0, 4, 7, 5, 6],
    [3, 2, 4, 1, 5, 0, 6, 7],
    [4, 3, 5, 2, 6, 1, 7, 0],
    [5, 4, 6, 3, 7, 2, 0, 1],
    [6, 5, 7, 4, 0, 3, 1, 2],
    [7, 6, 0, 5, 1, 4, 2, 3],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterSkillCallOutcome {
    NotHandled,
    Handled,
    BeginRejected,
}

pub(crate) fn finish_monster_skill_call<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    outcome: MonsterSkillCallOutcome,
    runtime: &mut Runtime,
) -> bool {
    match outcome {
        MonsterSkillCallOutcome::NotHandled => false,
        MonsterSkillCallOutcome::Handled => true,
        MonsterSkillCallOutcome::BeginRejected => {
            release_owned_monster_target(game, region, monster_id, runtime);
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
            }
            true
        }
    }
}

pub(crate) fn process_owned_monster_stiffen<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> PassiveStiffenAction {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return PassiveStiffenAction::None;
    };
    let action = monster.begin_reached_stiffen_action();
    if action.interrupts_attack() {
        while let Some((release_target, ended_skill)) = region.find_monster_by_id_mut(monster_id)
            .and_then(|monster| monster.prepare_stiffen_attack(game.skill_factory()))
        {
            if ended_skill == Some(crate::gameserver::appserver::skills::littlestar::LITTLE_STAR_SKILL_ID)
                && let Some(monster) = region.find_monster_by_id(monster_id)
                && let Some(skill) = monster.move_shape().current_skill(game.skill_factory())
            {
                crate::gameserver::appserver::skills::littlestar::send_end(
                    game, region, monster.move_shape().shape(), skill.level() as u16,
                );
            }
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.finish_stiffen_attack(ended_skill, game.skill_factory(), || runtime.now_milliseconds());
            }
            if release_target {
                release_owned_monster_target(game, region, monster_id, runtime);
            }
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.resume_stiffen_after_target_release(release_target);
            }
        }
    }
    region.find_monster_by_id_mut(monster_id).map_or(PassiveStiffenAction::None, |monster| {
        monster.finish_reached_stiffen_action(action, || runtime.now_milliseconds())
    })
}

pub(crate) fn release_owned_monster_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) {
    let Some(ai) = region.find_monster_by_id(monster_id).and_then(CMonster::active_ai) else {
        return;
    };
    match ai {
        ActiveMonsterAi::Pet => super::pet::release_pet_target(region, monster_id),
        ActiveMonsterAi::Primary(kind) if kind.has_guard_station() => {
            super::cityguardwithsword::release_guard_sword_target(game, region, monster_id, runtime)
        }
        ActiveMonsterAi::Primary(MonsterAiKind::JiuMai) => {
            let _ = super::jiumai::release_jiumai_target(region, monster_id);
        }
        _ => {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.release_ai_target_for_death();
            }
        }
    }
}

/// Точный одноклеточный `Slip` общего `CBaseAI::MoveTo`: желаемое
/// направление и семь обходных направлений проверяются в legacy-порядке
/// против figure-specific move-check клеток региона.
fn find_slip_step_in_direction(
    game: &CGame,
    region: &CServerRegion,
    origin: ShapeAreaCoordinates,
    desired_direction: i32,
    figure: crate::gameserver::appserver::shape::ShapeFigure,
) -> Option<(i32, ShapeAreaCoordinates)> {
    let figure_index = usize::from(figure.get(0).min(2));
    SLIP_ORDER[desired_direction as usize]
        .into_iter()
        .find_map(|direction| {
            let destination = CShape::get_direction_position(direction as i32, origin).ok()?;
            let cells = game.move_check_cells().get(figure_index, direction)?;
            cells
                .iter()
                .all(|cell| {
                    region
                        .region
                        .get_block(
                            origin.x.wrapping_add(cell.x),
                            origin.y.wrapping_add(cell.y),
                        )
                        .is_ok_and(|block| block == 0)
                })
                .then_some((direction as i32, destination))
        })
}

/// MoveTo (0x004C9020): один Slip для ходьбы, два для ненулевого run,
/// затем Move и FIFO. Второй Slip сохраняет исходное желаемое направление;
/// его отказ не публикует даже первый шаг. Задержка зависит от направления
/// между исходной и окончательной клетками, без удвоения при беге.
/// Отказ вызывает пустой CMoveShape::OnCannotMove (+0xA4, 0x00485540)
/// и не меняет очередь.
pub(crate) fn move_owned_monster_to(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeAreaCoordinates,
    run: i32,
    now: impl FnOnce() -> u32,
) {
    let Some((origin, figure, speed, stop_frame)) = region.find_monster_by_id(monster_id)
        .and_then(|monster| {
            if !monster.move_shape().is_moveable() { return None; }
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let shape = monster.move_shape().shape();
            Some((ShapeAreaCoordinates { x: shape.get_tile_x().ok()?, y: shape.get_tile_y().ok()? },
                CMonster::figure(property),
                monster.speed(),
                monster.stop_frame(property)))
        })
    else { return; };
    let desired_direction = get_line_direction(origin.x, origin.y, target.x, target.y);
    let Some((_, mut destination)) = find_slip_step_in_direction(
        game, region, origin, desired_direction, figure,
    )
    else { return; };
    if run != 0 {
        let Some((_, second)) = find_slip_step_in_direction(
            game, region, destination, desired_direction, figure,
        ) else { return; };
        destination = second;
    }
    let direction = get_line_direction(origin.x, origin.y, destination.x, destination.y);
    let delay = one_step_move_delay_ms(direction, speed, stop_frame);
    let _ = game.move_owned_monster_step_with_run(
        region, monster_id, destination.x, destination.y, figure, run,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_move(delay, now());
    }
}

/// Отдельный timestamp расписания `CMonsterAI`; он не является cooldown
/// конкретного `CSkill` и обновляется до его `CheckCast`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterAiScheduleState {
    last_attack_attempt_ms: u32,
}

impl MonsterAiScheduleState {
    pub(crate) fn begin_attack_attempt_with_clock(
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

    pub(crate) const fn begin_attack_attempt(&mut self, now_ms: u32, interval_ms: u32) -> bool {
        // Exact `m_dwTimeStamp + GetAttackSpeed() <= timeGetTime()` сохраняет
        // wrapped absolute deadline, а не устойчивый elapsed-интервал.
        if self.last_attack_attempt_ms.wrapping_add(interval_ms) > now_ms {
            return false;
        }
        self.last_attack_attempt_ms = now_ms;
        true
    }
}

/// Общий OnSchedule 0x0060B890 и его наследник CGBGuardWithSward.
pub(crate) const fn uses_stationary_attack_schedule(ai_type: u32) -> bool {
    matches!(MonsterAiKind::from_ai_type(ai_type),
        MonsterAiKind::FixedPositionArcher
            | MonsterAiKind::GuardWithBow
            | MonsterAiKind::CityGuardWithBow
            | MonsterAiKind::VillageCountyGuardWithBow
            | MonsterAiKind::GuardCountry
            | MonsterAiKind::GuardCountry2
            | MonsterAiKind::GodsBattleGuardWithSword)
}

/// CMonsterAI::OnSchedule 0x005DD044 расширяет только AX из GetAttackSpeed,
/// затем складывает его с DWORD timestamp. Обычные наследники сохраняют
/// это усечение, включая значения setup больше 65535.
/// Стационарное OnSchedule 0x0060B890 (AI5/8/11/13/17/100/101 и наследник
/// CGBGuardWithSward AI103), SmartGladiator 0x006106E0, JiuMai 0x0060AB10
/// и оба босса переходят от дальности/Tracing/CheckCast прямо к Begin:
/// дополнительного GetAttackSpeed/timestamp у этих владельцев нет.
/// Задержка повторного применения самого навыка остаётся отдельной проверкой.
pub(crate) const fn schedule_attack_interval(
    ai_type: u32,
    ordinary_interval_ms: u32,
) -> Option<u32> {
    if uses_stationary_attack_schedule(ai_type) || matches!(
        MonsterAiKind::from_ai_type(ai_type),
        MonsterAiKind::SmartGladiator
            | MonsterAiKind::JiuMai
            | MonsterAiKind::BossBlue
            | MonsterAiKind::BossFiend
    ) {
        None
    } else {
        Some(ordinary_interval_ms & 0xffff)
    }
}

/// Сохраняет точный порядок и границу сравнения `SelectAttackSkill`.
/// `roll` получает вызывающая сторона из исходного генератора случайных чисел,
/// а стандартный навык вычисляет владелец формы по категориям установленных
/// навыков.
pub(crate) fn select_attack_skill(
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
pub(crate) const fn accepts_hurt_target(
    current_target: Option<ShapeIdentity>,
    attacker: ShapeIdentity,
    attacker_is_tamed: bool,
) -> bool {
    current_target.is_none()
        && (attacker.object_type == 400
            || (attacker.object_type == 600 && attacker_is_tamed))
}

/// Определяет достигнутые `OnIdle`, которые при отсутствии игроков переводят
/// владельца в sleeping-индекс области. Умный гладиатор сначала обязан
/// исчерпать сохранённые шаги отхода; приручение, цель, cast и фактическую
/// пустоту соседних областей проверяет непосредственный runtime caller.
pub(crate) const fn hibernates_without_nearby_players(
    ai_type: u32,
    smart_gladiator_ready_to_idle: bool,
) -> bool {
    MonsterAiKind::is_generic_ai_type(ai_type)
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

pub(crate) const fn has_owned_search_enemy(ai_type: u32, pet_ai: bool) -> bool {
    pet_ai
        || MonsterAiKind::is_generic_ai_type(ai_type)
        || matches!(
            ai_type,
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 23
                | 100 | 101 | 103 | 104
        )
}

/// Ставит достигнутый общий `CMonsterAI::OnIdle`: при необходимости отдельный
/// `ChangeSkill`, затем ровно один выбор `Move/Stand` и завершающий
/// `SearchEnemy`. Каждый `AddAIEvent` получает собственный замер часов.
pub(crate) fn queue_monster_idle<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &crate::setup::monsterlist::MonsterProperties,
    runtime: &mut Runtime,
) -> bool {
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
        monster.begin_active_ai_change_skill(runtime.now_milliseconds());
    }
    if (game.skill_random_below(10_000) as u32) < property.move_timer {
        let direction = game.skill_random_below(8);
        if let Ok(destination) = CShape::get_direction_position(direction, origin) {
            move_owned_monster_to(game, region, monster_id, destination, 0,
                || runtime.now_milliseconds());
        }
    } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_stand(stop_frame, runtime.now_milliseconds());
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
    }
    true
}

/// Живая форма сохраняет геометрию цели; точка нужна только для уже начатого
/// навыка, когда объект исчез, а подтверждённый прогресс ещё хранит назначение.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterTraceTarget {
    Shape(ShapeView),
    Point(ShapeAreaCoordinates),
}

impl MonsterTraceTarget {
    pub(crate) const fn point(x: i32, y: i32) -> Self {
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

/// Общий адаптер подхода оставшихся владельцев навыков. Его проверка прямого
/// пути и отсутствие минимальной дистанции не подменяют точный Tracing ниже.
pub(crate) fn approach_attack_range<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: MonsterTraceTarget,
    maximum_distance: u32,
    runtime: &mut Runtime,
) -> bool {
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
            matches!(monster.active_ai()?, ActiveMonsterAi::Pet),
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
        MonsterTraceTarget::Point(_) => real_distance(monster_x, monster_y, target_x, target_y),
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
                monster.lose_ai_target_and_search(runtime.now_milliseconds(), game.skill_factory());
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
        ShapeAreaCoordinates { x: target_x, y: target_y }, 0, || runtime.now_milliseconds());
    false
}

/// Virtual Tracing перед новым Begin KnockOut/SpiderWeb. Стоящий питомец и
/// стационарный OnSchedule проверяют свой диапазон снаружи без вызова Tracing.
pub(crate) fn trace_owned_target_state_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut ServerRegionOwner,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some(monster) = owner.base().find_monster_by_id(monster_id) else { return false; };
    let Some(active_ai) = monster.active_ai() else { return false; };
    if matches!(active_ai, ActiveMonsterAi::Primary(MonsterAiKind::PuninessCreature)) {
        let _ = super::puninesscreature::execute_owned_puniness_creature(
            game, owner.base_mut(), monster_id, runtime,
        );
        return false;
    }
    // OnSchedule повозки не вызывает Begin атакующего навыка.
    if matches!(active_ai, ActiveMonsterAi::Carriage
        | ActiveMonsterAi::Primary(MonsterAiKind::Carriage))
    { return false; }
    let Some(skill) = monster.move_shape().current_skill(game.skill_factory()) else {
        release_owned_monster_target(game, owner.base_mut(), monster_id, runtime);
        return false;
    };
    let (skill_id, skill_level) = (skill.id(), skill.level());
    if !matches!(skill_id, 0x192 | 0x199) { return false; }
    let target = monster.ai_target()
        .and_then(|identity| crate::gameserver::appserver::skills::monsterattack::resolve_owned_monster_attack_target(
            game, owner, identity,
        ))
        .filter(|target| !target.dead)
        .map(|target| target.view);
    let Some(target) = target else {
        release_owned_monster_target(game, owner.base_mut(), monster_id, runtime);
        if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
        }
        return false;
    };
    let Some(source) = game.shape_view_in_owner(owner, monster.move_shape().shape().identity()) else {
        return false;
    };
    let maximum = |game: &CGame| {
        game.skill_base_properties(skill_id, skill_level)
            .map(|properties| properties.query_property(5_003) as i32)
            .filter(|value| *value > 0)
            .unwrap_or(1)
    };
    let distance = source.real_distance(Some(target));
    if distance >= 1 && distance <= maximum(game) { return true; }
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
    if matches!(active_ai, ActiveMonsterAi::Primary(kind) if kind.has_guard_station()) {
        let maximum_distance = maximum(game);
        return super::cityguardwithsword::trace_city_sword_target(
            game, owner.base_mut(), monster_id, source, target, 1, maximum_distance, chase_range, runtime,
        ) == super::cityguardwithsword::CitySwordTraceOutcome::Ready;
    }
    if distance > chase_range {
        release_owned_monster_target(game, owner.base_mut(), monster_id, runtime);
        if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
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
    move_owned_monster_to(game, owner.base_mut(), monster_id, destination, 0, || runtime.now_milliseconds());
    false
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp

// ============================================================================
// FUNCTION: CMonsterAI::CMonsterAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:15
// RVA: 0x001DCB80
// ADDRESS: 005dcb80
// PROTOTYPE: undefined __thiscall CMonsterAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::~CMonsterAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:20
// RVA: 0x001DCBA0
// ADDRESS: 005dcba0
// PROTOTYPE: void __thiscall ~CMonsterAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::Run
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:25
// RVA: 0x001DCBB0
// ADDRESS: 005dcbb0
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::OnChangeSkill
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// MATERIALIZED: weighted selector, `CSkill::IsRestored`, default-skill
// fallback и производная задержка AI5/AI103 выполняются достигнутым FIFO caller-ом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:167
// RVA: 0x001DCBC0
// ADDRESS: 005dcbc0
// PROTOTYPE: int __thiscall OnChangeSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::Hibernate
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CMonster::hibernate_ai` делегирует canonical `CBaseAI`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:284
// RVA: 0x001DCC50
// ADDRESS: 005dcc50
// PROTOTYPE: void __thiscall Hibernate(void)
//
// ============================================================================
// FUNCTION: CMonsterAI::OnIdle
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CMonster::hibernate_ai` и
// `execute_owned_monster_base_attack` сохраняют проверку соседних игроков и
// спящий переход. `queue_monster_idle` проводит `ChangeSkill`, точный RNG
// случайного шага либо `Stand`, а затем `SearchEnemy` для достигнутых AI0,
// AI1, AI2, AI3, AI4, AI6, AI7, AI8, AI9, AI10, AI12, AI14, AI15, AI16,
// AI17, AI18, AI19, AI20, AI100, AI101, AI104 и двух боссов. Остальные виртуальные
// владельцы остаются RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:34
// RVA: 0x001DCC60
// ADDRESS: 005dcc60
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::WhenBeenHurted
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `accepts_hurt_target` и `CMonster::when_been_hurted_by`
// сохраняют достигнутую постановку `Defense` и правила назначения цели при
// нулевой длительности оглушения.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:218
// RVA: 0x001DCE00
// ADDRESS: 005dce00
// PROTOTYPE: void __thiscall WhenBeenHurted(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::Attack
// STATUS: IMPLEMENTED
// MATERIALIZED: `CServerRegion::set_listed_pets_target` назначает target
// каждому разрешённому pet AI в canonical порядке списка владельца.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:252
// RVA: 0x001DCEC0
// ADDRESS: 005dcec0
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterAI::WakeUp
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CMonster::wake_ai` и `CGame::wake_owned_monsters_around_area`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:296
// RVA: 0x001DCEE0
// ADDRESS: 005dcee0
// PROTOTYPE: void __thiscall WakeUp(void)
//
// ============================================================================
// FUNCTION: CMonsterAI::OnSchedule
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_monster_base_attack` сохраняет проверку цели,
// выбор текущего навыка, проверку допустимости цели, `OnLoseTarget` с отдельным
// `SearchEnemy`, преследование, интервалы ИИ и навыка и запуск достигнутых
// владельцев. Недостигнутые виртуальные владельцы и прочие навыки остаются RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\monsterai.cpp:102
// RVA: 0x001DCF80
// ADDRESS: 005dcf80
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
