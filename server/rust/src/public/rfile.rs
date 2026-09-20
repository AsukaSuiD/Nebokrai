//! Частично восстановленный владелец `public/rfile.cpp`.
//!
//! Точная пара WorldServer: `Nworldserver.exe` / `WorldServer.pdb`, исходный
//! владелец PDB: `e:\svn\fengyun_russia_dev\public\rfile.cpp`. Подтверждены
//! компоновка cursor-а `CRFile`, границы `ReadData` и преобразование
//! `CheckRFileStr`; они материализованы ниже. Rust-владелец файла и буфера
//! заменяет `FILE*`, ручное освобождение и небезопасные копирования, не меняя
//! их контракт на корректном вводе.
//!
//! `rf_open` материализует точный выбор package/loose источника для explicit
//! `CClientResource`; `Option<CRFile>` заменяет nullable старый указатель.
//! Process-global default-resource принадлежит безопасному
//! `DefaultClientResourceOwner` в соседнем owner-е. `ReadToStream` materialized
//! как `CRFile::read_to_stream`: его память и файл вставляют C-строку до первого
//! NUL, однако точный EXE возвращает `true` только для memory-ветви. Rust `Vec`
//! заменяет stream buffer без raw временного массива и unchecked `fread`.
//!
//! Сырой C++ ниже остаётся доказательной заготовкой, а не Rust-реализацией.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::public::clientresource::ClientResource;

/// Явный borrowed resource-context вместо `g_pDefaultClientResource`.
pub(crate) struct RFileResource<'a> {
    resource: Option<&'a ClientResource>,
    root: &'a std::path::Path,
}

impl<'a> RFileResource<'a> {
    pub(crate) fn loaded(resource: &'a ClientResource, root: &'a std::path::Path) -> Self {
        Self {
            resource: Some(resource),
            root,
        }
    }

    /// Представляет уже созданный `CClientResource`, у которого `LoadEx`
    /// оставил пустой/недоступный file index.
    pub(crate) fn without_index(root: &'a std::path::Path) -> Self {
        Self {
            resource: None,
            root,
        }
    }
}

/// Безопасная замена двух подтверждённых источников `CRFile`.
///
/// `position` сохраняет `m_dwPos`; файловое чтение, как в оригинале, перед
/// каждым запросом позиционируется по нему с начала файла.
pub(crate) struct CRFile {
    source: CRFileSource,
    size: u32,
    position: u32,
}

enum CRFileSource {
    Memory(Vec<u8>),
    File(File),
}

impl CRFile {
    /// Точный конструктор памяти `CRFile(unsigned char *, unsigned long)`.
    pub(crate) fn from_memory(data: Vec<u8>) -> Self {
        let size = u32::try_from(data.len())
            .expect("размер буфера CRFile превышает unsigned long исходного сервера");
        Self {
            source: CRFileSource::Memory(data),
            size,
            position: 0,
        }
    }

    /// Создаёт файловый cursor с размером, уже полученным владельцем открытия.
    pub(crate) fn from_file(file: File, size: u32) -> Self {
        Self {
            source: CRFileSource::File(file),
            size,
            position: 0,
        }
    }

    /// Точный вид поля `m_dwSize` после успешного `rfOpen`.
    pub(crate) const fn size(&self) -> u32 {
        self.size
    }

