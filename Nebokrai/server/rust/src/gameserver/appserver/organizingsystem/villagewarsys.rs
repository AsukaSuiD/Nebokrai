//! Владелец GameServer-снимка деревенских войн `CVillageWarSys`.
//!
//! Snapshot/setup (`0x0005F320`), initial state (`0x0005E310`), faction update
//! (`0x0005E480`) и фазовые callbacks `0x0005E5B0..0x0005E640`,
//! `0x0005F510..0x0005F740` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; schedule queries `0x0005DA90..0x0005DB80` и
//! `0x0005E830` — `IMPLEMENTED`. Остальные функции owner-а остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Исходник PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp`;
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//!
//! Wire начинается с signed `long` количества. Для каждого положительного
//! элемента исходник копирует prefix `tagVilWarSetup` длиной `0x8C`, затем
//! читает signed count и ordered faction IDs. PDB подтверждает шесть пар
//! `event ID + tagTime`, signed schedule/region/state и `std::list` с offset
//! `0x88`; weekly flag по offset `0x94` в snapshot не входит. `BTreeMap` и
//! `Vec` заменяют STL с тем же key/order-контрактом; повторный `lID` заменяет
//! ранее декодированную запись, как `map::operator[]` с присваиванием.
//!
//! Четыре bytes stateless allocator-base внутри prefix сознательно читаются и
//! отбрасываются. Event IDs сохраняются как wire `u32`, но в этом GameServer
//! owner-е ни один достигнутый consumer их не читает. Недоставленный weekly
//! flag выражен `None`, а не выдуманным значением. Exact EXE показывает внешний
//! цикл `0x0045F4D3..0x0045F4DF` и общий `mov al,1` по `0x0045F4EF`.
//! Безразмерный C++ pointer на short buffer уходил в неизвестный UB; safe Rust
//! возвращает локальный `UnexpectedEnd`, сохраняя уже пройденный cursor и уже
//! вставленные полные записи. STL tree/list allocation, SEH и cleanup отдельной
//! Rust-семантики не имеют.
//!
//! Initial-state owner в map-order выбирает только closed interval
//! `DeclarWarTime <= now <= WarEndTime`. Для war и village ID он отдельно ищет
//! сначала `CGame::s_mapRegion`, а при miss/null — `FindProxyRegion`. Найденный
//! war region получает virtual `ReSetWarState(war number, state)`. Если найдены
//! оба региона, byte `CRegion::m_btCountry` offset `+0x74` копируется из village
//! в war region уже после reset. Exact EXE подтверждает два stack-аргумента у
//! vtable slot `+0xE8` и byte offsets `+0x74`; сырой `CGame/CServerRegion`
//! остаётся внешним context-контрактом до собственного прохода.
//! Faction-update всегда читает war ID, но при miss сразу возвращает `false`,
//! не читая count. Hit очищает прежний list, принимает signed count и ordered
//! IDs, затем ищет war region только в `s_mapRegion` без proxy fallback и при
//! non-null вызывает virtual `UpdateContendPlayer()` slot `+0x104`. Safe short
//! buffer сохраняет уже выполненные clear/append и cursor, не назначая старому
//! UB дополнительный результат.
//!
//! Declare ставит `DUTH=1`, находит war и village независимо, затем перед
//! `OnWarDeclare` переносит в war-region faction/union/country деревни. Start
//! ставит `Fight=3`; end сперва очищает faction list и вызывает `OnWarEnd`
//! только из `Fight`. Start/end допускают proxy, timeout намеренно ищет лишь
//! основной war-region. Clear отдельно берёт основной village-region, шлёт
//! `0xBF806(0xFFDAEDFE, 0, GS0127(region name))` и запускает вытеснение через
//! 60000 ms. Region/message runtime остаётся явным context-контрактом.
//!
//! Query-поверхность сохраняет map-order, strict `now < end`, старую packed
//! minute-формулу с x86 wrapping и main-then-proxy lookup имени. Выходной
//! `std::string& + bool` представлен `Option<String>`. В отличие от города,
//! village membership не читает недоставленный weekly flag.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::public::date::TagTime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VillageWarDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for VillageWarDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for VillageWarDecodeError {}

