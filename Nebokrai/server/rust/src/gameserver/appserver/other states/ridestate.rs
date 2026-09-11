//! Достигнутый persisted/runtime owner `CRideState` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные owners
//! `appserver/other states/ridestate.h/.cpp`. State ID `100004`, variable
//! wire `ID/type/level/roleLimit/goodsName\0`, additional-data packing и
//! десятисекундный goods-check gate подтверждены EXE. Важный legacy quirk:
//! `AI` не обновляет `m_dwCheckGoodsTimeStamp`, поэтому после первого gate
//! проверяет packet каждый последующий AI turn. Общая арена CMoveShape заменяет
//! `CState*`, остальные safe payload заменяют cached raw goods pointer; gameplay
//! ordering, GUID cache invalidation и wrapping DWORD compare сохранены.
//! Wire-примитивы делегированы общему legacy codec поверх `bytes`.
//! decode_at читает одну запись с заданного фабрикой offset; он сохраняет
//! исходный нулевой check timestamp и не ищет похожий ID внутри данных.

use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::state::default_client_state_time;
use crate::public::guid::CGuid;

pub(crate) const RIDE_STATE_ID: u32 = 100_004;
const RIDE_STATE_FIXED_BYTES: usize = 16;
const RIDE_GOODS_NAME_CAPACITY: usize = 256;
pub(crate) const RIDE_GOODS_CHECK_INTERVAL_MS: u32 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RideState {
    mount_type: u32,
    level: u32,
    role_limit: u32,
    goods_name: Vec<u8>,
    cached_goods_id: CGuid,
    check_goods_timestamp_ms: u32,
    serialized_offset: Option<usize>,
}

impl RideState {
    pub(crate) fn new(mount_type: u32, level: u32, role_limit: u32, goods_name: &[u8]) -> Self {
        let prefix = goods_name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(goods_name.len())
            .min(RIDE_GOODS_NAME_CAPACITY - 1);
        Self {
            mount_type,
            level,
            role_limit,
            goods_name: goods_name[..prefix].to_vec(),
            cached_goods_id: CGuid::GUID_INVALID,
            check_goods_timestamp_ms: 0,
            serialized_offset: None,
        }
    }

    pub(crate) const fn mount_type(&self) -> u32 {
        self.mount_type
    }

    pub(crate) const fn level(&self) -> u32 {
        self.level
    }

    pub(crate) const fn role_limit(&self) -> u32 {
        self.role_limit
    }

    pub(crate) fn goods_name(&self) -> &[u8] {
        &self.goods_name
    }

    pub(crate) const fn cached_goods_id(&self) -> CGuid {
        self.cached_goods_id
    }

    pub(crate) const fn set_cached_goods_id(&mut self, goods_id: CGuid) {
        self.cached_goods_id = goods_id;
    }

    pub(crate) const fn clear_cached_goods_id(&mut self) {
        self.cached_goods_id = CGuid::GUID_INVALID;
    }

    /// Хвост объектного Begin 0x004F8D60; cached goods не сбрасывается.
    pub(crate) const fn reset_goods_check(&mut self) {
        self.check_goods_timestamp_ms = 0;
    }

    pub(crate) const fn additional_data(&self) -> u32 {
        self.mount_type.wrapping_shl(16) | self.level
    }

    pub(crate) const fn client_state_time(&self) -> i32 {
        default_client_state_time()
    }

    /// Exact EXE `timestamp + 10000 <= timeGetTime`; timestamp намеренно не
    /// обновляется после успешного gate.
    pub(crate) const fn goods_check_due(&self, now_ms: u32) -> bool {
        self.check_goods_timestamp_ms
            .wrapping_add(RIDE_GOODS_CHECK_INTERVAL_MS)
            <= now_ms
    }

    pub(crate) fn decode_at(payload: &[u8], offset: usize) -> Option<Self> {
        if read_u32(payload, offset)? != RIDE_STATE_ID {
            return None;
        }
        let name_start = offset.checked_add(RIDE_STATE_FIXED_BYTES)
            .filter(|start| *start <= payload.len())?;
        let available = payload.len().saturating_sub(name_start);
        let Some(name_length) = payload[name_start..]
            .iter()
            .take(RIDE_GOODS_NAME_CAPACITY)
            .position(|byte| *byte == 0)
        else {
            return None;
        };
        if name_length >= available {
            return None;
        }
        Some(Self {
            mount_type: read_u32(payload, offset + 4)?,
            level: read_u32(payload, offset + 8)?,
            role_limit: read_u32(payload, offset + 12)?,
            goods_name: payload[name_start..name_start + name_length].to_vec(),
            cached_goods_id: CGuid::GUID_INVALID,
            check_goods_timestamp_ms: 0,
            serialized_offset: Some(offset),
        })
    }

    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>) {
        let offset = payload.len();
        let mut writer = LegacyWriter::new(payload);
        writer.write_u32(RIDE_STATE_ID);
        writer.write_u32(self.mount_type);
        writer.write_u32(self.level);
        writer.write_u32(self.role_limit);
        writer.write_c_string(&self.goods_name);
        self.serialized_offset = Some(offset);
    }

    pub(crate) fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, RIDE_STATE_FIXED_BYTES + self.goods_name.len() + 1))
    }

    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(payload, offset).ok()?.read_u32().ok()
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp

// ============================================================================
// FUNCTION: CRideState::SetGoodName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.h:24
// RVA: 0x00038CA0
// ADDRESS: 00438ca0
// PROTOTYPE: void __thiscall SetGoodName(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:66
// RVA: 0x000F8B50
// ADDRESS: 004f8b50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:87
// RVA: 0x000F8C30
// ADDRESS: 004f8c30
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:107
// RVA: 0x000F8D10
// ADDRESS: 004f8d10
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::GetAdditionalData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:252
// RVA: 0x000F8D50
// ADDRESS: 004f8d50
// PROTOTYPE: ulong __thiscall GetAdditionalData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:48
// RVA: 0x000F8D60
// ADDRESS: 004f8d60
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:259
// RVA: 0x000F8E20
// ADDRESS: 004f8e20
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:216
// RVA: 0x000F8F60
// ADDRESS: 004f8f60
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::~CRideState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:44
// RVA: 0x000F8FC0
// ADDRESS: 004f8fc0
// PROTOTYPE: void __thiscall ~CRideState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:124
// RVA: 0x000F9000
// ADDRESS: 004f9000
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:149
// RVA: 0x000F9110
// ADDRESS: 004f9110
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::CRideState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:17
// RVA: 0x000F9280
// ADDRESS: 004f9280
// PROTOTYPE: undefined __thiscall CRideState(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::CRideState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:31
// RVA: 0x000F9320
// ADDRESS: 004f9320
// PROTOTYPE: undefined __thiscall CRideState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRideState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\ridestate.cpp:229
// RVA: 0x000F93B0
// ADDRESS: 004f93b0
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
