//! Достигнутая storage-часть `CArea` исторического GameServer.
//!
//! Constructor RVA `0x00075400` и его использование из
//! `CServerRegion::CreateAreaArray` RVA `0x0007BE10` имеют достигнутый статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, PDB
//! `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`.
//! Исходник `server/gameserver/appserver/area.cpp`; старый размер объекта
//! `0x118` подтверждён allocation stride.
//!
//! Constructor создаёт base-state type `300`, координаты `-1/-1`, девять
//! пустых vector-storage, три пустые ordered map и один Win32 critical section.
//! `Vec`, `BTreeMap` и `parking_lot::Mutex` заменяют достигнутые технические
//! механизмы. Membership family `GetNumShapes`, `AddObject`, `RemoveObject`,
//! `FindShapes`, `GetAllShapes` RVA `0x00070B80/0x00073B70/0x000721F0/
//! 0x00073DE0/0x000743A0` также имеет статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`. Она хранит только исходные ID/GUID/hash и получает
//! живой shape-view через resolver исторического
//! `CServerRegion::FindChildObject`; pointer ownership в `CArea` не вводится.
//! `PlayerEnter` RVA `0x00075580` сохраняет точный девяти-area traversal, а
//! сам `WakeUpMonsters` остаётся явным AI callback.
//! `OnRefreshMonster` RVA `0x00101A70`, вызываемый region AI только для area
//! без plug-ов, в точном EXE является намеренным no-op (`ret 4`). Метод
//! оставлен явным, чтобы не потерять подтверждённую границу owner-а и аргумент
//! refresh index при последующей реконструкции другой версии.
//! Inline `CSession::GetPlugList` RVA `0x00070910`, скомпилированный из этого
//! же source-owner, также `IMPLEMENTED`: он возвращает ordered plug-list без
//! копии; Rust slice сохраняет порядок и запрещает чужую мутацию во время
//! around-send обхода.
//! `CreateAreaArray` после построения всего массива записывает в inherited
//! father-slot `+0x40` один и тот же `CServerRegion*`, затем X/Y по
//! `+0x44/+0x48`. Самоссылочный pointer выражен структурным
//! `AreaParentLink::OwningServerRegion`: `CArea` уже физически принадлежит
//! `CServerRegion::areas`, а будущие методы получают живой region-context без
//! `unsafe`. Это сознательная смена формы API, но не identity/topology
//! контракта.
//! `AddWarSoul/DelWarSoul/FindWarSoul` RVA `0x00072F60/0x000710D0/0x00072EA0`
//! теперь сохраняют exact ordered-map semantics: delete только помечает point
//! `(-1,-1)`, а find не публикует такие записи и не перезаписывает output key.
//! `RemoveObject` буквально не удаляет unknown-type hash из
//! `m_vOtherShapes`; это подтверждённая странность оригинала, а не забытый
//! Rust cleanup. Goods timestamp/protection удаляются только при успешном
//! `CGoods` RTTI, а type `700` membership — по base GUID независимо от RTTI.
//! `ShapeRuntimeFacts` переносит только уже доказанные derived RTTI/getter
//! результаты; AI, loot lifetime и protection policy остаются у своих RAW
//! owners.
//! Девять однородных `Unwind@00635210..00635280`, делавших только
//! `operator_delete` временного allocation при exception, сняты общей
//! классификацией: их эффект покрывает RAII достигнутых Rust-контейнеров.
//! Полные constructor/destructor блоки остаются RAW: оба проходят через ещё не
//! материализованный base child-tree, а destructor также фиксирует cleanup
//! всех domain containers. Реализован только reached storage prefix.

use std::collections::BTreeMap;
use std::fmt;

use parking_lot::Mutex;

