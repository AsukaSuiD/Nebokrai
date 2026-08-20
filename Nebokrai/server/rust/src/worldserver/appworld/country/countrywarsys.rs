//! WorldServer-владелец country-war victory state `CountryWarSys`.
//!
//! `on_flag_destory` (исходное PDB-написание) RVA `0x00091E00` имеет статус
//! `IMPLEMENTED`; остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходник
//! `appworld/country/countrywarsys.cpp`.
//!
//! PDB/static map хранит `CountryWarRegion` размером `0x0C`: clear-byte с
//! padding, signed defend и attack country. `BTreeMap` сохраняет map-order.
//! Для каждой записи с двумя ненулевыми сторонами callback сначала сбрасывает
//! clear-byte, затем ищет живой region и обе страны. При успехе он рассылает
//! `0x7FF22` с low byte входной страны; full signed equality выбирает defender
//! result `2/1` либо attacker result `1/2`. После optional `WS0105/WS0106`
//! форматирования стороны всегда обнуляются и country-info `0x7FA03` уходит
//! даже при пустом тексте. Потерянный Ghidra stack-key region lookup и порядок
//! state/message/result/clear подтверждены exact EXE `0x00491E69..0x00492142`.
//! Region, localization, country-map и network owners остаются явной context-
//! границей; неизвестность в них сохраняет уже выполненные предыдущие эффекты.

use std::collections::BTreeMap;

