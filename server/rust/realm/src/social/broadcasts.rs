//! Системная broadcast-рассылка мира (исходный live-список
//! `CGame::m_listSysBroadcast` из записей `tagSysBroadcast` и его AI-проход) —
//! primary state владельца `social` (прежнее transitional-поле `CGame`).
//! Типы записи и AI-отчётов перенесены из `app/world_hub_entries`; прежние
//! пути сохранены re-export-ами для прежних consumers.
//!
//! Поле списка публично: reload-оркестрация перечитывает `sysboardcast.ini`
//! целиком, а AI-цикл app мутирует записи in place в прежнем доказанном
//! порядке (прецедент `regions::worldzones::WorldRegionRegistry`).

use std::collections::VecDeque;

use crate::app::world_message::SendMessageError;

/// Действующая AI-проекция исходного `CGame::tagSysBroadcast`.
///
/// Поля идут по смыслу struct-layout `+0x04..+0x40`; `_login_type` AI не читает,
/// а Rust-layout не выдаётся за старый 68-байтовый Windows ABI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldSystemBroadcast {
    pub import_level: i32,
    pub region_id: i32,
    pub min_time_seconds: u32,
    pub max_time_seconds: u32,
    pub odds: u32,
    pub text_color: u32,
    pub back_color: u32,
    pub message: Vec<u8>,
    pub interval_seconds: u32,
    pub last_notify_time_seconds: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldSystemBroadcastTarget {
    All {
        delivery: Result<i32, SendMessageError>,
    },
    Region {
        region_id: i32,
        game_server_index: Option<u32>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldSystemBroadcastDisposition {
    Waiting {
        elapsed_seconds: u32,
        interval_seconds: u32,
    },
    OddsMissed {
        roll: i32,
        odds: u32,
    },
    Broadcast {
        roll: i32,
        target: WorldSystemBroadcastTarget,
        assigned_last_notify_time_seconds: u32,
        assigned_interval_seconds: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGameAiReport {
    pub region_ids_run: Vec<i32>,
    pub broadcast_tick_ms: u32,
    pub broadcasts: Vec<WorldSystemBroadcastDisposition>,
    pub legacy_result: i32,
}

/// Действующий live-список системных broadcast-объявлений мира.
///
/// Constructor-ное состояние — пустой список; наполнение выполняет
/// reload-оркестрация через pub-поле в прежнем порядке.
pub struct WorldSystemBroadcasts {
    pub entries: VecDeque<WorldSystemBroadcast>,
}

impl WorldSystemBroadcasts {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }
}
