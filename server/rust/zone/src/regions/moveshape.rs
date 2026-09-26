//! Агрегат `CMoveShape` (тип `MoveShapeState`): пространственная база `CShape`,
//! реестр навыков, каноническая арена состояний и скалярные колонки, плюс общая
//! основа wire-команд движения. Исходный владелец — `appserver/moveshape.h/.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`. Старый пакет
//! хранит тонкую оболочку `CMoveShape` над этим типом: делегаты навыков и арены
//! состояний, клиентские snapshot-фасады и dispatcher-оркестрацию wire-команд.
//! Основа `0xBF603/604/605` возвращает решение и готовый пакет; around-доставку
//! и запись пространственного членства выполняет dispatcher-обвязка старого
//! пакета, чтение span приходит узким швом `RegionSpanView`.
//!
//! Подтверждённые quirks сохранены: верхняя граница Y `ForceMove` записывает
//! `width - 1`; убийца записывается единственный раз после пакета смерти
//! `0xBF60B`. `Stiffen` и пространственное семейство сверены по машинному коду.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#npc-и-базовые-фигуры

use std::ops::{Deref, DerefMut};

use nebokrai_shared::resources::GlobeStiffenSetup;
use nebokrai_shared::runtime::get_line_direction;

use super::region::{CRegion, RegionCellAccessBlock};
use super::serverregion::membership::RegionMembershipBlock;
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeFigure, ShapePositionDispatch,
};
use super::skillregistry::SkillRegistry;
use crate::app::game_message::CMessage;
use crate::combat::AttackInformation;
use crate::skills::execution::MoveShapeSkill;
use crate::skills::state::CanonicalStateStorage;

const NPC_TYPE: i32 = 500;
const SET_POSITION_MESSAGE: i32 = 0xBF603;
const FORCE_MOVE_MESSAGE: i32 = 0xBF604;
const MOVE_MESSAGE: i32 = 0xBF605;

/// Отказ пространственной регистрации `CMoveShape::SetPosXY`: coordinate
/// conversion, запись клетки footprint либо несогласованный area-span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoveShapePositionBlock {
    Coordinate(ShapeCoordinateBlock),
    ShapeBlock(ShapeBlockError),
    InvalidAreaSpan { width: i32, height: i32 },
}

/// RTTI-факты derived owner-а для пространственной регистрации `CMoveShape`:
/// здоровье выбирает запись нового блока, figure задаёт footprint, а area-span
/// и текущая area — переключение change-state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MoveShapePositionFacts {
    pub current_hit_points: u32,
    pub figure: ShapeFigure,
    pub current_area: Option<ShapeAreaCoordinates>,
    pub area_width: i32,
    pub area_height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MoveShapePet {
    pub object_type: i32,
    pub id: i32,
    pub figure: i32,
}

/// Единственная проекция `SetKilledMeAttackInfo` (RVA `0x000CCE50`):
/// сохраняемая после пакета смерти `0xBF60B` и потребляемая поздним OnDied
/// идентичность убийцы — тип, ID и guild ID атакующего.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KillingAttackIdentity {
    pub attacker_type: i32,
    pub attacker_id: i32,
    pub attacker_faction_id: i32,
}

impl From<&AttackInformation> for KillingAttackIdentity {
    fn from(attack: &AttackInformation) -> Self {
        Self {
            attacker_type: attack.attacker_type,
            attacker_id: attack.attacker_id,
            attacker_faction_id: attack.attacker_faction_id,
        }
    }
}

/// Единственный набор `tagProperties` PDB (type `0x6D94`, fieldlist `0x6D93`):
/// 25 signed LONG исходного порядка. Общий UpdateProperty обнуляет его и
/// вызывает живые property-state в исходном порядке.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MoveShapePropertyModifiers {
    pub maximum_hp: i32,
    pub maximum_mp: i32,
    pub maximum_yp: i32,
    pub maximum_rp: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub hit: i32,
    pub burden: i32,
    pub critical_hit: i32,
    pub defense: i32,
    pub dodge: i32,
    pub attack_speed: i32,
    pub element_resistance: i32,
    pub hp_recovery_speed: i32,
    pub mp_recovery_speed: i32,
    pub soul_resistance: i32,
    pub additional_element_attack: i32,
    pub additional_soul_attack: i32,
    pub element_modify: i32,
    pub attack_avoid: i32,
    pub element_avoid: i32,
}

