//! GameServer startup-owner войны четырёх стран `CFourNationWarSys`.
//!
//! Wire и startup side effects подтверждены точными
//! `gameserver.exe + GameServer.pdb` и `worldserver.exe + worldserver.pdb`;
//! исходные owners `organizingsystem/fournationwarsys.cpp/.h`. Snapshot несёт
//! signed count, 196-байтные setup records, затем signed count и 16-байтные
//! `tagRECT`. World serializer и Game decoder используют одинаковый порядок.
//!
//! Decoder принимает повторный snapshot, пока setup vector остаётся пустым;
//! при уже опубликованных setup оригинал удалял buffer и оставлял dangling
//! pointers, поэтому Rust возвращает typed error без разрушения state. Rect
//! count больше пяти в оригинале писал за static array и здесь блокируется
//! после сохранения допустимого префикса. Malformed tail также сохраняет все
//! полностью опубликованные records и cursor.
//!
//! `InitWarState` идёт по vector-order, ищет main region с proxy fallback,
//! принимает только nation region, применяет `(index, region_state)` и копирует
//! пять relive rectangles. Process singleton заменён owned-полем `CGame`.
//! Полная фазовая цепочка `0x7FE3C..0x7FE45` сохраняет проверку индекса,
//! DUTH/Mass/Fight, различия local/proxy lookup, очистку process-wide времени
//! и morale, пятизначный signup payload и atomic take пяти результатов.
//! Region/shape/NPC/player effects фаз исполняют concrete `CGame` и
//! `ServerNationRegion`; отдельного process runtime у FourNation больше нет.
//! Direct morale `0x7FE49` и replacement player-war-time `0x7FE47` принадлежат
//! тому же game owner-у; остальные player-war-time queries ниже ещё сохраняют
//! RAW.

use std::collections::BTreeMap;
use thiserror::Error;

use nebokrai_shared::protocol::LegacyReader;
use crate::public::date::TagTime;

