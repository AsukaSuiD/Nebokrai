//! Статический registry-owner сессий и plug-объектов исторического WorldServer.
//!
//! Статус всех девяти экспортированных функций `CSessionFactory` —
//! `IMPLEMENTED`: `AI` RVA `0x0007B8E0`, `QuerySession` RVA `0x0007B970`,
//! `GarbageCollect` RVA `0x0007B9A0`, `QueryPlug` RVA `0x0007BA40`,
//! `InsertPlug` RVA `0x0007BA70`, `CreateSession` RVA `0x0007C250`,
//! `CreatePlug` RVA `0x0007C370`, `UnserializePlug` RVA `0x0007C480` и
//! `UnserializeSession` RVA `0x0007C560`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\session\csessionfactory.cpp:30,61,74,89,124,168,203,215,242`.
//!
//! Exact PDB задаёт `s_lSessionID/s_lPlugID` как signed `long`, оба registry —
//! как `stdext::hash_map<long, pointer>`, а enum-значения —
//! `ST_NORMAL_SESSION=0`, `ST_TEAM=1`, шесть `PLUG_TYPE=0..5` и
//! `TYPE_SESSION/TYPE_PLUG=10/11`; exact EXE data задаёт обоим ID initial `1`.
//! Factory выбирает только `CSession` либо
//! `CTeam` и только `CTeamate`; сами constructors, virtual API и unserialize-
//! тела принадлежат соседним сырым owners. Rust оставляет их явными trait-
//! границами, но сохраняет factory-owned lifetime через `Box` и вызывает
//! `Drop` ровно там, где исходник звал scalar deleting destructor.
//!
//! `AI` проходит только session registry. Null value стирается сразу;
//! недоступная session сначала получает `Abort`, завершённая — `End`, затем
//! owner уничтожается до erase. Иначе вызывается virtual `AI`, после чего
//! обход продолжается со следующей записью. Конкретные vtable slots проверены
//! в exact EXE (`VERIFIED_DISASSEMBLY`): `CSession` vtable RVA `0x00149604`
//! содержит `AI +0x40`, `IsSessionEnded +0x50`, `IsSessionAvailable +0x60`,
//! `InsertPlug +0x64`, `End +0x6C`, `Abort +0x70`, `Unserialize +0x94`; у
//! `CPlug` vtable RVA `0x00149744` `Unserialize` находится по `+0x84`.
//!
//! Обычный `HashMap` не подходит: достигнутые MSVC `lower_bound/insert` RVA
//! `0x0007B850/0x0007BFF0` хранят элементы в общем list по linear-hash bucket
//! order, а внутри bucket — по signed key.
//! Этот порядок наблюдаем через последовательные virtual side effects `AI`.
//! Стандартные Rust collections не дают выбрать такой iterator contract;
//! узкий `LegacyMsvcHashRegistry` хранит owners в `Vec`, воспроизводит только
//! доказанные `hash = key ^ 0xDEADBEEF`, mask/max-index growth и итоговый
//! `(bucket, signed key)` порядок. Bucket nodes, iterator-vector, allocation и
//! rehash pointer surgery являются удалённым STL noise. При 32-битном ID-
//! collision исходник перезаписывал pointer без destructor; displaced owner
//! поэтому удерживается до process-lifetime factory drop, а не уничтожается
//! в момент замены.
//!
//! Unserialize читает little-endian DWORD после каждого исходного `offset += 4`,
//! создаёт owner, ищет его по возвращённому ID и делегирует virtual body. Ноль
//! virtual-результата удаляет только что найденный owner. `None` заменяет
//! исходный null stream и возвращает ноль без изменения offset. Отрицательный,
//! переполненный либо короткий slice не имеет безопасно доказанной реакции
//! старого pointer-read и получает локальный `BLOCKED_MISSING_FACT`, сохраняя
//! уже выполненное wrapping-присваивание offset.

use std::error::Error;
use std::fmt;

const LEGACY_HASH_XOR: u32 = 0xDEAD_BEEF;
const TYPE_SESSION: i32 = 10;
const TYPE_PLUG: i32 = 11;