/// Dispatch достигнутого override-а `CMoveShape::SetPosXY` для владельцев,
/// разрешающих фигуру через registry (`ShapePositionDispatch`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MoveShapePositionDispatch {
    pub facts: MoveShapePositionFacts,
}

impl ShapePositionDispatch for MoveShapePositionDispatch {
    type Error = MoveShapePositionBlock;

    fn set_pos_xy(
        &mut self,
        region: &mut CRegion,
        shape: &mut CShape,
        x: f32,
        y: f32,
    ) -> Result<(), Self::Error> {
        set_pos_xy_core(Some(region), shape, x, y, self.facts)
    }
}

/// Агрегат `CMoveShape` исторического GameServer: пространственная база
/// `CShape`, реестр навыков `SkillRegistry<MoveShapeSkill>`, каноническая
/// арена состояний и скалярные колонки — property modifiers, счётчики
/// запретов движения и боя, god-флаг, список питомцев с режимом, stiffen-окно
/// и потребляемая поздним OnDied запись убийцы. Поля открыты по прецеденту
/// `skills::state::CanonicalStateStorage`: dispatcher-обвязка и делегаты
/// старого пакета обращаются к хранилищам напрямую, а поведение колонок
/// инкапсулируют методы ниже и свободные операции этого модуля.
/// `Deref` ведёт в арену состояний — прежний шов старого пакета.
/// Старый пакет хранит тонкую оболочку `CMoveShape` над этим типом с
/// делегатами навыков/состояний, клиентскими snapshot-фасадами и
/// dispatcher-оркестрацией wire-команд.
#[derive(Debug, Eq, PartialEq)]
pub struct MoveShapeState {
    pub shape: CShape,
    pub skills: SkillRegistry<MoveShapeSkill>,
    pub state_storage: CanonicalStateStorage,
    pub property_modifiers: MoveShapePropertyModifiers,
    pub moveable_count: i32,
    pub moveable: bool,
    pub can_fight_count: i32,
    pub can_fight: bool,
    pub is_god: bool,
    pub pets: Vec<MoveShapePet>,
    pub current_pets_mode: i32,
    pub stiffen_started_ms: u32,
    pub stiffen_count: i32,
    pub killed_by: Option<KillingAttackIdentity>,
}

impl Deref for MoveShapeState {
    type Target = CanonicalStateStorage;

    fn deref(&self) -> &Self::Target {
        &self.state_storage
    }
}

impl DerefMut for MoveShapeState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_storage
    }
}

impl Default for MoveShapeState {
    fn default() -> Self {
        Self {
            shape: CShape::default(),
            skills: Default::default(),
            state_storage: CanonicalStateStorage::default(),
            property_modifiers: MoveShapePropertyModifiers::default(),
            moveable_count: 0,
            moveable: true,
            can_fight_count: 0,
            can_fight: true,
            is_god: false,
            pets: Vec::new(),
            current_pets_mode: 1,
            stiffen_started_ms: 0,
            stiffen_count: 0,
            killed_by: None,
        }
    }
}

impl MoveShapeState {
    pub const fn property_modifiers(&self) -> &MoveShapePropertyModifiers {
        &self.property_modifiers
    }

    pub const fn property_modifiers_mut(&mut self) -> &mut MoveShapePropertyModifiers {
        &mut self.property_modifiers
    }

    pub fn reset_property_modifiers(&mut self) {
        self.property_modifiers = MoveShapePropertyModifiers::default();
    }

    pub fn set_killed_by(&mut self, attack: KillingAttackIdentity) {
        self.killed_by = Some(attack);
    }

    pub const fn killed_by(&self) -> Option<KillingAttackIdentity> {
        self.killed_by
    }

    /// Exact `CMoveShape::Stiffen` (`RVA 0x000CD2F0`) идёт общей операцией
    /// этого модуля: скаляры окна/live-limitа остаются полями этого агрегата,
    /// setup передаётся value-формой (`GlobeStiffenSetup` — POD проекция
    /// Shared resources, а не ссылка на setup-владельца).
    pub fn stiffen(
        &mut self,
        damage: u16,
        maximum_hp: u32,
        reank: u16,
        setup: GlobeStiffenSetup,
        now_ms: impl FnMut() -> u32,
        random: impl FnMut(i32) -> i32,
    ) -> u32 {
        stiffen(
            &mut self.stiffen_started_ms,
            &mut self.stiffen_count,
            damage,
            maximum_hp,
            reank,
            setup,
            now_ms,
            random,
        )
    }

    pub const fn current_pets_mode(&self) -> i32 {
        self.current_pets_mode
    }