const FOUR_NATION_SETUP_WIRE_SIZE: usize = 0xc4;
const FOUR_NATION_RECT_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FourNationGameSetup {
    pub(crate) time_index: i32,
    pub(crate) region_id: i32,
    pub(crate) sign_up_start_event_id: u32,
    pub(crate) sign_up_start_time: TagTime,
    pub(crate) sign_up_end_event_id: u32,
    pub(crate) sign_up_end_time: TagTime,
    pub(crate) start_event_id: u32,
    pub(crate) start_time: TagTime,
    pub(crate) end_event_id: u32,
    pub(crate) end_time: TagTime,
    pub(crate) end_info_event_id: u32,
    pub(crate) end_info_time: TagTime,
    pub(crate) enter_start_event_id: u32,
    pub(crate) enter_start_time: TagTime,
    pub(crate) enter_end_event_id: u32,
    pub(crate) enter_end_time: TagTime,
    pub(crate) refresh_event_id: u32,
    pub(crate) refresh_region_time: TagTime,
    pub(crate) clear_war_event_id: u32,
    pub(crate) clear_war_time: TagTime,
    pub(crate) region_state: i32,
    pub(crate) is_every_week: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct FourNationRect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CFourNationWarSys {
    setups: Vec<FourNationGameSetup>,
    rects: [FourNationRect; FOUR_NATION_RECT_COUNT],
    morale: i32,
    player_war_times_ms: BTreeMap<i32, u32>,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum FourNationGameDecodeError {
    #[error("FourNationWar snapshot повторно получен при {retained_setups} опубликованных setup")]
    AlreadyInitialized {
        retained_setups: usize,
    },
    #[error("FourNationWar snapshot обрывается на {field} в {offset}: нужно {required}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    #[error("FourNationWar snapshot объявляет {declared} rectangles при capacity {capacity}")]
    RectCapacity {
        declared: i32,
        capacity: usize,
    },
}

pub(crate) trait FourNationGameStartupContext {
    type Region: Copy;

    /// Ищет main region, затем proxy, и возвращает только nation region.
    fn find_nation_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    fn reset_nation_war_state(&mut self, region: Self::Region, index: i32, state: i32);

    fn set_nation_relive_rects(
        &mut self,
        region: Self::Region,
        rects: [FourNationRect; FOUR_NATION_RECT_COUNT],
    );
}

pub(crate) trait FourNationPhaseContext {
    type Region: Copy;

    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;
    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region>;
    fn find_server_nation_region(&mut self, region_id: i32) -> Option<Self::Region>;
    fn on_war_declare(&mut self, region: Self::Region, war_number: i32);
    fn on_nation_war_declare(
        &mut self,
        region: Self::Region,
        war_number: i32,
        sign_up_counts: [i32; 5],
    );
    fn on_war_mass(&mut self, region: Self::Region, war_number: i32);
    fn on_war_start(&mut self, region: Self::Region, war_number: i32);
    fn on_refresh_region(&mut self, region: Self::Region, war_number: i32);
    fn on_war_end(&mut self, region: Self::Region, war_number: i32);
    fn on_clear_war(&mut self, region: Self::Region, war_number: i32);
    fn take_war_results(&mut self, region: Self::Region) -> [u32; 5];
    fn add_war_end_log(&mut self, war_number: i32);
}

impl CFourNationWarSys {
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), FourNationGameDecodeError> {
        if !self.setups.is_empty() {
            return Err(FourNationGameDecodeError::AlreadyInitialized {
                retained_setups: self.setups.len(),
            });
        }

        let declared_setups = read_four_nation_i32(source, cursor, "setup count")?;
        let mut decoded_setups = 0;
        for _ in 0..declared_setups.max(0) as usize {
            let bytes = take_four_nation_bytes(
                source,
                cursor,
                FOUR_NATION_SETUP_WIRE_SIZE,
                "setup record",
            )?;
            self.setups.push(decode_four_nation_setup(bytes));
            decoded_setups += 1;
        }

        let declared_rects = read_four_nation_i32(source, cursor, "rectangle count")?;
        let mut decoded_rects = 0;
        for index in 0..declared_rects.max(0) as usize {
            if index >= FOUR_NATION_RECT_COUNT {
                return Err(FourNationGameDecodeError::RectCapacity {
                    declared: declared_rects,
                    capacity: FOUR_NATION_RECT_COUNT,
                });
            }
            let bytes = take_four_nation_bytes(source, cursor, 0x10, "rectangle")?;
            self.rects[index] = FourNationRect {
                left: four_nation_i32_at(bytes, 0),
                top: four_nation_i32_at(bytes, 4),
                right: four_nation_i32_at(bytes, 8),
                bottom: four_nation_i32_at(bytes, 12),
            };
            decoded_rects += 1;
        }

        tracing::trace!(declared_setups, decoded_setups, declared_rects, decoded_rects, "расписание войны четырёх государств декодировано");
        Ok(())
    }

    pub(crate) fn init_war_state<Context: FourNationGameStartupContext>(
        &self,
        context: &mut Context,
    ) {
        let mut nation_regions = 0;
        for (index, setup) in self.setups.iter().enumerate() {
            let Some(region) = context.find_nation_region_then_proxy(setup.region_id) else {
                continue;
            };
            context.reset_nation_war_state(region, index as i32, setup.region_state);
            context.set_nation_relive_rects(region, self.rects);
            nation_regions += 1;
        }
        tracing::trace!(setups = self.setups.len(), nation_regions, "состояние регионов войны четырёх государств инициализировано");
    }

    pub(crate) fn setups(&self) -> &[FourNationGameSetup] {
        &self.setups
    }

    pub(crate) const fn rects(&self) -> &[FourNationRect; FOUR_NATION_RECT_COUNT] {
        &self.rects
    }

    /// Exact direct assignment из OrganSys `0x7FE49`.
    pub(crate) const fn set_morale(&mut self, morale: i32) {
        self.morale = morale;
    }

    pub(crate) const fn morale(&self) -> i32 {
        self.morale
    }

    /// Exact `ClearMoraleValue`: process-static long становится нулём.
    pub(crate) const fn clear_morale_value(&mut self) -> i32 {
        let previous = self.morale;
        self.morale = 0;
        previous
    }

    /// Exact hash-map replacement из `SetOnePlayerWarTime`.
    pub(crate) fn set_one_player_war_time(&mut self, player_id: i32, time_ms: u32) -> Option<u32> {
        self.player_war_times_ms.insert(player_id, time_ms)
    }

    /// Exact `GetPlayerWarTime`: missing key даёт ноль, stored milliseconds
    /// усекаются целочисленным делением до полных секунд.
    pub(crate) fn player_war_time_seconds(&self, player_id: i32) -> u32 {
        self.player_war_times_ms
            .get(&player_id)
            .copied()
            .unwrap_or(0)
            / 1_000
    }

    /// Exact hash-map erase из `ClearOnePlayerWarTime`.
    pub(crate) fn clear_one_player_war_time(&mut self, player_id: i32) -> Option<u32> {
        self.player_war_times_ms.remove(&player_id)
    }

    pub(crate) fn clear_player_war_times(&mut self) {
        self.player_war_times_ms.clear();
    }

    pub(crate) fn on_sign_up_war_start<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> bool {
        let Some(setup) = self.setups.get_mut(war_number as usize) else {
            return false;
        };
        setup.region_state = 1;
        if let Some(region) = context.find_region_then_proxy(setup.region_id) {
            context.on_war_declare(region, war_number);
        }
        self.clear_player_war_times();
        self.morale = 0;
        true
    }

    pub(crate) fn on_sign_up_war_end<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        sign_up_counts: [i32; 5],
        context: &mut Context,
    ) -> bool {
        let Some(setup) = self.setups.get(war_number as usize) else {
            return false;
        };
        if let Some(region) = context.find_server_nation_region(setup.region_id) {
            context.on_nation_war_declare(region, war_number, sign_up_counts);
        }
        true
    }

    pub(crate) fn on_enter_start<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> bool {
        let Some(setup) = self.setups.get_mut(war_number as usize) else {
            return false;
        };
        setup.region_state = 2;
        if let Some(region) = context.find_region_then_proxy(setup.region_id) {
            context.on_war_mass(region, war_number);
        }
        true
    }

    pub(crate) fn on_war_start<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> bool {
        let Some(setup) = self.setups.get_mut(war_number as usize) else {
            return false;
        };
        setup.region_state = 3;
        if let Some(region) = context.find_region_then_proxy(setup.region_id) {
            context.on_war_start(region, war_number);
        }
        true
    }

    pub(crate) fn on_refresh_region<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> bool {
        let Some(setup) = self.setups.get(war_number as usize) else {
            return false;
        };
        if let Some(region) = context.find_server_region(setup.region_id) {
            context.on_refresh_region(region, war_number);
        }
        self.clear_player_war_times();
        true
    }

    pub(crate) fn on_war_end<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> bool {
        context.add_war_end_log(war_number);
        let Some(setup) = self.setups.get(war_number as usize) else {
            return false;
        };
        if let Some(region) = context.find_region_then_proxy(setup.region_id) {
            context.on_war_end(region, war_number);
        }
        true
    }

    pub(crate) fn on_clear_war<Context: FourNationPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) -> bool {
        let Some(setup) = self.setups.get(war_number as usize) else {
            return false;
        };
        if let Some(region) = context.find_server_nation_region(setup.region_id) {
            context.on_clear_war(region, war_number);
        }
        true
    }

    pub(crate) fn take_war_results<Context: FourNationPhaseContext>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) -> Option<[u32; 5]> {
        let setup = self.setups.get(war_number as usize)?;
        let region = context.find_server_nation_region(setup.region_id)?;
        Some(context.take_war_results(region))
    }
}