#[derive(Clone, Debug)]
pub(crate) struct VillageWarSetup {
    pub(crate) id: i32,
    pub(crate) war_region_id: i32,
    pub(crate) declare_event_id: u32,
    pub(crate) declare_time: TagTime,
    pub(crate) start_info_event_id: u32,
    pub(crate) start_info_time: TagTime,
    pub(crate) start_event_id: u32,
    pub(crate) start_time: TagTime,
    pub(crate) end_info_event_id: u32,
    pub(crate) end_info_time: TagTime,
    pub(crate) end_event_id: u32,
    pub(crate) end_time: TagTime,
    pub(crate) clear_event_id: u32,
    pub(crate) clear_player_time: TagTime,
    pub(crate) village_region_id: i32,
    pub(crate) region_state: i32,
    pub(crate) declaring_factions: Vec<i32>,
    pub(crate) is_every_week: Option<i32>,
}

#[derive(Default)]
pub(crate) struct CVillageWarSys {
    pub(crate) village_wars: BTreeMap<i32, VillageWarSetup>,
}

pub(crate) trait VillageWarRegionContext {
    type Region: Copy;

    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Вызывает virtual `ReSetWarState(long, eCityState)` найденного региона.
    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32);

    fn region_country(&self, region: Self::Region) -> u8;

    fn set_region_country(&mut self, region: Self::Region, country: u8);
}

pub(crate) trait VillageWarFactionUpdateContext {
    type Region: Copy;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region>;

    fn update_contend_player(&mut self, region: Self::Region);
}

pub(crate) trait VillageWarPhaseContext {
    type Region: Copy;

    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region>;

    fn owned_city_faction(&mut self, region: Self::Region) -> i32;
    fn owned_city_union(&mut self, region: Self::Region) -> i32;
    fn set_owned_city_org(&mut self, region: Self::Region, faction_id: i32, union_id: i32);
    fn region_country(&self, region: Self::Region) -> u8;
    fn set_region_country(&mut self, region: Self::Region, country: u8);

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32);
    fn on_war_start(&mut self, region: Self::Region, war_number: i32);
    fn on_war_time_out(&mut self, region: Self::Region, war_number: i32);
    fn on_war_end(&mut self, region: Self::Region, war_number: i32);

    /// Формирует `0xBF806(0xFFDAEDFE, 0, GS0127(region name))` и шлёт в регион.
    fn send_clear_player_notice(&mut self, region: Self::Region);

    fn start_clear_player_out(&mut self, region: Self::Region, delay_ms: i32);
}

pub(crate) trait VillageWarNameContext {
    /// Возвращает копию имени после lookup `s_mapRegion`, затем proxy fallback.
    fn region_name_then_proxy(&mut self, region_id: i32) -> Option<String>;
}

const CITY_STATE_DECLARE: i32 = 1;
const CITY_STATE_FIGHT: i32 = 3;

