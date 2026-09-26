//! Мировой реестр игроков и присутствие (login/online/offline/creation/
//! restore/deletion): primary state и typed-операции владельца `characters`
//! (прежние transitional-поля `CGame`).
//!
//! `CGame` хранит только composition handle `player_registry` и делегирует
//! прежний pub facade; организационные callbacks присутствия и wire-оркестрация
//! остаются у app. Поля публичны — app-оркестрация мутирует их в прежнем
//! доказанном порядке (прецедент `content::WorldContentCatalogs`); тела
//! typed-операций с наблюдаемым исходом (dup-логи, возврат непринятого
//! владения, wrapping счётчик) перенесены владельцу дословно.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use crate::characters::player::CPlayer;
use crate::persistence::savedata::DeletionPlayerSnapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLoginPlayerEntry {
    pub player_id: u32,
    pub login_time_ms: u32,
}

pub enum WorldCreationPlayerAppendOutcome {
    Inserted {
        player_id: u32,
    },
    DuplicateReleased {
        player_id: u32,
    },
    ExistingMapOwnerKept {
        player_id: u32,
        incoming: Box<CPlayer>,
    },
}

pub enum WorldMapPlayerAppendOutcome {
    Inserted {
        player_id: u32,
    },
    ExistingOwnerKept {
        player_id: u32,
        incoming: Box<CPlayer>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCreationPlayerAppendLog {
    Duplicate { player_id: u32 },
    ExistingMapOwner,
}

impl fmt::Display for WorldCreationPlayerAppendLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate { player_id } => {
                write!(formatter, "{player_id} Player Is In CreationPlayerList.")
            }
            Self::ExistingMapOwner => formatter.write_str("MapPlayer Not Found or NULL."),
        }
    }
}

/// Действующий мировой реестр игроков и очереди присутствия.
///
/// Constructor-ное состояние — пустые коллекции и `player_id: 0`; наполнение
/// выполняет app-оркестрация через pub-поля и typed-операции ниже.
pub struct WorldPlayerRegistry {
    pub players: BTreeMap<u32, Box<CPlayer>>,
    pub player_id: u32,
    pub creation_players: VecDeque<i32>,
    pub restore_players: VecDeque<u32>,
    pub deletion_players: VecDeque<DeletionPlayerSnapshot>,
    pub online_players: VecDeque<u32>,
    pub offline_players: VecDeque<u32>,
    pub login_players: VecDeque<WorldLoginPlayerEntry>,
}

impl WorldPlayerRegistry {
    pub fn new() -> Self {
        Self {
            players: BTreeMap::new(),
            player_id: 0,
            creation_players: VecDeque::new(),
            restore_players: VecDeque::new(),
            deletion_players: VecDeque::new(),
            online_players: VecDeque::new(),
            offline_players: VecDeque::new(),
            login_players: VecDeque::new(),
        }
    }

    pub fn allocate_player_id(&mut self) -> i32 {
        self.player_id = self.player_id.wrapping_add(1);
        self.player_id as i32
    }

    pub fn append_map_player(
        &mut self,
        incoming: Box<CPlayer>,
        mut add_log_text: impl FnMut(&'static str),
    ) -> WorldMapPlayerAppendOutcome {
        let player_id = incoming.get_id() as u32;
        if self.players.contains_key(&player_id) {
            add_log_text("MapPlayer Not Found or NULL.");
            return WorldMapPlayerAppendOutcome::ExistingOwnerKept {
                player_id,
                incoming,
            };
        }
        self.players.insert(player_id, incoming);
        WorldMapPlayerAppendOutcome::Inserted { player_id }
    }

    /// Передаёт уникального creation-игрока владеющему map после list-вставки.
    ///
    /// На обеих collision-ветвях синхронно передаёт точный payload исходного
    /// `AddLogText`; duplicate уничтожается только после возврата callback-а.
    pub fn append_creation_player(
        &mut self,
        incoming: Box<CPlayer>,
        mut add_log: impl FnMut(WorldCreationPlayerAppendLog),
    ) -> WorldCreationPlayerAppendOutcome {
        let signed_player_id = incoming.get_id();
        let player_id = signed_player_id as u32;
        if self.creation_players.contains(&signed_player_id) {
            add_log(WorldCreationPlayerAppendLog::Duplicate { player_id });
            // удаляет incoming до исходного UAF.
            // Box::drop сохраняет destruction; typed outcome запрещает caller-у
            // продолжить с уже уничтоженным non-owning alias.
            drop(incoming);
            return WorldCreationPlayerAppendOutcome::DuplicateReleased { player_id };
        }

        self.creation_players.push_back(signed_player_id);
        if self.players.contains_key(&player_id) {
            add_log(WorldCreationPlayerAppendLog::ExistingMapOwner);
            // Original уже добавил list-ID, оставил старый map-owner и вернул
            // incoming pointer caller-у. Box выражает именно это непринятое
            // владение; дальнейшая судьба объекта принадлежит OnLogMessage.
            return WorldCreationPlayerAppendOutcome::ExistingMapOwnerKept {
                player_id,
                incoming,
            };
        }

        self.players.insert(player_id, incoming);
        WorldCreationPlayerAppendOutcome::Inserted { player_id }
    }
}
