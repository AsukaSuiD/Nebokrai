//! Базовое состояние GameServer-region `CServerRegion`.
//!
//! `StartClearPlayerOut` RVA `0x0007BA50`, resource forwarding
//! `Save/New/Load` `0x0007BA80/0x0007BA90/0x0007C290`, обе `GetArea`
//! `0x00001DB0/0x0007BB60` и `CreateAreaArray` `0x0007BE10`, фазовые defaults
//! `OnWarDeclare/Start/End/Mass` RVA `0x00085560..0x000855B0`, ownership и
//! state accessors `0x000855D0..0x00085680` и `GetReturnPoint`
//! `0x000814F0` имеют статус `IMPLEMENTED, VERIFIED_DISASSEMBLY`.
//! Membership/spatial family `FindChildObject/AddObject/RemoveObject` RVA
//! `0x000821E0/0x00083270/0x0007CE60` и обе `GetShape`
//! `0x0007F390/0x00081300`, а также `GetPlayerAmout` `0x0000C760` имеют этот
//! статус. Spatial transition `RefeashBlock` `0x0007FAD0/0x00081680`,
//! `OnShapeChangeArea` `0x000802A0` и достигнутая `CS_CHANGEAREA` ветвь `AI`
//! `0x00084520` также `IMPLEMENTED, VERIFIED_DISASSEMBLY`.
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`;
//! исходники `serverregion.h/.cpp`. PDB фиксирует
//! `m_listChangeAreaShape +0x1D0`, поля `m_Param +0x214`,
//! `m_lWarNum +0x238`, `m_CityState +0x23C` и clear timer `+0x240..+0x248`.
//!
//! `i32/u32` сохраняют x86 `long/DWORD`; `String` и owned fields заменяют
//! MFC/STL storage без изменения достигнутых эффектов. `timeGetTime` передаётся
//! явным wrapping `now_ms`, пока GameServer runtime-clock owner не подключён.
//! Встроенный `CRegion` сохраняет исходную inheritance-границу и является
//! владельцем byte-exact tile/security storage; subtype-ы не дублируют клетки.
//! `Save/New` являются точными tail-jump в `CRegion`; `Load` создаёт area-grid
//! только после успешного resource load. `AREA_WIDTH/AREA_HEIGHT` приходят из
//! достигнутого GlobeSetup writer-а и передаются явно. Для положительных
//! размеров grid использует исходное ceiling-деление и row-major
//! `area_x * y + x`; старые zero/negative span и 32-bit allocation overflow
//! становятся локальной typed-границей, а не platform-dependent trap/UB.
//! Старый unbounded `GetArea(index)` заменён `Result`, coordinate overload
//! сохраняет доказанный `nullptr -> Option`.
//! Старые pointer-valued hash maps выражены registry identity: сами `CShape`
//! остаются у runtime owner-а и разрешаются через `ShapeResolver`. Это
//! сознательная смена формы API без копии shared/derived семантики. Player
//! registry сохраняет vector и first-erase, остальные map assignment —
//! уникальные keys. `CArea::PlayerEnter`, move-shape callback и GodsBattle
//! message остаются у внешних owners через `ServerRegionMembershipContext` и
//! вызываются в исходном порядке.
//! `SetPosXY` только пишет `CS_CHANGEAREA`; region AI уникализирует pointer,
//! сбрасывает state лишь после первого insert и применяет очередь после
//! delete/remove queues. `OnShapeChangeArea` строит old/new девяти-area
//! neighborhoods, шлёт create/snapshot только в new-exclusive areas, затем
//! делает old `RemoveObject -> new AddObject -> m_pArea` и player-enter wake.
//! Message serialization/send остаются явным context-owner-ом; area storage и
//! deferred queue принадлежат `CServerRegion`. `RefeashBlock` сначала снимает
//! все block `3`, затем возвращает single-cell block живым `CMoveShape` и NPC.
//! Достигнутые movement commands вызывают здесь именно owner
//! `CShape::SetTileXY`: region дополняет runtime area facts и передаёт virtual
//! dispatch, не дублируя tile-center либо `CMoveShape::SetPosXY`.
//! Общий `OnWarTimeOut/OnClearOtherPlayer/OnRefreshRegion` — один PDB-symbol
//! RVA `0x00201A70`, три байта `ret 4`, поэтому базовые defaults — no-op.
//! `GetReturnPoint` сохраняет null-player zero result, local `m_stSetup`
//! priority и fallback в три mutating country-default карты. Constructor
//! `0x000852B0` не инициализирует `m_stSetup`; prefix
//! `DecordSetupFromByteArray` `0x0007EAC0` материализует точные `0x20` байт,
//! затем полностью заменяет ordered set запрещённых для производства товаров;
//! `FindForbidGood` `0x0007D6A0` выполняет точный lookup C-string в этом set.
//! До вызова decoder-а setup остаётся отдельной typed-границей.
//! Достигнутый player-leave call из `RemoveObject` попадает в тот же exact
//! `ret 4` RVA `0x00201A70`, поэтому отдельного наблюдаемого эффекта не имеет.
//! Оставшиеся отдельные `std::basic_streambuf`/`Unwind@` экспорты сняты общей
//! технической классификацией; domain lifecycle и callbacks не затрагивались.
//! Остальная поверхность файла ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).

use std::collections::BTreeSet;

use super::area::CArea;
use super::country::countryparam::CCountryParam;
use super::moveshape::{
    MoveShapePositionBlock, MoveShapePositionDispatch, MoveShapePositionFacts, MoveShapeResolver,
};
use super::region::{
    CRegion, RegionCellAccessBlock, RegionDecodeError, RegionRandomContext, RegionResourceWrite,
    RegionReturnPoint, RegionStorageBlock,
};
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeIdentity, ShapePositionDispatch, ShapeResolver, ShapeRuntimeFacts,
    ShapeView,
};
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;
const GOODS_TYPE: i32 = 700;

const NEIGHBOR_AREAS: [(i32, i32); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

const CITY_STATE_NONE: i32 = 0;
const CITY_STATE_DECLARE: i32 = 1;
const CITY_STATE_MASS: i32 = 2;
const CITY_STATE_FIGHT: i32 = 3;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionParamState {
    pub(crate) region_id: i32,
    pub(crate) max_tax_rate: i32,
    pub(crate) current_tax_rate: i32,
    pub(crate) total_tax: u32,
    pub(crate) today_total_tax: u32,
    pub(crate) superior_region_id: i32,
    pub(crate) turn_in_tax_rate: i32,
    pub(crate) owned_faction_id: i32,
    pub(crate) owned_union_id: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerReturnPlayer {
    pub(crate) id: i32,
    pub(crate) country: u8,
    pub(crate) faction_id: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerReturnSetup {
    pub(crate) region_id: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) does_recall_when_lost: i32,
    pub(crate) move_monster_when_refeash: i32,
    pub(crate) use_return: i32,
}

/// BLOCKED_MISSING_FACT: constructor не записывает `m_stSetup`; безопасный
/// результат возможен только после доказанного decoder/writer-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerReturnSetupBlock;

/// BLOCKED_MISSING_FACT: исходный setup decoder не принимает buffer length;
/// truncated `m_stSetup` в safe Rust останавливается на точной границе.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerReturnSetupInputBlock {
    pub(crate) offset: usize,
    pub(crate) required: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionSetupDecodeError {
    Setup(ServerReturnSetupInputBlock),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AreaGridBlock {
    InvalidAreaSpan { width: i32, height: i32 },
    InvalidRegionDimensions { width: i32, height: i32 },
    GridSizeOverflow { area_x: i32, area_y: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AreaIndexBlock {
    pub(crate) index: i32,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServerRegionLoadError {
    Region(RegionDecodeError),
    AreaGrid(AreaGridBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionMembershipBlock {
    InvalidAreaSpan { width: i32, height: i32 },
    StaleAreaIndex { index: usize, available: usize },
    ShapeCoordinate(ShapeCoordinateBlock),
    ShapeBlock(ShapeBlockError),
    RegionCell(RegionCellAccessBlock),
    MoveShape(MoveShapePositionBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AreaTransitionBlock {
    StaleAreaIndex { index: usize, available: usize },
}

pub(crate) trait ServerRegionAreaTransitionContext {
    /// Сохраняет player-special, goods old-client и обычную virtual
    /// serialization ветви `OnShapeChangeArea`.
    fn serialize_moving_shape_for_area(&mut self, identity: ShapeIdentity) -> Vec<u8>;

    fn serialize_area_shape(&mut self, shape: ShapeView) -> Vec<u8>;

    /// Шлёт `0xBF502` в новую exclusive area, исключая сам moving shape.
    fn send_shape_entered_area(
        &mut self,
        area_x: i32,
        area_y: i32,
        identity: ShapeIdentity,
        payload: &[u8],
    );

    /// Шлёт player-у `0xBF502` одного уже присутствующего shape новой area.
    fn send_area_shape_to_player(&mut self, player_id: i32, shape: ShapeView, payload: &[u8]);

    /// Материализует `CArea::PlayerEnter -> WakeUpMonsters` у AI owner-а.
    fn wake_up_area_monsters(&mut self, area_x: i32, area_y: i32);
}

pub(crate) trait ServerRegionMembershipContext:
    RegionRandomContext + ShapePositionDispatch<Error = RegionMembershipBlock>
{
    /// Материализует `CArea::WakeUpMonsters` у AI owner-а.
    fn wake_up_area_monsters(&mut self, area_x: i32, area_y: i32);

    /// Материализует достигнутый virtual `CMoveShape` area-enter callback.
    fn move_shape_entered_area(&mut self, identity: ShapeIdentity);

    fn is_gods_battle_region(&self, region_id: i32) -> bool;

    /// Отправляет исходный `0xBF80C` player message после входа не в GodsBattle.
    fn notify_player_left_gods_region(&mut self, player_id: i32);
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ServerRegionRegistry {
    monsters: BTreeSet<i32>,
    players: Vec<i32>,
    npcs: BTreeSet<i32>,
    goods: BTreeSet<CGuid>,
    other_shapes: BTreeSet<i64>,
}

impl ServerRegionRegistry {
    fn add(&mut self, identity: ShapeIdentity, facts: ShapeRuntimeFacts) {
        match identity.object_type {
            MONSTER_TYPE if facts.monster.is_some() => {
                self.monsters.insert(identity.id);
            }
            PLAYER_TYPE if facts.is_player => self.players.push(identity.id),
            NPC_TYPE if facts.is_npc => {
                self.npcs.insert(identity.id);
            }
            GOODS_TYPE if facts.goods.is_some() => {
                self.goods.insert(identity.ex_id);
            }
            PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE | GOODS_TYPE => {}
            _ => {
                self.other_shapes
                    .insert(super::baseobject::CBaseObject::get_hash_value(
                        identity.object_type,
                        identity.id,
                    ));
            }
        }
    }

    fn remove(&mut self, identity: ShapeIdentity) {
        match identity.object_type {
            MONSTER_TYPE => {
                self.monsters.remove(&identity.id);
            }
            PLAYER_TYPE => {
                if let Some(index) = self.players.iter().position(|id| *id == identity.id) {
                    self.players.remove(index);
                }
            }
            NPC_TYPE => {
                self.npcs.remove(&identity.id);
            }
            GOODS_TYPE => {
                self.goods.remove(&identity.ex_id);
            }
            _ => {
                self.other_shapes
                    .remove(&super::baseobject::CBaseObject::get_hash_value(
                        identity.object_type,
                        identity.id,
                    ));
            }
        }
    }

    fn contains(&self, identity: ShapeIdentity) -> bool {
        match identity.object_type {
            MONSTER_TYPE => self.monsters.contains(&identity.id),
            PLAYER_TYPE => self.players.contains(&identity.id),
            NPC_TYPE => self.npcs.contains(&identity.id),
            GOODS_TYPE => self.goods.contains(&identity.ex_id),
            _ => self
                .other_shapes
                .contains(&super::baseobject::CBaseObject::get_hash_value(
                    identity.object_type,
                    identity.id,
                )),
        }
    }
}

struct RegisteredShapeResolver<'a, Resolver> {
    registry: &'a ServerRegionRegistry,
    resolver: &'a Resolver,
}

impl<Resolver: ShapeResolver> ShapeResolver for RegisteredShapeResolver<'_, Resolver> {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        if !self.registry.contains(identity) {
            return None;
        }
        self.resolver.resolve_shape(identity)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerRegion {
    pub(crate) region: CRegion,
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) country: u8,
    area_x: i32,
    area_y: i32,
    areas: Vec<CArea>,
    registry: ServerRegionRegistry,
    change_area_shapes: Vec<ShapeIdentity>,
    pub(crate) param: RegionParamState,
    pub(crate) return_setup: Option<ServerReturnSetup>,
    forbidden_make_goods: BTreeSet<Vec<u8>>,
    pub(crate) war_number: i32,
    pub(crate) city_state: i32,
    pub(crate) kick_out_player: bool,
    pub(crate) kick_out_player_time: i32,
    pub(crate) last_time_ms: u32,
}

impl CServerRegion {
    /// Делегирует exact `CServerRegion::New` в базовый `CRegion::New`.
    pub(crate) fn new_region(&mut self) -> Result<i32, RegionStorageBlock> {
        self.region.new_region()
    }

    /// Делегирует exact `CServerRegion::Save` в базовый resource serializer.
    pub(crate) fn save_region_resource(
        &mut self,
    ) -> Result<RegionResourceWrite, RegionStorageBlock> {
        self.region.set_id(self.id);
        self.region.save_resource()
    }

    /// Сохраняет `Load -> success gate -> CreateAreaArray` без domain effects.
    pub(crate) fn load_region_resource(
        &mut self,
        source: Option<&[u8]>,
        area_width: i32,
        area_height: i32,
    ) -> Result<bool, ServerRegionLoadError> {
        self.region.set_id(self.id);
        let loaded = self
            .region
            .load_resource(source)
            .map_err(ServerRegionLoadError::Region)?;
        if !loaded {
            return Ok(false);
        }
        self.create_area_array(area_width, area_height)
            .map_err(ServerRegionLoadError::AreaGrid)?;
        Ok(true)
    }

    /// Пересоздаёт весь area-grid и назначает каждой клетке region parent/X/Y.
    pub(crate) fn create_area_array(
        &mut self,
        area_width: i32,
        area_height: i32,
    ) -> Result<(), AreaGridBlock> {
        if area_width <= 0 || area_height <= 0 {
            // BLOCKED_MISSING_FACT: x86 `idiv` trap для нуля и последующая
            // signed allocation для отрицательного span не задают safe contract.
            return Err(AreaGridBlock::InvalidAreaSpan {
                width: area_width,
                height: area_height,
            });
        }
        if self.region.width < 0 || self.region.height < 0 {
            return Err(AreaGridBlock::InvalidRegionDimensions {
                width: self.region.width,
                height: self.region.height,
            });
        }

        self.area_x = ceil_positive_division(self.region.width, area_width);
        self.area_y = ceil_positive_division(self.region.height, area_height);
        self.areas.clear();

        let Some(area_count_i32) = self.area_x.checked_mul(self.area_y) else {
            return Err(AreaGridBlock::GridSizeOverflow {
                area_x: self.area_x,
                area_y: self.area_y,
            });
        };
        let Ok(area_count) = usize::try_from(area_count_i32) else {
            return Err(AreaGridBlock::GridSizeOverflow {
                area_x: self.area_x,
                area_y: self.area_y,
            });
        };
        if area_count > (u32::MAX as usize - 4) / 0x118 {
            return Err(AreaGridBlock::GridSizeOverflow {
                area_x: self.area_x,
                area_y: self.area_y,
            });
        }

        self.areas
            .resize_with(area_count, CArea::with_storage_defaults);
        let mut x = 0;
        while x < self.area_x {
            let mut y = 0;
            while y < self.area_y {
                let index =
                    usize::try_from(self.area_x * y + x).expect("положительные grid dimensions");
                self.areas[index].assign_to_server_region(x, y);
                y += 1;
            }
            x += 1;
        }
        Ok(())
    }

    /// Safe-граница исходного unbounded `GetArea(long)`.
    pub(crate) fn get_area_by_index(&self, index: i32) -> Result<&CArea, AreaIndexBlock> {
        let Ok(index_usize) = usize::try_from(index) else {
            return Err(AreaIndexBlock {
                index,
                available: self.areas.len(),
            });
        };
        self.areas.get(index_usize).ok_or(AreaIndexBlock {
            index,
            available: self.areas.len(),
        })
    }

    /// Сохраняет bounds-check и `nullptr` coordinate-overload-а.
    pub(crate) fn get_area(&self, x: i32, y: i32) -> Option<&CArea> {
        if x < 0 || x >= self.area_x || y < 0 || y >= self.area_y {
            return None;
        }
        let index = usize::try_from(self.area_x * y + x).expect("положительный grid index");
        self.areas.get(index)
    }

    /// Собирает player IDs одной area без чтения их координат: исходный
    /// `CArea::FindShapes(400)` использовал только RTTI и inherited socket ID.
    pub(crate) fn find_player_ids_in_area(
        &self,
        area_x: i32,
        area_y: i32,
        destination: &mut Vec<i32>,
    ) {
        let Some(area) = self.get_area(area_x, area_y) else {
            return;
        };
        self.append_registered_player_ids(area, destination);
    }

    /// Обходит все `CArea` в физическом row-major storage order и сохраняет
    /// exact `FindShapes(400)` filtering через registry owning region-а.
    pub(crate) fn find_all_player_ids(&self, destination: &mut Vec<i32>) {
        for area in &self.areas {
            self.append_registered_player_ids(area, destination);
        }
    }

    /// Безопасно заменяет исходный `CArea::m_pFather`: пара принимается только
    /// если area действительно принадлежит этому server-region.
    pub(crate) fn find_player_ids_in_area_object(
        &self,
        area: &CArea,
        destination: &mut Vec<i32>,
    ) -> bool {
        let Some(owned_area) = self
            .areas
            .iter()
            .find(|owned_area| std::ptr::eq(*owned_area, area))
        else {
            return false;
        };
        self.append_registered_player_ids(owned_area, destination);
        true
    }

    fn append_registered_player_ids(&self, area: &CArea, destination: &mut Vec<i32>) {
        let mut area_player_ids = Vec::new();
        if !area.append_player_ids(&mut area_player_ids) {
            return;
        }
        for player_id in area_player_ids {
            if self.registry.contains(ShapeIdentity {
                object_type: PLAYER_TYPE,
                id: player_id,
                ex_id: CGuid::GUID_INVALID,
            }) {
                destination.push(player_id);
            }
        }
    }

    pub(crate) fn find_child_object<Resolver: ShapeResolver>(
        &self,
        object_type: i32,
        id: i32,
        ex_id: CGuid,
        resolver: &Resolver,
    ) -> Option<ShapeView> {
        let identity = ShapeIdentity {
            object_type,
            id: if object_type == GOODS_TYPE { 0 } else { id },
            ex_id: if object_type == GOODS_TYPE {
                ex_id
            } else {
                CGuid::GUID_INVALID
            },
        };
        if !self.registry.contains(identity) {
            return None;
        }
        resolver.resolve_shape(identity)
    }

    pub(crate) fn contains_child_object<Resolver: ShapeResolver>(
        &self,
        shape: &CShape,
        resolver: &Resolver,
    ) -> bool {
        let identity = shape.identity();
        self.find_child_object(identity.object_type, identity.id, identity.ex_id, resolver)
            .is_some()
    }

    pub(crate) fn get_player_amount(&self) -> u32 {
        self.registry.players.len() as u32
    }

    pub(crate) fn add_object<Context: ServerRegionMembershipContext>(
        &mut self,
        shape: &mut CShape,
        facts: ShapeRuntimeFacts,
        area_width: i32,
        area_height: i32,
        now_ms: u32,
        context: &mut Context,
    ) -> Result<(), RegionMembershipBlock> {
        validate_area_span(area_width, area_height)?;
        let mut tile_x = shape
            .get_tile_x()
            .map_err(RegionMembershipBlock::ShapeCoordinate)?;
        let mut tile_y = shape
            .get_tile_y()
            .map_err(RegionMembershipBlock::ShapeCoordinate)?;
        shape.assign_to_server_region();

        if (tile_x < 0 || tile_x >= self.region.width || tile_y < 0 || tile_y >= self.region.height)
            && !self.region.cells.is_empty()
        {
            let position = self
                .region
                .get_random_pos(context)
                .map_err(RegionMembershipBlock::RegionCell)?;
            tile_x = position.x;
            tile_y = position.y;
        }
        shape.set_tile_xy(&mut self.region, tile_x, tile_y, context)?;

        let identity = shape.identity();
        self.registry.add(identity, facts);
        let area_index = self.area_index_for_tile(tile_x, tile_y, area_width, area_height);
        if let Some(area_index) = area_index {
            self.areas[area_index].add_object(identity, facts, now_ms);
            shape.set_area_index(Some(area_index));

            if identity.object_type == PLAYER_TYPE {
                let neighbors = self.areas[area_index].player_enter_neighbors();
                for (area_x, area_y) in neighbors {
                    if self.get_area(area_x, area_y).is_some() {
                        context.wake_up_area_monsters(area_x, area_y);
                    }
                }
            }
            if facts.is_move_shape {
                context.move_shape_entered_area(identity);
            }
        } else {
            self.remove_object(shape, facts)?;
        }

        if identity.object_type == PLAYER_TYPE
            && facts.is_player
            && !context.is_gods_battle_region(self.id)
        {
            context.notify_player_left_gods_region(identity.id);
        }
        Ok(())
    }

    pub(crate) fn remove_object(
        &mut self,
        shape: &mut CShape,
        facts: ShapeRuntimeFacts,
    ) -> Result<(), RegionMembershipBlock> {
        let identity = shape.identity();
        if let Some(area_index) = shape.area_index() {
            let available = self.areas.len();
            let area =
                self.areas
                    .get_mut(area_index)
                    .ok_or(RegionMembershipBlock::StaleAreaIndex {
                        index: area_index,
                        available,
                    })?;
            area.remove_object(identity, facts);
            shape.set_area_index(None);

            if matches!(identity.object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE) {
                let tile_x = shape
                    .get_tile_x()
                    .map_err(RegionMembershipBlock::ShapeCoordinate)?;
                let tile_y = shape
                    .get_tile_y()
                    .map_err(RegionMembershipBlock::ShapeCoordinate)?;
                shape
                    .set_block(&mut self.region, tile_x, tile_y, 0, facts.figure)
                    .map_err(RegionMembershipBlock::ShapeBlock)?;
            }
        }
        self.registry.remove(identity);
        Ok(())
    }

    pub(crate) fn set_move_shape_position(
        &mut self,
        shape: &mut CShape,
        x: f32,
        y: f32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), RegionMembershipBlock> {
        let facts = self.complete_move_shape_position_facts(shape, facts)?;
        let mut dispatch = MoveShapePositionDispatch { facts };
        dispatch
            .set_pos_xy(&mut self.region, shape, x, y)
            .map_err(RegionMembershipBlock::MoveShape)
    }

    pub(crate) fn set_move_shape_tile_position(
        &mut self,
        shape: &mut CShape,
        tile_x: i32,
        tile_y: i32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), RegionMembershipBlock> {
        let facts = self.complete_move_shape_position_facts(shape, facts)?;
        let mut dispatch = MoveShapePositionDispatch { facts };
        shape
            .set_tile_xy(&mut self.region, tile_x, tile_y, &mut dispatch)
            .map_err(RegionMembershipBlock::MoveShape)
    }

    fn complete_move_shape_position_facts(
        &self,
        shape: &CShape,
        mut facts: MoveShapePositionFacts,
    ) -> Result<MoveShapePositionFacts, RegionMembershipBlock> {
        facts.current_area = match shape.area_index() {
            Some(index) => {
                let available = self.areas.len();
                let area = self
                    .areas
                    .get(index)
                    .ok_or(RegionMembershipBlock::StaleAreaIndex { index, available })?;
                Some(ShapeAreaCoordinates {
                    x: area.x(),
                    y: area.y(),
                })
            }
            None => None,
        };
        Ok(facts)
    }

    /// Собирает только достигнутую `CS_CHANGEAREA` ветвь area AI scan.
    /// Первый insert сбрасывает state; уже присутствующий pointer оригинал
    /// пропускал без сброса.
    pub(crate) fn stage_area_transition(&mut self, shape: &mut CShape) -> bool {
        if shape.change_state() != SHAPE_CHANGE_AREA {
            return false;
        }
        let identity = shape.identity();
        if self.change_area_shapes.contains(&identity) {
            return false;
        }
        self.change_area_shapes.push(identity);
        shape.set_change_state(SHAPE_CHANGE_NONE);
        true
    }

    /// Возвращает ordered snapshot, не очищая исходный list до применения всех
    /// `OnShapeChangeArea`, как в конце original region AI.
    pub(crate) fn staged_area_transitions(&self) -> Vec<ShapeIdentity> {
        self.change_area_shapes.clone()
    }

    pub(crate) fn clear_staged_area_transitions(&mut self) {
        self.change_area_shapes.clear();
    }

    pub(crate) fn apply_area_transition<
        Resolver: ShapeResolver,
        Context: ServerRegionAreaTransitionContext,
    >(
        &mut self,
        shape: &mut CShape,
        facts: ShapeRuntimeFacts,
        now_ms: u32,
        resolver: &Resolver,
        context: &mut Context,
    ) -> Result<bool, AreaTransitionBlock> {
        let Some(current_index) = shape.area_index() else {
            return Ok(false);
        };
        let available = self.areas.len();
        let current_area =
            self.areas
                .get(current_index)
                .ok_or(AreaTransitionBlock::StaleAreaIndex {
                    index: current_index,
                    available,
                })?;
        let current = ShapeAreaCoordinates {
            x: current_area.x(),
            y: current_area.y(),
        };
        let next = shape.next_area_coordinates();
        if current == next {
            return Ok(false);
        }

        let old_neighbors = self.neighbor_area_indices(current);
        let mut new_exclusive = self.neighbor_area_indices(next);
        new_exclusive.retain(|index| !old_neighbors.contains(index));

        let moving = shape.identity();
        let moving_payload = context.serialize_moving_shape_for_area(moving);
        let registered = RegisteredShapeResolver {
            registry: &self.registry,
            resolver,
        };
        for area_index in new_exclusive {
            let area = &self.areas[area_index];
            if area.get_num_shapes() == 0 {
                continue;
            }
            context.send_shape_entered_area(area.x(), area.y(), moving, &moving_payload);
            if moving.object_type == PLAYER_TYPE {
                let mut shapes = Vec::new();
                area.get_all_shapes(&registered, &mut shapes);
                for other in shapes {
                    if other.identity == moving {
                        continue;
                    }
                    let payload = context.serialize_area_shape(other);
                    context.send_area_shape_to_player(moving.id, other, &payload);
                }
            }
        }

        let Some(target_index) = self.area_index_by_coordinates(next) else {
            return Ok(false);
        };
        self.areas[current_index].remove_object(moving, facts);
        self.areas[target_index].add_object(moving, facts, now_ms);
        shape.set_area_index(Some(target_index));

        if moving.object_type == PLAYER_TYPE {
            for (area_x, area_y) in self.areas[target_index].player_enter_neighbors() {
                if self.get_area(area_x, area_y).is_some() {
                    context.wake_up_area_monsters(area_x, area_y);
                }
            }
        }
        Ok(true)
    }

    pub(crate) fn refresh_blocks<Resolver: MoveShapeResolver>(
        &mut self,
        resolver: &Resolver,
    ) -> Result<(), RegionMembershipBlock> {
        let mut x = 0;
        while x < self.region.width {
            let mut y = 0;
            while y < self.region.height {
                if self
                    .region
                    .get_block(x, y)
                    .map_err(RegionMembershipBlock::RegionCell)?
                    == 3
                {
                    self.region
                        .set_block(x, y, 0)
                        .map_err(RegionMembershipBlock::RegionCell)?;
                }
                y += 1;
            }
            x += 1;
        }

        let registered = RegisteredShapeResolver {
            registry: &self.registry,
            resolver,
        };
        for area in &self.areas {
            let mut shapes = Vec::new();
            area.get_all_shapes(&registered, &mut shapes);
            for shape in shapes {
                let Some(is_alive) = resolver.move_shape_is_alive(shape.identity) else {
                    continue;
                };
                if shape.identity.object_type != NPC_TYPE && !is_alive {
                    continue;
                }
                self.region
                    .set_block(shape.tile_x, shape.tile_y, 3)
                    .map_err(RegionMembershipBlock::RegionCell)?;
            }
        }
        Ok(())
    }

    pub(crate) fn refresh_block<Resolver: MoveShapeResolver>(
        &mut self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        resolver: &Resolver,
    ) -> Result<(), RegionMembershipBlock> {
        if tile_x < 0 || tile_y < 0 || tile_x > self.region.width || tile_y > self.region.height {
            return Ok(());
        }
        if self
            .region
            .get_block(tile_x, tile_y)
            .map_err(RegionMembershipBlock::RegionCell)?
            == 3
        {
            self.region
                .set_block(tile_x, tile_y, 0)
                .map_err(RegionMembershipBlock::RegionCell)?;
        }

        let mut shapes = Vec::new();
        self.get_shapes(
            tile_x,
            tile_y,
            area_width,
            area_height,
            resolver,
            &mut shapes,
        )?;
        for shape in shapes {
            let Some(is_alive) = resolver.move_shape_is_alive(shape.identity) else {
                continue;
            };
            if shape.identity.object_type == NPC_TYPE || is_alive {
                self.region
                    .set_block(shape.tile_x, shape.tile_y, 3)
                    .map_err(RegionMembershipBlock::RegionCell)?;
            }
        }
        Ok(())
    }

    pub(crate) fn get_shape<Resolver: ShapeResolver>(
        &self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        resolver: &Resolver,
    ) -> Result<Option<ShapeView>, RegionMembershipBlock> {
        validate_area_span(area_width, area_height)?;
        let registered = RegisteredShapeResolver {
            registry: &self.registry,
            resolver,
        };

        for (offset_x, offset_y) in NEIGHBOR_AREAS {
            let area_x = (tile_x / area_width).wrapping_add(offset_x);
            let area_y = (tile_y / area_height).wrapping_add(offset_y);
            let Some(area) = self.get_area(area_x, area_y) else {
                continue;
            };
            let mut area_shapes = Vec::new();
            area.get_all_shapes(&registered, &mut area_shapes);
            for shape in area_shapes {
                if shape_covers_tile(shape, tile_x, tile_y) {
                    return Ok(Some(shape));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn get_shapes<Resolver: ShapeResolver>(
        &self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
        resolver: &Resolver,
        destination: &mut Vec<ShapeView>,
    ) -> Result<(), RegionMembershipBlock> {
        validate_area_span(area_width, area_height)?;
        let registered = RegisteredShapeResolver {
            registry: &self.registry,
            resolver,
        };

        for (offset_x, offset_y) in NEIGHBOR_AREAS {
            let area_x = (tile_x / area_width).wrapping_add(offset_x);
            let area_y = (tile_y / area_height).wrapping_add(offset_y);
            let Some(area) = self.get_area(area_x, area_y) else {
                continue;
            };
            let mut area_shapes = Vec::new();
            area.get_all_shapes(&registered, &mut area_shapes);
            for shape in area_shapes {
                if shape_covers_tile(shape, tile_x, tile_y) {
                    destination.push(shape);
                }
            }
        }
        Ok(())
    }

    fn neighbor_area_indices(&self, center: ShapeAreaCoordinates) -> Vec<usize> {
        let mut indices = Vec::new();
        for (offset_x, offset_y) in NEIGHBOR_AREAS {
            let coordinates = ShapeAreaCoordinates {
                x: center.x.wrapping_add(offset_x),
                y: center.y.wrapping_add(offset_y),
            };
            if let Some(index) = self.area_index_by_coordinates(coordinates) {
                indices.push(index);
            }
        }
        indices
    }

    fn area_index_by_coordinates(&self, coordinates: ShapeAreaCoordinates) -> Option<usize> {
        if coordinates.x < 0
            || coordinates.x >= self.area_x
            || coordinates.y < 0
            || coordinates.y >= self.area_y
        {
            return None;
        }
        usize::try_from(self.area_x * coordinates.y + coordinates.x).ok()
    }

    fn area_index_for_tile(
        &self,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> Option<usize> {
        let area_x = tile_x / area_width;
        let area_y = tile_y / area_height;
        if area_x < 0 || area_x >= self.area_x || area_y < 0 || area_y >= self.area_y {
            return None;
        }
        usize::try_from(self.area_x * area_y + area_x).ok()
    }

    pub(crate) fn decode_return_setup_prefix(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), ServerReturnSetupInputBlock> {
        let offset = *cursor;
        let required = 0x20usize;
        let Some(end) = offset.checked_add(required) else {
            return Err(ServerReturnSetupInputBlock {
                offset,
                required,
                available: source.len().saturating_sub(offset),
            });
        };
        let Some(bytes) = source.get(offset..end) else {
            return Err(ServerReturnSetupInputBlock {
                offset,
                required,
                available: source.len().saturating_sub(offset),
            });
        };
        let read_i32 = |at: usize| {
            i32::from_le_bytes(
                bytes[at..at + 4]
                    .try_into()
                    .expect("проверенный 0x20-байтовый setup"),
            )
        };
        self.return_setup = Some(ServerReturnSetup {
            region_id: read_i32(0x00),
            left: read_i32(0x04),
            top: read_i32(0x08),
            right: read_i32(0x0C),
            bottom: read_i32(0x10),
            does_recall_when_lost: read_i32(0x14),
            move_monster_when_refeash: read_i32(0x18),
            use_return: read_i32(0x1C),
        });
        *cursor = end;
        Ok(())
    }

    /// Читает полный startup/reload snapshot `regions/{id}.rs`: восемь
    /// signed DWORD setup-а, count и C-string set запрещённых товаров.
    pub(crate) fn decord_setup_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
    ) -> Result<bool, ServerRegionSetupDecodeError> {
        self.decode_return_setup_prefix(source, cursor)
            .map_err(ServerRegionSetupDecodeError::Setup)?;

        // RVA 0x0007EAC0 очищает tree до чтения count и повторяет clear после
        // него. Второй clear не наблюдаем для owned BTreeSet.
        self.forbidden_make_goods.clear();
        let count = read_setup_i32(source, cursor, "m_ForbidMakeGoods.size")?;
        for _ in 0..count.max(0) {
            let value = read_setup_c_string(source, cursor, "m_ForbidMakeGoods[]")?;
            self.forbidden_make_goods.insert(value);
        }
        Ok(true)
    }

    /// Сохраняет `std::set<std::string>::find`: вход рассматривается как
    /// C-string, поэтому байты после первого NUL не участвуют в lookup.
    pub(crate) fn find_forbid_good(&self, name: &[u8]) -> bool {
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.forbidden_make_goods.contains(&name[..end])
    }

    pub(crate) fn get_return_point(
        &self,
        player: Option<ServerReturnPlayer>,
        country_param: &mut CCountryParam,
    ) -> Result<RegionReturnPoint, ServerReturnSetupBlock> {
        let Some(player) = player else {
            return Ok(self.region.get_return_point());
        };
        let setup = self.return_setup.ok_or(ServerReturnSetupBlock)?;
        if setup.use_return != 0 {
            return Ok(RegionReturnPoint {
                region_id: setup.region_id,
                left: setup.left,
                top: setup.top,
                right: setup.right,
                bottom: setup.bottom,
                direction: -1,
            });
        }

        let main = country_param.main_return_point(player.country);
        Ok(RegionReturnPoint {
            region_id: main.region_id,
            left: main.rect.left,
            top: main.rect.top,
            right: main.rect.right,
            bottom: main.rect.bottom,
            direction: main.direction,
        })
    }

    pub(crate) fn does_recall_when_lost(&self) -> Result<i32, ServerReturnSetupBlock> {
        self.return_setup
            .map(|setup| setup.does_recall_when_lost)
            .ok_or(ServerReturnSetupBlock)
    }

    pub(crate) fn on_war_declare(&mut self, war_number: i32) {
        self.city_state = CITY_STATE_DECLARE;
        self.war_number = war_number;
    }

    pub(crate) fn on_war_start(&mut self, _war_number: i32) {
        self.city_state = CITY_STATE_FIGHT;
    }

    pub(crate) fn on_war_time_out(&mut self, _war_number: i32) {}

    pub(crate) fn on_war_end(&mut self, _war_number: i32) {
        self.city_state = CITY_STATE_NONE;
        self.war_number = 0;
    }

    pub(crate) fn on_war_mass(&mut self, _war_number: i32) {
        self.city_state = CITY_STATE_MASS;
    }

    pub(crate) fn on_clear_other_player(&mut self, _war_number: i32) {}

    pub(crate) fn on_refresh_region(&mut self, _war_number: i32) {}

    pub(crate) fn set_owned_city_org(&mut self, faction_id: i32, union_id: i32) {
        self.param.owned_faction_id = faction_id;
        self.param.owned_union_id = union_id;
    }

    pub(crate) fn owned_city_faction(&self) -> i32 {
        self.param.owned_faction_id
    }

    pub(crate) fn owned_city_union(&self) -> i32 {
        self.param.owned_union_id
    }

    pub(crate) fn set_war_number(&mut self, war_number: i32) {
        self.war_number = war_number;
    }

    pub(crate) fn get_war_number(&self) -> i32 {
        self.war_number
    }

    pub(crate) fn get_city_state(&self) -> i32 {
        self.city_state
    }

    pub(crate) fn set_city_state(&mut self, state: i32) {
        self.city_state = state;
    }

    pub(crate) fn reset_war_state(&mut self, war_number: i32, state: i32) {
        self.war_number = war_number;
        self.city_state = state;
    }

    pub(crate) fn start_clear_player_out_at(&mut self, delay_ms: i32, now_ms: u32) {
        self.kick_out_player = true;
        self.kick_out_player_time = delay_ms;
        self.last_time_ms = now_ms;
    }
}

fn read_setup_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ServerRegionSetupDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(ServerRegionSetupDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(ServerRegionSetupDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    *cursor = end;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("проверены четыре байта"),
    ))
}

fn read_setup_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, ServerRegionSetupDecodeError> {
    let mut value = Vec::new();
    loop {
        let offset = *cursor;
        let Some(byte) = source.get(offset).copied() else {
            return Err(ServerRegionSetupDecodeError::UnexpectedEnd {
                field,
                offset,
                needed: 1,
                available: 0,
            });
        };
        *cursor = offset + 1;
        if byte == 0 {
            return Ok(value);
        }
        value.push(byte);
    }
}

fn ceil_positive_division(value: i32, divisor: i32) -> i32 {
    let quotient = value / divisor;
    quotient + i32::from(value % divisor != 0)
}

fn validate_area_span(width: i32, height: i32) -> Result<(), RegionMembershipBlock> {
    if width <= 0 || height <= 0 {
        // BLOCKED_MISSING_FACT: zero вызывает x86 `idiv` trap, negative
        // GlobeSetup span не имеет доказанного переносимого runtime contract.
        return Err(RegionMembershipBlock::InvalidAreaSpan { width, height });
    }
    Ok(())
}

fn wrapping_abs_difference(left: i32, right: i32) -> i32 {
    let difference = left.wrapping_sub(right);
    let sign = difference >> 31;
    (difference ^ sign).wrapping_sub(sign)
}

fn shape_covers_tile(shape: ShapeView, tile_x: i32, tile_y: i32) -> bool {
    wrapping_abs_difference(shape.tile_x, tile_x) <= i32::from(shape.figure.get(2))
        && wrapping_abs_difference(shape.tile_y, tile_y) <= i32::from(shape.figure.get(0))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp

// IMPLEMENTED: `CServerRegion::GetArea` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CServerRegion::GetPlayerAmout` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::StartClearPlayerOut
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0007BA50
//
// IMPLEMENTED выше: clear timer flags и caller-supplied wrapping tick; технические STL/SEH детали удалены.