    /// Повторяет успешную ветвь `ReadData`.
    ///
    /// Переполнение суммы позиции и длины, а также неполное чтение исходника были
    /// внутренними дефектами C++ реализации. Здесь они возвращают `false` и не
    /// меняют логический cursor: корректные resource-файлы сохраняют тот же результат.
    pub(crate) fn read_data(&mut self, output: &mut [u8]) -> bool {
        let requested = match u32::try_from(output.len()) {
            Ok(requested) => requested,
            Err(_) => return false,
        };
        let end = match self.position.checked_add(requested) {
            Some(end) if end <= self.size => end,
            _ => return false,
        };

        let read_ok = match &mut self.source {
            CRFileSource::Memory(data) => {
                let start = self.position as usize;
                let end = end as usize;
                output.copy_from_slice(&data[start..end]);
                true
            }
            CRFileSource::File(file) => file
                .seek(SeekFrom::Start(u64::from(self.position)))
                .and_then(|_| file.read_exact(output))
                .is_ok(),
        };

        if read_ok {
            self.position = end;
        }
        read_ok
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
    pub(crate) fn read_to_stream(&mut self, output: &mut Vec<u8>) -> bool {
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

/// Повторяет побайтовую часть `CheckRFileStr`.
///
/// Оригинал приводил байты к нижнему регистру активной locale CRT. Внешний контракт
/// путей WorldServer подтверждён для ASCII; байты вне ASCII сохраняются, чтобы
/// не навязывать Rust Unicode-normalization. После замены `/` на `\\` функция
/// добавляет начальный `\\`, если первый обратный слеш не стоит на нулевой позиции.
pub(crate) fn check_rfile_str(path: &mut Vec<u8>) {
    for byte in path.iter_mut() {
        byte.make_ascii_lowercase();
        if *byte == b'/' {
            *byte = b'\\';
        }
    }

    if path.first() != Some(&b'\\') {
        path.insert(0, b'\\');
    }
}

/// Безопасный read-side `rfOpen` для explicit resource либо прямого loose пути.
///
/// При resource сначала выполняется exact `CheckRFileStr`. Несуществующая
/// index-запись и установленный bit 0 `dwPackageType` открываются из
/// `m_strCurFolder`; package-ветвь выдаёт memory cursor. Как и исходный
/// nullable `CRFile*`, ошибка файла, пакета или декодирования возвращает
/// `None`: подробная диагностика остаётся на API `ClientResource`.
///
/// Без resource exact owner использует process-global default. В Rust его
/// передаёт `DefaultClientResourceOwner` как `RFileResource`; если контекст
/// действительно отсутствует, остаётся прямой loose fallback параметра без
/// package normalizing.
pub(crate) fn rf_open(path: &[u8], resource: Option<RFileResource<'_>>) -> Option<CRFile> {
    let path = c_string_prefix(path);
    let Some(resource) = resource else {
        return open_loose_file(std::path::Path::new(String::from_utf8_lossy(path).as_ref()));
    };

    let mut normalized = path.to_vec();
    check_rfile_str(&mut normalized);
    let Some(client_resource) = resource.resource else {
        return open_loose_file(&resource_loose_path(resource.root, &normalized));
    };
    let loose = client_resource
        .file_info(&normalized)
        .is_none_or(|info| info.package_type() & 1 != 0);
    if loose {
        return open_loose_file(&resource_loose_path(resource.root, &normalized));
    }

    client_resource
        .read_packaged(&normalized)
        .ok()
        .flatten()
        .map(CRFile::from_memory)
}

/// Закрывает nullable результат `rf_open`.
///
/// Владение `CRFile` уже принадлежит вызывающему коду, поэтому `Drop` корректно
/// освобождает и memory buffer, и файловый дескриптор. В точном C++ пути
/// памяти ранний `return` оставлял сам объект `CRFile` неосвобождённым; это
/// внутренний lifetime-дефект без требуемого внешнего эффекта и намеренно не
/// переносится.
pub(crate) fn rf_close(file: Option<CRFile>) {
    drop(file);
}

fn c_string_prefix(value: &[u8]) -> &[u8] {
    value
        .iter()
        .position(|byte| *byte == 0)
        .map_or(value, |end| &value[..end])
}

fn resource_loose_path(root: &std::path::Path, normalized: &[u8]) -> std::path::PathBuf {
    let mut path = root.to_path_buf();
    for part in normalized[1..].split(|byte| *byte == b'\\') {
        if !part.is_empty() {
            path.push(String::from_utf8_lossy(part).as_ref());
        }
    }
    path
}

fn open_loose_file(path: &std::path::Path) -> Option<CRFile> {
    let file = File::open(path).ok()?;
    let size = u32::try_from(file.metadata().ok()?.len()).ok()?;
    Some(CRFile::from_file(file, size))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\rfile.cpp

// ============================================================================
// FUNCTION: CRFile::CRFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:27
// RVA: 0x000A20F0
// ADDRESS: 004a20f0
// PROTOTYPE: undefined __thiscall CRFile(uchar * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRFile::ReadData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:50
// RVA: 0x000A2110
// ADDRESS: 004a2110
// PROTOTYPE: bool __thiscall ReadData(void * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: rfClose
// STATUS: IMPLEMENTED / OWNERSHIP_SUBSTITUTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:208
// RVA: 0x000A21A0
// ADDRESS: 004a21a0
// PROTOTYPE: void __cdecl rfClose(CRFile * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CheckRFileStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:216
// RVA: 0x000A2370
// ADDRESS: 004a2370
// PROTOTYPE: void __cdecl CheckRFileStr(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: rfOpen
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:119
// RVA: 0x000A2470
// ADDRESS: 004a2470
// PROTOTYPE: CRFile * __cdecl rfOpen(char * param_1, CClientResource * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\rfile.cpp

// ============================================================================
// FUNCTION: CRFile::CRFile
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:27
// RVA: 0x0005A820
// ADDRESS: 0045a820
// PROTOTYPE: undefined __thiscall CRFile(uchar * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRFile::ReadData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:50
// RVA: 0x0005A840
// ADDRESS: 0045a840
// PROTOTYPE: bool __thiscall ReadData(void * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetDefaultClientResource
// STATUS: IMPLEMENTED / OWNERSHIP_SUBSTITUTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:233
// RVA: 0x0005A8D0
// ADDRESS: 0045a8d0
// PROTOTYPE: CClientResource * __cdecl GetDefaultClientResource(void)
//
// `get_default_client_resource` принимает safe owner на Rust context boundary
// вместо mutable static и возвращает nullable borrowed view. Она сохраняет
// опубликованный `CClientResource` после неуспешного `LoadEx`, но не позволяет
// сохранить висячую ссылку через последующие LoadServerResource/Release.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: rfClose
// STATUS: IMPLEMENTED / OWNERSHIP_SUBSTITUTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:208
// RVA: 0x0005A8E0
// ADDRESS: 0045a8e0
// PROTOTYPE: void __cdecl rfClose(CRFile * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRFile::ReadToStream
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:79
// RVA: 0x0005A970
// ADDRESS: 0045a970
//
// `CRFile::read_to_stream` выше заменяет stream/temporary-buffer ownership
// owned `Vec<u8>`. Exact `0x0045A9A6` задаёт memory return `true`, а машинный
// epilogue `0x0045AA11..0x0045AA15` задаёт file/empty return `false`; прежняя
// декомпиляция скрывала этот `xor al, al` как нестабильный register-result.
// C-string truncation, file-position и memory cursor сохранены отдельно.
// PROTOTYPE: bool __thiscall ReadToStream(basic_stringstream<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CheckRFileStr
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:216
// RVA: 0x0005AA20
// ADDRESS: 0045aa20
// PROTOTYPE: void __cdecl CheckRFileStr(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: rfOpen
// STATUS: IMPLEMENTED / OWNERSHIP_CONTEXT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:119
// RVA: 0x0005AB20
// ADDRESS: 0045ab20
// PROTOTYPE: CRFile * __cdecl rfOpen(char * param_1, CClientResource * param_2)
//
// loose fallback. Process-global `g_pDefaultClientResource` выражен через
// `DefaultClientResourceOwner::open`, который передаёт borrowed context без
// mutable Rust static. `None` здесь остаётся точным случаем, когда и explicit,
// и default resource отсутствуют, поэтому открывается исходный loose path.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
