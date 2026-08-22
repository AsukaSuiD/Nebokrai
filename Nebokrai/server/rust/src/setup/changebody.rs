//! Ограничения товаров для смены тела исторического Miracle.
//!
//! Статус World `CChangeBodyConf::LoadChangeBodySetup` RVA `0x0003E6E0` и
//! `AddToByteArray` RVA `0x0003E610`: `IMPLEMENTED`; GameServer decoder и
//! singleton plumbing ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Owner очищает vector до resource-open, принимает direct `Goods` children
//! `RestrictionsGoodsList`, и кодирует signed count с `u32` item-ами. `quick-xml`
//! заменяет TinyXML; `GS1148..1151` и специальный clear при missing `index`
//! остаются подтверждённым контрактом. Resource backend передаёт уже выделенный
//! bytes slice, поэтому только allocation-failure diagnostic `GS1149` не имеет
//! отдельного безопасного Rust состояния.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CChangeBodyConf {
    restrictions_goods: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChangeBodyLoadError {
    MissingRootOrGoods,
    MissingIndex,
}

impl ChangeBodyLoadError {
    pub(crate) const fn string_id(self) -> &'static [u8] {
        match self {
            Self::MissingRootOrGoods => b"GS1150",
            Self::MissingIndex => b"GS1151",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChangeBodySerializeError {
    CountOverflow,
}

impl CChangeBodyConf {
    pub(crate) fn clear(&mut self) {
        self.restrictions_goods.clear();
    }

    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) -> Result<(), ChangeBodyLoadError> {
        self.clear();
        let result = self.load_from_bytes_after_clear(source);
        if matches!(result, Err(ChangeBodyLoadError::MissingIndex)) {
            self.clear();
        }
        result
    }

    fn load_from_bytes_after_clear(&mut self, source: &[u8]) -> Result<(), ChangeBodyLoadError> {
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut goods_seen = false;

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    self.process_start(&start, depth, &mut root_seen, &mut goods_seen)?;
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    self.process_start(&empty, depth, &mut root_seen, &mut goods_seen)?;
                }
                Ok(Event::End(_)) => {
                    if depth == 0 {
                        return Err(ChangeBodyLoadError::MissingRootOrGoods);
                    }
                    depth -= 1;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(ChangeBodyLoadError::MissingRootOrGoods),
            }
            buffer.clear();
        }
        if root_seen && goods_seen && depth == 0 {
            Ok(())
        } else {
            Err(ChangeBodyLoadError::MissingRootOrGoods)
        }
    }

    fn process_start(
        &mut self,
        start: &BytesStart<'_>,
        depth: usize,
        root_seen: &mut bool,
        goods_seen: &mut bool,
    ) -> Result<(), ChangeBodyLoadError> {
        let name = start.name();
        if !*root_seen {
            if name.as_ref() != b"RestrictionsGoodsList" {
                return Err(ChangeBodyLoadError::MissingRootOrGoods);
            }
            *root_seen = true;
        } else if depth == 1 && name.as_ref() == b"Goods" {
            let index = required_index(start)?;
            self.restrictions_goods.push(index);
            *goods_seen = true;
        }
        Ok(())
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), ChangeBodySerializeError> {
        let count = i32::try_from(self.restrictions_goods.len())
            .map_err(|_| ChangeBodySerializeError::CountOverflow)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for &goods_id in &self.restrictions_goods {
            destination.extend_from_slice(&goods_id.to_le_bytes());
        }
        Ok(())
    }
}

fn required_index(start: &BytesStart<'_>) -> Result<u32, ChangeBodyLoadError> {
    let value = start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == b"index")
        .map(|attribute| attribute.value.into_owned())
        .ok_or(ChangeBodyLoadError::MissingIndex)?;
    Ok(legacy_atol(&value) as u32)
}