    pub fn set_current_pets_mode(&mut self, mode: i32) -> bool {
        set_current_pets_mode(&mut self.current_pets_mode, mode)
    }

    pub fn add_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        add_pet(&mut self.pets, object_type, id, figure);
    }

    pub fn remove_pet(&mut self, object_type: i32, id: i32) -> bool {
        remove_pet(&mut self.pets, object_type, id)
    }

    pub fn pets(&self) -> &[MoveShapePet] {
        &self.pets
    }

    pub const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    /// Exact inline `CMoveShape::God`: runtime-only invulnerability flag не
    /// сериализуется и проверяется ordinary `OnBeenAttacked` owner-ом.
    pub const fn set_god(&mut self, enabled: bool) {
        self.is_god = enabled;
    }

    pub const fn is_god(&self) -> bool {
        self.is_god
    }

    /// Счётчик запретов боя идёт общей операцией этого модуля.
    pub const fn set_fightable(&mut self, fightable: bool) {
        set_fightable(&mut self.can_fight_count, &mut self.can_fight, fightable);
    }

    pub const fn can_fight(&self) -> bool {
        self.can_fight
    }

    /// Scalar-prefix сброса при входе в регион идёт общей операцией этого
    /// модуля; конкретные Begin заново устанавливают свои запреты после
    /// этого сброса в общем живом проходе.
    pub const fn reset_region_entry_control(&mut self) {
        reset_region_entry_control(
            &mut self.moveable,
            &mut self.can_fight,
            &mut self.moveable_count,
            &mut self.can_fight_count,
        );
    }

    pub const fn is_moveable(&self) -> bool {
        self.moveable
    }

    pub const fn moveable_count(&self) -> i32 {
        self.moveable_count
    }

    /// Счётчик запретов движения идёт общей операцией этого модуля.
    pub const fn set_moveable(&mut self, moveable: bool) {
        set_moveable(&mut self.moveable_count, &mut self.moveable, moveable);
    }
}

/// Общее ядро exact `CMoveShape::SetPosXY` (RVA `0x000CD050`): сначала снимается
/// блок по старой клетке, живой либо NPC owner ставит блок по новой, затем
/// записывается позиция и переключается change-state по area-span.
/// Монстр и постройка используют ту же пространственную базу напрямую.
pub fn set_pos_xy_core(
    region: Option<&mut CRegion>,
    shape: &mut CShape,
    x: f32,
    y: f32,
    facts: MoveShapePositionFacts,
) -> Result<(), MoveShapePositionBlock> {
    if let Some(region) =
        region.filter(|region| shape.is_assigned_to_server_region() && region.width != 0)
    {
        let old_y = shape
            .get_tile_y()
            .map_err(MoveShapePositionBlock::Coordinate)?;
        let old_x = shape
            .get_tile_x()
            .map_err(MoveShapePositionBlock::Coordinate)?;
        shape
            .set_block(region, old_x, old_y, 0, facts.figure)
            .map_err(MoveShapePositionBlock::ShapeBlock)?;

        if facts.current_hit_points != 0 || shape.identity().object_type == NPC_TYPE {
            let new_y = CShape::tile_from_value(y).map_err(MoveShapePositionBlock::Coordinate)?;
            let new_x = CShape::tile_from_value(x).map_err(MoveShapePositionBlock::Coordinate)?;
            shape
                .set_block(region, new_x, new_y, 3, facts.figure)
                .map_err(MoveShapePositionBlock::ShapeBlock)?;
        }
    }

    shape.set_pos_xy_move_order(x, y);
    let tile_y = CShape::tile_from_value(y).map_err(MoveShapePositionBlock::Coordinate)?;
    let tile_x = CShape::tile_from_value(x).map_err(MoveShapePositionBlock::Coordinate)?;
    if facts.area_width <= 0 || facts.area_height <= 0 {
        return Err(MoveShapePositionBlock::InvalidAreaSpan {
            width: facts.area_width,
            height: facts.area_height,
        });
    }

    let next_area = ShapeAreaCoordinates {
        x: tile_x / facts.area_width,
        y: tile_y / facts.area_height,
    };
    if facts
        .current_area
        .is_some_and(|current| current != next_area)
    {
        shape.set_next_area_coordinates(next_area);
        shape.set_change_state(SHAPE_CHANGE_AREA);
    } else {
        shape.set_change_state(SHAPE_CHANGE_NONE);
    }
    Ok(())
}