impl CVillageWarSys {
    /// Очищает прежний map и принимает полный payload `SI_VILLAGEWARSYS_SETUP`.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, VillageWarDecodeError> {
        self.village_wars.clear();
        let schedule_count = read_i32(source, cursor, "m_VillageWars.size")?;
        for _ in 0..schedule_count.max(0) {
            let setup = decode_setup(source, cursor)?;
            self.village_wars.insert(setup.id, setup);
        }
        Ok(true)
    }

    /// Проецирует active schedule state в найденные server/proxy regions.
    pub(crate) fn init_village_region_state<Context: VillageWarRegionContext>(
        &self,
        context: &mut Context,
    ) {
        self.init_village_region_state_at(TagTime::local_now(), context);
    }

    pub(crate) fn init_village_region_state_at<Context: VillageWarRegionContext>(
        &self,
        now: TagTime,
        context: &mut Context,
    ) {
        for (&war_number, setup) in &self.village_wars {
            if !now.legacy_ge(setup.declare_time) || !now.legacy_le(setup.end_time) {
                continue;
            }

            let war_region = context.find_region_then_proxy(setup.war_region_id);
            if let Some(region) = war_region {
                context.reset_war_state(region, war_number, setup.region_state);
            }

            let village_region = context.find_region_then_proxy(setup.village_region_id);
            if let (Some(war_region), Some(village_region)) = (war_region, village_region) {
                let village_country = context.region_country(village_region);
                context.set_region_country(war_region, village_country);
            }
        }
    }

    /// Заменяет ordered faction list одного schedule и обновляет contenders региона.
    pub(crate) fn update_apply_war_factions<Context: VillageWarFactionUpdateContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        context: &mut Context,
    ) -> Result<bool, VillageWarDecodeError> {
        let war_number = read_i32(source, cursor, "UpdateApplyWarFacs.war_number")?;
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return Ok(false);
        };

        setup.declaring_factions.clear();
        let faction_count = read_i32(source, cursor, "UpdateApplyWarFacs.faction_count")?;
        for _ in 0..faction_count.max(0) {
            setup.declaring_factions.push(read_i32(
                source,
                cursor,
                "UpdateApplyWarFacs.faction_id",
            )?);
        }
        let war_region_id = setup.war_region_id;

        if let Some(region) = context.find_server_region(war_region_id) {
            context.update_contend_player(region);
        }
        Ok(true)
    }

    /// Возвращает ID связанной деревни либо исходный sentinel `0`.
    pub(crate) fn get_village_region_id_by_time(&self, war_number: i32) -> i32 {
        self.village_wars
            .get(&war_number)
            .map_or(0, |setup| setup.village_region_id)
    }

    pub(crate) fn is_already_declar_for_war(&self, war_number: i32, faction_id: i32) -> bool {
        self.village_wars
            .get(&war_number)
            .is_some_and(|setup| setup.declaring_factions.contains(&faction_id))
    }

    /// Возвращает legacy packed start-time первого ещё не завершённого schedule.
    pub(crate) fn get_war_start_time(&self, war_region_id: i32) -> i32 {
        self.get_war_start_time_at(war_region_id, TagTime::local_now())
    }

    pub(crate) fn get_war_start_time_at(&self, war_region_id: i32, now: TagTime) -> i32 {
        self.village_wars
            .values()
            .find(|setup| setup.war_region_id == war_region_id && now.legacy_lt(setup.end_time))
            .map_or(0, |setup| legacy_packed_war_start(setup.start_time))
    }

    /// Возвращает имя первого active schedule, содержащего faction ID.
    pub(crate) fn get_war_name_for_declar<Context: VillageWarNameContext>(
        &self,
        faction_id: i32,
        context: &mut Context,
    ) -> Option<String> {
        self.village_wars.values().find_map(|setup| {
            if setup.region_state == 0 || !setup.declaring_factions.contains(&faction_id) {
                return None;
            }
            context.region_name_then_proxy(setup.war_region_id)
        })
    }

    pub(crate) fn on_delcare_war<Context: VillageWarPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return;
        };
        setup.region_state = CITY_STATE_DECLARE;
        let war_region_id = setup.war_region_id;
        let village_region_id = setup.village_region_id;

        let war_region = context.find_region_then_proxy(war_region_id);
        let village_region = context.find_region_then_proxy(village_region_id);
        if let Some(war_region) = war_region {
            if let Some(village_region) = village_region {
                // RVA 0x5F510 читает union раньше faction, но передаёт
                // `SetOwnedCityOrg(faction, union)` именно в таком порядке.
                let union_id = context.owned_city_union(village_region);
                let faction_id = context.owned_city_faction(village_region);
                context.set_owned_city_org(war_region, faction_id, union_id);
                let country = context.region_country(village_region);
                context.set_region_country(war_region, country);
            }
            context.on_war_declare(war_region, war_number);
        }
    }

    pub(crate) fn on_attack_village_start<Context: VillageWarPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return;
        };
        setup.region_state = CITY_STATE_FIGHT;
        let war_region_id = setup.war_region_id;

        if let Some(region) = context.find_region_then_proxy(war_region_id) {
            context.on_war_start(region, war_number);
        }
    }

    pub(crate) fn on_attack_village_out_time<Context: VillageWarPhaseContext>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.village_wars.get(&war_number) else {
            return;
        };

        if let Some(region) = context.find_server_region(setup.war_region_id) {
            context.on_war_time_out(region, war_number);
        }
    }

    pub(crate) fn on_attack_village_end<Context: VillageWarPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return;
        };
        setup.declaring_factions.clear();
        if setup.region_state != CITY_STATE_FIGHT {
            return;
        }
        let war_region_id = setup.war_region_id;

        if let Some(region) = context.find_region_then_proxy(war_region_id) {
            context.on_war_end(region, war_number);
        }
    }

    pub(crate) fn on_clear_player<Context: VillageWarPhaseContext>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.village_wars.get(&war_number) else {
            return;
        };

        if let Some(region) = context.find_server_region(setup.village_region_id) {
            context.send_clear_player_notice(region);
            context.start_clear_player_out(region, 60_000);
        }
    }
}