fn legacy_atol(value: &[u8]) -> i32 {
    let mut bytes = value.iter().copied().skip_while(u8::is_ascii_whitespace).peekable();
    let negative = matches!(bytes.peek(), Some(b'-'));
    if matches!(bytes.peek(), Some(b'-' | b'+')) {
        bytes.next();
    }
    let mut parsed = false;
    let mut result = 0_i32;
    for byte in bytes {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        parsed = true;
        result = result.saturating_mul(10).saturating_add(i32::from(digit));
    }
    if parsed {
        if negative { result.saturating_neg() } else { result }
    } else {
        0
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\changebody.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp

// ============================================================================
// FUNCTION: CChangeBodyConf::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.h:24
// RVA: 0x0002AAD0
// ADDRESS: 0042aad0
// PROTOTYPE: CChangeBodyConf * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::GetVector
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:31
// RVA: 0x000C5D30
// ADDRESS: 004c5d30
// PROTOTYPE: vector<unsigned_long,std::allocator<unsigned_long>_> * __thiscall GetVector(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::~CChangeBodyConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:24
// RVA: 0x000C5D40
// ADDRESS: 004c5d40
// PROTOTYPE: void __thiscall ~CChangeBodyConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::CChangeBodyConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:19
// RVA: 0x000C5DA0
// ADDRESS: 004c5da0
// PROTOTYPE: undefined __thiscall CChangeBodyConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:124
// RVA: 0x000C5DC0
// ADDRESS: 004c5dc0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eae4d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000EAE4D
// ADDRESS: 004eae4d
// PROTOTYPE: undefined Catch@004eae4d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eaf3d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000EAF3D
// ADDRESS: 004eaf3d
// PROTOTYPE: undefined Catch@004eaf3d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eb23f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000EB23F
// ADDRESS: 004eb23f
// PROTOTYPE: undefined Catch@004eb23f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eb2f2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000EB2F2
// ADDRESS: 004eb2f2
// PROTOTYPE: undefined Catch@004eb2f2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\changebody.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp

// ============================================================================
// FUNCTION: CChangeBodyConf::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.h:24
// RVA: 0x000012C0
// ADDRESS: 004012c0
// PROTOTYPE: CChangeBodyConf * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:112
// RVA: 0x0003E610
// ADDRESS: 0043e610
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::~CChangeBodyConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:24
// RVA: 0x0003E660
// ADDRESS: 0043e660
// PROTOTYPE: void __thiscall ~CChangeBodyConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::CChangeBodyConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:19
// RVA: 0x0003E6C0
// ADDRESS: 0043e6c0
// PROTOTYPE: undefined __thiscall CChangeBodyConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CChangeBodyConf::LoadChangeBodySetup
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp:39
// RVA: 0x0003E6E0
// ADDRESS: 0043e6e0
// PROTOTYPE: bool __thiscall LoadChangeBodySetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::stComposeNode::~stComposeNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000862D0
// ADDRESS: 004862d0
// PROTOTYPE: void __thiscall ~stComposeNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00486531
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x00086531
// ADDRESS: 00486531
// PROTOTYPE: undefined Catch@00486531()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004867db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000867DB
// ADDRESS: 004867db
// PROTOTYPE: undefined Catch@004867db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00486a92
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x00086A92
// ADDRESS: 00486a92
// PROTOTYPE: undefined Catch@00486a92()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00486c36
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x00086C36
// ADDRESS: 00486c36
// PROTOTYPE: undefined Catch@00486c36()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004870fd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x000870FD
// ADDRESS: 004870fd
// PROTOTYPE: undefined Catch@004870fd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048719d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x0008719D
// ADDRESS: 0048719d
// PROTOTYPE: undefined Catch@0048719d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00487484
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x00087484
// ADDRESS: 00487484
// PROTOTYPE: undefined Catch@00487484()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00487537
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x00087537
// ADDRESS: 00487537
// PROTOTYPE: undefined Catch@00487537()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00531320
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\changebody.cpp
// RVA: 0x00131320
// ADDRESS: 00531320
// PROTOTYPE: undefined Unwind@00531320()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
