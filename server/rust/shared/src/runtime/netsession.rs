//! Асинхронная `CNetSession`, общая для GameServer и WorldServer.
//! Источники контракта — точные пары обоих EXE/PDB.
//!
//! Session хранит signed 64-bit ID, два cookie, unsigned timeout и nullable
//! callback. `Beging` сначала назначает timeout, затем вызывает async endpoint;
//! result/timeout сохраняют tags `0/1/2` и исходный порядок callback-ов.
//! `Arc<dyn NetSessionEndpoint>` заменяет два interface-subobject и удерживает
//! lifetime после снятия manager lock без изменения session semantics.

//! Экземпляр реестра создаёт и держит владелец роли (variant Game/World);
//! shared несёт только типы записей и порядок их жизненного цикла.

use std::sync::Arc;

/// Точная пара двух signed Windows `long`, которой защищён callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetSessionCookie {
    pub first: i32,
    pub second: i32,
}

/// Исходный numeric tag `tagAsyncResult`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetSessionAsyncResultKind {
    Do = 0,
    Result = 1,
    TimeOut = 2,
}

/// Типизированная проекция достигнутых callback-результатов. Все подключённые
/// владельцы передают один Windows `long`; timeout не несёт значения.
pub struct NetSessionAsyncResult {
    pub kind: NetSessionAsyncResultKind,
    pub value: Option<i32>,
}

/// Совмещённые действующие контракты старых `IAsyncCaller/IAsyncCallback`.
pub trait NetSessionEndpoint: Send + Sync {
    /// Выполняет исходный `DoAsyncCall(id, cookie.second, args)`.
    fn do_async_call(&self, session_id: i64, cookie_second: i32);

    /// Получает один исходный result-tag и типизированное значение.
    fn on_async_callback(&self, result: NetSessionAsyncResult);
}

/// Повторный setter, для которого исходник терял прежний owner.
pub struct NetSessionCallbackAlreadyAssigned {
    pub incoming: Box<dyn NetSessionEndpoint>,
}

/// Локальная неизвестность `Beging` после уже записанного timeout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetSessionBeginBlock {
    MissingEndpointAfterTimeoutAssigned,
}

/// Подготовленный под session lock вызов, исполняемый уже после его отпускания.
pub struct NetSessionBeginDispatch {
    session_id: i64,
    cookie_second: i32,
    endpoint: Arc<dyn NetSessionEndpoint>,
}

impl NetSessionBeginDispatch {
    /// Синхронно вызывает старый `IAsyncCaller::DoAsyncCall`.
    pub fn dispatch(self) {
        self.endpoint
            .do_async_call(self.session_id, self.cookie_second);
    }
}

/// Owned-состояние исходного `CNetSession`.
pub struct CNetSession {
    id: i64,
    cookie: NetSessionCookie,
    timeout: u32,
    endpoint: Option<Arc<dyn NetSessionEndpoint>>,
}

impl CNetSession {
    /// Создаёт session с нулевым timeout и null callback.
    pub const fn new(id: i64, cookie: NetSessionCookie) -> Self {
        Self {
            id,
            cookie,
            timeout: 0,
            endpoint: None,
        }
    }

    /// Возвращает точный signed 64-bit ключ manager map.
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// Возвращает оба cookie без изменения 32-битных шаблонов.
    pub const fn cookie(&self) -> NetSessionCookie {
        self.cookie
    }

    /// Проверяет оба cookie в исходном порядке.
    pub const fn check_cookie(&self, first: i32, second: i32) -> bool {
        first == self.cookie.first && second == self.cookie.second
    }

    /// Передаёт единственный callback/caller owner сессии.
    pub fn set_callback_handle(
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
    pub fn beging(&mut self, timeout: u32) -> Result<(), NetSessionBeginBlock> {
        self.prepare_beging(timeout)?.dispatch();
        Ok(())
    }

    /// Записывает timeout и копирует endpoint для вызова вне manager lock.
    pub fn prepare_beging(
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
    pub fn on_do(&self, value: i32) -> bool {
        self.deliver(NetSessionAsyncResultKind::Do, Some(value))
    }

    /// Доставляет terminal tag `1`; удаление session выполняет manager.
    pub fn on_result(&self, value: i32) -> bool {
        self.deliver(NetSessionAsyncResultKind::Result, Some(value))
    }

    /// Доставляет tag `2` без чтения неинициализированного старого pointer-а.
    pub fn on_time_out(&self) -> bool {
        self.deliver(NetSessionAsyncResultKind::TimeOut, None)
    }

    /// Уменьшает только ненулевой unsigned timeout.
    pub fn decrement_timeout(&mut self) -> bool {
        if self.timeout == 0 {
            return false;
        }
        self.timeout -= 1;
        true
    }

    /// Проверяет действующее условие немедленного timeout.
    pub const fn is_timed_out(&self) -> bool {
        self.timeout == 0
    }

    /// Создаёт временный callback owner для вызова вне manager lock.
    pub fn endpoint_handle(&self) -> Option<Arc<dyn NetSessionEndpoint>> {
        self.endpoint.clone()
    }

    fn deliver(&self, kind: NetSessionAsyncResultKind, value: Option<i32>) -> bool {
        let Some(endpoint) = self.endpoint.as_ref() else {
            return false;
        };
        endpoint.on_async_callback(NetSessionAsyncResult { kind, value });
        true
    }
}
