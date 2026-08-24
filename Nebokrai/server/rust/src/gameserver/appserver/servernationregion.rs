//! Метаданные исследования оригинала; сами по себе не доказывают совместимость.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Startup inheritance подтверждён constructor-ом: `ServerNationRegion`
//! начинается с `CServerWarRegion` и не имеет собственного wire decoder-а.
//! Поэтому typed startup owner делегирует exact War -> ServerRegion chain.
//! FourNation startup дополнительно материализует подтверждённый
//! `GetReliveRect`, а timing-chain владеет exact 16-byte
//! `_tagPlayerWarTime`, wrapping `timeGetTime` arithmetic и x87-truncated
//! morale→exploit. Линейный owned `Vec` заменяет MSVC `stdext::hash_map`:
//! lookup-семантика совпадает, а недоказанный bucket-order не выдаётся
//! за gameplay-контракт. Остальное nation combat state ниже остаётся RAW.

use super::organizingsystem::fournationwarsys::FourNationRect;
use super::serverregion::ServerRegionDecodeError;
use super::serverwarregion::{CServerWarRegion, WarRegionDecodeContext, WarRegionDecodeError};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerNationRegion {
    pub(crate) war: CServerWarRegion,
    relive_rects: [FourNationRect; 5],
    morale: [i32; 5],
    nation_failed: [bool; 5],
    player_war_times: Vec<NationPlayerWarTime>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct NationPlayerWarTime {
    pub(crate) player_id: i32,
    pub(crate) country: i32,
    pub(crate) country_figure: i32,
    pub(crate) start_time_ms: u32,
    pub(crate) elapsed_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NationPlayerWarAward {
    pub(crate) player_id: i32,
    pub(crate) country: i32,
    pub(crate) elapsed_time_ms: u32,
    pub(crate) exploit: u32,
}

impl ServerNationRegion {
    pub(crate) fn decord_from_byte_array<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        context: &mut Context,
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError<Context::RuntimeError>>> {
        self.war
            .decord_from_byte_array(source, cursor, include_child, context)
    }

    /// Exact `GetReliveRect` копирует все пять country rectangles в owner.
    pub(crate) const fn set_relive_rects(&mut self, rects: [FourNationRect; 5]) {
        self.relive_rects = rects;
    }

    pub(crate) const fn relive_rects(&self) -> &[FourNationRect; 5] {
        &self.relive_rects
    }

    pub(crate) const fn morale(&self) -> &[i32; 5] {
        &self.morale
    }

    pub(crate) const fn nation_failed(&self) -> &[bool; 5] {
        &self.nation_failed
    }

    /// Region battle owner публикует текущую morale без clamp:
    /// исходные combat callbacks пишут signed `long` напрямую.
    pub(crate) const fn set_country_morale(&mut self, country: usize, morale: i32) -> bool {
        if country >= self.morale.len() {
            return false;
        }
        self.morale[country] = morale;
        true
    }

    pub(crate) const fn set_nation_failed(&mut self, country: usize, failed: bool) -> bool {
        if country >= self.nation_failed.len() {
            return false;
        }
        self.nation_failed[country] = failed;
        true
    }

    /// Exact materialized subset `OnWarDeclare`: regional timing and combat
    /// counters start empty; signup counts сам owner в этой функции не читает.
    pub(crate) fn reset_for_war_declare(&mut self) {
        self.player_war_times.clear();
        self.morale.fill(0);
        self.nation_failed.fill(false);
    }

    /// Exact materialized prefix `OnRefreshRegion`: morale всех five slots
    /// становится 1000, failed flags и regional timing очищаются.
    pub(crate) fn reset_for_region_refresh(&mut self) {
        self.player_war_times.clear();
        self.morale.fill(1000);
        self.nation_failed.fill(false);
    }

    /// Exact `OnPlayerTimgingStart`: repeated start меняет только clock,
    /// не переснимая country/identity и не обнуляя elapsed.
    pub(crate) fn start_player_timing(
        &mut self,
        player_id: i32,
        country: i32,
        country_figure: i32,
        now_ms: u32,
    ) {
        if let Some(record) = self
            .player_war_times
            .iter_mut()
            .find(|record| record.player_id == player_id)
        {
            record.start_time_ms = now_ms;
            return;
        }
        self.player_war_times.push(NationPlayerWarTime {
            player_id,
            country,
            country_figure,
            start_time_ms: now_ms,
            elapsed_time_ms: 0,
        });
    }

    pub(crate) fn has_player_timing(&self, player_id: i32) -> bool {
        self.player_war_times
            .iter()
            .any(|record| record.player_id == player_id)
    }

    /// Exact `OnPlayerTimeingFinish`: missing/inactive records не меняются;
    /// death penalty добавляется после elapsed с `DWORD` wrapping.
    pub(crate) fn finish_player_timing(
        &mut self,
        player_id: i32,
        died: bool,
        now_ms: impl FnOnce() -> u32,
    ) -> Option<NationPlayerWarTime> {
        let record = self
            .player_war_times
            .iter_mut()
            .find(|record| record.player_id == player_id && record.start_time_ms != 0)?;
        let now_ms = now_ms();
        record.elapsed_time_ms = record
            .elapsed_time_ms
            .wrapping_add(now_ms.wrapping_sub(record.start_time_ms));
        record.start_time_ms = 0;
        if died {
            record.elapsed_time_ms = record.elapsed_time_ms.wrapping_add(30_000);
        }
        Some(*record)
    }

    /// Exact `ConvertMoraleToExploitForEachPlayer` state pass: каждый active
    /// record сам читает `timeGetTime`, затем owner-map очищается.
    pub(crate) fn take_player_war_awards(
        &mut self,
        mut now_ms: impl FnMut() -> u32,
    ) -> Vec<NationPlayerWarAward> {
        let mut awards = Vec::with_capacity(self.player_war_times.len());
        for mut record in self.player_war_times.drain(..) {
            if record.start_time_ms != 0 {
                let now_ms = now_ms();
                record.elapsed_time_ms = record
                    .elapsed_time_ms
                    .wrapping_add(now_ms.wrapping_sub(record.start_time_ms));
                record.start_time_ms = 0;
            }
            let morale = self
                .morale
                .get(record.country as usize)
                .copied()
                .unwrap_or_default();
            awards.push(NationPlayerWarAward {
                player_id: record.player_id,
                country: record.country,
                elapsed_time_ms: record.elapsed_time_ms,
                exploit: convert_morale_to_exploit(
                    record.elapsed_time_ms,
                    morale,
                    record.country_figure,
                ),
            });
        }
        awards
    }

    /// Materialized subset final reset-loop `OnWarEnd`: все пять slots,
    /// включая unused index 0, обнуляются после award pass.
    pub(crate) fn reset_materialized_war_state(&mut self) {
        self.morale.fill(0);
        self.nation_failed.fill(false);
    }
}

/// Exact `ConvertMoraleToExploit` RVA `0xF1230`. EXE ставит x87 RC=11
/// перед `fistp`, поэтому для положительной morale нужно truncation,
/// а не Rust `round()`.
pub(crate) fn convert_morale_to_exploit(
    elapsed_time_ms: u32,
    morale: i32,
    country_figure: i32,
) -> u32 {
    if elapsed_time_ms == 0 {
        return 0;
    }
    let band = match elapsed_time_ms {
        0..600_000 => 0,
        600_000..1_800_000 => 1,
        1_800_000..2_700_000 => 2,
        2_700_000..3_600_000 => 3,
        _ => 4,
    };
    let multiplier = match country_figure {
        1 => [0.3, 0.75, 1.05, 1.35, 1.5][band],
        2..=8 => [0.24, 0.6, 0.84, 1.08, 1.2][band],
        _ => [0.2, 0.5, 0.7, 0.9, 1.0][band],
    };
    (f64::from(morale) * multiplier).trunc() as i64 as u32
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp

// ============================================================================
// FUNCTION: CArea::`vector_deleting_destructor'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007BB00
// ADDRESS: 0047bb00
// PROTOTYPE: void * __thiscall `vector_deleting_destructor'(uint param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagNpc::tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007C550
// ADDRESS: 0047c550
// PROTOTYPE: undefined __thiscall tagNpc(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagNpc::~tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007C570
// ADDRESS: 0047c570
// PROTOTYPE: void __thiscall ~tagNpc(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonsterList::~tagMonsterList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007C5C0
// ADDRESS: 0047c5c0
// PROTOTYPE: void __thiscall ~tagMonsterList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonsterList::tagMonsterList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007CA60
// ADDRESS: 0047ca60
// PROTOTYPE: undefined __thiscall tagMonsterList(tagMonsterList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagNpc::tagNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007CB00
// ADDRESS: 0047cb00
// PROTOTYPE: undefined __thiscall tagNpc(tagNpc * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d606
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007D606
// ADDRESS: 0047d606
// PROTOTYPE: undefined Catch@0047d606()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047de0e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007DE0E
// ADDRESS: 0047de0e
// PROTOTYPE: undefined Catch@0047de0e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047dfa1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007DFA1
// ADDRESS: 0047dfa1
// PROTOTYPE: undefined Catch@0047dfa1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e3c9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E3C9
// ADDRESS: 0047e3c9
// PROTOTYPE: undefined Catch@0047e3c9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e546
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E546
// ADDRESS: 0047e546
// PROTOTYPE: undefined Catch@0047e546()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e635
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E635
// ADDRESS: 0047e635
// PROTOTYPE: undefined Catch@0047e635()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047e6d8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007E6D8
// ADDRESS: 0047e6d8
// PROTOTYPE: undefined Catch@0047e6d8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047ea26
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007EA26
// ADDRESS: 0047ea26
// PROTOTYPE: undefined Catch@0047ea26()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047fd99
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007FD99
// ADDRESS: 0047fd99
// PROTOTYPE: undefined Catch@0047fd99()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047ffe2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x0007FFE2
// ADDRESS: 0047ffe2
// PROTOTYPE: undefined Catch@0047ffe2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00480071
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00080071
// ADDRESS: 00480071
// PROTOTYPE: undefined Catch@00480071()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004801be
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x000801BE
// ADDRESS: 004801be
// PROTOTYPE: undefined Catch@004801be()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00482114
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00082114
// ADDRESS: 00482114
// PROTOTYPE: undefined Catch@00482114()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00482190
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00082190
// ADDRESS: 00482190
// PROTOTYPE: undefined Catch@00482190()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonster::tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00082F70
// ADDRESS: 00482f70
// PROTOTYPE: undefined __thiscall tagMonster(tagMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagMonster::~tagMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083260
// ADDRESS: 00483260
// PROTOTYPE: void __thiscall ~tagMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483a84
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083A84
// ADDRESS: 00483a84
// PROTOTYPE: undefined Catch@00483a84()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483b35
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083B35
// ADDRESS: 00483b35
// PROTOTYPE: undefined Catch@00483b35()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483c74
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083C74
// ADDRESS: 00483c74
// PROTOTYPE: undefined Catch@00483c74()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00483d7e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083D7E
// ADDRESS: 00483d7e
// PROTOTYPE: undefined Catch@00483d7e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerRegion::tagWeatherTime::~tagWeatherTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00083F30
// ADDRESS: 00483f30
// PROTOTYPE: void __thiscall ~tagWeatherTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004844de
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x000844DE
// ADDRESS: 004844de
// PROTOTYPE: undefined Catch@004844de()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00484db0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00084DB0
// ADDRESS: 00484db0
// PROTOTYPE: undefined Catch@00484db0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00485014
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x00085014
// ADDRESS: 00485014
// PROTOTYPE: undefined Catch@00485014()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004850c5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp
// RVA: 0x000850C5
// ADDRESS: 004850c5
// PROTOTYPE: undefined Catch@004850c5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::GetReliveRect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:892
// RVA: 0x000F1150
// ADDRESS: 004f1150
// PROTOTYPE: void __thiscall GetReliveRect(tagRECT * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:904
// RVA: 0x000F1220
// ADDRESS: 004f1220
// PROTOTYPE: void __thiscall OnWarStart(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::ConvertMoraleToExploit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1815
// RVA: 0x000F1230
// ADDRESS: 004f1230
// PROTOTYPE: ulong __thiscall ConvertMoraleToExploit(ulong param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::IsNationFail
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2267
// RVA: 0x000F15B0
// ADDRESS: 004f15b0
// PROTOTYPE: bool __thiscall IsNationFail(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::GetDiedStateTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2347
// RVA: 0x000F15D0
// ADDRESS: 004f15d0
// PROTOTYPE: long __thiscall GetDiedStateTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::GetIsPlayerIsContendSymbol
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1534
// RVA: 0x000F1610
// ADDRESS: 004f1610
// PROTOTYPE: bool __thiscall GetIsPlayerIsContendSymbol(long param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CancelContendByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1549
// RVA: 0x000F1640
// ADDRESS: 004f1640
// PROTOTYPE: bool __thiscall CancelContendByPlayerID(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnPlayerDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1630
// RVA: 0x000F16C0
// ADDRESS: 004f16c0
// PROTOTYPE: void __thiscall OnPlayerDamage(CPlayer * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnPlayerTimeingFinish
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1733
// RVA: 0x000F1800
// ADDRESS: 004f1800
// PROTOTYPE: void __thiscall OnPlayerTimeingFinish(CPlayer * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::SendNotifyWhenFisrtGuardOfCountryDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1095
// RVA: 0x000F18E0
// ADDRESS: 004f18e0
// PROTOTYPE: void __thiscall SendNotifyWhenFisrtGuardOfCountryDie(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnCarriageBackTown
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1442
// RVA: 0x000F1A70
// ADDRESS: 004f1a70
// PROTOTYPE: void __thiscall OnCarriageBackTown(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CheckAdd_YUYINGSHI_DueToAdmiralDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1948
// RVA: 0x000F1DA0
// ADDRESS: 004f1da0
// PROTOTYPE: void __thiscall CheckAdd_YUYINGSHI_DueToAdmiralDie(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CheckAdd_YUYINGSHI_DueToMoraleChange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2052
// RVA: 0x000F23E0
// ADDRESS: 004f23e0
// PROTOTYPE: void __thiscall CheckAdd_YUYINGSHI_DueToMoraleChange(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::DelNpcMagicStoneAndAddMonsterMagicStone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:2274
// RVA: 0x000F3030
// ADDRESS: 004f3030
// PROTOTYPE: void __thiscall DelNpcMagicStoneAndAddMonsterMagicStone(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::~ServerNationRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:42
// RVA: 0x000F3900
// ADDRESS: 004f3900
// PROTOTYPE: void __thiscall ~ServerNationRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OpMorale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1113
// RVA: 0x000F3960
// ADDRESS: 004f3960
// PROTOTYPE: bool __thiscall OpMorale(CMonster * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnMonsterDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1381
// RVA: 0x000F4720
// ADDRESS: 004f4720
// PROTOTYPE: void __thiscall OnMonsterDie(CMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnClearWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:948
// RVA: 0x000F4E60
// ADDRESS: 004f4e60
// PROTOTYPE: void __thiscall OnClearWar(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::AddContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1588
// RVA: 0x000F5550
// ADDRESS: 004f5550
// PROTOTYPE: void __thiscall AddContend(CPlayer * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::ServerNationRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:14
// RVA: 0x000F5810
// ADDRESS: 004f5810
// PROTOTYPE: undefined __thiscall ServerNationRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarDeclare
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:388
// RVA: 0x000F5920
// ADDRESS: 004f5920
// PROTOTYPE: void __thiscall OnWarDeclare(long param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnRefreshRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:424
// RVA: 0x000F5990
// ADDRESS: 004f5990
// PROTOTYPE: void __thiscall OnRefreshRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::KickOutAllPlayerToReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1067
// RVA: 0x000F62E0
// ADDRESS: 004f62e0
// PROTOTYPE: void __thiscall KickOutAllPlayerToReturnPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnMonsterDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1309
// RVA: 0x000F63E0
// ADDRESS: 004f63e0
// PROTOTYPE: void __thiscall OnMonsterDamage(CMonster * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnEnterContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1492
// RVA: 0x000F6B40
// ADDRESS: 004f6b40
// PROTOTYPE: void __thiscall OnEnterContend(CPlayer * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::CancelContendToAllPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1574
// RVA: 0x000F6D70
// ADDRESS: 004f6d70
// PROTOTYPE: void __thiscall CancelContendToAllPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::ConvertMoraleToExploitForEachPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1751
// RVA: 0x000F6DF0
// ADDRESS: 004f6df0
// PROTOTYPE: void __thiscall ConvertMoraleToExploitForEachPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnContendTimeOver
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:219
// RVA: 0x000F7420
// ADDRESS: 004f7420
// PROTOTYPE: void __thiscall OnContendTimeOver(_tagNationContend * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarMass
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:412
// RVA: 0x000F7DB0
// ADDRESS: 004f7db0
// PROTOTYPE: void __thiscall OnWarMass(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnWarEnd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:910
// RVA: 0x000F7DD0
// ADDRESS: 004f7dd0
// PROTOTYPE: void __thiscall OnWarEnd(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::OnPlayerTimgingStart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:1662
// RVA: 0x000F7EA0
// ADDRESS: 004f7ea0
// PROTOTYPE: void __thiscall OnPlayerTimgingStart(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ServerNationRegion::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servernationregion.cpp:156
// RVA: 0x000F8290
// ADDRESS: 004f8290
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