/// Два factory-поддержанных вида session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum WorldSessionType {
    Normal = 0,
    Team = 1,
}

impl WorldSessionType {
    const fn from_legacy(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Team),
            _ => None,
        }
    }
}

/// Полный PDB enum; factory создаёт только `Teamate`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum WorldPlugType {
    Normal = 0,
    Trader = 1,
    PersonalShopSeller = 2,
    PersonalShopBuyer = 3,
    EquipmentUpgrade = 4,
    Teamate = 5,
}

impl WorldPlugType {
    const fn from_legacy(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Trader),
            2 => Some(Self::PersonalShopSeller),
            3 => Some(Self::PersonalShopBuyer),
            4 => Some(Self::EquipmentUpgrade),
            5 => Some(Self::Teamate),
            _ => None,
        }
    }
}

/// Virtual contract, принадлежащий ещё сырому `CSession/CTeam` owner-у.
pub(crate) trait WorldSessionOwner {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32);
    fn is_session_available(&mut self) -> i32;
    fn abort(&mut self) -> i32;
    fn is_session_ended(&mut self) -> i32;
    fn end(&mut self) -> i32;
    fn ai(&mut self);
    fn insert_plug(&mut self, plug_id: i32) -> i32;
    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32;
}

/// Virtual contract, принадлежащий ещё сырому `CPlug/CTeamate` owner-у.
pub(crate) trait WorldPlugOwner {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32);
    fn set_owner(&mut self, owner_type: i32, owner_id: i32);
    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32;
}

/// Allocation/constructor-граница четырёх соседних class owners.
pub(crate) trait WorldSessionFactoryAllocator {
    fn construct_session(
        &mut self,
        session_type: WorldSessionType,
        minimum_plugs: u32,
        maximum_plugs: u32,
        lifetime_ms: u32,
    ) -> Option<Box<dyn WorldSessionOwner>>;

    fn construct_plug(&mut self, plug_type: WorldPlugType) -> Option<Box<dyn WorldPlugOwner>>;
}

struct LegacyMsvcHashEntry<T> {
    key: i32,
    value: Option<T>,
}

/// Только наблюдаемая ordered-проекция старого MSVC linear hash container.
struct LegacyMsvcHashRegistry<T> {
    entries: Vec<LegacyMsvcHashEntry<T>>,
    mask: u32,
    max_index: u32,
    bucket_vector_slots: u32,
}

impl<T> LegacyMsvcHashRegistry<T> {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            mask: 1,
            max_index: 1,
            bucket_vector_slots: 9,
        }
    }

    fn position(&self, key: i32) -> Option<usize> {
        self.entries.iter().position(|entry| entry.key == key)
    }

    fn get(&self, key: i32) -> Option<&T> {
        self.position(key)
            .and_then(|index| self.entries[index].value.as_ref())
    }

    fn get_mut(&mut self, key: i32) -> Option<&mut T> {
        let index = self.position(key)?;
        self.entries[index].value.as_mut()
    }

    fn insert_or_replace(&mut self, key: i32, value: Option<T>) -> Option<T> {
        if let Some(index) = self.position(key) {
            return std::mem::replace(&mut self.entries[index].value, value);
        }

        self.grow_before_insert_if_needed();
        let order = self.order_key(key);
        let index = self
            .entries
            .partition_point(|entry| self.order_key(entry.key) < order);
        self.entries
            .insert(index, LegacyMsvcHashEntry { key, value });
        None
    }

    fn grow_before_insert_if_needed(&mut self) {
        if self.max_index > (self.entries.len() as u32) >> 2 {
            return;
        }

        if self.max_index < self.bucket_vector_slots - 1 {
            if self.mask < self.max_index {
                self.mask = self.mask.wrapping_mul(2).wrapping_add(1);
            }
        } else {
            self.mask = self.bucket_vector_slots.wrapping_mul(2).wrapping_sub(3);
            self.bucket_vector_slots = self.bucket_vector_slots.wrapping_mul(2).wrapping_sub(1);
        }
        self.max_index = self.max_index.wrapping_add(1);

        let mask = self.mask;
        let max_index = self.max_index;
        self.entries
            .sort_by_key(|entry| (legacy_bucket(entry.key, mask, max_index), entry.key));
    }

    fn order_key(&self, key: i32) -> (u32, i32) {
        (legacy_bucket(key, self.mask, self.max_index), key)
    }
}

