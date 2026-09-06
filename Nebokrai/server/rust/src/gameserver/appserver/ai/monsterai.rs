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
//! питомца без дополнительного RNG. Реакция `WhenBeenHurted` назначает новую
//! цель только свободному ИИ: игрок принимается всегда, а монстр — только после
//! подтверждения приручения. `HasTarget` считает object-целью только пару со
//! строго положительными типом объекта и ID; отрицательные legacy-значения
//! могут храниться, но расписание их целью не считает. Timestamp
//! попытки атаки принадлежит расписанию ИИ и не подменяет отдельные reuse-таймеры
//! установленных навыков; его DWORD deadline сохраняет исходное раннее
//! срабатывание рядом с переполнением часов. Default-ветвь
//! `CAIFactory::CreateAI` также проходит
//! этот общий runtime как обычный `CMonsterAI`, сохраняя исходный `ai_type`.
//! Остальные AI-ветви ниже остаются RAW.
//! Внешний virtual `Attack(skill, target)` не запускает навык: он только
//! передаёт identity в `SetTarget`; client-команда и `CPetsControl` проводят
//! этот контракт через ordered список питомцев игрока.
//! CPet::OnStayingSchedule (0x004E9650) не вызывает Tracing: его диапазон
//! проверяет общий dispatcher перед Begin. Поэтому помощник преследования
//! в Stay ничего не делает и не добавляет проверку прямого пути вместо Begin.

use crate::gameserver::appserver::ai::aifactory::MonsterAiKind;
use crate::gameserver::appserver::ai::baseai::one_step_move_delay_ms;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView,
};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
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

/// Точный одноклеточный `Slip` общего `CBaseAI::MoveTo`: желаемое
/// направление и семь обходных направлений проверяются в legacy-порядке
/// против figure-specific move-check клеток региона.
pub(crate) fn find_slip_step(
    game: &CGame,
    region: &CServerRegion,
    origin: ShapeAreaCoordinates,
    target: ShapeAreaCoordinates,
    figure: crate::gameserver::appserver::shape::ShapeFigure,
) -> Option<(i32, ShapeAreaCoordinates)> {
    let desired_direction = get_line_direction(origin.x, origin.y, target.x, target.y);
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

/// Отдельный timestamp расписания `CMonsterAI`; он не является cooldown
/// конкретного `CSkill` и обновляется до его `CheckCast`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterAiScheduleState {
    last_attack_attempt_ms: u32,
}

impl MonsterAiScheduleState {
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

/// `CVilCouGuardWithBow::OnSchedule`, `CBossBlue::OnSchedule` и
/// `CBossFiend::OnSchedule` переходят от своей проверки дальности/`CheckCast`
/// прямо к `ASA_ATTACK` и не имеют дополнительной проверки
/// `CMonster::GetAttackSpeed`, присутствующей в обычном `CMonsterAI`.
/// Задержка повторного применения самого навыка остаётся отдельной проверкой.
pub(crate) const fn schedule_attack_interval(
    ai_type: u32,
    ordinary_interval_ms: u32,
) -> Option<u32> {
    if matches!(
        MonsterAiKind::from_ai_type(ai_type),
        MonsterAiKind::VillageCountyGuardWithBow
            | MonsterAiKind::BossBlue
            | MonsterAiKind::BossFiend
    ) {
        None
    } else {
        Some(ordinary_interval_ms)
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

pub(crate) const fn has_owned_search_enemy(ai_type: u32, tamed: bool) -> bool {
    tamed
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
    let Some((origin, speed, stop_frame, has_skill)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let shape = monster.move_shape().shape();
            Some((
                ShapeAreaCoordinates {
                    x: shape.get_tile_x().ok()?,
                    y: shape.get_tile_y().ok()?,
                },
                shape.get_speed(),
                monster.stop_frame(property),
                monster.move_shape().current_skill().is_some(),
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
        if let Ok(destination) = CShape::get_direction_position(direction, origin)
            && game.move_owned_monster_step(
                region,
                monster_id,
                destination.x,
                destination.y,
                CMonster::figure(property),
            )
            && let Some(monster) = region.find_monster_by_id_mut(monster_id)
        {
            monster.begin_active_ai_move(
                one_step_move_delay_ms(direction, speed, stop_frame),
                runtime.now_milliseconds(),
            );
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

/// Выполняет общий шаг `CBaseAI::Tracing` перед запуском выбранного навыка.
/// Наблюдаемый порядок движения задаёт существующий индекс региона; функция не
/// выбирает навык и не потребляет RNG.
pub(crate) fn approach_attack_range(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: MonsterTraceTarget,
    maximum_distance: u32,
    now_ms: u32,
) -> bool {
    let Some((
        property,
        monster_view,
        monster_x,
        monster_y,
        tamed,
        pet_action,
        moveable,
        speed,
        stop_frame,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let pet = monster
            .is_tamed()
            .then(|| monster.pet_attack_properties(&property));
        let monster_view = monster.shape_view(&property)?;
        Some((
            property.clone(),
            monster_view,
            monster.move_shape().shape().get_tile_x().ok()?,
            monster.move_shape().shape().get_tile_y().ok()?,
            monster.is_tamed(),
            monster.pet_action(),
            monster.move_shape().is_moveable(),
            pet.map_or(monster.move_shape().shape().get_speed(), |pet| {
                f32::from_bits(pet.speed_bits)
            }),
            pet.map_or(property.stop_frame, |pet| pet.stop_frame),
        ))
    }) else {
        return false;
    };

    if tamed && pet_action == 2 {
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
    let chase_range = if tamed {
        game.globe_setup().maximum_pet_tracing_distance()
    } else {
        property.chase_range
    };
    if distance > chase_range as i32 {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if has_owned_search_enemy(property.ai, tamed) {
                monster.lose_ai_target_and_search(now_ms);
            } else {
                monster.clear_ai_target();
            }
        }
        return false;
    }
    if !moveable {
        return false;
    }

    let origin = ShapeAreaCoordinates {
        x: monster_x,
        y: monster_y,
    };
    let figure = CMonster::figure(&property);
    let destination = find_slip_step(
        game,
        region,
        origin,
        ShapeAreaCoordinates {
            x: target_x,
            y: target_y,
        },
        figure,
    );
    let Some((direction, destination)) = destination else {
        return false;
    };
    if game.move_owned_monster_step(
        region,
        monster_id,
        destination.x,
        destination.y,
        figure,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_active_ai_move(
                one_step_move_delay_ms(direction, speed, stop_frame),
                now_ms,
            );
        }
    }
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
