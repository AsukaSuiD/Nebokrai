//! Ordered registry `CNetSession` для GameServer и WorldServer.
//! Источники контракта — точные пары обоих EXE/PDB.
//!
//! `CreateSession` сохраняет signed key/cookie arithmetic. Result получает
//! session под lock, отпускает lock на callback и только затем удаляет owner.
//! `Run` держит lock весь проход, уменьшает ненулевой timeout на один, а при
//! нуле вызывает callback и удаляет запись. `BTreeMap`, `Mutex` и `AtomicI32`
//! заменяют MSVC map, critical section и static counter, сохраняя key-order,
//! duplicate behavior и lifecycle.

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI32, Ordering};

use parking_lot::Mutex;

use super::netsession::{
    CNetSession, NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionBeginBlock,
    NetSessionCallbackAlreadyAssigned, NetSessionCookie, NetSessionEndpoint,
};

/// Компонентная поверхность одного и того же исходного owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NetSessionManagerVariant {
    GameServer,
    WorldServer,
}

/// Успешно созданный ID; raw pointer заменён стабильным map-key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CreatedNetSession {
    pub(crate) id: i64,
    pub(crate) cookie: NetSessionCookie,
}

/// Duplicate, на котором исходный `operator[]` терял прежний owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetSessionCreateBlock {
    pub(crate) duplicate_id: i64,
}

/// Результат нетерминального либо terminal callback lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NetSessionCallbackOutcome {
    UnsupportedVariant,
    SessionNotFound,
    CookieMismatch,
    CallbackMissing,
    Delivered,
}

/// Итог полного ordered timeout pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NetSessionRunReport {
    pub(crate) decremented: usize,
    pub(crate) timed_out: Vec<i64>,
    pub(crate) callbacks_missing: usize,
}

/// Caller-owned замена process singleton `CNetSessionManager::instance`.
pub(crate) struct CNetSessionManager {
    variant: NetSessionManagerVariant,
    sessions: Mutex<BTreeMap<i64, CNetSession>>,
    generated_id: AtomicI32,
}

impl CNetSessionManager {
    /// Создаёт пустой manager конкретного подтверждённого процесса.
    pub(crate) const fn new(variant: NetSessionManagerVariant) -> Self {
        Self {
            variant,
            sessions: Mutex::new(BTreeMap::new()),
            generated_id: AtomicI32::new(0),
        }
    }

    /// Возвращает выбранную компонентную поверхность без смешения процессов.
    pub(crate) const fn variant(&self) -> NetSessionManagerVariant {
        self.variant
    }