fn legacy_bucket(key: i32, mask: u32, max_index: u32) -> u32 {
    let mut bucket = ((key as u32) ^ LEGACY_HASH_XOR) & mask;
    if max_index <= bucket {
        bucket = bucket.wrapping_sub((mask >> 1).wrapping_add(1));
    }
    bucket
}

/// Итог одной позиции исходного ordered session AI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldSessionFactoryAiEvent {
    NullRemoved { session_id: i32 },
    UnavailableRemoved { session_id: i32, abort_result: i32 },
    EndedRemoved { session_id: i32, end_result: i32 },
    Ran { session_id: i32 },
}

/// Полный ordered результат `CSessionFactory::AI`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldSessionFactoryAiReport {
    pub(crate) events: Vec<WorldSessionFactoryAiEvent>,
}

/// Результат protected `GarbageCollect` без выдачи raw pointer-а наружу.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldSessionFactoryGarbageCollect {
    UnknownObjectType,
    Missing,
    Removed {
        object_type: i32,
        object_id: i32,
        had_owner: bool,
    },
}

/// Safe-граница старого unchecked `stream + signed offset`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldSessionFactoryInputBlock {
    pub(crate) attempted_offset: i32,
    pub(crate) assigned_offset: i32,
    pub(crate) stream_length: usize,
}

impl fmt::Display for WorldSessionFactoryInputBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSessionFactory читает DWORD с offset {}, после присваивания {}; длина stream {} не покрывает старый unchecked read",
            self.attempted_offset, self.assigned_offset, self.stream_length
        )
    }
}

impl Error for WorldSessionFactoryInputBlock {}

/// Владелец двух process-global registry и монотонных signed ID.
pub(crate) struct CSessionFactory {
    next_session_id: i32,
    next_plug_id: i32,
    sessions: LegacyMsvcHashRegistry<Box<dyn WorldSessionOwner>>,
    plugs: LegacyMsvcHashRegistry<Box<dyn WorldPlugOwner>>,
    displaced_sessions: Vec<Box<dyn WorldSessionOwner>>,
    displaced_plugs: Vec<Box<dyn WorldPlugOwner>>,
}

impl CSessionFactory {
    /// Создаёт точные initial static values до первого factory-вызова.
    pub(crate) const fn new() -> Self {
        Self {
            next_session_id: 1,
            next_plug_id: 1,
            sessions: LegacyMsvcHashRegistry::new(),
            plugs: LegacyMsvcHashRegistry::new(),
            displaced_sessions: Vec::new(),
            displaced_plugs: Vec::new(),
        }
    }

    /// Возвращает живой session owner либо исходный `nullptr` как `None`.
    pub(crate) fn query_session(&self, session_id: i32) -> Option<&dyn WorldSessionOwner> {
        self.sessions
            .get(session_id)
            .map(|session| session.as_ref())
    }

    /// Возвращает живой plug owner либо исходный `nullptr` как `None`.
    pub(crate) fn query_plug(&self, plug_id: i32) -> Option<&dyn WorldPlugOwner> {
        self.plugs.get(plug_id).map(|plug| plug.as_ref())
    }

    /// Делегирует `CSession::InsertPlug` найденной ненулевой session.
    pub(crate) fn insert_plug(&mut self, session_id: i32, plug_id: i32) -> i32 {
        self.sessions
            .get_mut(session_id)
            .map_or(0, |session| session.insert_plug(plug_id))
    }