fn decode_four_nation_setup(bytes: &[u8]) -> FourNationGameSetup {
    let mut offset = 8;
    let (sign_up_start_event_id, sign_up_start_time) = decode_four_nation_event(bytes, &mut offset);
    let (sign_up_end_event_id, sign_up_end_time) = decode_four_nation_event(bytes, &mut offset);
    let (start_event_id, start_time) = decode_four_nation_event(bytes, &mut offset);
    let (end_event_id, end_time) = decode_four_nation_event(bytes, &mut offset);
    let (end_info_event_id, end_info_time) = decode_four_nation_event(bytes, &mut offset);
    let (enter_start_event_id, enter_start_time) = decode_four_nation_event(bytes, &mut offset);
    let (enter_end_event_id, enter_end_time) = decode_four_nation_event(bytes, &mut offset);
    let (refresh_event_id, refresh_region_time) = decode_four_nation_event(bytes, &mut offset);
    let (clear_war_event_id, clear_war_time) = decode_four_nation_event(bytes, &mut offset);
    debug_assert_eq!(offset, 0xbc);
    FourNationGameSetup {
        time_index: four_nation_i32_at(bytes, 0),
        region_id: four_nation_i32_at(bytes, 4),
        sign_up_start_event_id,
        sign_up_start_time,
        sign_up_end_event_id,
        sign_up_end_time,
        start_event_id,
        start_time,
        end_event_id,
        end_time,
        end_info_event_id,
        end_info_time,
        enter_start_event_id,
        enter_start_time,
        enter_end_event_id,
        enter_end_time,
        refresh_event_id,
        refresh_region_time,
        clear_war_event_id,
        clear_war_time,
        region_state: four_nation_i32_at(bytes, 0xbc),
        is_every_week: four_nation_i32_at(bytes, 0xc0),
    }
}

