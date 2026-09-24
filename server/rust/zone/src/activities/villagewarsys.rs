//! Владелец GameServer-снимка деревенских войн `CVillageWarSys`, перенесённый в Zone `activities/`.
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
use thiserror::Error;

use nebokrai_shared::protocol::LegacyReader;
use nebokrai_shared::values::TagTime;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum VillageWarDecodeError {
    #[error("поле {field} с offset {offset} требует {needed} байт, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug)]
pub struct VillageWarSetup {
    pub id: i32,
    pub war_region_id: i32,
    pub declare_event_id: u32,
    pub declare_time: TagTime,
    pub start_info_event_id: u32,
    pub start_info_time: TagTime,
    pub start_event_id: u32,
    pub start_time: TagTime,
    pub end_info_event_id: u32,
    pub end_info_time: TagTime,
    pub end_event_id: u32,
    pub end_time: TagTime,
    pub clear_event_id: u32,
    pub clear_player_time: TagTime,
    pub village_region_id: i32,
    pub region_state: i32,
    pub declaring_factions: Vec<i32>,
    pub is_every_week: Option<i32>,
}

#[derive(Default)]
pub struct CVillageWarSys {
    pub village_wars: BTreeMap<i32, VillageWarSetup>,
}

pub trait VillageWarRegionContext {
    type Region: Copy;

    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Вызывает virtual `ReSetWarState(long, eCityState)` найденного региона.
    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32);

    fn region_country(&self, region: Self::Region) -> u8;

    fn set_region_country(&mut self, region: Self::Region, country: u8);
}

pub trait VillageWarPhaseContext {
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

pub trait VillageWarNameContext {
    /// Возвращает копию имени после lookup `s_mapRegion`, затем proxy fallback.
    fn region_name_then_proxy(&mut self, region_id: i32) -> Option<String>;
}

const CITY_STATE_DECLARE: i32 = 1;
const CITY_STATE_FIGHT: i32 = 3;

impl CVillageWarSys {
    /// Очищает прежний map и принимает полный payload `SI_VILLAGEWARSYS_SETUP`.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), VillageWarDecodeError> {
        self.village_wars.clear();
        let schedule_count = read_i32(source, cursor, "m_VillageWars.size")?;
        for _ in 0..schedule_count.max(0) {
            let setup = decode_setup(source, cursor)?;
            self.village_wars.insert(setup.id, setup);
        }
        tracing::trace!(
            schedules = self.village_wars.len(),
            "расписание деревенской войны декодировано"
        );
        Ok(())
    }

    /// Проецирует active schedule state в найденные server/proxy regions.
    pub fn init_village_region_state<Context: VillageWarRegionContext>(
        &self,
        context: &mut Context,
    ) {
        self.init_village_region_state_at(TagTime::local_now(), context)
    }

    pub fn init_village_region_state_at<Context: VillageWarRegionContext>(
        &self,
        now: TagTime,
        context: &mut Context,
    ) {
        let mut active_schedules = 0usize;
        let mut projected_regions = 0usize;
        let mut projected_countries = 0usize;
        for (&war_number, setup) in &self.village_wars {
            if !now.legacy_ge(setup.declare_time) || !now.legacy_le(setup.end_time) {
                continue;
            }
            active_schedules += 1;

            let war_region = context.find_region_then_proxy(setup.war_region_id);
            if let Some(region) = war_region {
                context.reset_war_state(region, war_number, setup.region_state);
                projected_regions += 1;
            }

            let village_region = context.find_region_then_proxy(setup.village_region_id);
            if let (Some(war_region), Some(village_region)) = (war_region, village_region) {
                let village_country = context.region_country(village_region);
                context.set_region_country(war_region, village_country);
                projected_countries += 1;
            }
        }
        tracing::trace!(
            schedules = self.village_wars.len(),
            active_schedules,
            projected_regions,
            projected_countries,
            "состояние регионов деревенской войны инициализировано"
        );
    }

    /// Заменяет ordered faction list одного schedule и обновляет contenders региона.
    pub fn update_apply_war_factions(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<Option<i32>, VillageWarDecodeError> {
        let war_number = read_i32(source, cursor, "UpdateApplyWarFacs.war_number")?;
        let Some(setup) = self.village_wars.get_mut(&war_number) else {
            return Ok(None);
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
        Ok(Some(setup.war_region_id))
    }

    /// Возвращает ID связанной деревни либо исходный sentinel `0`.
    pub fn get_village_region_id_by_time(&self, war_number: i32) -> i32 {
        self.village_wars
            .get(&war_number)
            .map_or(0, |setup| setup.village_region_id)
    }

    pub fn is_already_declar_for_war(&self, war_number: i32, faction_id: i32) -> bool {
        self.village_wars
            .get(&war_number)
            .is_some_and(|setup| setup.declaring_factions.contains(&faction_id))
    }

    /// Возвращает legacy packed start-time первого ещё не завершённого schedule.
    pub fn get_war_start_time(&self, war_region_id: i32) -> i32 {
        self.get_war_start_time_at(war_region_id, TagTime::local_now())
    }

    pub fn get_war_start_time_at(&self, war_region_id: i32, now: TagTime) -> i32 {
        self.village_wars
            .values()
            .find(|setup| setup.war_region_id == war_region_id && now.legacy_lt(setup.end_time))
            .map_or(0, |setup| legacy_packed_war_start(setup.start_time))
    }

    /// Возвращает имя первого active schedule, содержащего faction ID.
    pub fn get_war_name_for_declar<Context: VillageWarNameContext>(
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

    pub fn on_delcare_war<Context: VillageWarPhaseContext>(
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

    pub fn on_attack_village_start<Context: VillageWarPhaseContext>(
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

    pub fn on_attack_village_out_time<Context: VillageWarPhaseContext>(
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

    pub fn on_attack_village_end<Context: VillageWarPhaseContext>(
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

    pub fn on_clear_player<Context: VillageWarPhaseContext>(
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
    let mut reader = village_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| village_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, VillageWarDecodeError> {
    let mut reader = village_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| village_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_tag_time(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<TagTime, VillageWarDecodeError> {
    let bytes = read_exact::<16>(source, cursor, field)?;
    let mut reader = LegacyReader::new(&bytes);
    Ok(TagTime {
        year: reader.read_u16().expect("проверен TagTime"),
        month: reader.read_u16().expect("проверен TagTime"),
        day_of_week: reader.read_u16().expect("проверен TagTime"),
        day: reader.read_u16().expect("проверен TagTime"),
        hour: reader.read_u16().expect("проверен TagTime"),
        minute: reader.read_u16().expect("проверен TagTime"),
        second: reader.read_u16().expect("проверен TagTime"),
        milliseconds: reader.read_u16().expect("проверен TagTime"),
    })
}

fn read_exact<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], VillageWarDecodeError> {
    let mut reader = village_reader(source, *cursor, field, N)?;
    let bytes = reader
        .read_bytes(N)
        .map_err(|block| village_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn village_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, VillageWarDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| VillageWarDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn village_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> VillageWarDecodeError {
    VillageWarDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
