//! Асинхронная `CNetSession`, общая для GameServer и WorldServer.
//! Источники контракта — точные пары обоих EXE/PDB.
//!
//! Session хранит signed 64-bit ID, два cookie, unsigned timeout и nullable
//! callback. `Beging` сначала назначает timeout, затем вызывает async endpoint;
//! result/timeout сохраняют tags `0/1/2` и исходный порядок callback-ов.
//! `Arc<dyn NetSessionEndpoint>` заменяет два interface-subobject и удерживает
//! lifetime после снятия manager lock без изменения session semantics.

use std::any::Any;
use std::sync::Arc;

/// Точная пара двух signed Windows `long`, которой защищён callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetSessionCookie {
    pub(crate) first: i32,
    pub(crate) second: i32,
}

/// Исходный numeric tag `tagAsyncResult`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NetSessionAsyncResultKind {
    Do = 0,
    Result = 1,
    TimeOut = 2,
}

/// Безопасная проекция `tagAsyncResult { long type; char *args; }`.
pub(crate) struct NetSessionAsyncResult<'payload> {
    pub(crate) kind: NetSessionAsyncResultKind,
    pub(crate) payload: Option<&'payload dyn Any>,
}

/// Совмещённые действующие контракты старых `IAsyncCaller/IAsyncCallback`.
pub(crate) trait NetSessionEndpoint: Send + Sync {
 /// Выполняет исходный `DoAsyncCall(id, cookie.second, args)`.
    fn do_async_call(&self, session_id: i64, cookie_second: i32, payload: &dyn Any);

 /// Получает один оригинал result-tag и его typed-erased payload.
    fn on_async_callback(&self, result: NetSessionAsyncResult<'_>);
}

/// Повторный setter, для которого исходник терял прежний owner.
pub(crate) struct NetSessionCallbackAlreadyAssigned {
    pub(crate) incoming: Box<dyn NetSessionEndpoint>,
}

/// Локальная неизвестность `Beging` после уже записанного timeout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NetSessionBeginBlock {
    MissingEndpointAfterTimeoutAssigned,
}

/// Подготовленный под session lock вызов, исполняемый уже после его отпускания.
pub(crate) struct NetSessionBeginDispatch {
    session_id: i64,
    cookie_second: i32,
    endpoint: Arc<dyn NetSessionEndpoint>,
}

impl NetSessionBeginDispatch {
 /// Синхронно вызывает старый `IAsyncCaller::DoAsyncCall`.
    pub(crate) fn dispatch(self, payload: &dyn Any) {
        self.endpoint
            .do_async_call(self.session_id, self.cookie_second, payload);
    }
}

/// Owned-состояние исходного `CNetSession`.
pub(crate) struct CNetSession {
    id: i64,
    cookie: NetSessionCookie,
    timeout: u32,
    endpoint: Option<Arc<dyn NetSessionEndpoint>>,
}

impl CNetSession {
 /// Создаёт session с нулевым timeout и null callback.
    pub(crate) const fn new(id: i64, cookie: NetSessionCookie) -> Self {
        Self {
            id,
            cookie,
            timeout: 0,
            endpoint: None,
        }
    }

 /// Возвращает точный signed 64-bit ключ manager map.
    pub(crate) const fn id(&self) -> i64 {
        self.id
    }

 /// Возвращает оба cookie без изменения 32-битных шаблонов.
    pub(crate) const fn cookie(&self) -> NetSessionCookie {
        self.cookie
    }

 /// Проверяет оба cookie в исходном порядке.
    pub(crate) const fn check_cookie(&self, first: i32, second: i32) -> bool {
        first == self.cookie.first && second == self.cookie.second
    }

 /// Передаёт единственный callback/caller owner сессии.
    pub(crate) fn set_callback_handle(
        &mut self,
        endpoint: Box<dyn NetSessionEndpoint>,
    ) -> Result<(), NetSessionCallbackAlreadyAssigned> {
        if self.endpoint.is_some() {
            return Err(NetSessionCallbackAlreadyAssigned { incoming: endpoint });
        }
        self.endpoint = Some(Arc::from(endpoint));
        Ok(())
    }

 /// Ставит timeout и синхронно вызывает `DoAsyncCall`.
    pub(crate) fn beging(
        &mut self,
        timeout: u32,
        payload: &dyn Any,
    ) -> Result<(), NetSessionBeginBlock> {
        self.prepare_beging(timeout)?.dispatch(payload);
        Ok(())
    }

 /// Записывает timeout и копирует endpoint для вызова вне manager lock.
    pub(crate) fn prepare_beging(
        &mut self,
        timeout: u32,
    ) -> Result<NetSessionBeginDispatch, NetSessionBeginBlock> {
        self.timeout = timeout;
        let Some(endpoint) = self.endpoint.as_ref() else {
            return Err(NetSessionBeginBlock::MissingEndpointAfterTimeoutAssigned);
        };
        Ok(NetSessionBeginDispatch {
            session_id: self.id,
            cookie_second: self.cookie.second,
            endpoint: Arc::clone(endpoint),
        })
    }

 /// Доставляет GameServer-only tag `0` без изменения timeout.
    pub(crate) fn on_do(&self, payload: &dyn Any) -> bool {
        self.deliver(NetSessionAsyncResultKind::Do, Some(payload))
    }

 /// Доставляет terminal tag `1`; удаление session выполняет manager.
    pub(crate) fn on_result(&self, payload: &dyn Any) -> bool {
        self.deliver(NetSessionAsyncResultKind::Result, Some(payload))
    }

 /// Доставляет tag `2` без чтения неинициализированного старого pointer-а.
    pub(crate) fn on_time_out(&self) -> bool {
        self.deliver(NetSessionAsyncResultKind::TimeOut, None)
    }

 /// Уменьшает только ненулевой unsigned timeout.
    pub(crate) fn decrement_timeout(&mut self) -> bool {
        if self.timeout == 0 {
            return false;
        }
        self.timeout -= 1;
        true
    }

 /// Проверяет действующее условие немедленного timeout.
    pub(crate) const fn is_timed_out(&self) -> bool {
        self.timeout == 0
    }

 /// Создаёт временный callback owner для вызова вне manager lock.
    pub(crate) fn endpoint_handle(&self) -> Option<Arc<dyn NetSessionEndpoint>> {
        self.endpoint.clone()
    }

    fn deliver(&self, kind: NetSessionAsyncResultKind, payload: Option<&dyn Any>) -> bool {
        let Some(endpoint) = self.endpoint.as_ref() else {
            return false;
        };
        endpoint.on_async_callback(NetSessionAsyncResult { kind, payload });
        true
    }
}