    /// Создаёт `CSession`/`CTeam`, назначает type/ID и публикует owner.
    pub(crate) fn create_session<Allocator>(
        &mut self,
        minimum_plugs: u32,
        maximum_plugs: u32,
        lifetime_ms: u32,
        session_type: i32,
        allocator: &mut Allocator,
    ) -> i32
    where
        Allocator: WorldSessionFactoryAllocator,
    {
        let Some(session_type) = WorldSessionType::from_legacy(session_type) else {
            return 0;
        };
        let Some(mut session) =
            allocator.construct_session(session_type, minimum_plugs, maximum_plugs, lifetime_ms)
        else {
            return 0;
        };

        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        session.assign_factory_identity(TYPE_SESSION, session_id);
        if let Some(displaced) = self.sessions.insert_or_replace(session_id, Some(session)) {
            self.displaced_sessions.push(displaced);
        }
        session_id
    }

    /// Создаёт единственный поддержанный `CTeamate` и публикует plug owner.
    pub(crate) fn create_plug<Allocator>(
        &mut self,
        plug_type: i32,
        owner_type: i32,
        owner_id: i32,
        allocator: &mut Allocator,
    ) -> i32
    where
        Allocator: WorldSessionFactoryAllocator,
    {
        let Some(plug_type) = WorldPlugType::from_legacy(plug_type) else {
            return 0;
        };
        if plug_type != WorldPlugType::Teamate {
            return 0;
        }
        let Some(mut plug) = allocator.construct_plug(plug_type) else {
            return 0;
        };

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        plug.assign_factory_identity(TYPE_PLUG, plug_id);
        plug.set_owner(owner_type, owner_id);
        if let Some(displaced) = self.plugs.insert_or_replace(plug_id, Some(plug)) {
            self.displaced_plugs.push(displaced);
        }
        plug_id
    }

    /// Исполняет exact ordered session traversal и erase-after-destructor.
    pub(crate) fn ai(&mut self) -> WorldSessionFactoryAiReport {
        let mut events = Vec::new();
        let mut index = 0;
        while index < self.sessions.entries.len() {
            let session_id = self.sessions.entries[index].key;
            if self.sessions.entries[index].value.is_none() {
                self.sessions.entries.remove(index);
                events.push(WorldSessionFactoryAiEvent::NullRemoved { session_id });
                continue;
            }

            let available = self.sessions.entries[index]
                .value
                .as_mut()
                .expect("null session отсеян до virtual-вызова")
                .is_session_available();
            if available == 0 {
                let abort_result = self.sessions.entries[index]
                    .value
                    .as_mut()
                    .expect("session остаётся живой до Abort")
                    .abort();
                let session = self.sessions.entries[index]
                    .value
                    .take()
                    .expect("session остаётся живой до deleting destructor");
                drop(session);
                self.sessions.entries.remove(index);
                events.push(WorldSessionFactoryAiEvent::UnavailableRemoved {
                    session_id,
                    abort_result,
                });
                continue;
            }

            let ended = self.sessions.entries[index]
                .value
                .as_mut()
                .expect("session остаётся живой после availability")
                .is_session_ended();
            if ended != 0 {
                let end_result = self.sessions.entries[index]
                    .value
                    .as_mut()
                    .expect("session остаётся живой до End")
                    .end();
                let session = self.sessions.entries[index]
                    .value
                    .take()
                    .expect("session остаётся живой до deleting destructor");
                drop(session);
                self.sessions.entries.remove(index);
                events.push(WorldSessionFactoryAiEvent::EndedRemoved {
                    session_id,
                    end_result,
                });
                continue;
            }

            self.sessions.entries[index]
                .value
                .as_mut()
                .expect("session остаётся живой до AI")
                .ai();
            events.push(WorldSessionFactoryAiEvent::Ran { session_id });
            index += 1;
        }
        WorldSessionFactoryAiReport { events }
    }