    /// Создаёт session и вставляет её по оригинал signed 64-bit key.
    pub(crate) fn create_session(
        &self,
        first: i32,
        requested_id: i32,
        mut random: impl FnMut(i32) -> i32,
    ) -> Result<CreatedNetSession, NetSessionCreateBlock> {
        let id_low = if requested_id == 0 {
            self.generated_id
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1)
        } else {
            requested_id
        };
        let id_high = first | (id_low >> 31);
        let id = i64::from(id_high)
            .wrapping_shl(32)
            .wrapping_add(i64::from(id_low as u32));
        let cookie = NetSessionCookie {
            first,
            second: random(30_000),
        };
        let session = CNetSession::new(id, cookie);
        let mut sessions = self.sessions.lock();
        if sessions.contains_key(&id) {
            return Err(NetSessionCreateBlock { duplicate_id: id });
        }
        sessions.insert(id, session);
        Ok(CreatedNetSession { id, cookie })
    }

    /// Передаёт callback-owner созданной session либо возвращает его caller-у.
    pub(crate) fn set_callback_handle(
        &self,
        session_id: i64,
        endpoint: Box<dyn NetSessionEndpoint>,
    ) -> Result<(), NetSessionSetCallbackBlock> {
        let mut sessions = self.sessions.lock();
        let Some(session) = sessions.get_mut(&session_id) else {
            return Err(NetSessionSetCallbackBlock::SessionNotFound { endpoint });
        };
        session
            .set_callback_handle(endpoint)
            .map_err(NetSessionSetCallbackBlock::AlreadyAssigned)
    }

    /// Выполняет `Beging` для stable key под исходной map lifetime.
    pub(crate) fn beging(
        &self,
        session_id: i64,
        timeout: u32,
        payload: &dyn Any,
    ) -> Result<(), NetSessionManagerBeginBlock> {
        let dispatch = {
            let mut sessions = self.sessions.lock();
            let Some(session) = sessions.get_mut(&session_id) else {
                return Err(NetSessionManagerBeginBlock::SessionNotFound);
            };
            session
                .prepare_beging(timeout)
                .map_err(NetSessionManagerBeginBlock::Session)?
        };
        dispatch.dispatch(payload);
        Ok(())
    }

    /// Доставляет GameServer-only nonterminal callback и сохраняет session.
    pub(crate) fn on_do(
        &self,
        session_id: i64,
        cookie_first: i32,
        cookie_second: i32,
        payload: &dyn Any,
    ) -> NetSessionCallbackOutcome {
        if self.variant != NetSessionManagerVariant::GameServer {
            return NetSessionCallbackOutcome::UnsupportedVariant;
        }
        let Some((cookie, endpoint)) = self.callback_snapshot(session_id) else {
            return NetSessionCallbackOutcome::SessionNotFound;
        };
        if cookie
            != (NetSessionCookie {
                first: cookie_first,
                second: cookie_second,
            })
        {
            return NetSessionCallbackOutcome::CookieMismatch;
        }
        let Some(endpoint) = endpoint else {
            return NetSessionCallbackOutcome::CallbackMissing;
        };
        endpoint.on_async_callback(NetSessionAsyncResult {
            kind: NetSessionAsyncResultKind::Do,
            payload: Some(payload),
        });
        NetSessionCallbackOutcome::Delivered
    }

    /// Доставляет terminal result и только после callback удаляет map-owner.
    pub(crate) fn on_sync_callback_result(
        &self,
        session_id: i64,
        cookie_first: i32,
        cookie_second: i32,
        payload: &dyn Any,
    ) -> NetSessionCallbackOutcome {
        let Some((cookie, endpoint)) = self.callback_snapshot(session_id) else {
            return NetSessionCallbackOutcome::SessionNotFound;
        };
        if cookie
            != (NetSessionCookie {
                first: cookie_first,
                second: cookie_second,
            })
        {
            return NetSessionCallbackOutcome::CookieMismatch;
        }
        let Some(endpoint) = endpoint else {
            let removed = self.sessions.lock().remove(&session_id);
            drop(removed);
            return NetSessionCallbackOutcome::CallbackMissing;
        };
        endpoint.on_async_callback(NetSessionAsyncResult {
            kind: NetSessionAsyncResultKind::Result,
            payload: Some(payload),
        });
        let removed = self.sessions.lock().remove(&session_id);
        drop(removed);
        drop(endpoint);
        NetSessionCallbackOutcome::Delivered
    }

    /// Обходит все sessions по signed key-order и удаляет каждый zero-timeout.
    pub(crate) fn run(&self) -> NetSessionRunReport {
        let mut sessions = self.sessions.lock();
        let keys: Vec<i64> = sessions.keys().copied().collect();
        let mut report = NetSessionRunReport::default();
        for id in keys {
            let Some(session) = sessions.get_mut(&id) else {
                continue;
            };
            if session.is_timed_out() {
                if !session.on_time_out() {
                    report.callbacks_missing += 1;
                }
                let removed = sessions.remove(&id);
                drop(removed);
                report.timed_out.push(id);
            } else {
                let _ = session.decrement_timeout();
                report.decremented += 1;
            }
        }
        report
    }

    /// Освобождает все callbacks/sessions в map-order и оставляет manager пустым.
    pub(crate) fn release(&self) -> usize {
        let mut sessions = self.sessions.lock();
        let released = sessions.len();
        while let Some((_id, session)) = sessions.pop_first() {
            drop(session);
        }
        released
    }

    /// Возвращает текущее число session records под map lock.
    pub(crate) fn len(&self) -> usize {
        self.sessions.lock().len()
    }

    fn callback_snapshot(
        &self,
        session_id: i64,
    ) -> Option<(
        NetSessionCookie,
        Option<std::sync::Arc<dyn NetSessionEndpoint>>,
    )> {
        let sessions = self.sessions.lock();
        let session = sessions.get(&session_id)?;
        Some((session.cookie(), session.endpoint_handle()))
    }
}

/// Safe-блок передачи callback owner-а через manager lookup.
pub(crate) enum NetSessionSetCallbackBlock {
    SessionNotFound {
        endpoint: Box<dyn NetSessionEndpoint>,
    },
    AlreadyAssigned(NetSessionCallbackAlreadyAssigned),
}

/// Safe-блок manager-обёртки `Beging`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NetSessionManagerBeginBlock {
    SessionNotFound,
    Session(NetSessionBeginBlock),
}