use crate::nets::networld::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarRegion {
    pub(crate) state_clear: bool,
    pub(crate) defend_country: i32,
    pub(crate) attack_country: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarVictoryRegion {
    pub(crate) name: Vec<u8>,
}

pub(crate) trait CountryWarVictoryContext {
    type Block;

    /// Повторяет `s_mapRegionList.find/operator[]` и non-null `pRegion` gate.
    fn region(&mut self, region_id: i32) -> Result<Option<CountryWarVictoryRegion>, Self::Block>;

    /// Повторяет отдельный `GetCountry(low byte)`; ID `0` даёт false.
    fn country_exists(&mut self, country: u8) -> Result<bool, Self::Block>;

    /// Синхронно повторяет `CMessage::SendAll`; старый return игнорировался.
    fn send_all(&mut self, message: &CMessage) -> i32;

    /// Пишет result уже доказанно существующей стране.
    fn set_country_war_result(&mut self, country: u8, result: i32) -> Result<(), Self::Block>;

    /// Повторяет `GetStringByID` и старую 256-byte `_sprintf` границу.
    fn format_victory_notice(
        &mut self,
        string_id: &'static [u8],
        attack_country: i32,
        defend_country: i32,
        region_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block>;

    /// Вызывает concrete `CCountryHandler::send_info_to_client` с исходными
    /// title `-366` и color `0xFFFF0000`.
    fn send_country_info(
        &mut self,
        text: &[u8],
        title: u32,
        color: u32,
    ) -> Result<i32, Self::Block>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarVictoryReport {
    pub(crate) active_regions: usize,
    pub(crate) flag_deliveries: Vec<i32>,
    pub(crate) result_pairs: usize,
    pub(crate) formatted_notices: usize,
    pub(crate) info_deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarSys {
    pub(crate) war_regions: BTreeMap<i32, CountryWarRegion>,
}

impl CountryWarSys {
    pub(crate) fn on_flag_destory<Context: CountryWarVictoryContext + ?Sized>(
        &mut self,
        country: i32,
        context: &mut Context,
    ) -> Result<CountryWarVictoryReport, Context::Block> {
        let mut report = CountryWarVictoryReport::default();

        for (&region_id, state) in &mut self.war_regions {
            if state.defend_country == 0 || state.attack_country == 0 {
                continue;
            }
            report.active_regions += 1;
            state.state_clear = false;

            let defend_country = state.defend_country;
            let attack_country = state.attack_country;
            let region = context.region(region_id)?;
            let mut victory_side = None;

            if region.is_some() {
                // Оригинал выполняет оба lookup независимо, затем общий gate.
                let defend_exists = context.country_exists(defend_country as u8)?;
                let attack_exists = context.country_exists(attack_country as u8)?;
                if defend_exists && attack_exists {
                    let mut message = CMessage::new(0x7ff22);
                    message.base_mut().add_byte(country as u8);
                    report.flag_deliveries.push(context.send_all(&message));

                    if defend_country == country {
                        context.set_country_war_result(defend_country as u8, 2)?;
                        context.set_country_war_result(attack_country as u8, 1)?;
                        victory_side = Some(false);
                        report.result_pairs += 1;
                    } else if attack_country == country {
                        context.set_country_war_result(defend_country as u8, 1)?;
                        context.set_country_war_result(attack_country as u8, 2)?;
                        victory_side = Some(true);
                        report.result_pairs += 1;
                    }
                }
            }

            let text = match (victory_side, region.as_ref()) {
                (Some(false), Some(region)) => {
                    report.formatted_notices += 1;
                    context.format_victory_notice(
                        b"WS0105",
                        attack_country,
                        defend_country,
                        &region.name,
                    )?
                }
                (Some(true), Some(region)) => {
                    report.formatted_notices += 1;
                    context.format_victory_notice(
                        b"WS0106",
                        attack_country,
                        defend_country,
                        &region.name,
                    )?
                }
                _ => Vec::new(),
            };

            state.defend_country = 0;
            state.attack_country = 0;
            report.info_deliveries.push(context.send_country_info(
                &text,
                0xffff_fe92,
                0xffff_0000,
            )?);
        }

        Ok(report)
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp

// ============================================================================
// FUNCTION: CountryWarSys::get_instance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:38
// RVA: 0x0008F410
// ADDRESS: 0048f410
// PROTOTYPE: CountryWarSys * __cdecl get_instance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:477
// RVA: 0x0008F490
// ADDRESS: 0048f490
// PROTOTYPE: void __stdcall on_war_clear(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: get_country_war_sys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:775
// RVA: 0x0008F4F0
// ADDRESS: 0048f4f0
// PROTOTYPE: CountryWarSys * __cdecl get_country_war_sys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:335
// RVA: 0x0008F800
// ADDRESS: 0048f800
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::end_war
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:482
// RVA: 0x0008F890
// ADDRESS: 0048f890
// PROTOTYPE: void __thiscall end_war(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::is_already_declare
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:604
// RVA: 0x0008F940
// ADDRESS: 0048f940
// PROTOTYPE: bool __thiscall is_already_declare(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::get_war_region
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:693
// RVA: 0x0008F990
// ADDRESS: 0048f990
// PROTOTYPE: long __thiscall get_war_region(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_declare_end
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:514
// RVA: 0x0008FA00
// ADDRESS: 0048fa00
// PROTOTYPE: void __stdcall on_declare_end(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_prepare_begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:526
// RVA: 0x0008FB40
// ADDRESS: 0048fb40
// PROTOTYPE: void __stdcall on_prepare_begin(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_prepare_end
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:538
// RVA: 0x0008FC80
// ADDRESS: 0048fc80
// PROTOTYPE: void __stdcall on_prepare_end(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_start
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:376
// RVA: 0x00090EC0
// ADDRESS: 00490ec0
// PROTOTYPE: void __stdcall on_war_start(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_end
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:409
// RVA: 0x00091100
// ADDRESS: 00491100
// PROTOTYPE: void __stdcall on_war_end(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_declare_begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:495
// RVA: 0x00091530
// ADDRESS: 00491530
// PROTOTYPE: void __stdcall on_declare_begin(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_start_info
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:550
// RVA: 0x000916C0
// ADDRESS: 004916c0
// PROTOTYPE: void __stdcall on_war_start_info(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_war_end_info
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:578
// RVA: 0x000918E0
// ADDRESS: 004918e0
// PROTOTYPE: void __stdcall on_war_end_info(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::player_declare
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:617
// RVA: 0x00091B00
// ADDRESS: 00491b00
// PROTOTYPE: bool __thiscall player_declare(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::on_flag_destory
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:706
// RVA: 0x00091E00
//
// Реализовано выше в связной cross-server country victory chain;
// потерянный stack-key и порядок эффектов имеют статус VERIFIED_DISASSEMBLY.
//

// ============================================================================
// FUNCTION: CountryWarSys::initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:45
// RVA: 0x00092220
// ADDRESS: 00492220
// PROTOTYPE: bool __thiscall initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountryWarSys::reload
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:351
// RVA: 0x00092DF0
// ADDRESS: 00492df0
// PROTOTYPE: bool __thiscall reload(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