// IMPLEMENTED: `CServerRegion::Save` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CServerRegion::New` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::AddRegionParamToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:659
// RVA: 0x0007BAA0
// ADDRESS: 0047baa0
// PROTOTYPE: bool __thiscall AddRegionParamToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DoesRecallWhenLost
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:666
// RVA: 0x0007BAC0
//
// Реализовано выше прямым чтением setup; player pointer исходник не использует.

// ============================================================================
// FUNCTION: CServerRegion::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1209
// RVA: 0x0007BAD0
// ADDRESS: 0047bad0
// PROTOTYPE: void __thiscall DeleteChildObject(long param_1, long param_2, CGUID * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::GetArea` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::AdjustTaxRate(CPlayer*)'::__l6::PlayerAdjustTaxRate::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2272
// RVA: 0x0007BBE0
// ADDRESS: 0047bbe0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::ObtainTaxPayment(CPlayer*)'::__l6::PlayerObtainTaxPayment::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2431
// RVA: 0x0007BBF0
// ADDRESS: 0047bbf0
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetNpcNameListNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2609
// RVA: 0x0007BCB0
// ADDRESS: 0047bcb0
// PROTOTYPE: long __thiscall GetNpcNameListNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTotalTaxPayment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:400
// RVA: 0x0007BD00
// ADDRESS: 0047bd00
// PROTOTYPE: void __thiscall SetTotalTaxPayment(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::FindChildObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::FindAroundObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1611
// RVA: 0x0007BD90
// ADDRESS: 0047bd90
// PROTOTYPE: CBaseObject * __thiscall FindAroundObject(CShape * param_1, long param_2, long param_3, CGUID * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::CreateAreaArray` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::SetGoodsProtection
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2160
// RVA: 0x0007BFB0
// ADDRESS: 0047bfb0
// PROTOTYPE: void __thiscall SetGoodsProtection(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::CanPickUpGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2172
// RVA: 0x0007BFE0
// ADDRESS: 0047bfe0
// PROTOTYPE: int __thiscall CanPickUpGoods(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::AdjustTaxRate(CPlayer*)'::__l6::PlayerAdjustTaxRate::DoAsyncCall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2275
// RVA: 0x0007C010
// ADDRESS: 0047c010
// PROTOTYPE: void __thiscall DoAsyncCall(__int64 param_1, long param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::UpdateTaxToWorldServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2405
// RVA: 0x0007C0C0
// ADDRESS: 0047c0c0
// PROTOTYPE: void __thiscall UpdateTaxToWorldServer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::ObtainTaxPayment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2416
// RVA: 0x0007C160
// ADDRESS: 0047c160
// PROTOTYPE: void __thiscall ObtainTaxPayment(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetNpcNameListLength
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2603
// RVA: 0x0007C210
// ADDRESS: 0047c210
// PROTOTYPE: long __thiscall GetNpcNameListLength(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::Load` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::AdjustTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2260
// RVA: 0x0007C2B0
// ADDRESS: 0047c2b0
// PROTOTYPE: void __thiscall AdjustTaxRate(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTaxRateAndUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2334
// RVA: 0x0007C370
// ADDRESS: 0047c370
// PROTOTYPE: void __thiscall SetTaxRateAndUpdate(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetMonsterRefeash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:990
// RVA: 0x0007C4B0
// ADDRESS: 0047c4b0
// PROTOTYPE: tagMonster * __thiscall GetMonsterRefeash(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SendWeatherInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2086
// RVA: 0x0007C610
// ADDRESS: 0047c610
// PROTOTYPE: void __thiscall SendWeatherInfo(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetNpcNameList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2571
// RVA: 0x0007C710
// ADDRESS: 0047c710
// PROTOTYPE: uchar * __thiscall GetNpcNameList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1218
// RVA: 0x0007CBE0
// ADDRESS: 0047cbe0
// PROTOTYPE: void __thiscall DeleteChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::RemoveObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::FindMonsterByID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1803
// RVA: 0x0007D090
// ADDRESS: 0047d090
// PROTOTYPE: CMonster * __thiscall FindMonsterByID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindNpcById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2613
// RVA: 0x0007D0D0
// ADDRESS: 0047d0d0
// PROTOTYPE: CNpc * __thiscall FindNpcById(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::AdjustTaxRate(CPlayer*)'::__l6::PlayerAdjustTaxRate::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2288
// RVA: 0x0007D740
// ADDRESS: 0047d740
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::CollectTodayTax
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2382
// RVA: 0x0007D9B0
// ADDRESS: 0047d9b0
// PROTOTYPE: void __thiscall CollectTodayTax(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_virtual_void___thiscall_CServerRegion::ObtainTaxPayment(CPlayer*)'::__l6::PlayerObtainTaxPayment::OnAsyncCallback
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2443
// RVA: 0x0007DAF0
// ADDRESS: 0047daf0
// PROTOTYPE: void __thiscall OnAsyncCallback(tagAsyncResult * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:842
// RVA: 0x0007EC50
// ADDRESS: 0047ec50
// PROTOTYPE: CMonster * __thiscall AddMonster(char * param_1, long param_2, long param_3, long param_4, bool param_5, bool param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddSummonedCreature
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:918
// RVA: 0x0007EEC0
// ADDRESS: 0047eec0
// PROTOTYPE: CSummonedCreature * __thiscall AddSummonedCreature(tagMasterInfo * param_1, ulong param_2, long param_3, long param_4, long param_5, ulong param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SendMsgToAroundShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1662
// RVA: 0x0007F100
// ADDRESS: 0047f100
// PROTOTYPE: void __thiscall SendMsgToAroundShape(CShape * param_1, CMessage * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindNearestObjectInArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1704
// RVA: 0x0007F280
// ADDRESS: 0047f280
// PROTOTYPE: CBaseObject * __thiscall FindNearestObjectInArea(CShape * param_1, long param_2, long param_3, long param_4, ushort * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::GetShape` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::GetDropGoodsPos
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1881
// RVA: 0x0007F530
// ADDRESS: 0047f530
// PROTOTYPE: long __thiscall GetDropGoodsPos(long param_1, long param_2, long * param_3, long * param_4, long param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::RefeashBlock` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::ChangeWeather
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:74
// RVA: 0x00080210
// ADDRESS: 00480210
// PROTOTYPE: void __thiscall ChangeWeather(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::OnShapeChangeArea` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::AddMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:746
// RVA: 0x000808B0
// ADDRESS: 004808b0
// PROTOTYPE: int __thiscall AddMonster(char * param_1, long param_2, long param_3, long param_4, long param_5, long param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1003
// RVA: 0x00080A40
// ADDRESS: 00480a40
// PROTOTYPE: int __thiscall AddNpc(tagNpc * param_1, bool param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindNearestAroundObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1679
// RVA: 0x00080DD0
// ADDRESS: 00480dd0
// PROTOTYPE: CBaseObject * __thiscall FindNearestAroundObject(CShape * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindAroundObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1738
// RVA: 0x00080E70
// ADDRESS: 00480e70
// PROTOTYPE: void __thiscall FindAroundObject(CShape * param_1, long param_2, vector<CShape*,std::allocator<CShape*>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindAroundPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1758
// RVA: 0x00080FE0
// ADDRESS: 00480fe0
// PROTOTYPE: void __thiscall FindAroundPlayer(CShape * param_1, vector<CPlayer*,std::allocator<CPlayer*>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::FindAroundPets
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1770
// RVA: 0x00081190
// ADDRESS: 00481190
// PROTOTYPE: void __thiscall FindAroundPets(CShape * param_1, vector<CMonster*,std::allocator<CMonster*>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::GetShape` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::GetReturnPoint
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2104
// RVA: 0x000814F0
//
// Реализовано выше: null default, local setup и country-map fallback;
// layout/call sequence имеют статус VERIFIED_DISASSEMBLY.