/// Clamp X цели `ForceMove` к валидному span региона.
pub const fn clamp_force_x(destination: i32, width: i32) -> i32 {
    if destination < 0 {
        0
    } else if destination >= width {
        width.wrapping_sub(1)
    } else {
        destination
    }
}

/// Clamp Y цели `ForceMove`. Подтверждённая странность GameServer RVA
/// `0x000CD1A0`: `if (height <= lDestY) lDestY = width - 1;`.
pub const fn clamp_force_y(destination: i32, width: i32, height: i32) -> i32 {
    if destination < 0 {
        0
    } else if destination >= height {
        width.wrapping_sub(1)
    } else {
        destination
    }
}

/// Exact `CMoveShape::IsDied` (RVA `0x000CCF20`): нулевое здоровье.
pub const fn is_died(current_hit_points: u32) -> bool {
    current_hit_points == 0
}

/// Exact `CMoveShape::GetDestDir` (RVA `0x000CCF60`): направление от цели
/// к атакующему по знаковым дельтам. Подтверждённая странность: совпавшие
/// точки возвращают DIR_DOWN `4`, а не отдельный sentinel.
pub fn get_dest_direction(
    source_x: i32,
    source_y: i32,
    destination_x: i32,
    destination_y: i32,
) -> i32 {
    let delta_x = source_x.wrapping_sub(destination_x);
    let delta_y = source_y.wrapping_sub(destination_y);
    match (delta_x.signum(), delta_y.signum()) {
        (1, 1) => 7,
        (1, 0) => 6,
        (1, -1) => 5,
        (-1, 1) => 1,
        (-1, 0) => 2,
        (-1, -1) => 3,
        (0, 1) => 0,
        (0, 0 | -1) => 4,
        _ => unreachable!("signum возвращает только -1/0/1"),
    }
}

/// Общая геометрия exact overrides `CBuild/CMonster::GetBeAttackedPoint`
/// (RVA `0x001DD350`/`0x000E6AA0`): ближайшая клетка footprint с предпочтением
/// прямого направления при равной Chebyshev-дистанции.
pub fn nearest_figure_attack_point(
    tile_x: i32,
    tile_y: i32,
    figure: ShapeFigure,
    attacker_x: i32,
    attacker_y: i32,
) -> (i32, i32) {
    let horizontal = i32::from(figure.get(2));
    let vertical = i32::from(figure.get(0));
    let mut best_point = (tile_x, tile_y);
    let mut best_distance = 10_000_000;
    let mut best_direction: i32 = 0;

    for offset_x in -horizontal..=horizontal {
        let candidate_x = tile_x.wrapping_add(offset_x);
        for offset_y in -vertical..=vertical {
            let candidate_y = tile_y.wrapping_add(offset_y);
            let distance_x = candidate_x.wrapping_sub(attacker_x).unsigned_abs() as i32;
            let distance_y = candidate_y.wrapping_sub(attacker_y).unsigned_abs() as i32;
            let distance = distance_x.max(distance_y);
            let direction = get_dest_direction(
                attacker_x,
                attacker_y,
                candidate_x,
                candidate_y,
            );
            if distance < best_distance
                || (distance == best_distance
                    && best_direction.rem_euclid(2) == 1
                    && direction.rem_euclid(2) == 0)
            {
                best_point = (candidate_x, candidate_y);
                best_distance = distance;
                best_direction = direction;
            }
        }
    }
    best_point
}

/// Смена режима питомцев: совпавший режим отклоняется без записи.
pub fn set_current_pets_mode(current_pets_mode: &mut i32, mode: i32) -> bool {
    if *current_pets_mode == mode {
        return false;
    }
    *current_pets_mode = mode;
    true
}

/// Регистрация питомца в ordered-списке владельца.
pub fn add_pet(pets: &mut Vec<MoveShapePet>, object_type: i32, id: i32, figure: i32) {
    pets.push(MoveShapePet {
        object_type,
        id,
        figure,
    });
}

/// Снятие питомца по паре type/ID первого совпадения.
pub fn remove_pet(pets: &mut Vec<MoveShapePet>, object_type: i32, id: i32) -> bool {
    let Some(index) = pets
        .iter()
        .position(|pet| pet.object_type == object_type && pet.id == id)
    else {
        return false;
    };
    pets.remove(index);
    true
}

