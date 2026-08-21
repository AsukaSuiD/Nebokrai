//! WorldServer-владелец country-war state `CountryWarSys`.
//!
//! `AddToByteArray` RVA `0x0008F800`, `player_declare` RVA `0x00091B00` и
//! `on_flag_destory` (исходное PDB-написание) RVA `0x00091E00` имеют статус
//! `IMPLEMENTED`; остальной корпус
//! ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара `WorldServer/Nworldserver.exe +
//! WorldServer/WorldServer.pdb`, исходник `appworld/country/countrywarsys.cpp`.
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
//! границей; concrete adapter теперь подключён к `ProcessMessage(0x60318)`.
//! Неизвестность в этих владельцах сохраняет уже выполненные предыдущие эффекты.
//! Переполнение исходного 256-byte `_sprintf` не воспроизводится: нормальный
//! output сохраняется, oversized localization безопасно ограничивается 255
//! байтами под C-string NUL как внутренний UB без доказанного gameplay-эффекта.
//! `player_declare` подтверждён exact `0x00491B00..0x00491DFD`: online-player,
//! чужая target-country, последовательные `IsKing/IsMinister(5)`, первый
//! свободный region, живой `pRegion`, затем проверки только defend-country.
//! При успехе state меняется до `0x7FF1F`, результаты обеих рассылок
//! игнорируются, лишний lookup `WS0103` перезаписывается `WS0104`, и функция
//! возвращает true. Linux-донор добавлял source/tail gates, fallback региона и
//! rollback при отказе очереди; эти полезные, но неоригинальные политики сюда
//! не перенесены. 512-byte `_sprintf` overflow безопасно ограничен нормальным
//! C-string payload в 511 байт без изменения штатного результата.
//!
//! Snapshot намеренно сохраняет layout World EXE: `state_clear + 3 bytes
//! padding`, затем defender и attacker. Парный Game EXE RVA `0x000EBD60`
//! трактует те же 12 bytes как defender, attacker, `state_clear + padding` —
//! это подтверждённое несовпадение поставленных бинарников, а не повод молча
//! менять World wire. Неинициализированный padding старого World нормализован
//! нулями: он не несёт семантики Miracle и не должен утекать в сеть.

use std::collections::BTreeMap;