// IMPLEMENTED: `CServerRegion::RefeashBlock` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::FindChildObjectByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1547
// RVA: 0x00082340
// ADDRESS: 00482340
// PROTOTYPE: CBaseObject * __thiscall FindChildObjectByName(long param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::BackToCity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2134
// RVA: 0x000825C0
// ADDRESS: 004825c0
// PROTOTYPE: bool __thiscall BackToCity(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddTaxMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2350
// RVA: 0x000826E0
// ADDRESS: 004826e0
// PROTOTYPE: void __thiscall AddTaxMoney(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::KickOutAllPlayerToReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2490
// RVA: 0x00082820
// ADDRESS: 00482820
// PROTOTYPE: void __thiscall KickOutAllPlayerToReturnPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::KickOutAllPlayerToReturnPointExceptOwner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2523
// RVA: 0x000829B0
// ADDRESS: 004829b0
// PROTOTYPE: void __thiscall KickOutAllPlayerToReturnPointExceptOwner(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DelOneGoodFromAllPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2556
// RVA: 0x00082B60
// ADDRESS: 00482b60
// PROTOTYPE: void __thiscall DelOneGoodFromAllPlayer(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetWarSoulXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:2575
// RVA: 0x00082BE0
// ADDRESS: 00482be0
// PROTOTYPE: void __thiscall GetWarSoulXY(long param_1, long param_2, map<unsigned_long,tagPOINT,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,tagPOINT>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::ClearPlayerAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:336
// RVA: 0x00082FF0
// ADDRESS: 00482ff0
// PROTOTYPE: void __thiscall ClearPlayerAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CServerRegion::AddObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CServerRegion::DeleteAllChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:1387
// RVA: 0x00083580
// ADDRESS: 00483580
// PROTOTYPE: void __thiscall DeleteAllChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AddMonsterRect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:672
// RVA: 0x00084040
// ADDRESS: 00484040
// PROTOTYPE: int __thiscall AddMonsterRect(tagMonster * param_1, long param_2, bool param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: сбор, unique insert, state reset и отложенное
// применение `CS_CHANGEAREA` материализованы выше; weather/monster/delete/
// remove/change-region/ClearPlayerAI остаются RAW в этом блоке.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:87
// RVA: 0x00084520
// ADDRESS: 00484520
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::CServerRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:28
// RVA: 0x000852B0
// ADDRESS: 004852b0
// PROTOTYPE: undefined __thiscall CServerRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::OperatorCityGate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:270
// RVA: 0x00085550
// ADDRESS: 00485550
// PROTOTYPE: bool __thiscall OperatorCityGate(long param_1, eOperCityGate param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::OnWarDeclare
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085560
//
// IMPLEMENTED выше: state DUTH и war number; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::OnWarStart
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085580
//
// IMPLEMENTED выше: state Fight; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::OnWarEnd
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085590
//
// IMPLEMENTED выше: state No и нулевой war number; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::OnWarMass
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000855B0
//
// IMPLEMENTED выше: state Mass; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::SymbolIsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:280
// RVA: 0x000855C0
// ADDRESS: 004855c0
// PROTOTYPE: bool __thiscall SymbolIsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetOwnedCityOrg
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000855D0
//
// IMPLEMENTED выше: faction/union ownership; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetOwnedCityFaction
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000855F0
//
// IMPLEMENTED выше: прямое owned-faction field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetOwnedCityUnion
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085600
//
// IMPLEMENTED выше: прямое owned-union field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:392
// RVA: 0x00085610
// ADDRESS: 00485610
// PROTOTYPE: long __thiscall GetTaxRate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::GetMaxTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:393
// RVA: 0x00085620
// ADDRESS: 00485620
// PROTOTYPE: long __thiscall GetMaxTaxRate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTaxRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:395
// RVA: 0x00085630
// ADDRESS: 00485630
// PROTOTYPE: void __thiscall SetTaxRate(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetWarNum
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085640
//
// IMPLEMENTED выше: прямой war-number field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetWarNum
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085650
//
// IMPLEMENTED выше: прямой war-number field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::GetCityState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085660
//
// IMPLEMENTED выше: прямой city-state field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::SetCityState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085670
//
// IMPLEMENTED выше: прямой city-state field access; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::ReSetWarState
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00085680
//
// IMPLEMENTED выше: ordered war-number/state assignment; технические STL/SEH детали удалены.

// ============================================================================
// FUNCTION: CServerRegion::~CServerRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:53
// RVA: 0x000856B0
// ADDRESS: 004856b0
// PROTOTYPE: void __thiscall ~CServerRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.cpp:543
// RVA: 0x000858F0
// ADDRESS: 004858f0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::SetTodayTaxPayment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\serverregion.h:403
// RVA: 0x00089590
// ADDRESS: 00489590
// PROTOTYPE: void __thiscall SetTodayTaxPayment(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
