//! восстановлено — технический владелец `CMyWinInet` из
//! `loginserver/applogin/mywininet.cpp` и `.h`.
//!
//! Точная пара LoginServer.exe/PDB:
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876` /
//! Исходный путь PDB:
//! Наблюдаемый контракт — синхронный HTTP/1 POST с user-agent `App`,
//! `Accept: text/*`, form-urlencoded content type, системным proxy и
//! совместимостью со старым HTTPS-сервером с недоверенным CA. WinInet заменён
//! зрелым blocking-клиентом `reqwest` с rustls; вызовы выполняются только
//! выделенным `CGasThread`. Для HTTPS сознательно сохранён обход проверки CA,
//! но проверка имени узла остаётся включённой. Ограниченный таймаут reqwest
//! заменяет неограниченное владение системным handle и позволяет безопасно
//! завершить Rust-thread.
//!
//! `Recv` оригинала перечитывал ответ кусками прямо в один 1024-байтовый буфер,
//! поэтому после EOF наружу выходил его последний C-string-срез, включая
//! историческое сохранение хвоста предыдущего куска. Ошибка очередного read,
//! как `InternetReadFile == 0`, также завершает цикл и оставляет уже прочитанный
//! буфер доступным анализатору. `Close` после запроса полностью обнуляет его.
//! Это поведение сохранено. Если полный последний кусок не оставляет NUL,
//! оригинал читал за границей массива; safe Rust возвращает весь bounded
//! 1024-байтовый блок анализатору без OOB.

use std::error::Error;
use std::fmt;
use std::io::Read;

use reqwest::Url;
use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE};

const RESPONSE_CAPACITY: usize = 0x400;

pub(crate) enum MyWinInetError {
    InvalidUrlEncoding,
    InvalidUrl,
    UnsupportedScheme,
    Client(reqwest::Error),
    Request(reqwest::Error),
    ResponseMissing,
}

impl fmt::Debug for MyWinInetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // reqwest::Error может содержать полный setup URL. Operator-visible
        // отчёт сохраняет тип ошибки, но не публикует локальное значение.
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for MyWinInetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrlEncoding => formatter.write_str("URL GAS не является UTF-8"),
            Self::InvalidUrl => formatter.write_str("URL GAS имеет недопустимый формат"),
            Self::UnsupportedScheme => {
                formatter.write_str("URL GAS использует неподдерживаемую схему")
            }
            Self::Client(_) => formatter.write_str("не удалось создать HTTP-клиент GAS"),
            Self::Request(_) => formatter.write_str("HTTP-запрос GAS завершился ошибкой"),
            Self::ResponseMissing => formatter.write_str("ответ GAS ещё не получен"),
        }
    }
}

impl Error for MyWinInetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Client(error) | Self::Request(error) => Some(error),
            _ => None,
        }
    }
}

pub(crate) struct CMyWinInet {
    url: Option<Url>,
    client: Option<Client>,
    response: Option<Response>,
    response_buffer: [u8; RESPONSE_CAPACITY],
}

impl Default for CMyWinInet {
    fn default() -> Self {
        Self {
            url: None,
            client: None,
            response: None,
            response_buffer: [0; RESPONSE_CAPACITY],
        }
    }
}

impl CMyWinInet {
    pub(crate) fn init(&mut self, raw_url: &[u8]) -> Result<(), MyWinInetError> {
        self.close();

        let end = raw_url
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(raw_url.len());
        let text =
            std::str::from_utf8(&raw_url[..end]).map_err(|_| MyWinInetError::InvalidUrlEncoding)?;
        let url = Url::parse(text).map_err(|_| MyWinInetError::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(MyWinInetError::UnsupportedScheme);
        }

        let client = Client::builder()
            .http1_only()
            .user_agent("App")
            .tls_danger_accept_invalid_certs(url.scheme() == "https")
            .build()
            .map_err(MyWinInetError::Client)?;

        self.url = Some(url);
        self.client = Some(client);
        Ok(())
    }

    pub(crate) fn send(&mut self, content: &[u8]) -> Result<(), MyWinInetError> {
        let url = self.url.as_ref().ok_or(MyWinInetError::InvalidUrl)?.clone();
        let client = self.client.as_ref().ok_or(MyWinInetError::InvalidUrl)?;
        let end = content
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(content.len());

        self.response = Some(
            client
                .post(url)
                .header(ACCEPT, "text/*")
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(content[..end].to_vec())
                .send()
                .map_err(MyWinInetError::Request)?,
        );
        Ok(())
    }

    pub(crate) fn recv(&mut self) -> Result<Option<Vec<u8>>, MyWinInetError> {
        let response = self
            .response
            .as_mut()
            .ok_or(MyWinInetError::ResponseMissing)?;
        loop {
            match response.read(&mut self.response_buffer) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }

        if self.response_buffer[0] == 0 {
            return Ok(None);
        }
        let end = self
            .response_buffer
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.response_buffer.len());
        Ok(Some(self.response_buffer[..end].to_vec()))
    }

    pub(crate) fn close(&mut self) {
        self.response = None;
        self.client = None;
        self.url = None;
        self.response_buffer.fill(0);
    }
}

impl Drop for CMyWinInet {
    fn drop(&mut self) {
        self.close();
    }
}
