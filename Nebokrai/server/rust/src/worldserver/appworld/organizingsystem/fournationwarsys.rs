//! Система войны четырёх стран исторического WorldServer.
//!
//! Статус `CFourNationWarSys::AddToByteArray` RVA `0x00094250`,
//! `RecvResultFromGS` RVA `0x00093C60` и `ConvertMoraleToExploit` RVA
//! `0x00093F80`, `OnRefreshRegion`/`OnClearWar`/`RequestWarResultFromGS` RVA
//! `0x00093B10/0x00093B80/0x00093BF0`, `GetWarRegionIDByTime` RVA
//! `0x00094200`: `IMPLEMENTED`; loader, timers и
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
//! `Initialize` RVA `0x000963E0` восстанавливает country-name snapshot,
//! очищает setup/fund registries, читает `#` weekly setup и `*` fund records,
//! ставит достижимые calendar events и загружает `regions/<first>.nation`.
//! Rust хранит timer IDs как ноль до регистрации: старые неинициализированные
//! поля не имеют downstream-семантики и не переносятся.
//! Проверка country-index nation-файла заменяет доказанный выход за `RECT[5]`
//! безопасным пропуском без нового внешнего log: исходная ветка ошибки была
//! внутренне противоречивой (`index < 0 && index > 4`) и не задавала
//! совместимого результата для повреждённого ресурса.
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
//!
//! Exact static `OneCountrySignUp` `0x00494340..0x00494431` принимает только
//! countries `1..=4`, получает `XBWS0035` и форматирует локальный `char[256]`,
//! но никуда не передаёт результат и не меняет состояние. Rust сохраняет
//! внешний no-op и country gate, удаляя только неиспользуемые allocation,
//! string lookup и `_snprintf` как внутреннюю мёртвую работу.
//!
//! Exact `SendPlayerWarTimeToGS` `0x00493D50..0x00493E19` отображает country
//! `1..=4` в regions `11000..14000`, принимает только map route `1..=4` и
//! отправляет `0x7FE47 { player_id:i32, war_time:u32 }`. Source/socket,
//! connected-state и send-result не влияют на ветвление.
//!
//! Exact `OneCountryFail` `0x00494440..0x00494600` выбирает `XBWS0036`, когда
//! обе страны равны, иначе `XBWS0037`; аргументы — копии country-name slots
//! `m_CountryName[5][10]`. Затем он безусловно вызывает уже подтверждённый
//! `COrganizingCtrl::SendTopInfoToClient(-1, 1, 2, text)`, то есть реальный
//! broadcast wire — `0x7FA04`, а не донорский `0x7FE48`. Старые `strcpy` в
//! десятибайтовые slots и последующий `strlen` после возможного `_snprintf`
//! overflow были внутренним UB: Rust безопасно ограничивает имя девятью,
//! notice — 255 байтами, сохраняя нормальный C-string wire.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::{TagTime, TagTimeArithmeticBlock};
use crate::public::readwrite::read_to;
use crate::public::timer::CTimer;
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct FourNationWarFund {
    pub(crate) morale: i32,
    pub(crate) money: i32,
}

/// Все девять точных calendar callback-ов `CFourNationWarSys`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FourNationWarCallbacks<Callback> {
    pub(crate) sign_up_start: Callback,
    pub(crate) sign_up_end: Callback,
    pub(crate) war_start: Callback,
    pub(crate) war_end: Callback,
    pub(crate) war_end_info: Callback,
    pub(crate) enter_start: Callback,
    pub(crate) enter_end: Callback,
    pub(crate) refresh_region: Callback,
    pub(crate) clear_war: Callback,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct FourNationWarLoadReport {
    pub(crate) setup_resource_found: bool,
    pub(crate) nation_resource_found: bool,
    pub(crate) setup_rows_read: u32,
    pub(crate) setup_rows_accepted: u32,
    pub(crate) setup_rows_ignored_order: u32,
    pub(crate) funds_loaded: u32,
    pub(crate) rects_loaded: u32,
    pub(crate) rects_ignored_country: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationWarLoadError {
    SetupResourceMissing,
    NoAcceptedSetups,
    NationResourceMissing { region_id: i32 },
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
    Arithmetic(TagTimeArithmeticBlock),
    ScheduleIndexOverflow,
}

/// Безопасная граница старого индексирования `s_vSetup` без bounds-check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FourNationWarRegionIndexBlock {
    pub(crate) index: i32,
    pub(crate) setup_count: usize,
}

