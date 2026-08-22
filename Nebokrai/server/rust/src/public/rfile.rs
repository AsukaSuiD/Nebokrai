//! Частично восстановленный владелец `public/rfile.cpp`.
//!
//! Точная пара WorldServer: `Nworldserver.exe` / `WorldServer.pdb`, исходный
//! владелец PDB: `e:\svn\fengyun_russia_dev\public\rfile.cpp`. Подтверждены
//! компоновка cursor-а `CRFile`, границы `ReadData` и преобразование
//! `CheckRFileStr`; они материализованы ниже. Rust-владелец файла и буфера
//! заменяет `FILE*`, ручное освобождение и небезопасные копирования, не меняя
//! их контракт на корректном вводе.
//!
//! `rfOpen`, default-resource и package/decompression ветви остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально): они зависят от ещё не восстановленных `CClientResource`,
//! `CFilesInfo` и `CPackage`. `ReadToStream` также намеренно не материализован:
//! его файловая ветвь в точной дизассемблировке возвращает неустойчивое значение
//! регистра, а практический C++ reference ему противоречит. Это нельзя
//! превратить в Rust-контракт без дополнительной проверки EXE/PDB.
//!
//! Сырой C++ ниже остаётся доказательной заготовкой, а не Rust-реализацией.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:233
// RVA: 0x0005A8D0
// ADDRESS: 0045a8d0
// PROTOTYPE: CClientResource * __cdecl GetDefaultClientResource(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: rfClose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:79
// RVA: 0x0005A970
// ADDRESS: 0045a970
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\rfile.cpp:119
// RVA: 0x0005AB20
// ADDRESS: 0045ab20
// PROTOTYPE: CRFile * __cdecl rfOpen(char * param_1, CClientResource * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
