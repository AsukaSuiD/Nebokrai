//! Блоки результата городской войны organizing-сообщений и session
//! терминальный слой async confirm-подтверждений: типы-конечники и их
//! in-memory очередь, вынесенные в Realm заранее. Диспетчеры `organsysmessage`
//! остаются в старом `appworld/message/organsysmessage.rs` до шага переноса
//! области; их callback-структуры уже обслуживаются этим владельцем.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::app::world_message::SendMessageError;
use crate::organizations::country::CountryGovernanceContextBlock;
use crate::organizations::faction::{
    FactionBillboardStatBlock, FactionInitialPropertyBlock, OwnedCityMutationBuildError,
};
use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;
use crate::organizations::union::{
    CityTransferTerminal, ConfederationCreationTerminal, UnionApplicationTerminal,
};

#[derive(Debug, Eq, PartialEq)]
pub enum OrganizingCityWarResultContextBlock {
    MissingRegionOwner { region_id: i32 },
    MissingFactionForMutation {
        faction_id: i32,
        operation: &'static str,
    },
    MissingFactionMaster { faction_id: i32 },
    MissingFactionCountry { faction_id: i32 },
    NullUnion { map_key: i32 },
    MissingEnemyOrganizing { organizing_id: i32 },
    EnemyMutation {
        organizing_id: i32,
        enemy_organizing_id: i32,
        source: FactionEnemyMutationBlock,
    },
    OwnedCity {
        faction_id: i32,
        operation: &'static str,
        source: OwnedCityMutationBuildError,
    },
    VictorCount {
        faction_id: i32,
        operation: &'static str,
        source: FactionInitialPropertyBlock,
    },
    Billboard(FactionBillboardStatBlock),
    NoticeWouldOverflow {
        string_id: &'static [u8],
        visible_len: usize,
    },
    MissingCountryOwner { country_id: u8 },
    CountryGovernance {
        country_id: u8,
        source: CountryGovernanceContextBlock,
    },
}

/// Терминалы, публикуемые async session callback-ом в FIFO владельца игры.
///
/// Исходный порядок — это сначала main-loop FIFO, а callback лишь кладёт
/// подтверждение; терминал публикует точную выявленную форму outcome из
/// `.exe/worldserver.exe`, и его изменение игры происходит только при
/// чтении владельца игры, как и раньше.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueuedUnionApplicationTerminal {
    pub union_id: i32,
    pub applicant_faction_id: i32,
    pub terminal: UnionApplicationTerminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueuedUnionInvitationTerminal {
    pub union_id: i32,
    pub inviter_faction_id: i32,
    pub invited_faction_id: i32,
    pub terminal: UnionApplicationTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedCityTransferTerminal {
    pub source_faction_id: i32,
    pub target_faction_id: i32,
    pub region_id: i32,
    pub region_name: Vec<u8>,
    pub terminal: CityTransferTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedConfederationCreationTerminal {
    pub first_player_id: i32,
    pub second_player_id: i32,
    pub first_faction_id: i32,
    pub second_faction_id: i32,
    pub union_name: Vec<u8>,
    pub terminal: ConfederationCreationTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueuedOrganizingSessionTerminal {
    Union(QueuedUnionApplicationTerminal),
    UnionInvitation(QueuedUnionInvitationTerminal),
    ConfederationCreation(QueuedConfederationCreationTerminal),
    CityTransfer(QueuedCityTransferTerminal),
}

/// Выгружает конкретное подтверждение Session confirmation с `SendMessageError`-
/// результатом; нулевое пuegosето SendMessageError не имеет смысла, ей выдаётся
/// исходный сообщение-маркер без инференцев.
#[derive(Debug, Eq, PartialEq)]
pub struct UnionApplicationConfirmationDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CityTransferConfirmationDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ConfederationCreationConfirmationDelivery {
    pub recipient_player_id: i32,
    pub game_server_id: i32,
    pub result: Result<i32, SendMessageError>,
}

/// Блокировка session endpoint-а, собранная в единый терминальный владелец.
///
/// Внутренний владелец очередей использует `parking_lot::Mutex`, а callback не
/// обновляет игру вне основного loop-a — зафиксировано в заявлении выше и в
/// подтверждающем заголовке старого диспетчера.
#[derive(Debug, Default)]
struct WorldUnionApplicationRuntimeState {
    terminals: Mutex<VecDeque<QueuedOrganizingSessionTerminal>>,
    confirmations: Mutex<VecDeque<UnionApplicationConfirmationDelivery>>,
    blocks: Mutex<VecDeque<crate::organizations::union::UnionApplicationEndpointBlock>>,
    city_confirmations: Mutex<VecDeque<CityTransferConfirmationDelivery>>,
    city_blocks: Mutex<VecDeque<crate::organizations::union::CityTransferEndpointBlock>>,
    confederation_creation_confirmations:
        Mutex<VecDeque<ConfederationCreationConfirmationDelivery>>,
    confederation_creation_blocks:
        Mutex<VecDeque<crate::organizations::union::ConfederationCreationEndpointBlock>>,
}

/// Идемпотентный владелец скопления endpoint-терминалов для одного World-а.
///
/// Это перенесённый тип: открытые поля `Arc` и есть исходные точки подключения
/// подтверждений; объектное поле `state` по объявлению берёт собственную форму
/// shared state из orginal-and-rec host `worldserver.exe + worldserver.pdb`.
#[derive(Clone, Default)]
pub struct WorldOrganizingSessionRuntimeOwner {
    state: Arc<WorldUnionApplicationRuntimeState>,
}

impl WorldOrganizingSessionRuntimeOwner {
    pub fn pop_terminal(&self) -> Option<QueuedOrganizingSessionTerminal> {
        self.state.terminals.lock().pop_front()
    }

    pub fn take_confirmations(&self) -> Vec<UnionApplicationConfirmationDelivery> {
        self.state.confirmations.lock().drain(..).collect()
    }

    pub fn take_city_confirmations(&self) -> Vec<CityTransferConfirmationDelivery> {
        self.state.city_confirmations.lock().drain(..).collect()
    }

    pub fn take_confederation_creation_confirmations(
        &self,
    ) -> Vec<ConfederationCreationConfirmationDelivery> {
        self.state
            .confederation_creation_confirmations
            .lock()
            .drain(..)
            .collect()
    }

    pub fn take_blocks(&self) -> Vec<crate::organizations::union::UnionApplicationEndpointBlock> {
        self.state.blocks.lock().drain(..).collect()
    }

    pub fn take_city_blocks(&self) -> Vec<crate::organizations::union::CityTransferEndpointBlock> {
        self.state.city_blocks.lock().drain(..).collect()
    }

    pub fn take_confederation_creation_blocks(
        &self,
    ) -> Vec<crate::organizations::union::ConfederationCreationEndpointBlock> {
        self.state
            .confederation_creation_blocks
            .lock()
            .drain(..)
            .collect()
    }
}
