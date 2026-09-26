//! HTTP-owner `loginserver/applogin/mywininet.cpp/.h`.
//!
//! WinInet заменён blocking `reqwest`/rustls с теми же HTTP/1 POST headers,
//! системным proxy и обходом проверки CA старого HTTPS-сервера; hostname всё
//! ещё проверяется, а конечный timeout позволяет присоединить `CGasThread`.
//! `Recv` сохраняет последний C-string-срез общего 1024-байтового буфера вместе
//! с хвостом предыдущего куска. При полном куске без NUL безопасная граница
//! возвращает весь буфер вместо исходного чтения за массивом.

use std::error::Error;
use std::fmt;
use std::io::Read;

use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::Url;

const RESPONSE_CAPACITY: usize = 0x400;

pub enum MyWinInetError {
    InvalidUrlEncoding,
    InvalidUrl,
    UnsupportedScheme,
    Client(reqwest::Error),
    Request(reqwest::Error),
    ResponseMissing,
}

impl fmt::Debug for MyWinInetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // reqwest::Error может содержать полный setup URL. Диагностический
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

pub struct CMyWinInet {
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
    pub fn init(&mut self, raw_url: &[u8]) -> Result<(), MyWinInetError> {
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

    pub fn send(&mut self, content: &[u8]) -> Result<(), MyWinInetError> {
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

    pub fn recv(&mut self) -> Result<Option<Vec<u8>>, MyWinInetError> {
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

    pub fn close(&mut self) {
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
