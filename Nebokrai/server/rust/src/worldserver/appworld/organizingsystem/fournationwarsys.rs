//! Система войны четырёх стран исторического WorldServer.
//!
//! Статус `CFourNationWarSys::AddToByteArray` RVA `0x00094250`,
//! `RecvResultFromGS` RVA `0x00093C60` и `ConvertMoraleToExploit` RVA
//! `0x00093F80`: `IMPLEMENTED`; loader, timers и
//! остальной Game runtime ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные owner-ы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:721`
//! и соседний `fournationwarsys.h`.
//!
//! Exact World serializer и Game decoder подтверждают wire: signed setup count,
//! insertion-order 196-byte setup records, затем signed rect count `5` и пять
//! 16-byte `tagRECT`. Setup record — два `i32`, девять пар `u32 event_id +
//! tagTime`, затем `i32 region_state + i32 is_every_week`; `tagTime` содержит
//! восемь последовательных `u16`. Старый Linux C++ использован только для имён
//! полей; размеры и порядок подтверждены поздними EXE/PDB и обоими концами
//! wire. Rust пишет поля явно little-endian вместо копирования ABI-memory:
//! padding/host ABI устранены, но все 196 наблюдаемых bytes сохраняются.
//! `Vec` заменяет `std::vector`, фиксированный массив — process-global RECT[5].
//! Невозможный signed count блокирует append до изменения destination. Точная
//! parser/timer-семантика `FourNationWarSys.ini` остаётся отдельным проходом.
//!
//! Exact `0x00493C60..0x00493D3E` последовательно читает ровно пять signed
//! `long` в static `s_lMorale[5]`. После slots `1..=4`, до чтения следующего,
//! выбираются fixed regions `11000/12000/13000/14000`; только route `1..=4`
//! получает `0x7FE49 { morale:i32 }`. Slot `0` сохраняется, но не публикуется.
//! Source map/socket, request correlation, connected-state и send-result не
//! проверяются. Linux-донор добавлял request tracker, socket ownership и
//! fail-closed публикацию; это новая политика, а не контракт EXE. Rust хранит
//! morale в instance-owner-е вместо process-global массива, сохраняя порядок
//! mutation/read/send и не копируя static storage.
//!
//! Exact `0x00493F80..0x004941B1` сначала ищет map-player. Для найденного
//! игрока он либо отправляет `0x7FE46 { player_id:i32, increment:i32 }` его
//! online GameServer-у, либо выполняет unsigned wrapping-прибавление к
//! `dwExploit`. Для отсутствующего игрока owner один раз обновляет
//! `CSL_PLAYER_ABILITY`, затем повторяет тот же поиск и маршрут. Exact
//! dispatcher `0x004A5096..0x004A50B4` читает два signed `long` в порядке
//! player/increment. Source metadata, полный tail, connected-state и send
//! result не проверяются. Linux-донор добавлял saturation, route ownership,
//! очередь, coalescing и retry; они не переносятся как неоригинальная
//! инфраструктура. SQL выполняется параметризованным `tiberius`-запросом в
//! dispatcher-owner-е; COM exception plumbing заменён явным Rust-исходом без
//! повторного player-поиска после ошибки выполнения.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::TagTime;
use crate::worldserver::appworld::player::PlayerExploitUpdate;

const FOUR_NATION_SETUP_WIRE_SIZE: usize = 196;
const FOUR_NATION_RECT_COUNT: i32 = 5;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FourNationWarSetup {
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
    setups: Vec<FourNationWarSetup>,
    rects: [FourNationRect; FOUR_NATION_RECT_COUNT as usize],
    morale: [i32; FOUR_NATION_RECT_COUNT as usize],
}

pub(crate) trait FourNationWarResultContext {
    fn game_server_number_by_region_id(&mut self, region_id: i32) -> i32;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
}

