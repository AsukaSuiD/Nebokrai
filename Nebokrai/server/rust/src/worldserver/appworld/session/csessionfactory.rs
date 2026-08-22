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
//! Factory напрямую строит конкретные `CSession`, `CTeam` и `CTeamate`.
//! Factory-owned lifetime выражен через `Box`, а `Drop` вызывается ровно там,
//! где исходник звал scalar deleting destructor. Подключённый `OnTeamMessage`
//! использует те же registry через узкие `WorldTeamSessionOwner` и
//! `WorldTeamateOwner` проекции: это safe-эквивалент точных RTTI-переходов, а
//! не отдельное team-состояние.
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
//! collision новая запись уже заменяет старую по тому же ключу, поэтому
//! вытесненный owner недостижим из всех factory lookup. Rust освобождает его
//! сразу: это устраняет старую pointer-утечку без изменения ID, registry-order
//! либо какого-либо наблюдаемого session/plug side effect.
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

use crate::nets::networld::message::CMessage;
use crate::worldserver::appworld::session::csession::{WorldSessionEffect, WorldSessionPlug};
use crate::worldserver::appworld::session::cteam::CTeam;
use crate::worldserver::appworld::session::cteamate::CTeamate;
use crate::worldserver::worldserver::game::CGame;

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
    fn ai_prefix(&mut self) {
        self.ai();
    }
    fn ai_suffix(&mut self) {}
    fn insert_plug(&mut self, plug_id: i32) -> i32;
    fn on_plug_change_state(
        &mut self,
        plug_id: i32,
        state: i32,
        value: &[u8],
        recursive: i32,
    ) -> i32;
    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32;
    fn take_effects(&mut self) -> Vec<WorldSessionEffect> {
        Vec::new()
    }
    fn take_pending_unserialize_plugs(&mut self) -> u32 {
        0
    }
    fn insert_plug_identity(&mut self, _plug: WorldSessionPlug) {}
    fn can_insert_plug(&self) -> bool {
        false
    }
    fn session_plugs(&self) -> &[WorldSessionPlug] {
        &[]
    }
    fn remove_plug_id(&mut self, _plug_id: i32) {}
    fn should_traverse_plugs(&self) -> bool {
        false
    }

    /// Возвращает доказанный `CTeam`-интерфейс либо RTTI-failure как `None`.
    fn as_team_mut(&mut self) -> Option<&mut dyn WorldTeamSessionOwner> {
        None
    }
}

/// Узкая проекция virtual API `CTeam`, достигнутая Team message-owner-ом.
pub(crate) trait WorldTeamSessionOwner {
    fn query_plug_by_owner(&mut self, owner_type: i32, owner_id: i32) -> Option<i32>;
    fn set_leader(&mut self, player_id: i32);
    fn kick_player(&mut self, player_id: i32);
    fn serialize(&mut self, output: &mut Vec<u8>);
    fn allocation_scheme(&mut self) -> i32;
    fn set_allocation_scheme(&mut self, scheme: i32);
    fn on_plug_change_state(&mut self, plug_id: i32, state: i32, value: &[u8]);
}

/// Общий virtual contract factory-owned plug-а.
pub(crate) trait WorldPlugOwner {
    fn assign_factory_identity(&mut self, object_type: i32, object_id: i32);
    fn set_owner(&mut self, owner_type: i32, owner_id: i32);
    fn object_id(&self) -> i32;
    fn owner_type(&self) -> i32;
    fn owner_id(&self) -> i32;
    fn set_session(&mut self, session_id: i32);
    fn is_plug_available(&mut self, game: &CGame) -> i32;
    fn is_plug_ended(&self) -> i32;
    fn on_change_state(&mut self, plug_id: i32, state: i32, value: &[u8]) -> i32;
    fn serialize(&self, output: &mut Vec<u8>) -> i32;
    fn unserialize(&mut self, stream: &[u8], offset: &mut i32) -> i32;

    /// Возвращает доказанный `CTeamate`-интерфейс либо RTTI-failure как `None`.
    fn as_teamate_mut(&mut self) -> Option<&mut dyn WorldTeamateOwner> {
        None
    }
}