    /// Уничтожает найденный owner до erase только для object types `10/11`.
    pub(crate) fn garbage_collect(
        &mut self,
        object_type: i32,
        object_id: i32,
    ) -> WorldSessionFactoryGarbageCollect {
        let had_owner = match object_type {
            TYPE_SESSION => {
                let Some(index) = self.sessions.position(object_id) else {
                    return WorldSessionFactoryGarbageCollect::Missing;
                };
                let owner = self.sessions.entries[index].value.take();
                let had_owner = owner.is_some();
                drop(owner);
                self.sessions.entries.remove(index);
                had_owner
            }
            TYPE_PLUG => {
                let Some(index) = self.plugs.position(object_id) else {
                    return WorldSessionFactoryGarbageCollect::Missing;
                };
                let owner = self.plugs.entries[index].value.take();
                let had_owner = owner.is_some();
                drop(owner);
                self.plugs.entries.remove(index);
                had_owner
            }
            _ => return WorldSessionFactoryGarbageCollect::UnknownObjectType,
        };
        WorldSessionFactoryGarbageCollect::Removed {
            object_type,
            object_id,
            had_owner,
        }
    }

    /// Читает plug header и делегирует virtual `CPlug::Unserialize`.
    pub(crate) fn unserialize_plug<Allocator>(
        &mut self,
        stream: Option<&[u8]>,
        offset: &mut i32,
        allocator: &mut Allocator,
    ) -> Result<i32, WorldSessionFactoryInputBlock>
    where
        Allocator: WorldSessionFactoryAllocator,
    {
        let Some(stream) = stream else {
            return Ok(0);
        };
        let plug_type = read_i32(stream, offset)?;
        let owner_type = read_i32(stream, offset)?;
        let owner_id = read_i32(stream, offset)?;
        let plug_id = self.create_plug(plug_type, owner_type, owner_id, allocator);
        let Some(plug) = self.plugs.get_mut(plug_id) else {
            return Ok(0);
        };
        if plug.unserialize(stream, offset) != 0 {
            return Ok(plug_id);
        }
        let _ = self.garbage_collect(TYPE_PLUG, plug_id);
        Ok(0)
    }

    /// Читает session header и делегирует virtual `CSession::Unserialize`.
    pub(crate) fn unserialize_session<Allocator>(
        &mut self,
        stream: Option<&[u8]>,
        offset: &mut i32,
        allocator: &mut Allocator,
    ) -> Result<i32, WorldSessionFactoryInputBlock>
    where
        Allocator: WorldSessionFactoryAllocator,
    {
        let Some(stream) = stream else {
            return Ok(0);
        };
        let session_type = read_i32(stream, offset)?;
        let minimum_plugs = read_u32(stream, offset)?;
        let maximum_plugs = read_u32(stream, offset)?;
        let lifetime_ms = read_u32(stream, offset)?;
        let session_id = self.create_session(
            minimum_plugs,
            maximum_plugs,
            lifetime_ms,
            session_type,
            allocator,
        );
        let Some(session) = self.sessions.get_mut(session_id) else {
            return Ok(0);
        };
        if session.unserialize(stream, offset) != 0 {
            return Ok(session_id);
        }
        let _ = self.garbage_collect(TYPE_SESSION, session_id);
        Ok(0)
    }
}

fn read_i32(stream: &[u8], offset: &mut i32) -> Result<i32, WorldSessionFactoryInputBlock> {
    read_dword(stream, offset).map(i32::from_le_bytes)
}

fn read_u32(stream: &[u8], offset: &mut i32) -> Result<u32, WorldSessionFactoryInputBlock> {
    read_dword(stream, offset).map(u32::from_le_bytes)
}

fn read_dword(stream: &[u8], offset: &mut i32) -> Result<[u8; 4], WorldSessionFactoryInputBlock> {
    let attempted_offset = *offset;
    *offset = offset.wrapping_add(4);
    let Ok(start) = usize::try_from(attempted_offset) else {
        return Err(WorldSessionFactoryInputBlock {
            attempted_offset,
            assigned_offset: *offset,
            stream_length: stream.len(),
        });
    };
    let Some(end) = start.checked_add(4) else {
        return Err(WorldSessionFactoryInputBlock {
            attempted_offset,
            assigned_offset: *offset,
            stream_length: stream.len(),
        });
    };
    let Some(bytes) = stream.get(start..end) else {
        return Err(WorldSessionFactoryInputBlock {
            attempted_offset,
            assigned_offset: *offset,
            stream_length: stream.len(),
        });
    };
    Ok(bytes
        .try_into()
        .expect("ровно четыре bytes проверены slice-границей"))
}