use super::baseobject::CBaseObject;
use super::session::csession::CSession;
use super::shape::{MonsterAreaClass, ShapeIdentity, ShapeResolver, ShapeRuntimeFacts, ShapeView};
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;
const PET_QUERY_TYPE: i32 = 602;
const CARRIAGE_QUERY_TYPE: i32 = 603;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AreaParentLink {
    Unassigned,
    OwningServerRegion,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WarSoulPoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct AreaGoodsProtection {
    timestamp_ms: u32,
    protection_level: u32,
    owner_id: i32,
}

pub(crate) struct CArea {
    base_object: CBaseObject,
    parent: AreaParentLink,
    x: i32,
    y: i32,
    critical_section: Mutex<()>,
    players: Vec<i32>,
    active_monsters: Vec<i32>,
    sleeping_monsters: Vec<i32>,
    goods: Vec<CGuid>,
    npcs: Vec<i32>,
    pets: Vec<i32>,
    carriages: Vec<i32>,
    other_shapes: Vec<i64>,
    removing_shapes: Vec<i64>,
    dropped_goods_timestamps: BTreeMap<CGuid, u32>,
    war_souls: BTreeMap<u32, WarSoulPoint>,
    goods_protection: BTreeMap<CGuid, AreaGoodsProtection>,
}

impl CArea {
    /// Создаёт только достигнутый storage-prefix исходного constructor-а.
    pub(crate) const fn with_storage_defaults() -> Self {
        let mut base_object = CBaseObject::with_reached_constructor_defaults();
        base_object.set_type(300);
        Self {
            base_object,
            parent: AreaParentLink::Unassigned,
            x: -1,
            y: -1,
            critical_section: Mutex::new(()),
            players: Vec::new(),
            active_monsters: Vec::new(),
            sleeping_monsters: Vec::new(),
            goods: Vec::new(),
            npcs: Vec::new(),
            pets: Vec::new(),
            carriages: Vec::new(),
            other_shapes: Vec::new(),
            removing_shapes: Vec::new(),
            dropped_goods_timestamps: BTreeMap::new(),
            war_souls: BTreeMap::new(),
            goods_protection: BTreeMap::new(),
        }
    }

    pub(crate) const fn assign_to_server_region(&mut self, x: i32, y: i32) {
        self.parent = AreaParentLink::OwningServerRegion;
        self.x = x;
        self.y = y;
    }

    pub(crate) const fn x(&self) -> i32 {
        self.x
    }

    pub(crate) const fn y(&self) -> i32 {
        self.y
    }

    /// Возвращает inherited `CBaseObject::m_lID`, читаемый oversized send-log.
    pub(crate) const fn id(&self) -> i32 {
        self.base_object.get_id()
    }

    /// Типизированный вид inherited `CSession::GetPlugList` для region AI.
    pub(crate) fn plug_list(&self) -> &[i32] {
        &self.players
    }

    /// Сохраняет пустой контракт `CArea::OnRefreshMonster(long)` exact EXE.
    pub(crate) const fn on_refresh_monster(&mut self, _refresh_index: i32) {}

    /// Exact `AddWarSoul`: ID `0` отклоняется, а existing map entry получает
    /// новые координаты вместо создания второй записи.
    pub(crate) fn add_war_soul(&mut self, player_id: u32, point: WarSoulPoint) -> bool {
        if player_id == 0 {
            return false;
        }
        let _guard = self.critical_section.lock();
        self.war_souls.insert(player_id, point);
        true
    }

    /// Exact `DelWarSoul` не erases map entry: совпавшая координата помечается
    /// `(-1, -1)`, а неверный ID/point всё равно возвращает legacy success.
    pub(crate) fn del_war_soul(&mut self, player_id: u32, point: WarSoulPoint) -> bool {
        if player_id == 0 {
            return false;
        }
        let _guard = self.critical_section.lock();
        if self.war_souls.get(&player_id).copied() == Some(point) {
            self.war_souls
                .insert(player_id, WarSoulPoint { x: -1, y: -1 });
        }
        true
    }

    /// Exact `FindWarSoul` обходит ordered map и публикует только не
    /// помеченные `(-1, -1)` записи; duplicate key уже существующего output
    /// оставляет неизменным, как `std::map::insert` в исходнике.
    pub(crate) fn find_war_souls(&self, destination: &mut BTreeMap<u32, WarSoulPoint>) {
        let _guard = self.critical_section.lock();
        for (&player_id, &point) in &self.war_souls {
            if point.x != -1 && point.y != -1 {
                destination.entry(player_id).or_insert(point);
            }
        }
    }

    pub(crate) fn get_num_shapes(&self) -> u32 {
        [
            self.players.len(),
            self.active_monsters.len(),
            self.sleeping_monsters.len(),
            self.pets.len(),
            self.carriages.len(),
            self.goods.len(),
            self.npcs.len(),
            self.other_shapes.len(),
        ]
        .into_iter()
        .fold(0u32, |count, len| count.wrapping_add(len as u32))
    }

    pub(crate) fn add_object(
        &mut self,
        identity: ShapeIdentity,
        facts: ShapeRuntimeFacts,
        now_ms: u32,
    ) {
        let _guard = self.critical_section.lock();
        if let Some(goods) = facts.goods {
            let timestamp = if goods.particular_attribute & 0x10 == 0 {
                now_ms
            } else {
                0
            };
            self.dropped_goods_timestamps
                .insert(identity.ex_id, timestamp);
        }

        match identity.object_type {
            PLAYER_TYPE => self.players.push(identity.id),
            NPC_TYPE => self.npcs.push(identity.id),
            MONSTER_TYPE => match facts.monster {
                Some(MonsterAreaClass::Active) => self.active_monsters.push(identity.id),
                Some(MonsterAreaClass::Pet) => self.pets.push(identity.id),
                Some(MonsterAreaClass::Carriage) => self.carriages.push(identity.id),
                None => {}
            },
            GOODS_TYPE => self.goods.push(identity.ex_id),
            _ => self.other_shapes.push(CBaseObject::get_hash_value(
                identity.object_type,
                identity.id,
            )),
        }
    }

    pub(crate) fn remove_object(&mut self, identity: ShapeIdentity, facts: ShapeRuntimeFacts) {
        let _guard = self.critical_section.lock();
        if facts.goods.is_some() {
            self.dropped_goods_timestamps.remove(&identity.ex_id);
            self.goods_protection.remove(&identity.ex_id);
        }

        match identity.object_type {
            MONSTER_TYPE => {
                remove_first(&mut self.active_monsters, &identity.id);
                remove_first(&mut self.sleeping_monsters, &identity.id);
                remove_first(&mut self.pets, &identity.id);
                remove_first(&mut self.carriages, &identity.id);
            }
            PLAYER_TYPE => remove_first(&mut self.players, &identity.id),
            NPC_TYPE => remove_first(&mut self.npcs, &identity.id),
            GOODS_TYPE => remove_first(&mut self.goods, &identity.ex_id),
            // Подтверждённая странность RVA 0x000721F0: other_shapes здесь
            // вообще не просматривается и не получает deferred marker.
            _ => {}
        }
    }

    pub(crate) fn find_shapes<Resolver: ShapeResolver>(
        &self,
        object_type: i32,
        resolver: &Resolver,
        destination: &mut Vec<ShapeView>,
    ) -> bool {
        let _guard = self.critical_section.lock();
        if self.parent != AreaParentLink::OwningServerRegion {
            return false;
        }

        match object_type {
            PLAYER_TYPE => append_ids(&self.players, PLAYER_TYPE, resolver, destination),
            NPC_TYPE => append_ids(&self.npcs, NPC_TYPE, resolver, destination),
            MONSTER_TYPE => {
                append_ids(&self.active_monsters, MONSTER_TYPE, resolver, destination);
                append_ids(&self.sleeping_monsters, MONSTER_TYPE, resolver, destination);
                append_ids(&self.pets, MONSTER_TYPE, resolver, destination);
                append_ids(&self.carriages, MONSTER_TYPE, resolver, destination);
            }
            PET_QUERY_TYPE => append_ids(&self.pets, MONSTER_TYPE, resolver, destination),
            CARRIAGE_QUERY_TYPE => {
                append_ids(&self.carriages, MONSTER_TYPE, resolver, destination);
            }
            GOODS_TYPE => {
                for ex_id in &self.goods {
                    let identity = ShapeIdentity {
                        object_type: GOODS_TYPE,
                        id: 0,
                        ex_id: *ex_id,
                    };
                    if let Some(shape) = resolver.resolve_shape(identity) {
                        destination.push(shape);
                    }
                }
            }
            _ => {
                for hash in &self.other_shapes {
                    if CBaseObject::calculate_type(*hash) != object_type {
                        continue;
                    }
                    let identity = ShapeIdentity {
                        object_type,
                        id: CBaseObject::calculate_id(*hash),
                        ex_id: CGuid::GUID_INVALID,
                    };
                    if let Some(shape) = resolver.resolve_shape(identity) {
                        destination.push(shape);
                    }
                }
            }
        }
        true
    }

    pub(crate) fn append_player_ids(&self, destination: &mut Vec<i32>) -> bool {
        let _guard = self.critical_section.lock();
        if self.parent != AreaParentLink::OwningServerRegion {
            return false;
        }
        destination.extend_from_slice(&self.players);
        true
    }

    pub(crate) fn get_all_shapes<Resolver: ShapeResolver>(
        &self,
        resolver: &Resolver,
        destination: &mut Vec<ShapeView>,
    ) -> bool {
        for object_type in [PLAYER_TYPE, MONSTER_TYPE, GOODS_TYPE, NPC_TYPE] {
            if !self.find_shapes(object_type, resolver, destination) {
                destination.clear();
                return false;
            }
        }

        let _guard = self.critical_section.lock();
        if self.parent != AreaParentLink::OwningServerRegion {
            destination.clear();
            return false;
        }
        for hash in &self.other_shapes {
            let identity = ShapeIdentity {
                object_type: CBaseObject::calculate_type(*hash),
                id: CBaseObject::calculate_id(*hash),
                ex_id: CGuid::GUID_INVALID,
            };
            if let Some(shape) = resolver.resolve_shape(identity) {
                destination.push(shape);
            }
        }
        true
    }

    pub(crate) fn player_enter_neighbors(&self) -> [(i32, i32); 9] {
        NEIGHBOR_AREAS.map(|(x, y)| (self.x.wrapping_add(x), self.y.wrapping_add(y)))
    }
}

fn append_ids<Resolver: ShapeResolver>(
    ids: &[i32],
    object_type: i32,
    resolver: &Resolver,
    destination: &mut Vec<ShapeView>,
) {
    for id in ids {
        let identity = ShapeIdentity {
            object_type,
            id: *id,
            ex_id: CGuid::GUID_INVALID,
        };
        if let Some(shape) = resolver.resolve_shape(identity) {
            destination.push(shape);
        }
    }
}

fn remove_first<Value: PartialEq>(values: &mut Vec<Value>, needle: &Value) {
    if let Some(index) = values.iter().position(|value| value == needle) {
        values.remove(index);
    }
}

impl Clone for CArea {
    fn clone(&self) -> Self {
        Self {
            base_object: self.base_object.clone(),
            parent: self.parent,
            x: self.x,
            y: self.y,
            critical_section: Mutex::new(()),
            players: self.players.clone(),
            active_monsters: self.active_monsters.clone(),
            sleeping_monsters: self.sleeping_monsters.clone(),
            goods: self.goods.clone(),
            npcs: self.npcs.clone(),
            pets: self.pets.clone(),
            carriages: self.carriages.clone(),
            other_shapes: self.other_shapes.clone(),
            removing_shapes: self.removing_shapes.clone(),
            dropped_goods_timestamps: self.dropped_goods_timestamps.clone(),
            war_souls: self.war_souls.clone(),
            goods_protection: self.goods_protection.clone(),
        }
    }
}

impl fmt::Debug for CArea {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CArea")
            .field("parent", &self.parent)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("players", &self.players.len())
            .field("active_monsters", &self.active_monsters.len())
            .field("sleeping_monsters", &self.sleeping_monsters.len())
            .field("goods", &self.goods.len())
            .field("npcs", &self.npcs.len())
            .field("pets", &self.pets.len())
            .field("carriages", &self.carriages.len())
            .field("other_shapes", &self.other_shapes.len())
            .field("removing_shapes", &self.removing_shapes.len())
            .field(
                "dropped_goods_timestamps",
                &self.dropped_goods_timestamps.len(),
            )
            .field("war_souls", &self.war_souls.len())
            .field("goods_protection", &self.goods_protection.len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for CArea {
    fn eq(&self, other: &Self) -> bool {
        self.base_object == other.base_object
            && self.parent == other.parent
            && self.x == other.x
            && self.y == other.y
            && self.players == other.players
            && self.active_monsters == other.active_monsters
            && self.sleeping_monsters == other.sleeping_monsters
            && self.goods == other.goods
            && self.npcs == other.npcs
            && self.pets == other.pets
            && self.carriages == other.carriages
            && self.other_shapes == other.other_shapes
            && self.removing_shapes == other.removing_shapes
            && self.dropped_goods_timestamps == other.dropped_goods_timestamps
            && self.war_souls == other.war_souls
            && self.goods_protection == other.goods_protection
    }
}

impl Eq for CArea {}

impl CSession {
    pub(crate) fn get_plug_list(&self) -> &[i32] {
        self.plug_ids_storage()
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp

// IMPLEMENTED: `CSession::GetPlugList` материализован выше; покрытый raw-блок
// удалён.

// IMPLEMENTED: `CArea::GetNumShapes` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CArea::CanPickUpGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:239
// RVA: 0x00071060
// ADDRESS: 00471060
// PROTOTYPE: int __thiscall CanPickUpGoods(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::DelWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:1129
// RVA: 0x000710D0
// ADDRESS: 004710d0
// PROTOTYPE: int __thiscall DelWarSoul(ulong param_1, tagPOINT param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CArea::RemoveObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CArea::FindWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:1087
// RVA: 0x00072EA0
// ADDRESS: 00472ea0
// PROTOTYPE: void __thiscall FindWarSoul(map<unsigned_long,tagPOINT,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,tagPOINT>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::AddWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:1104
// RVA: 0x00072F60
// ADDRESS: 00472f60
// PROTOTYPE: int __thiscall AddWarSoul(ulong param_1, tagPOINT param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::~CArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:31
// RVA: 0x000736F0
// ADDRESS: 004736f0
// PROTOTYPE: void __thiscall ~CArea(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::WakeUpMonsters
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:100
// RVA: 0x00073A70
// ADDRESS: 00473a70
// PROTOTYPE: void __thiscall WakeUpMonsters(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CArea::AddObject` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CArea::SetGoodsProtection
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:217
// RVA: 0x00073D60
// ADDRESS: 00473d60
// PROTOTYPE: void __thiscall SetGoodsProtection(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CArea::FindShapes` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CArea::GetAllShapes` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CArea::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:734
// RVA: 0x00074500
// ADDRESS: 00474500
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::GetActivedShapes
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:886
// RVA: 0x00074AB0
// ADDRESS: 00474ab0
// PROTOTYPE: int __thiscall GetActivedShapes(vector<CShape*,std::allocator<CShape*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::GetSleepMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:1037
// RVA: 0x00075160
// ADDRESS: 00475160
// PROTOTYPE: int __thiscall GetSleepMonster(vector<CShape*,std::allocator<CShape*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::WakeUpOneMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:1154
// RVA: 0x000752C0
// ADDRESS: 004752c0
// PROTOTYPE: void __thiscall WakeUpOneMonster(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArea::CArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\area.cpp:23
// RVA: 0x00075400
// ADDRESS: 00475400
// PROTOTYPE: undefined __thiscall CArea(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CArea::PlayerEnter` материализован выше; покрытый raw-блок удалён.

// COMPONENT_VARIANT_END: GameServer