/// Узкая проекция методов `CTeamate`, вызываемых входящими Team-сообщениями.
pub(crate) trait WorldTeamateOwner {
    fn exit(&mut self);
    fn set_owner_region_id(&mut self, region_id: i32);
    fn set_owner_name(&mut self, owner_name: &[u8]);
    fn player_still_existed(&mut self, existed: i32);
    fn owner_region_id(&self) -> i32;
    fn owner_name(&self) -> &[u8];

    /// Забирает синхронные обращения plug-а к его session после мутации поля.
    fn take_session_effects(&mut self) -> Vec<WorldPlugSessionEffect> {
        Vec::new()
    }

    /// Завершает `Exit` только после найденной session, как исходный owner.
    fn confirm_exit(&mut self) {}
}

/// Safe-форма немедленного `CPlug -> CSessionFactory -> CSession` вызова.
pub(crate) struct WorldPlugSessionEffect {
    pub(crate) session_id: i32,
    pub(crate) plug_id: i32,
    pub(crate) state: i32,
    pub(crate) value: Vec<u8>,
    pub(crate) end_after_session_lookup: bool,
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
}

impl CSessionFactory {
    /// Создаёт точные initial static values до первого factory-вызова.
    pub(crate) const fn new() -> Self {
        Self {
            next_session_id: 1,
            next_plug_id: 1,
            sessions: LegacyMsvcHashRegistry::new(),
            plugs: LegacyMsvcHashRegistry::new(),
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
    pub(crate) fn insert_plug(&mut self, game: &mut CGame, session_id: i32, plug_id: i32) -> i32 {
        let Some(plug) = self.plugs.get_mut(plug_id) else {
            return 0;
        };
        let identity = WorldSessionPlug {
            plug_id,
            owner_type: plug.owner_type(),
            owner_id: plug.owner_id(),
        };
        let Some(session) = self.sessions.get_mut(session_id) else {
            return 0;
        };
        if !session.can_insert_plug() || plug.is_plug_available(game) == 0 {
            return 0;
        }
        plug.set_session(session_id);
        session.insert_plug_identity(identity);
        let _ = session.on_plug_change_state(plug_id, 0, &[], 0);
        self.drain_session_effects(game, session_id);
        1
    }

    /// Проверяет тот же RTTI-переход `CSession -> CTeam`, не выдавая owner.
    pub(crate) fn is_team(&mut self, session_id: i32) -> bool {
        self.sessions
            .get_mut(session_id)
            .and_then(|session| session.as_team_mut())
            .is_some()
    }

    /// Строит точный session/team header и затем plug wire в list-order.
    pub(crate) fn serialize_team(&mut self, session_id: i32) -> Option<Vec<u8>> {
        let (mut output, plug_ids) = {
            let session = self.sessions.get_mut(session_id)?;
            let plug_ids = session
                .session_plugs()
                .iter()
                .map(|plug| plug.plug_id)
                .collect::<Vec<_>>();
            let mut output = Vec::new();
            session.as_team_mut()?.serialize(&mut output);
            (output, plug_ids)
        };
        for plug_id in plug_ids {
            let plug = self.plugs.get(plug_id)?;
            if plug.serialize(&mut output) == 0 {
                return None;
            }
        }
        Some(output)
    }

    /// Делегирует операцию живому `CTeam` из единого factory registry.
    pub(crate) fn with_team<ResultValue>(
        &mut self,
        game: &mut CGame,
        session_id: i32,
        operation: impl FnOnce(&mut dyn WorldTeamSessionOwner) -> ResultValue,
    ) -> Option<ResultValue> {
        let result = self.sessions.get_mut(session_id)?.as_team_mut().map(operation)?;
        self.drain_session_effects(game, session_id);
        Some(result)
    }

    /// Делегирует операцию живому `CTeamate` из единого factory registry.
    pub(crate) fn with_teamate<ResultValue>(
        &mut self,
        game: &mut CGame,
        plug_id: i32,
        operation: impl FnOnce(&mut dyn WorldTeamateOwner) -> ResultValue,
    ) -> Option<ResultValue> {
        let (result, effects) = {
            let teamate = self.plugs.get_mut(plug_id)?.as_teamate_mut()?;
            let result = operation(teamate);
            (result, teamate.take_session_effects())
        };
        let mut affected_sessions = Vec::new();
        for effect in effects {
            affected_sessions.push(effect.session_id);
            let session_found = self.sessions.get_mut(effect.session_id).is_some_and(|session| {
                session.on_plug_change_state(
                    effect.plug_id,
                    effect.state,
                    &effect.value,
                    0,
                );
                true
            });
            if session_found && effect.end_after_session_lookup {
                if let Some(teamate) = self
                    .plugs
                    .get_mut(plug_id)
                    .and_then(|plug| plug.as_teamate_mut())
                {
                    teamate.confirm_exit();
                }
            }
        }
        affected_sessions.sort_unstable();
        affected_sessions.dedup();
        for session_id in affected_sessions {
            self.drain_session_effects(game, session_id);
        }
        Some(result)
    }

    /// Вызывает virtual `CSession::End` без придуманного downcast-а к team.
    pub(crate) fn end_session(&mut self, game: &mut CGame, session_id: i32) -> Option<i32> {
        let result = self.sessions.get_mut(session_id).map(|session| session.end())?;
        self.drain_session_effects(game, session_id);
        Some(result)
    }

    fn drain_session_effects(&mut self, game: &mut CGame, session_id: i32) {
        loop {
            let effects = self
                .sessions
                .get_mut(session_id)
                .map(|session| session.take_effects())
                .unwrap_or_default();
            if effects.is_empty() {
                break;
            }
            for effect in effects {
                match effect {
                    WorldSessionEffect::TeamStarted { team_id } => {
                        game.publish_team_session(team_id, session_id);
                    }
                    WorldSessionEffect::TeamEnded { team_id } => {
                        let mut message = CMessage::new(0x0007_FD02);
                        message.base_mut().add_long(team_id as i32);
                        let _ = message.send_all(game.current_game_server_sender().as_ref());
                        game.remove_team_session(team_id);
                    }
                    WorldSessionEffect::KickPlayer { team_id, player_id } => {
                        if let Some(map_id) = game
                            .player_game_server(player_id)
                            .map(|server| server.index as i32)
                        {
                            let mut message = CMessage::new(0x0007_FD06);
                            message.base_mut().add_long(team_id as i32);
                            message.base_mut().add_long(player_id);
                            let _ = game.send_msg_to_game_server(map_id, &message);
                        }
                    }
                    WorldSessionEffect::PlugState {
                        team_id,
                        plug_id,
                        state,
                        value,
                        include_sender,
                    } => self.dispatch_team_plug_state(
                        game,
                        session_id,
                        team_id,
                        plug_id,
                        state,
                        &value,
                        include_sender,
                    ),
                }
            }
        }
    }

    fn dispatch_team_plug_state(
        &mut self,
        game: &CGame,
        session_id: i32,
        team_id: u32,
        plug_id: i32,
        state: i32,
        value: &[u8],
        include_sender: bool,
    ) {
        let plug_ids: Vec<i32> = self
            .sessions
            .get(session_id)
            .map(|session| {
                session
                    .session_plugs()
                    .iter()
                    .map(|plug| plug.plug_id)
                    .collect()
            })
            .unwrap_or_default();
        if !plug_ids.contains(&plug_id) {
            return;
        }
        let Some(source) = self.plugs.get_mut(plug_id) else {
            return;
        };
        if source.is_plug_available(game) == 0 {
            return;
        }

        for target_id in plug_ids.iter().copied() {
            if target_id == plug_id && !include_sender {
                continue;
            }
            let Some(target) = self.plugs.get_mut(target_id) else {
                continue;
            };
            if target.is_plug_available(game) != 0 && target.is_plug_ended() == 0 {
                let _ = target.on_change_state(plug_id, state, value);
            }
        }

        let source_identity = self.plugs.get(plug_id).map(|plug| {
            (plug.owner_type(), plug.owner_id())
        });
        match state {
            0 => {
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                let Some((region_id, owner_name)) = self.with_teamate_snapshot(plug_id) else {
                    return;
                };
                let mut message = CMessage::new(0x0007_FD03);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add_long(region_id);
                message.base_mut().add(&owner_name);
                message.base_mut().add_byte(0);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            1 | 2 => {
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                let mut message = CMessage::new(0x0007_FD04);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            4 => {
                let Some(player_id) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let player_exists = self.sessions.get(session_id).is_some_and(|session| {
                    session
                        .session_plugs()
                        .iter()
                        .any(|plug| plug.owner_type == 400 && plug.owner_id == player_id)
                });
                if player_id == 0 || !player_exists {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD07);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(player_id);
                self.send_to_team_game_servers(
                    game,
                    &plug_ids,
                    if include_sender { 0 } else { plug_id },
                    &message,
                );
            }
            5 => {
                let Some(scheme) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let mut message = CMessage::new(0x0007_FD0A);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(scheme);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            6 => {
                let Some(region_id) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                if region_id == 0 || self.with_teamate_snapshot(plug_id).is_none() {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD05);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add_long(region_id);
                self.send_to_team_game_servers(game, &plug_ids, -1, &message);
            }
            8 => {
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                if self.with_teamate_snapshot(plug_id).is_none() {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD0B);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add(value);
                message.base_mut().add_byte(0);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            9 => {
                let Some(state_bits) = value.get(..4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("ровно DWORD"))
                }) else {
                    return;
                };
                let Some((owner_type, owner_id)) = source_identity else {
                    return;
                };
                if self.with_teamate_snapshot(plug_id).is_none() {
                    return;
                }
                let mut message = CMessage::new(0x0007_FD0C);
                message.base_mut().add_long(team_id as i32);
                message.base_mut().add_long(owner_type);
                message.base_mut().add_long(owner_id);
                message.base_mut().add_long(state_bits);
                self.send_to_team_game_servers(game, &plug_ids, plug_id, &message);
            }
            _ => {}
        }
    }

    fn with_teamate_snapshot(&mut self, plug_id: i32) -> Option<(i32, Vec<u8>)> {
        let teamate = self.plugs.get_mut(plug_id)?.as_teamate_mut()?;
        let region_id = teamate.owner_region_id();
        let owner_name = teamate.owner_name().to_vec();
        Some((region_id, owner_name))
    }

    fn send_to_team_game_servers(
        &self,
        game: &CGame,
        plug_ids: &[i32],
        source_plug_id: i32,
        message: &CMessage,
    ) {
        let source_map = self
            .plugs
            .get(source_plug_id)
            .and_then(|plug| game.player_game_server(plug.owner_id()))
            .map(|server| server.index);
        let mut maps = Vec::new();
        for plug_id in plug_ids.iter().copied() {
            if plug_id == source_plug_id {
                continue;
            }
            let Some(map_id) = self
                .plugs
                .get(plug_id)
                .and_then(|plug| game.player_game_server(plug.owner_id()))
                .map(|server| server.index)
            else {
                continue;
            };
            if Some(map_id) == source_map || maps.contains(&map_id) {
                continue;
            }
            maps.push(map_id);
            let _ = game.send_msg_to_game_server(map_id as i32, message);
        }
    }

    /// Создаёт `CSession`/`CTeam`, назначает type/ID и публикует owner.
    pub(crate) fn create_session(
        &mut self,
        minimum_plugs: u32,
        maximum_plugs: u32,
        lifetime_ms: u32,
        session_type: i32,
    ) -> i32 {
        let Some(session_type) = WorldSessionType::from_legacy(session_type) else {
            return 0;
        };
        let mut session: Box<dyn WorldSessionOwner> = match session_type {
            WorldSessionType::Normal => Box::new(
                crate::worldserver::appworld::session::csession::CSession::new(
                    minimum_plugs,
                    maximum_plugs,
                    lifetime_ms,
                ),
            ),
            WorldSessionType::Team => Box::new(CTeam::new(
                minimum_plugs,
                maximum_plugs,
                lifetime_ms,
            )),
        };

        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        session.assign_factory_identity(TYPE_SESSION, session_id);
        drop(self.sessions.insert_or_replace(session_id, Some(session)));
        session_id
    }

    /// Создаёт единственный поддержанный `CTeamate` и публикует plug owner.
    pub(crate) fn create_plug(
        &mut self,
        plug_type: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> i32 {
        let Some(plug_type) = WorldPlugType::from_legacy(plug_type) else {
            return 0;
        };
        if plug_type != WorldPlugType::Teamate {
            return 0;
        }
        let mut plug: Box<dyn WorldPlugOwner> = Box::new(CTeamate::new());

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        plug.assign_factory_identity(TYPE_PLUG, plug_id);
        plug.set_owner(owner_type, owner_id);
        drop(self.plugs.insert_or_replace(plug_id, Some(plug)));
        plug_id
    }

    /// Исполняет exact ordered session traversal и erase-after-destructor.
    pub(crate) fn ai(&mut self, game: &mut CGame) -> WorldSessionFactoryAiReport {
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
                self.drain_session_effects(game, session_id);
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let _ = self.garbage_collect(TYPE_PLUG, plug_id);
                }
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
                self.drain_session_effects(game, session_id);
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let _ = self.garbage_collect(TYPE_PLUG, plug_id);
                }
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
                .expect("session остаётся живой до AI prefix")
                .ai_prefix();
            self.drain_session_effects(game, session_id);

            let traverse = self.sessions.entries[index]
                .value
                .as_ref()
                .is_some_and(|session| session.should_traverse_plugs());
            if traverse {
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let disposition = match self.plugs.get_mut(plug_id) {
                        None => 0,
                        Some(plug) => {
                            if plug.is_plug_available(game) == 0 {
                                2
                            } else if plug.is_plug_ended() != 0 {
                                1
                            } else {
                                3
                            }
                        }
                    };
                    match disposition {
                        0 => {
                            if let Some(session) = self.sessions.entries[index].value.as_mut() {
                                session.remove_plug_id(plug_id);
                            }
                        }
                        state @ (1 | 2) => {
                            if let Some(session) = self.sessions.entries[index].value.as_mut() {
                                let _ = session.on_plug_change_state(plug_id, state, &[], 0);
                            }
                            self.drain_session_effects(game, session_id);
                            let _ = self.garbage_collect(TYPE_PLUG, plug_id);
                            if let Some(session) = self.sessions.entries[index].value.as_mut() {
                                session.remove_plug_id(plug_id);
                            }
                        }
                        _ => {}
                    }
                }
            }
            self.sessions.entries[index]
                .value
                .as_mut()
                .expect("session остаётся живой до AI suffix")
                .ai_suffix();
            self.drain_session_effects(game, session_id);
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
                let plug_ids = self.sessions.entries[index]
                    .value
                    .as_ref()
                    .map(|session| {
                        session
                            .session_plugs()
                            .iter()
                            .map(|plug| plug.plug_id)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for plug_id in plug_ids {
                    let Some(plug_index) = self.plugs.position(plug_id) else {
                        continue;
                    };
                    let plug = self.plugs.entries[plug_index].value.take();
                    drop(plug);
                    self.plugs.entries.remove(plug_index);
                }
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
    pub(crate) fn unserialize_plug(
        &mut self,
        stream: Option<&[u8]>,
        offset: &mut i32,
    ) -> Result<i32, WorldSessionFactoryInputBlock> {
        let Some(stream) = stream else {
            return Ok(0);
        };
        let plug_type = read_i32(stream, offset)?;
        let owner_type = read_i32(stream, offset)?;
        let owner_id = read_i32(stream, offset)?;
        let plug_id = self.create_plug(plug_type, owner_type, owner_id);
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
    pub(crate) fn unserialize_session(
        &mut self,
        game: &mut CGame,
        stream: Option<&[u8]>,
        offset: &mut i32,
    ) -> Result<i32, WorldSessionFactoryInputBlock> {
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
        );
        let (unserialized, plug_count) = {
            let Some(session) = self.sessions.get_mut(session_id) else {
                return Ok(0);
            };
            let unserialized = session.unserialize(stream, offset);
            let plug_count = session.take_pending_unserialize_plugs();
            (unserialized, plug_count)
        };
        if unserialized != 0 {
            self.drain_session_effects(game, session_id);
            for _ in 0..plug_count {
                let plug_id = self.unserialize_plug(Some(stream), offset)?;
                if plug_id == 0 {
                    break;
                }
                let _ = self.insert_plug(game, session_id, plug_id);
            }
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