/// Exact nesting contract `SetFightable` (RVA `0x000CCE10`): false добавляет
/// запрет, true снимает один; отрицательный legacy count нормализуется только
/// перед добавлением нового запрета.
pub const fn set_fightable(can_fight_count: &mut i32, can_fight: &mut bool, fightable: bool) {
    if !fightable {
        if *can_fight_count < 0 {
            *can_fight_count = 0;
        }
        *can_fight_count = can_fight_count.wrapping_add(1);
    } else {
        *can_fight_count = can_fight_count.wrapping_sub(1);
    }
    *can_fight = *can_fight_count < 1;
}

/// Exact counter semantics `SetMoveable` (RVA `0x000CCEE0`): `false` ставит
/// новый запрет, `true` снимает один; отрицательный счётчик не нормализуется
/// в ветви снятия и потому сохраняется как наблюдаемая legacy-семантика.
pub const fn set_moveable(moveable_count: &mut i32, moveable: &mut bool, allow: bool) {
    if !allow {
        if *moveable_count < 0 {
            *moveable_count = 0;
        }
        *moveable_count = moveable_count.wrapping_add(1);
    } else {
        *moveable_count = moveable_count.wrapping_sub(1);
    }
    *moveable = *moveable_count < 1;
}

/// Scalar-prefix `CMoveShape::OnEnterRegion` (RVA `0x000CEF40`); конкретные
/// Begin заново устанавливают свои запреты после этого сброса в общем живом
/// проходе.
pub const fn reset_region_entry_control(
    moveable: &mut bool,
    can_fight: &mut bool,
    moveable_count: &mut i32,
    can_fight_count: &mut i32,
) {
    *moveable = true;
    *can_fight = true;
    *moveable_count = 0;
    *can_fight_count = 0;
}

/// Exact `CMoveShape::Stiffen` (RVA `0x000CD2F0`): окно и limit проверяются
/// до RNG, просроченное окно делает второй замер часов, а вероятность
/// уменьшается на `GetReAnk` перед signed-сравнением с `random(100)`.
/// `GlobeStiffenSetup` передаётся value-формой — это POD проекция Shared
/// resources, а не ссылка на setup-владельца.
#[allow(
    clippy::too_many_arguments,
    reason = "scalars owner-а, damage-параметры, setup и два clock/RNG шва сохраняют исходные границы"
)]
pub fn stiffen(
    stiffen_started_ms: &mut u32,
    stiffen_count: &mut i32,
    damage: u16,
    maximum_hp: u32,
    reank: u16,
    setup: GlobeStiffenSetup,
    mut now_ms: impl FnMut() -> u32,
    mut random: impl FnMut(i32) -> i32,
) -> u32 {
    let now = now_ms();
    if stiffen_started_ms.wrapping_add(setup.bound_time_ms) < now {
        *stiffen_started_ms = now_ms();
        *stiffen_count = 0;
    } else if *stiffen_count >= setup.limit {
        return 0;
    }

    let damage_ratio = f32::from(damage) / maximum_hp as f32;
    let probability = setup
        .damage_thresholds
        .iter()
        .zip(setup.probabilities)
        .take(usize::from(setup.count).min(4))
        .find_map(|(threshold, probability)| {
            (damage_ratio >= *threshold).then_some(probability)
        })
        .unwrap_or_default();
    let chance = i32::from(probability) - i32::from(reank);
    if random(100) > chance {
        return 0;
    }
    *stiffen_count = stiffen_count.wrapping_add(1);
    setup.delay_ms
}

/// Typed-отказ wire-команд движения `CMoveShape` (`0xBF603/604/605`):
/// чтение координат фигуры, доступ запрета клетки региона, запись
/// пространственного членства либо отсоединённая позиция без региона.
/// Общая основа методов живёт здесь, а отказ от её шагов собирает
/// dispatcher-обвязка переходного `CMoveShape` старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoveShapeCommandBlock {
    Coordinate(ShapeCoordinateBlock),
    RegionCell(RegionCellAccessBlock),
    Position(RegionMembershipBlock),
    DetachedPosition(MoveShapePositionBlock),
}

/// Узкий шов чтения пространственного региона для основы wire-команд
/// движения: `ForceMove` клампит цель к span, а `OnSetPosition`
/// сверяет цель с границами и запретом клетки. Вид реализует переходный
/// владелец хранилищ (`CServerRegion` старого пакета) у себя; запись позиции
/// и around-доставка этой границей не покрываются.
pub trait RegionSpanView {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn get_block(&self, x: i32, y: i32) -> Result<u8, RegionCellAccessBlock>;
}