use crate::nets::networld::message::{CMessage, SendMessageError};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationAuthority {
    CountryMissing,
    Rejected,
    Authorized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationPlayer {
    Missing,
    CountryUnavailable,
    Country(u8),
}

pub(crate) trait CountryWarDeclarationContext {
    fn online_player_country(&mut self, player_id: i32) -> CountryWarDeclarationPlayer;

    /// Повторяет последовательные `IsKing`, затем `IsMinister(player, 5)`,
    /// включая их king-log при отрицательных проверках.
    fn declaration_authority(
        &mut self,
        country: u8,
        player_id: i32,
    ) -> CountryWarDeclarationAuthority;

    fn region(&mut self, region_id: i32) -> Option<CountryWarVictoryRegion>;
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8>;
    fn format_declaration_notice(
        &mut self,
        attack_country: u8,
        defend_country: i32,
        region_name: &[u8],
    ) -> Vec<u8>;
    fn send_private_to_country_king(
        &mut self,
        country: u8,
        text: &[u8],
    ) -> Option<Result<i32, SendMessageError>>;
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
    fn send_country_info(&mut self, text: &[u8], title: u32, color: u32) -> i32;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationRejection {
    PlayerMissing,
    PlayerCountryUnavailable,
    OwnCountry,
    CountryMissing,
    Unauthorized,
    NoFreeRegion,
    RegionMissing,
    AttackAlreadyDeclared {
        private_delivery: Option<Result<i32, SendMessageError>>,
    },
    TargetAlreadyDeclared {
        private_delivery: Option<Result<i32, SendMessageError>>,
    },
    StateEntryMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryWarDeclarationDisposition {
    Rejected(CountryWarDeclarationRejection),
    Declared {
        region_id: i32,
        state_wire: Vec<u8>,
        state_delivery: Result<i32, SendMessageError>,
        discarded_ws0103: Vec<u8>,
        notice: Vec<u8>,
        info_delivery: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryWarDeclarationReport {
    pub(crate) player_id: i32,
    pub(crate) target_country: i32,
    pub(crate) attack_country: Option<u8>,
    pub(crate) disposition: CountryWarDeclarationDisposition,
}

impl CountryWarDeclarationReport {
    pub(crate) const fn accepted(&self) -> bool {
        matches!(self.disposition, CountryWarDeclarationDisposition::Declared { .. })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryWarSys {
    pub(crate) war_regions: BTreeMap<i32, CountryWarRegion>,
}

impl CountryWarSys {
    pub(crate) fn add_to_byte_array(&self, output: &mut Vec<u8>) -> bool {
        output.extend_from_slice(&(self.war_regions.len() as u32).to_le_bytes());
        for (&region_id, state) in &self.war_regions {
            output.extend_from_slice(&region_id.to_le_bytes());
            output.push(u8::from(state.state_clear));
            output.extend_from_slice(&[0; 3]);
            output.extend_from_slice(&state.defend_country.to_le_bytes());
            output.extend_from_slice(&state.attack_country.to_le_bytes());
        }
        true
    }

    fn is_already_declared(&self, country: i32) -> bool {
        self.war_regions
            .values()
            .any(|state| state.defend_country == country)
    }

    fn free_war_region(&self) -> Option<i32> {
        self.war_regions
            .iter()
            .find(|(_, state)| state.defend_country == 0 && state.attack_country == 0)
            .map(|(&region_id, _)| region_id)
    }

    pub(crate) fn player_declare<Context: CountryWarDeclarationContext + ?Sized>(
        &mut self,
        player_id: i32,
        target_country: i32,
        context: &mut Context,
    ) -> CountryWarDeclarationReport {
        let rejected = |attack_country, reason| CountryWarDeclarationReport {
            player_id,
            target_country,
            attack_country,
            disposition: CountryWarDeclarationDisposition::Rejected(reason),
        };

        let attack_country = match context.online_player_country(player_id) {
            CountryWarDeclarationPlayer::Missing => {
                return rejected(None, CountryWarDeclarationRejection::PlayerMissing);
            }
            CountryWarDeclarationPlayer::CountryUnavailable => {
                return rejected(
                    None,
                    CountryWarDeclarationRejection::PlayerCountryUnavailable,
                );
            }
            CountryWarDeclarationPlayer::Country(country) => country,
        };
        if i32::from(attack_country) == target_country {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::OwnCountry,
            );
        }
        match context.declaration_authority(attack_country, player_id) {
            CountryWarDeclarationAuthority::CountryMissing => {
                return rejected(
                    Some(attack_country),
                    CountryWarDeclarationRejection::CountryMissing,
                );
            }
            CountryWarDeclarationAuthority::Rejected => {
                return rejected(
                    Some(attack_country),
                    CountryWarDeclarationRejection::Unauthorized,
                );
            }
            CountryWarDeclarationAuthority::Authorized => {}
        }

        let Some(region_id) = self.free_war_region() else {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::NoFreeRegion,
            );
        };
        let Some(region) = context.region(region_id) else {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::RegionMissing,
            );
        };
        if self.is_already_declared(i32::from(attack_country)) {
            let text = context.world_string(b"WS0101");
            let private_delivery = context.send_private_to_country_king(attack_country, &text);
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::AttackAlreadyDeclared { private_delivery },
            );
        }
        if self.is_already_declared(target_country) {
            let text = context.world_string(b"WS0102");
            let private_delivery = context.send_private_to_country_king(attack_country, &text);
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::TargetAlreadyDeclared { private_delivery },
            );
        }

        let Some(state) = self.war_regions.get_mut(&region_id) else {
            return rejected(
                Some(attack_country),
                CountryWarDeclarationRejection::StateEntryMissing,
            );
        };
        state.defend_country = target_country;
        state.attack_country = i32::from(attack_country);

        let mut state_message = CMessage::new(0x7ff1f);
        state_message.base_mut().add_long(region_id);
        state_message.base_mut().add_long(target_country);
        state_message
            .base_mut()
            .add_long(i32::from(attack_country));
        let state_wire = state_message.as_wire_bytes().to_vec();
        let state_delivery = context.send_all(&state_message);

        // Exact сначала копировал WS0103 в 512-byte buffer, затем полностью
        // перезаписывал его результатом sprintf(WS0104). Сам lookup сохраняем.
        let discarded_ws0103 = context.world_string(b"WS0103");
        let notice = context.format_declaration_notice(
            attack_country,
            target_country,
            &region.name,
        );
        let info_delivery = context.send_country_info(&notice, 0xffff_fe92, 0xffff_0000);

        CountryWarDeclarationReport {
            player_id,
            target_country,
            attack_country: Some(attack_country),
            disposition: CountryWarDeclarationDisposition::Declared {
                region_id,
                state_wire,
                state_delivery,
                discarded_ws0103,
                notice,
                info_delivery,
            },
        }
    }

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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countrywarsys.cpp:335
// RVA: 0x0008F800
// ADDRESS: 0048f800
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Реализовано выше с exact World layout и нулевым техническим padding.
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
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
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
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
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
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
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