fn decode_four_nation_event(bytes: &[u8], offset: &mut usize) -> (u32, TagTime) {
    let event_id = four_nation_u32_at(bytes, *offset);
    let time = decode_four_nation_time(&bytes[*offset + 4..*offset + 0x14]);
    *offset += 0x14;
    (event_id, time)
}

fn decode_four_nation_time(bytes: &[u8]) -> TagTime {
    let mut fields = [0u16; 8];
    for (index, field) in fields.iter_mut().enumerate() {
        let offset = index * 2;
        *field = LegacyReader::at(bytes, offset)
            .and_then(|mut reader| reader.read_u16())
            .expect("проверен TagTime");
    }
    TagTime::from_fields(fields)
}

fn read_four_nation_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, FourNationGameDecodeError> {
    let mut reader = four_nation_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| four_nation_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn take_four_nation_bytes<'source>(
    source: &'source [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'source [u8], FourNationGameDecodeError> {
    let mut reader = four_nation_reader(source, *cursor, field, required)?;
    let bytes = reader
        .read_bytes(required)
        .map_err(|block| four_nation_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn four_nation_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    required: usize,
) -> Result<LegacyReader<'source>, FourNationGameDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| FourNationGameDecodeError::UnexpectedEnd { field, offset: block.offset, required, available: block.available })
}

fn four_nation_error(field: &'static str, block: nebokrai_shared::protocol::LegacyReadBlock) -> FourNationGameDecodeError {
    FourNationGameDecodeError::UnexpectedEnd { field, offset: block.offset, required: block.needed, available: block.available }
}

fn four_nation_i32_at(bytes: &[u8], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("проверено поле FourNationWar")
}

fn four_nation_u32_at(bytes: &[u8], offset: usize) -> u32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("проверено поле FourNationWar")
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp

// ============================================================================
// FUNCTION: tagVilWarSetup::~tagVilWarSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005DA80
// ADDRESS: 0045da80
// PROTOTYPE: void __thiscall ~tagVilWarSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagVilWarSetup::tagVilWarSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005DB30
// ADDRESS: 0045db30
// PROTOTYPE: undefined __thiscall tagVilWarSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045e125
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005E125
// ADDRESS: 0045e125
// PROTOTYPE: undefined Catch@0045e125()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045e2f9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005E2F9
// ADDRESS: 0045e2f9
// PROTOTYPE: undefined Catch@0045e2f9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagVilWarSetup::tagVilWarSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005E990
// ADDRESS: 0045e990
// PROTOTYPE: undefined __thiscall tagVilWarSetup(tagVilWarSetup * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagVilWarSetup::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005EAE0
// ADDRESS: 0045eae0
// PROTOTYPE: tagVilWarSetup * __thiscall operator=(tagVilWarSetup * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045eddf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp
// RVA: 0x0005EDDF
// ADDRESS: 0045eddf
// PROTOTYPE: undefined Catch@0045eddf()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::GetWarRegionIDByTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.h:60
// RVA: 0x001BDFB0
// ADDRESS: 005bdfb0
// PROTOTYPE: long __thiscall GetWarRegionIDByTime(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnSignUpWarEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:78
// RVA: 0x001BEC80
// ADDRESS: 005bec80
// PROTOTYPE: void __thiscall OnSignUpWarEnd(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnEnterStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:100
// RVA: 0x001BEDC0
// ADDRESS: 005bedc0
// PROTOTYPE: void __thiscall OnEnterStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnWarStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:131
// RVA: 0x001BEE40
// ADDRESS: 005bee40
// PROTOTYPE: void __thiscall OnWarStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnWarEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:148
// RVA: 0x001BEEC0
// ADDRESS: 005beec0
// PROTOTYPE: void __thiscall OnWarEnd(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnClearWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:164
// RVA: 0x001BEF40
// ADDRESS: 005bef40
// PROTOTYPE: void __thiscall OnClearWar(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::SendReultToWS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:183
// RVA: 0x001BEFD0
// ADDRESS: 005befd0
// PROTOTYPE: void __thiscall SendReultToWS(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnSignUpWarStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:56
// RVA: 0x001BF600
// ADDRESS: 005bf600
// PROTOTYPE: void __thiscall OnSignUpWarStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnRefreshRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\fournationwarsys.cpp:116
// RVA: 0x001BF6A0
// ADDRESS: 005bf6a0
// PROTOTYPE: void __thiscall OnRefreshRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