/// Внешние owner-ы достигнутого `OnWarStart`.
pub(crate) trait FourNationWarStartContext {
    fn send_all(&mut self, message: &CMessage);
    fn region_name(&mut self, region_id: i32) -> Option<Vec<u8>>;
    fn war_start_notice(&mut self, region_name: &[u8]) -> Vec<u8>;
    fn war_start_log(&mut self, index: i32, region_name: &[u8]) -> Vec<u8>;
    fn send_organizing_info(&mut self, text: &[u8], color: u32, trailing: u32);
    fn put_war_log(&mut self, text: &[u8]);
}

impl From<TagTimeArithmeticBlock> for FourNationWarLoadError {
    fn from(value: TagTimeArithmeticBlock) -> Self {
        Self::Arithmetic(value)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CFourNationWarSys {
    setups: Vec<FourNationWarSetup>,
    funds: Vec<FourNationWarFund>,
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

pub(crate) trait FourNationCountryFailContext {
    fn country_name(&mut self, country: i32) -> Vec<u8>;
    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8>;
    fn send_top_info(&mut self, text: &[u8]) -> Result<i32, SendMessageError>;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FourNationSignUpDisposition {
    CountryIgnored,
    ValidCountryNoExternalEffect,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FourNationWarTimeDisposition {
    CountryIgnored,
    RouteRejected { region_id: i32, map_id: i32 },
    Sent {
        region_id: i32,
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FourNationWarTimeReport {
    pub(crate) player_id: i32,
    pub(crate) war_time: u32,
    pub(crate) country: i32,
    pub(crate) disposition: FourNationWarTimeDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FourNationCountryFailReport {
    pub(crate) country: i32,
    pub(crate) failed_country: i32,
    pub(crate) string_id: &'static [u8],
    pub(crate) country_name: Vec<u8>,
    pub(crate) failed_country_name: Option<Vec<u8>>,
    pub(crate) text: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
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

/// Общая безопасная materialization трёх static World callback-ов.
fn send_four_nation_broadcast<SendAll, Delivery>(
    opcode: i32,
    region_id: i32,
    send_all: &mut SendAll,
) -> Delivery
where
    SendAll: FnMut(&CMessage) -> Delivery,
{
    let mut message = CMessage::new(opcode);
    message.base_mut().add_long(region_id);
    send_all(&message)
}

impl CFourNationWarSys {
    /// Broadcast `OnRefreshRegion`: exact `0x7FE41 { region_id:i32 }`.
    pub(crate) fn on_refresh_region<SendAll, Delivery>(
        region_id: i32,
        send_all: &mut SendAll,
    ) -> Delivery
    where
        SendAll: FnMut(&CMessage) -> Delivery,
    {
        send_four_nation_broadcast(0x7FE41, region_id, send_all)
    }

    /// Broadcast `OnClearWar`: exact `0x7FE44 { region_id:i32 }`.
    pub(crate) fn on_clear_war<SendAll, Delivery>(
        region_id: i32,
        send_all: &mut SendAll,
    ) -> Delivery
    where
        SendAll: FnMut(&CMessage) -> Delivery,
    {
        send_four_nation_broadcast(0x7FE44, region_id, send_all)
    }

    /// Broadcast `RequestWarResultFromGS`: exact `0x7FE45 { region_id:i32 }`.
    pub(crate) fn request_war_result_from_gs<SendAll, Delivery>(
        region_id: i32,
        send_all: &mut SendAll,
    ) -> Delivery
    where
        SendAll: FnMut(&CMessage) -> Delivery,
    {
        send_four_nation_broadcast(0x7FE45, region_id, send_all)
    }

    /// Восстанавливает полный exact loader `FourNationWarSys.ini`.
    ///
    /// Source `regions/<first region>.nation` запрашивается лишь после
    /// постановки timers всех принятых setup-записей, как в EXE. Поэтому его
    /// отсутствие завершает bool-owner с уже зарегистрированным prefix side
    /// effect.
    pub(crate) fn initialize<Callback: Copy, NationSource>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: FourNationWarCallbacks<Callback>,
        mut nation_source: NationSource,
    ) -> Result<FourNationWarLoadReport, FourNationWarLoadError>
    where
        NationSource: FnMut(i32) -> Option<Vec<u8>>,
    {
        self.setups.clear();
        self.funds.clear();
        self.rects = [FourNationRect::default(); FOUR_NATION_RECT_COUNT as usize];
        let Some(source) = source else {
            return Err(FourNationWarLoadError::SetupResourceMissing);
        };

        let mut report = FourNationWarLoadReport {
            setup_resource_found: true,
            ..FourNationWarLoadReport::default()
        };
        let mut setup_tokens = war_tokens(source);
        let mut row_index = 0i32;
        while read_to(&mut setup_tokens, b"#") {
            row_index = row_index
                .checked_add(1)
                .ok_or(FourNationWarLoadError::ScheduleIndexOverflow)?;
            report.setup_rows_read = report.setup_rows_read.wrapping_add(1);
            let setup = read_weekly_setup(&mut setup_tokens, row_index, now)?;
            if setup.has_valid_time_order() {
                self.setups.push(setup);
                report.setup_rows_accepted = report.setup_rows_accepted.wrapping_add(1);
            } else {
                report.setup_rows_ignored_order = report.setup_rows_ignored_order.wrapping_add(1);
            }
        }

        let mut fund_tokens = war_tokens(source);
        while read_to(&mut fund_tokens, b"*") {
            let fund = FourNationWarFund {
                morale: next_war_i32(&mut fund_tokens, "lMorale")?,
                money: next_war_i32(&mut fund_tokens, "lMoney")?,
            };
            self.funds.push(fund);
            report.funds_loaded = report.funds_loaded.wrapping_add(1);
        }

        let first_region_id = self
            .setups
            .first()
            .map(|setup| setup.region_id)
            .ok_or(FourNationWarLoadError::NoAcceptedSetups)?;
        for (index, setup) in self.setups.iter_mut().enumerate() {
            setup.register_initial_events(index as i32, now, timer, callbacks);
        }

        let Some(nation_source) = nation_source(first_region_id) else {
            return Err(FourNationWarLoadError::NationResourceMissing {
                region_id: first_region_id,
            });
        };
        report.nation_resource_found = true;
        let mut nation_tokens = war_tokens(&nation_source);
        while read_to(&mut nation_tokens, b"#") {
            let country = next_war_i32(&mut nation_tokens, "country")?;
            let rect = FourNationRect {
                left: next_war_i32(&mut nation_tokens, "rect.left")?,
                top: next_war_i32(&mut nation_tokens, "rect.top")?,
                right: next_war_i32(&mut nation_tokens, "rect.right")?,
                bottom: next_war_i32(&mut nation_tokens, "rect.bottom")?,
            };
            if let Some(slot) = self.rects.get_mut(country as usize) {
                *slot = rect;
                report.rects_loaded = report.rects_loaded.wrapping_add(1);
            } else {
                report.rects_ignored_country = report.rects_ignored_country.wrapping_add(1);
            }
        }
        Ok(report)
    }

    pub(crate) fn funds(&self) -> &[FourNationWarFund] {
        &self.funds
    }

    /// Возвращает region setup-а по машинному индексу `GetWarRegionIDByTime`.
    ///
    /// Пустой vector возвращает `0`. При существующем vector raw разыменовывал
    /// index без проверки; invalid index остаётся typed safe-границей, а не
    /// превращается молча в иной region ID.
    pub(crate) fn war_region_id_by_time(
        &self,
        index: i32,
    ) -> Result<i32, FourNationWarRegionIndexBlock> {
        if self.setups.is_empty() {
            return Ok(0);
        }
        let setup_count = self.setups.len();
        let index = usize::try_from(index).map_err(|_| FourNationWarRegionIndexBlock {
            index,
            setup_count,
        })?;
        self.setups
            .get(index)
            .map(|setup| setup.region_id)
            .ok_or(FourNationWarRegionIndexBlock {
                index: index as i32,
                setup_count,
            })
    }

    /// `OnWarStart`: state `CIS_Fight`, broadcast `0x7FE3C`, затем region
    /// notice `XBWS0022` и war-log `XBWS0023`.
    pub(crate) fn on_war_start<Context: FourNationWarStartContext + ?Sized>(
        &mut self,
        index: i32,
        context: &mut Context,
    ) -> Result<(), FourNationWarRegionIndexBlock> {
        let setup_count = self.setups.len();
        let setup = self.setups.get_mut(usize::try_from(index).map_err(|_| {
            FourNationWarRegionIndexBlock { index, setup_count }
        })?).ok_or(FourNationWarRegionIndexBlock { index, setup_count })?;
        setup.region_state = 1;
        let mut message = CMessage::new(0x7FE3C);
        message.base_mut().add_long(index);
        context.send_all(&message);
        let Some(region_name) = context.region_name(setup.region_id) else {
            return Ok(());
        };
        let notice = context.war_start_notice(&region_name);
        context.send_organizing_info(&notice, 0xFFFF_FE92, 0xFFFF_0000);
        let log = context.war_start_log(index, &region_name);
        context.put_war_log(&log);
        Ok(())
    }
    /// Сохраняет единственный внешний контракт exact `OneCountrySignUp`.
    pub(crate) const fn one_country_sign_up(country: i32) -> FourNationSignUpDisposition {
        if 0 < country && country < 5 {
            FourNationSignUpDisposition::ValidCountryNoExternalEffect
        } else {
            FourNationSignUpDisposition::CountryIgnored
        }
    }

    pub(crate) fn send_player_war_time_to_game_server<
        Context: FourNationWarResultContext + ?Sized,
    >(
        &mut self,
        player_id: i32,
        war_time: u32,
        country: i32,
        context: &mut Context,
    ) -> FourNationWarTimeReport {
        let region_id = match country {
            1 => 11_000,
            2 => 12_000,
            3 => 13_000,
            4 => 14_000,
            _ => {
                return FourNationWarTimeReport {
                    player_id,
                    war_time,
                    country,
                    disposition: FourNationWarTimeDisposition::CountryIgnored,
                };
            }
        };
        let map_id = context.game_server_number_by_region_id(region_id);
        let disposition = if (1..5).contains(&map_id) {
            let mut update = CMessage::new(0x7fe47);
            update.base_mut().add_long(player_id);
            update.base_mut().add_ulong(war_time);
            let wire = update.as_wire_bytes().to_vec();
            let delivery = context.send_to_map_id(&update, map_id);
            FourNationWarTimeDisposition::Sent {
                region_id,
                map_id,
                wire,
                delivery,
            }
        } else {
            FourNationWarTimeDisposition::RouteRejected { region_id, map_id }
        };
        FourNationWarTimeReport {
            player_id,
            war_time,
            country,
            disposition,
        }
    }

    pub(crate) fn one_country_fail<Context: FourNationCountryFailContext + ?Sized>(
        country: i32,
        failed_country: i32,
        context: &mut Context,
    ) -> FourNationCountryFailReport {
        let country_name = context.country_name(country);
        let (string_id, failed_country_name, formatted) = if country == failed_country {
            let string_id = b"XBWS0036";
            let formatted = context.format_world_string(string_id, &[&country_name]);
            (string_id.as_slice(), None, formatted)
        } else {
            let string_id = b"XBWS0037";
            let failed_name = context.country_name(failed_country);
            let formatted =
                context.format_world_string(string_id, &[&country_name, &failed_name]);
            (string_id.as_slice(), Some(failed_name), formatted)
        };
        let visible_length = formatted
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(formatted.len())
            .min(0xff);
        let text = formatted[..visible_length].to_vec();
        let delivery = context.send_top_info(&text);
        FourNationCountryFailReport {
            country,
            failed_country,
            string_id,
            country_name,
            failed_country_name,
            text,
            delivery,
        }
    }

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

impl FourNationWarSetup {
    fn has_valid_time_order(&self) -> bool {
        self.sign_up_start_time.legacy_lt(self.sign_up_end_time)
            && self.sign_up_end_time.legacy_lt(self.enter_start_time)
            && self.enter_start_time.legacy_lt(self.refresh_region_time)
            && self.refresh_region_time.legacy_lt(self.start_time)
            && self.start_time.legacy_lt(self.end_info_time)
            && self.end_info_time.legacy_lt(self.end_time)
            && self.end_time.legacy_lt(self.enter_end_time)
            && self.enter_end_time.legacy_lt(self.clear_war_time)
    }

    fn register_initial_events<Callback: Copy>(
        &mut self,
        index: i32,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: FourNationWarCallbacks<Callback>,
    ) {
        if self.end_time.legacy_lt(now) {
            return;
        }
        self.end_event_id = timer
            .set_time_event(self.end_time, callbacks.war_end, index)
            .get();
        self.clear_war_event_id = timer
            .set_time_event(self.clear_war_time, callbacks.clear_war, index)
            .get();
        self.end_info_event_id = timer
            .set_time_event(
                if self.end_info_time.legacy_ge(now) {
                    self.end_info_time
                } else {
                    now
                },
                callbacks.war_end_info,
                index,
            )
            .get();
        self.enter_end_event_id = timer
            .set_time_event(
                if self.enter_end_time.legacy_gt(now) {
                    self.enter_end_time
                } else {
                    now
                },
                callbacks.enter_end,
                index,
            )
            .get();
        if !self.start_time.legacy_ge(now) {
            self.region_state = 3;
            return;
        }
        self.start_event_id = timer
            .set_time_event(self.start_time, callbacks.war_start, index)
            .get();
        self.refresh_event_id = timer
            .set_time_event(self.refresh_region_time, callbacks.refresh_region, index)
            .get();
        if !self.enter_start_time.legacy_gt(now) {
            self.region_state = 2;
            return;
        }
        self.enter_start_event_id = timer
            .set_time_event(self.enter_start_time, callbacks.enter_start, index)
            .get();
        if self.sign_up_end_time.legacy_ge(now) {
            self.sign_up_end_event_id = timer
                .set_time_event(self.sign_up_end_time, callbacks.sign_up_end, index)
                .get();
            if self.sign_up_start_time.legacy_ge(now) {
                self.sign_up_start_event_id = timer
                    .set_time_event(self.sign_up_start_time, callbacks.sign_up_start, index)
                    .get();
            } else {
                self.region_state = 1;
            }
        } else {
            self.sign_up_end_event_id = timer
                .set_time_event(now, callbacks.sign_up_end, index)
                .get();
        }
    }
}

fn read_weekly_setup<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    time_index: i32,
    now: TagTime,
) -> Result<FourNationWarSetup, FourNationWarLoadError> {
    let region_id = next_war_i32(tokens, "lRegionID")?;
    let weekday = next_war_i32(tokens, "weekday")?;
    let hour = next_war_i32(tokens, "hour")? as u16;
    let minute = next_war_i32(tokens, "minute")? as u16;
    let second = next_war_i32(tokens, "second")? as u16;
    let sign_up_start_offset = next_war_i32(tokens, "SignUpWarStartTime offset")?;
    let sign_up_end_offset = next_war_i32(tokens, "SignUpWarEndTime offset")?;
    let enter_start_offset = next_war_i32(tokens, "EnterStartTime offset")?;
    let end_info_offset = next_war_i32(tokens, "EndInfoTime offset")?;
    let enter_end_offset = next_war_i32(tokens, "EnterEndTime offset")?;
    let refresh_offset = next_war_i32(tokens, "RefreshRegionTime offset")?;
    let end_offset = next_war_i32(tokens, "EndTime offset")?;
    let clear_offset = next_war_i32(tokens, "ClearWarTime offset")?;

    let mut start_time = now;
    let day_delta = weekday - i32::from(now.day_of_week);
    let _ = start_time.add_day(if day_delta < 0 { day_delta + 7 } else { day_delta })?;
    start_time.hour = hour;
    start_time.minute = minute;
    start_time.second = second;
    start_time.milliseconds = 0;
    if shifted_minutes(start_time, sign_up_start_offset)?.legacy_lt(now) {
        let _ = start_time.add_day(7)?;
    }

    Ok(FourNationWarSetup {
        time_index,
        region_id,
        sign_up_start_event_id: 0,
        sign_up_start_time: shifted_minutes(start_time, sign_up_start_offset)?,
        sign_up_end_event_id: 0,
        sign_up_end_time: shifted_minutes(start_time, sign_up_end_offset)?,
        start_event_id: 0,
        start_time,
        end_event_id: 0,
        end_time: shifted_minutes(start_time, end_offset)?,
        end_info_event_id: 0,
        end_info_time: shifted_minutes(start_time, end_info_offset)?,
        enter_start_event_id: 0,
        enter_start_time: shifted_minutes(start_time, enter_start_offset)?,
        enter_end_event_id: 0,
        enter_end_time: shifted_minutes(start_time, enter_end_offset)?,
        refresh_event_id: 0,
        refresh_region_time: shifted_minutes(start_time, refresh_offset)?,
        clear_war_event_id: 0,
        clear_war_time: shifted_minutes(start_time, clear_offset)?,
        region_state: 0,
        is_every_week: 1,
    })
}

fn shifted_minutes(mut time: TagTime, offset: i32) -> Result<TagTime, TagTimeArithmeticBlock> {
    let _ = time.add_minute(offset)?;
    Ok(time)
}

fn war_tokens(source: &[u8]) -> impl Iterator<Item = &[u8]> {
    source
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty())
}

fn next_war_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, FourNationWarLoadError> {
    let token = tokens
        .next()
        .ok_or(FourNationWarLoadError::MissingValue { field })?;
    std::str::from_utf8(token)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(FourNationWarLoadError::InvalidValue { field })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.h

// ============================================================================
// FUNCTION: CFourNationWarSys::OnRefreshRegion
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:517
// RVA: 0x00093B10
// ADDRESS: 00493b10
// PROTOTYPE: void __stdcall OnRefreshRegion(long param_1)
//
// IMPLEMENTED_OWNER: `CFourNationWarSys::on_refresh_region` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::OnClearWar
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:646
// RVA: 0x00093B80
// ADDRESS: 00493b80
// PROTOTYPE: void __stdcall OnClearWar(long param_1)
//
// IMPLEMENTED_OWNER: `CFourNationWarSys::on_clear_war` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFourNationWarSys::RequestWarResultFromGS
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:669
// RVA: 0x00093BF0
// ADDRESS: 00493bf0
// PROTOTYPE: void __cdecl RequestWarResultFromGS(long param_1)
//
// IMPLEMENTED_OWNER: `CFourNationWarSys::request_war_result_from_gs` выше.
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.h:71
// RVA: 0x00094200
// ADDRESS: 00494200
// PROTOTYPE: long __thiscall GetWarRegionIDByTime(long param_1)
//
// IMPLEMENTED_OWNER: `CFourNationWarSys::war_region_id_by_time` выше.
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\fournationwarsys.cpp:333
// RVA: 0x00094CC0
// ADDRESS: 00494cc0
// PROTOTYPE: void __stdcall OnWarStart(long param_1)
//
// IMPLEMENTED_OWNER: `CFourNationWarSys::on_war_start` выше.
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
