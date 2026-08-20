//! Одна асинхронная сетевая сессия `CNetSession` для GameServer и WorldServer.
//!
//! Статус владельца: `IMPLEMENTED` для constructor/destructor,
//! `CheckCookie`, `SetCallbackHandle`, Game-only `OnDo`, общего `OnResult`,
//! `OnTimeOut` и `Beging` с сохранением исходной опечатки имени.
//!
//! Точные пары и существенные RVA:
//! - GameServer: `gameserver.exe + GameServer.pdb`, constructor `0x0010B190`,
//!   destructor `0x0010B1C0`, `CheckCookie` `0x0010B1D0`, setter
//!   `0x0010B1F0`, `OnDo` `0x0010B200`, `OnResult` `0x0010B230`,
//!   `OnTimeOut` `0x0010B260`, `Beging` `0x0010B280`;
//! - WorldServer: `Nworldserver.exe + WorldServer.pdb`, constructor
//!   `0x000C16E0`, destructor `0x000C1710`, `CheckCookie` `0x000C1720`,
//!   setter `0x000C1740`, `OnResult` `0x000C1750`, `OnTimeOut` `0x000C1780`,
//!   `Beging` `0x000C17A0`.
//!
//! Исходный путь обеих PDB:
//! `e:\svn\fengyun_russia_dev\public\netsession.cpp`.
//!
//! Варианты совпадают по layout и общей семантике; `OnDo` присутствует только
//! в GameServer, потому что World call-chain его не достигает. Старый объект
//! хранил signed 64-bit ID, два Windows `long` cookie, unsigned timeout и
//! nullable `IAsyncCallback*`. Constructor ставил timeout `0` и callback
//! `nullptr`. `Beging` сначала записывает timeout, затем через второй interface
//! того же callback-owner-а вызывает `DoAsyncCall(id, cookie.second, args)`.
//! Результаты callback имеют exact tags `0/1/2` для do/result/timeout;
//! timeout не предоставляет payload, поскольку второй DWORD исходного
//! `tagAsyncResult` оставался неинициализированным.
//!
//! Один transferred `Box<dyn NetSessionEndpoint>` безопасно выражает объект с
//! двумя старыми interface-подобъектами. Внутри session он становится `Arc`
//! только потому, что manager доказанно отпускает map lock перед result/do
//! callback и затем может удалить session: временный clone сохраняет lifetime
//! ровно до возврата callback. Сам endpoint выбирает подходящую синхронизацию
//! своей доменной мутации. `&dyn Any` заменяет только безразмерный адрес
//! varargs; конкретный caller по-прежнему обязан передать свой точный payload.
//!
//! Повторный `SetCallbackHandle` в исходнике терял прежний pointer без Release.
//! Среди достигнутых call sites setter вызывается один раз. Rust не создаёт
//! утечку: повторная установка возвращает incoming owner как
//! `BLOCKED_MISSING_FACT`, не меняя session. `Beging` без callback в исходнике
//! разыменовывал null caller после уже записанного timeout; Rust сохраняет
//! запись и возвращает отдельный block. STL/map, vtable dispatch, deleting
//! destructor, allocator и unwind-код удалены как технический механизм;
//! callback-owner освобождается обычным `Drop` последнего `Arc`.

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

/// Совмещённые достигнутые контракты старых `IAsyncCaller/IAsyncCallback`.
pub(crate) trait NetSessionEndpoint: Send + Sync {
    /// Выполняет исходный `DoAsyncCall(id, cookie.second, args)`.
    fn do_async_call(&self, session_id: i64, cookie_second: i32, payload: &dyn Any);

    /// Получает один exact result-tag и его typed-erased payload.
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

    /// Проверяет достигнутое условие немедленного timeout.
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