pub(crate) trait FourNationExploitContext {
    fn map_player_exists(&mut self, player_id: u32) -> bool;
    fn player_game_server_map_id(&mut self, player_id: i32) -> Option<i32>;
    fn add_local_player_exploit(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate>;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FourNationExploitLoadedDisposition {
    PlayerMissing,
    PlayerDisappearedBeforeLocalUpdate,
    LocalUpdated(PlayerExploitUpdate),
    Forwarded {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FourNationExploitLoadedReport {
    pub(crate) player_id: i32,
    pub(crate) increment: i32,
    pub(crate) disposition: FourNationExploitLoadedDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FourNationMoralePublicationDisposition {
    RouteRejected,
    Sent {
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FourNationMoralePublication {
    pub(crate) country: u8,
    pub(crate) region_id: i32,
    pub(crate) map_id: i32,
    pub(crate) disposition: FourNationMoralePublicationDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FourNationWarResultReport {
    pub(crate) previous_morale: [i32; FOUR_NATION_RECT_COUNT as usize],
    pub(crate) applied_morale: [i32; FOUR_NATION_RECT_COUNT as usize],
    pub(crate) values_complete: [bool; FOUR_NATION_RECT_COUNT as usize],
    pub(crate) publications: Vec<FourNationMoralePublication>,
}

impl CFourNationWarSys {
    pub(crate) fn push_setup(&mut self, setup: FourNationWarSetup) {
        self.setups.push(setup);
    }

    pub(crate) fn setups(&self) -> &[FourNationWarSetup] {
        &self.setups
    }

    pub(crate) fn rects(&self) -> &[FourNationRect; FOUR_NATION_RECT_COUNT as usize] {
        &self.rects
    }

    pub(crate) const fn morale(&self) -> &[i32; FOUR_NATION_RECT_COUNT as usize] {
        &self.morale
    }

    pub(crate) fn set_rect(&mut self, country: usize, rect: FourNationRect) -> bool {
        let Some(slot) = self.rects.get_mut(country) else {
            return false;
        };
        *slot = rect;
        true
    }

    /// Выполняет загруженную половину exact `ConvertMoraleToExploit`.
    ///
    /// Offline SQL остаётся у async dispatcher-а, который после успешной
    /// попытки вызывает этот метод повторно, как машинный owner.
    pub(crate) fn convert_loaded_morale_to_exploit<
        Context: FourNationExploitContext + ?Sized,
    >(
        &mut self,
        player_id: i32,
        increment: i32,
        context: &mut Context,
    ) -> FourNationExploitLoadedReport {
        let disposition = if !context.map_player_exists(player_id as u32) {
            FourNationExploitLoadedDisposition::PlayerMissing
        } else if let Some(map_id) = context.player_game_server_map_id(player_id) {
            let mut forward = CMessage::new(0x7fe46);
            forward.base_mut().add_long(player_id);
            forward.base_mut().add_long(increment);
            let wire = forward.as_wire_bytes().to_vec();
            let delivery = context.send_to_map_id(&forward, map_id);
            FourNationExploitLoadedDisposition::Forwarded {
                map_id,
                wire,
                delivery,
            }
        } else {
            match context.add_local_player_exploit(player_id as u32, increment) {
                Some(update) => FourNationExploitLoadedDisposition::LocalUpdated(update),
                None => FourNationExploitLoadedDisposition::PlayerDisappearedBeforeLocalUpdate,
            }
        };

        FourNationExploitLoadedReport {
            player_id,
            increment,
            disposition,
        }
    }

    /// Повторяет exact static `RecvResultFromGS`, включая interleaving чтения
    /// следующего slot-а только после публикации предыдущей страны.
    pub(crate) fn receive_result_from_game_server<
        Context: FourNationWarResultContext + ?Sized,
    >(
        &mut self,
        message: &mut CMessage,
        context: &mut Context,
    ) -> FourNationWarResultReport {
        const COUNTRY_REGION_IDS: [i32; 4] = [11_000, 12_000, 13_000, 14_000];

        let previous_morale = self.morale;
        let mut values_complete = [false; FOUR_NATION_RECT_COUNT as usize];
        let mut publications = Vec::with_capacity(4);

        for index in 0..FOUR_NATION_RECT_COUNT as usize {
            self.morale[index] = 0;
            let decoded = message.base_mut().get_long();
            self.morale[index] = decoded.unwrap_or(0);
            values_complete[index] = decoded.is_some();

            let Some(&region_id) = index
                .checked_sub(1)
                .and_then(|country_index| COUNTRY_REGION_IDS.get(country_index))
            else {
                continue;
            };
            let map_id = context.game_server_number_by_region_id(region_id);
            let disposition = if (1..5).contains(&map_id) {
                let mut publication = CMessage::new(0x7fe49);
                publication.base_mut().add_long(self.morale[index]);
                let wire = publication.as_wire_bytes().to_vec();
                let delivery = context.send_to_map_id(&publication, map_id);
                FourNationMoralePublicationDisposition::Sent { wire, delivery }
            } else {
                FourNationMoralePublicationDisposition::RouteRejected
            };
            publications.push(FourNationMoralePublication {
                country: index as u8,
                region_id,
                map_id,
                disposition,
            });
        }

        FourNationWarResultReport {
            previous_morale,
            applied_morale: self.morale,
            values_complete,
            publications,
        }
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), FourNationWarSerializationBlock> {
        let count = i32::try_from(self.setups.len()).map_err(|_| {
            FourNationWarSerializationBlock::SetupCountOutOfRange {
                count: self.setups.len(),
            }
        })?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&count.to_le_bytes());
        for setup in &self.setups {
            let record_start = payload.len();
            write_four_nation_setup(&mut payload, setup);
            debug_assert_eq!(payload.len() - record_start, FOUR_NATION_SETUP_WIRE_SIZE);
        }
        payload.extend_from_slice(&FOUR_NATION_RECT_COUNT.to_le_bytes());
        for rect in &self.rects {
            for value in [rect.left, rect.top, rect.right, rect.bottom] {
                payload.extend_from_slice(&value.to_le_bytes());
            }
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationWarSerializationBlock {
    SetupCountOutOfRange { count: usize },
}

impl fmt::Display for FourNationWarSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetupCountOutOfRange { count } => write!(
                formatter,
                "CFourNationWarSys содержит {count} setup-записей вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for FourNationWarSerializationBlock {}

fn write_four_nation_setup(destination: &mut Vec<u8>, setup: &FourNationWarSetup) {
    destination.extend_from_slice(&setup.time_index.to_le_bytes());
    destination.extend_from_slice(&setup.region_id.to_le_bytes());
    for (event_id, time) in [
        (setup.sign_up_start_event_id, setup.sign_up_start_time),
        (setup.sign_up_end_event_id, setup.sign_up_end_time),
        (setup.start_event_id, setup.start_time),
        (setup.end_event_id, setup.end_time),
        (setup.end_info_event_id, setup.end_info_time),
        (setup.enter_start_event_id, setup.enter_start_time),
        (setup.enter_end_event_id, setup.enter_end_time),
        (setup.refresh_event_id, setup.refresh_region_time),
        (setup.clear_war_event_id, setup.clear_war_time),
    ] {
        destination.extend_from_slice(&event_id.to_le_bytes());
        write_tag_time(destination, time);
    }
    destination.extend_from_slice(&setup.region_state.to_le_bytes());
    destination.extend_from_slice(&setup.is_every_week.to_le_bytes());
}

fn write_tag_time(destination: &mut Vec<u8>, time: TagTime) {
    for value in [
        time.year,
        time.month,
        time.day_of_week,
        time.day,
        time.hour,
        time.minute,
        time.second,
        time.milliseconds,
    ] {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.h

// ============================================================================
// FUNCTION: CFourNationWarSys::OnRefreshRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:517
// RVA: 0x00093B10
// ADDRESS: 00493b10
// PROTOTYPE: void __stdcall OnRefreshRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnClearWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:646
// RVA: 0x00093B80
// ADDRESS: 00493b80
// PROTOTYPE: void __stdcall OnClearWar(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::RequestWarResultFromGS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:669
// RVA: 0x00093BF0
// ADDRESS: 00493bf0
// PROTOTYPE: void __cdecl RequestWarResultFromGS(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::RecvResultFromGS
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:676
// RVA: 0x00093C60
// ADDRESS: 00493c60
// PROTOTYPE: void __cdecl RecvResultFromGS(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::SendPlayerWarTimeToGS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:839
// RVA: 0x00093D50
// ADDRESS: 00493d50
// PROTOTYPE: void __thiscall SendPlayerWarTimeToGS(long param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.h:65
// RVA: 0x00093EA0
// ADDRESS: 00493ea0
// PROTOTYPE: CFourNationWarSys * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFourNationWarSys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:889
// RVA: 0x00093EC0
// ADDRESS: 00493ec0
// PROTOTYPE: CFourNationWarSys * __cdecl GetFourNationWarSys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::ConvertMoraleToExploit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:786
// RVA: 0x00093F80
// ADDRESS: 00493f80
// PROTOTYPE: void __thiscall ConvertMoraleToExploit(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::GetWarRegionIDByTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.h:71
// RVA: 0x00094200
// ADDRESS: 00494200
// PROTOTYPE: long __thiscall GetWarRegionIDByTime(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:721
// RVA: 0x00094250
// ADDRESS: 00494250
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OneCountrySignUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:654
// RVA: 0x00094340
// ADDRESS: 00494340
// PROTOTYPE: void __cdecl OneCountrySignUp(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OneCountryFail
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:869
// RVA: 0x00094440
// ADDRESS: 00494440
// PROTOTYPE: void __cdecl OneCountryFail(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnWarStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:333
// RVA: 0x00094CC0
// ADDRESS: 00494cc0
// PROTOTYPE: void __stdcall OnWarStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnSignUpWarStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:363
// RVA: 0x00094F10
// ADDRESS: 00494f10
// PROTOTYPE: void __stdcall OnSignUpWarStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnSignUpWarEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:416
// RVA: 0x00095370
// ADDRESS: 00495370
// PROTOTYPE: void __stdcall OnSignUpWarEnd(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnEnterStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:463
// RVA: 0x00095610
// ADDRESS: 00495610
// PROTOTYPE: void __stdcall OnEnterStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnEnterEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:491
// RVA: 0x000958F0
// ADDRESS: 004958f0
// PROTOTYPE: void __stdcall OnEnterEnd(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnWarEndInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:525
// RVA: 0x00095B80
// ADDRESS: 00495b80
// PROTOTYPE: void __stdcall OnWarEndInfo(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnWarEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:565
// RVA: 0x00095E90
// ADDRESS: 00495e90
// PROTOTYPE: void __stdcall OnWarEnd(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::Initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:19
// RVA: 0x000963E0
// ADDRESS: 004963e0
// PROTOTYPE: bool __cdecl Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::ReLoad
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:304
// RVA: 0x00097370
// ADDRESS: 00497370
// PROTOTYPE: bool __cdecl ReLoad(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: COrganizing::`scalar_deleting_destructor'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp
// RVA: 0x000DFCB0
// ADDRESS: 004dfcb0
// PROTOTYPE: void * __thiscall `scalar_deleting_destructor'(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
