//! Курсор памяти/файла по public/rfile.cpp: World CRFile::ReadData,
//! RVA 0x0005A840. ReadToStream сохраняет различие памяти и файла.
//! Привязки и правила: docs/architecture/resources-and-configuration.md,
//! раздел «Курсор ресурса». Result и передача буфера — интерфейсы Rust.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

/// Безопасная замена двух подтверждённых источников `CRFile`.
///
/// `position` сохраняет `m_dwPos`; файловое чтение, как в оригинале, перед
/// каждым запросом позиционируется по нему с начала файла.
pub struct CRFile {
    source: CRFileSource,
    size: u32,
    position: u32,
}

enum CRFileSource {
    Memory(Vec<u8>),
    File(File),
}

impl CRFile {
    /// Принимает буфер во владение; размер должен помещаться в 32 бита.
    pub fn from_memory(data: Vec<u8>) -> io::Result<Self> {
        let size = u32::try_from(data.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Ресурс больше u32"))?;
        Ok(Self {
            source: CRFileSource::Memory(data),
            size,
            position: 0,
        })
    }

    /// Создаёт файловый cursor с размером, уже полученным владельцем открытия.
    pub fn from_file(file: File, size: u32) -> Self {
        Self {
            source: CRFileSource::File(file),
            size,
            position: 0,
        }
    }

    /// Точный вид поля `m_dwSize` после успешного `rfOpen`.
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// Совместимый bool-интерфейс: логический курсор меняется после полного чтения.
    pub fn read_data(&mut self, output: &mut [u8]) -> bool {
        self.read_exact(output).is_ok()
    }

    /// Читает от логического курсора с сохранением причины отказа.
    /// Файловое чтение может частично заполнить output до ошибки.
    pub fn read_exact(&mut self, output: &mut [u8]) -> io::Result<()> {
        let requested = u32::try_from(output.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Запрос больше u32"))?;
        let end = self
            .position
            .checked_add(requested)
            .filter(|end| *end <= self.size)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, "Чтение за границей ресурса")
            })?;
        match &mut self.source {
            CRFileSource::Memory(data) => {
                output.copy_from_slice(&data[self.position as usize..end as usize]);
            }
            CRFileSource::File(file) => {
                file.seek(SeekFrom::Start(u64::from(self.position)))?;
                file.read_exact(output)?;
            }
        }
        self.position = end;
        Ok(())
    }

    /// Возвращает весь ресурс с начала, потребляя курсор.
    /// Буфер памяти передаётся без копирования; файл читается в пределах
    /// размера, зафиксированного при открытии, с возвратом ошибки I/O.
    pub fn into_bytes(self) -> io::Result<Vec<u8>> {
        match self.source {
            CRFileSource::Memory(data) => Ok(data),
            CRFileSource::File(mut file) => {
                let size = usize::try_from(self.size).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Размер ресурса недоступен")
                })?;
                let mut data = Vec::new();
                data.try_reserve_exact(size)
                    .map_err(|error| io::Error::new(io::ErrorKind::OutOfMemory, error))?;
                data.resize(size, 0);
                file.seek(SeekFrom::Start(0))?;
                file.read_exact(&mut data)?;
                Ok(data)
            }
        }
    }

    /// Повторяет `CRFile::ReadToStream` через owned stream-buffer.
    ///
    /// Точный `operator<<(char const *)` публикует байты только до первого
    /// NUL, хотя memory-ветвь затем сдвигает `m_dwPos` на полный `m_dwSize`.
    /// Это подтверждённое unsigned wrapping сложения оставлено явно: оно может
    /// изменить следующий cursor-visible вызов и потому не нормализуется молча.
    /// Файловая ветвь читает от текущей позиции host file и, даже после
    /// успешной вставки, возвращает `false`: это подтверждено epilogue
    /// `0x0045AA11: xor al, al`, а не выводится из менее доверенного донора.
    ///
    /// Ошибка аллокации или short host read заменяет старые неопределённые
    /// данные safe `false` без частичной публикации. На корректном ресурсе
    /// сохраняются bytes, cursor и точное различие return value.
    pub fn read_to_stream(&mut self, output: &mut Vec<u8>) -> bool {
        match &mut self.source {
            CRFileSource::Memory(data) => {
                if !append_c_string(output, data) {
                    return false;
                }
                self.position = self.position.wrapping_add(self.size);
                true
            }
            CRFileSource::File(file) => {
                let mut data = Vec::new();
                let Ok(size) = usize::try_from(self.size) else {
                    return false;
                };
                if data.try_reserve_exact(size).is_err() {
                    return false;
                }
                data.resize(size, 0);
                if file.read_exact(&mut data).is_err() {
                    return false;
                }
                let _ = append_c_string(output, &data);
                false
            }
        }
    }
}

fn append_c_string(output: &mut Vec<u8>, data: &[u8]) -> bool {
    let length = data
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(data.len());
    if output.try_reserve(length).is_err() {
        return false;
    }
    output.extend_from_slice(&data[..length]);
    true
}