/// Typed-решение основы `CMoveShape::ForceMove`: зажатая к span точка цели и
/// готовый пакет `0xBF604`; публикация и spatial-запись остаются dispatcher-у.
pub struct MoveShapeForceMoveOutcome {
    pub destination: ShapeAreaCoordinates,
    pub message: CMessage,
}

/// Общая основа exact `CMoveShape::ForceMove` (RVA `0x000CD1A0`): clamp цели
/// к span региона и построение `0xBF604` в исходном порядке полей — ID,
/// object type, старые tile X/Y, цель X/Y, duration и нулевой хвост.
/// Around-доставка, virtual SetTileXY и последующий AI Stand принадлежат
/// dispatcher-владельцу старого пакета.
pub fn force_move_wire(
    shape: &CShape,
    span: &impl RegionSpanView,
    destination_x: i32,
    destination_y: i32,
    duration_ms: u32,
) -> Result<MoveShapeForceMoveOutcome, ShapeCoordinateBlock> {
    let width = span.width();
    let height = span.height();
    let clamped_x = clamp_force_x(destination_x, width);
    let clamped_y = clamp_force_y(destination_y, width, height);
    let old_x = shape.get_tile_x()?;
    let old_y = shape.get_tile_y()?;
    let identity = shape.identity();

    let mut message = CMessage::new(FORCE_MOVE_MESSAGE);
    message.add_long(identity.id);
    message.add_long(identity.object_type);
    message.add_long(old_x);
    message.add_long(old_y);
    message.add_long(clamped_x);
    message.add_long(clamped_y);
    message.add_ulong(duration_ms);
    message.add_long(0);
    Ok(MoveShapeForceMoveOutcome {
        destination: ShapeAreaCoordinates {
            x: clamped_x,
            y: clamped_y,
        },
        message,
    })
}

/// Общая основа exact virtual `CMoveShape::OnMove` (RVA `0x000CD490`):
/// направление считается от текущей клетки к цели исходной line-direction
/// сеткой и записывается в фигуру, затем строится `0xBF605` — ID,
/// object type, старые tile X/Y, байты `1` и режима `2 + (run != 0)` и две
/// пары координат цели. Отправка и spatial-запись — у dispatcher-а.
pub fn on_move_wire(
    shape: &mut CShape,
    destination_x: i32,
    destination_y: i32,
    run: i32,
) -> Result<CMessage, ShapeCoordinateBlock> {
    let old_x = shape.get_tile_x()?;
    let old_y = shape.get_tile_y()?;
    shape.set_direction(get_line_direction(
        old_x,
        old_y,
        destination_x,
        destination_y,
    ));
    let identity = shape.identity();

    let mut message = CMessage::new(MOVE_MESSAGE);
    message.add_long(identity.id);
    message.add_long(identity.object_type);
    message.add_long(old_x);
    message.add_long(old_y);
    message.add_byte(1);
    message.add_byte(2 + u8::from(run != 0));
    message.add_long(destination_x);
    message.add_long(destination_y);
    message.add_long(destination_x);
    message.add_long(destination_y);
    Ok(message)
}

/// Typed-решение основы `CMoveShape::OnSetPosition`: отказ без публикации
/// (цель вне span либо на занятой клетке) либо готовый пакет `0xBF603`.
pub enum MoveShapeSetPositionOutcome {
    Declined,
    Accepted(CMessage),
}

/// Общая основа exact virtual `CMoveShape::OnSetPosition` (RVA `0x000CD5C0`):
/// цель вне span региона или по ненулевому запрету клетки отклоняется без
/// публикации, иначе строится `0xBF603` — object type, ID, цель X/Y и
/// нулевой хвост. Отправка и запись членства — у dispatcher-а.
pub fn on_set_position_wire(
    shape: &CShape,
    span: &impl RegionSpanView,
    destination_x: i32,
    destination_y: i32,
) -> Result<MoveShapeSetPositionOutcome, RegionCellAccessBlock> {
    if destination_x < 0
        || destination_x >= span.width()
        || destination_y < 0
        || destination_y >= span.height()
    {
        return Ok(MoveShapeSetPositionOutcome::Declined);
    }
    if span.get_block(destination_x, destination_y)? != 0 {
        return Ok(MoveShapeSetPositionOutcome::Declined);
    }

    let identity = shape.identity();
    let mut message = CMessage::new(SET_POSITION_MESSAGE);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(destination_x);
    message.add_long(destination_y);
    message.add_long(0);
    Ok(MoveShapeSetPositionOutcome::Accepted(message))
}
