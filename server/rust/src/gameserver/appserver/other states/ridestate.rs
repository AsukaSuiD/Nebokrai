//! Достигнутый persisted/runtime owner `CRideState` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные owners
//! `appserver/other states/ridestate.h/.cpp`. State ID `100004`, variable
//! wire `ID/type/level/roleLimit/goodsName\0`, additional-data packing и
//! десятисекундный goods-check gate подтверждены EXE. Важный legacy quirk:
//! `AI` не обновляет `m_dwCheckGoodsTimeStamp`, поэтому после первого gate
//! проверяет packet каждый последующий AI turn. В AI0x004F9110 нет cached goods:
//! каждый проход получает GUID-список listener-а и заново проверяет type/level.
//! Общая арена CMoveShape заменяет `CState*`; имя хранит Vec<u8>, а wire-примитивы
//! делегированы общему legacy codec поверх `bytes`.
//! decode_at читает одну запись с заданного фабрикой offset; он сохраняет
//! исходный нулевой check timestamp и не ищет похожий ID внутри данных.
//! SetGoodName0x00438CA0 копирует std::string целиком, без ограничения длины.
//! Только safe Unserialize0x004F93B0 требует NUL в пределах native buffer256:
//! запись, которая переполнила бы стек C++, отклоняется, а не обрезается.
//! Serialize0x004F8F60 пишет четыре DWORD и C-string, без часов и мутации.
//! Cache-размер учитывает первый NUL, границы самой записи задаёт общий span.
//! Первичный object Begin0x004F8D60 требует nonnull sufferer, затем выполняет
//! base Begin(self,self), loop1 visual Update(0), SetFightable(false), check=0.
//! Единственный base clock поглощается на своём месте; base timestamp в Ride
//! не имеет достигнутых потребителей и не подменяет check timestamp.
//! Helper не регистрирует payload: caller после успеха сохраняет actual User/S
//! и остаточное loop1 visual-состояние в общей арене без повторного пакета.
//! Visual0x004F8E20 публикует BFE03(type,id,100004,0,type<<16|level)
//! либо BFE04(type,id,100004) через actual sufferer. Pure message-helper
//! не подменяет optional/ended/target gates и base visual tail владельца.
//! Общий CGame-вход visual выполняет base tail даже при NULL sufferer/ended,
//! но не при отсутствующем ресурсе. Повторный Begin(NULL,holder) сохраняет User
//! и не читает base clock: после visual ставит fight-lock и check=0.
//! End0x004F8D10: optional visual Update(1), свежий GetSufferer,
//! SetFightable(true), RemoveState у этой цели. Base End, запись ended,
//! принудительное удаление из holder и отдельный внешний Update отсутствуют.
//! Только фактическое RemoveState выполняет обычный property callback.
//! Destructor0x004F8FC0 освобождает имя и безусловно переходит в CState dtor
//! 0x005DBD40 (tail-jump0x004F8FED): освобождение visual, без End и пакетов.
//! Rust освобождает ресурсы безопасно; отказ native allocator не эмулируется,
//! исчезнувший после доставки target даёт отказ Begin перед fight-lock вместо
//! доступа по stale pointer; уже отправленный визуал не откатывается.
//! OnUpdateProperties0x004F9000 разрешает sufferer как CPlayer и ищет первый
//! live packet goods через original-name index именно этого экземпляра.
//! Он не использует goods-список AI и не вызывает visual/часы.
//! CPlayer::apply_ride_state_properties заимствует state и goods, затем
//! выполняет общий MountEquipRide(true) → MountEquipRide(false)0x0043C5E0.
//! Недостигнутые coordinate/typed-target перегрузки Begin сохранены только в локальном исследовательском корпусе.

use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    default_client_state_time, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
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
    check_goods_timestamp_ms: u32,
    serialized_offset: Option<usize>,
}

impl RideState {
    pub(crate) fn new(mount_type: u32, level: u32, role_limit: u32, goods_name: &[u8]) -> Self {
        Self {
            mount_type,
            level,
            role_limit,
            goods_name: goods_name.to_vec(),
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

    /// Хвост объектного Begin 0x004F8D60, после визуала и fight-lock.
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
            check_goods_timestamp_ms: 0,
            serialized_offset: Some(offset),
        })
    }

    pub(crate) fn encoded_for_install(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_size());
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(RIDE_STATE_ID);
        writer.write_u32(self.mount_type);
        writer.write_u32(self.level);
        writer.write_u32(self.role_limit);
        writer.write_c_string(&self.goods_name);
        bytes
    }

    pub(crate) fn serialized_size(&self) -> usize {
        let name_length = self.goods_name.iter().position(|byte| *byte == 0)
            .unwrap_or(self.goods_name.len());
        RIDE_STATE_FIXED_BYTES + name_length + 1
    }

    pub(crate) fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, self.serialized_size()))
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

pub(crate) fn ride_state_visual_message(
    target: ShapeIdentity,
    state: &RideState,
    begin: bool,
) -> CMessage {
    let mut message = CMessage::new(if begin { 0x0b_fe03 } else { 0x0b_fe04 });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_ulong(RIDE_STATE_ID);
    if begin {
        message.add_long(state.client_state_time());
        message.add_ulong(state.additional_data());
    }
    message
}

pub(crate) fn begin_primary_ride_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &mut RideState,
    now: &mut dyn FnMut() -> u32,
) -> Option<(i32, ShapeIdentity)> {
    resolve_state_move_shape(game, region_id, holder)?;
    let _base_timestamp = now();
    let shape = resolve_state_move_shape(game, region_id, holder)?.shape();
    let participant = (
        shape.get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() },
    );
    let message = ride_state_visual_message(participant.1, state, true);
    let _ = game.send_move_shape_around(participant.0, participant.1, &message);
    let sufferer = resolve_state_move_shape_mut(game, participant.0, participant.1)?;
    sufferer.set_fightable(false);
    state.reset_goods_check();
    Some(participant)
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

// COMPONENT_VARIANT_END: GameServer