fn legacy_packed_war_start(time: TagTime) -> i32 {
    i32::from(time.year)
        .wrapping_mul(0x10)
        .wrapping_add(0x93f)
        .wrapping_add(i32::from(time.month))
        .wrapping_mul(0x20)
        .wrapping_add(i32::from(time.day))
        .wrapping_mul(0x20)
        .wrapping_add(i32::from(time.hour))
        .wrapping_mul(0x40)
        .wrapping_add(i32::from(time.minute))
        .wrapping_mul(8)
        .wrapping_add(i32::from(time.day_of_week))
}

fn decode_setup(
    source: &[u8],
    cursor: &mut usize,
) -> Result<VillageWarSetup, VillageWarDecodeError> {
    let id = read_i32(source, cursor, "tagVilWarSetup.lID")?;
    let war_region_id = read_i32(source, cursor, "tagVilWarSetup.lWarRegionID")?;
    let declare_event_id = read_u32(source, cursor, "tagVilWarSetup.lDeclarWarEventID")?;
    let declare_time = read_tag_time(source, cursor, "tagVilWarSetup.DeclarWarTime")?;
    let start_info_event_id = read_u32(source, cursor, "tagVilWarSetup.lStartInfoEventID")?;
    let start_info_time = read_tag_time(source, cursor, "tagVilWarSetup.WarStartInfoTime")?;
    let start_event_id = read_u32(source, cursor, "tagVilWarSetup.lStartEventID")?;
    let start_time = read_tag_time(source, cursor, "tagVilWarSetup.WarStartTime")?;
    let end_info_event_id = read_u32(source, cursor, "tagVilWarSetup.lEndInfoEventID")?;
    let end_info_time = read_tag_time(source, cursor, "tagVilWarSetup.WarEndInfoTime")?;
    let end_event_id = read_u32(source, cursor, "tagVilWarSetup.lEndEventID")?;
    let end_time = read_tag_time(source, cursor, "tagVilWarSetup.WarEndTime")?;
    let clear_event_id = read_u32(source, cursor, "tagVilWarSetup.lClearEventID")?;
    let clear_player_time = read_tag_time(source, cursor, "tagVilWarSetup.ClearPlayerTime")?;
    let village_region_id = read_i32(source, cursor, "tagVilWarSetup.lVilRegionID")?;
    let region_state = read_i32(source, cursor, "tagVilWarSetup.RegionState")?;
    let _allocator_base = read_exact::<4>(
        source,
        cursor,
        "tagVilWarSetup.DecWarFactions allocator-base",
    )?;
    let faction_count = read_i32(source, cursor, "tagVilWarSetup.DecWarFactions.size")?;
    let mut declaring_factions = Vec::new();
    for _ in 0..faction_count.max(0) {
        declaring_factions.push(read_i32(
            source,
            cursor,
            "tagVilWarSetup.DecWarFactions.value",
        )?);
    }

    Ok(VillageWarSetup {
        id,
        war_region_id,
        declare_event_id,
        declare_time,
        start_info_event_id,
        start_info_time,
        start_event_id,
        start_time,
        end_info_event_id,
        end_info_time,
        end_event_id,
        end_time,
        clear_event_id,
        clear_player_time,
        village_region_id,
        region_state,
        declaring_factions,
        is_every_week: None,
    })
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, VillageWarDecodeError> {
    Ok(i32::from_le_bytes(read_exact(source, cursor, field)?))
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, VillageWarDecodeError> {
    Ok(u32::from_le_bytes(read_exact(source, cursor, field)?))
}

fn read_tag_time(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<TagTime, VillageWarDecodeError> {
    let bytes = read_exact::<16>(source, cursor, field)?;
    Ok(TagTime {
        year: u16::from_le_bytes([bytes[0], bytes[1]]),
        month: u16::from_le_bytes([bytes[2], bytes[3]]),
        day_of_week: u16::from_le_bytes([bytes[4], bytes[5]]),
        day: u16::from_le_bytes([bytes[6], bytes[7]]),
        hour: u16::from_le_bytes([bytes[8], bytes[9]]),
        minute: u16::from_le_bytes([bytes[10], bytes[11]]),
        second: u16::from_le_bytes([bytes[12], bytes[13]]),
        milliseconds: u16::from_le_bytes([bytes[14], bytes[15]]),
    })
}

fn read_exact<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], VillageWarDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(VillageWarDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // BLOCKED_MISSING_FACT: исходный безразмерный pointer читал за payload;
        // безопасный API не назначает неизвестному UB значение или side effect.
        return Err(VillageWarDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor = end;
    Ok(bytes.try_into().expect("проверен срез точной длины"))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp

// ============================================================================
// FUNCTION: CVillageWarSys::GetVilRegionIDByTime
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:234
// RVA: 0x0005DA90
//
// IMPLEMENTED выше: exact-key lookup и исходный 0-sentinel при miss.
//

// ============================================================================
// FUNCTION: CVillageWarSys::IsAlreadyDeclarForWar
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:242
// RVA: 0x0005DAC0
//
// IMPLEMENTED выше: exact-key lookup и ordered faction membership без weekly
// gate, как в деревенском варианте.
//

// ============================================================================
// FUNCTION: CVillageWarSys::GetWarStartTime
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:280
// RVA: 0x0005DB80
//
// IMPLEMENTED выше: первый map-order war-region schedule со strict now < end,
// исходный 0-sentinel и packed minute-формула с 32-bit wrapping.
//

// ============================================================================
// FUNCTION: CVillageWarSys::InitiVillRegionState
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:70
// RVA: 0x0005E310
//
// IMPLEMENTED выше: map-order, closed declare/end interval, отдельный
// region-then-proxy lookup обоих IDs, virtual ReSetWarState и последующее
// `m_btCountry` village -> war. Exact EXE: vcall `0x0045E3EC`, byte copy
// `0x0045E44B..0x0045E44E` по PDB-offset `CRegion +0x74`.
//

// ============================================================================
// FUNCTION: CVillageWarSys::UpdateApplyWarFacs
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:102
// RVA: 0x0005E480
//
// IMPLEMENTED выше: signed war ID/count, hit-only clear/append, main-region-only
// lookup и virtual UpdateContendPlayer. Exact EXE задаёт miss `false` по
// `0x0045E4BF`, hit `true` по `0x0045E59B` и vcall `0x0045E592` slot `+0x104`.
//

// ============================================================================
// FUNCTION: CVillageWarSys::OnAttackVillageOutTime
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0005E5B0
//
// IMPLEMENTED выше: main-region-only lookup war-region и virtual OnWarTimeOut. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CVillageWarSys::OnClearPlayer
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0005E640
//
// IMPLEMENTED выше: main-only village-region, сообщение GS0127 и StartClearPlayerOut(60000). STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CVillageWarSys::GetWarNameForDeclar
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:256
// RVA: 0x0005E830
//
// IMPLEMENTED выше: active-state и faction gates в map-order, main-then-proxy
// lookup war-region и продолжение после miss. bool + output string заменены
// на Option<String>; STL string/tree/list cleanup удалён как технический шум.
//

// ============================================================================
// FUNCTION: CVillageWarSys::CVillageWarSys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:24
// RVA: 0x0005EC10
// ADDRESS: 0045ec10
// PROTOTYPE: undefined __thiscall CVillageWarSys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVillageWarSys::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:34
// RVA: 0x0005ECA0
// ADDRESS: 0045eca0
// PROTOTYPE: CVillageWarSys * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVillageWarSys::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:43
// RVA: 0x0005ED10
// ADDRESS: 0045ed10
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetVilWarSys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:315
// RVA: 0x0005ED40
// ADDRESS: 0045ed40
// PROTOTYPE: CVillageWarSys * __cdecl GetVilWarSys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVillageWarSys::DecordFromByteArray
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp:51
// RVA: 0x0005F320
//
// IMPLEMENTED выше: clear, signed schedule/list counts, полный `0x8C` prefix,
// ordered factions и map assignment для каждой записи. Exact EXE подтверждает
// внешний цикл `0x0045F4D3..0x0045F4DF` и безусловный `true` `0x0045F4EF`.
//

// ============================================================================
// FUNCTION: CVillageWarSys::OnDelcareWar
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0005F510
//
// IMPLEMENTED выше: state DUTH, два region-then-proxy lookup, перенос owner/country и OnWarDeclare. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CVillageWarSys::OnAttackVillageStart
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0005F680
//
// IMPLEMENTED выше: state Fight, region-then-proxy lookup war-region и OnWarStart. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CVillageWarSys::OnAttackVillageEnd
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x0005F740
//
// IMPLEMENTED выше: clear faction list, Fight guard, region-then-proxy lookup и OnWarEnd. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: zcalloc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\villagewarsys.cpp
// RVA: 0x00213CD0
// ADDRESS: 00613cd0
// PROTOTYPE: undefined __cdecl zcalloc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
