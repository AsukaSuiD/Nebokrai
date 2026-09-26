//! Владелец GameServer-снимка городских войн `CAttackCitySys` в Zone
//! `activities/`. Исходник PDB: `organizingsystem/attackcitysys.cpp`; сверка по
//! точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! Wire начинается с signed `long` количества; каждый положительный элемент
//! копирует prefix `tagAttackCityTime` длиной `0xB0`, затем signed count и
//! ordered faction IDs. Четыре bytes stateless allocator-base внутри prefix
//! сознательно читаются и отбрасываются; недоставленный weekly flag выражен
//! `None`, а не выдуманным значением; обрыв short buffer возвращает
//! `UnexpectedEnd`, сохраняя пройденный cursor и полные записи, вместо старого
//! UB. Фазы сохраняют `DUTH=1`/`Mass=2`/`Fight=3` и различия поиска:
//! declare/start/timeout/end/mass допускают proxy, а clear/refresh — только
//! основной map; timeout действует лишь в `Fight`, clear — лишь в `Mass`.
//! Оба membership query в оригинале проверяли `bIsEveryWeek`, копируемый
//! assignment-ом из неинициализированного stack — UB не воспроизводится, оба
//! query используют переданный список фракций. Остальные функции owner-а —
//! `UNKNOWN`; сырой `CGame/CServerRegion` остаётся внешним context-контрактом.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#войны-и-расписания

use std::collections::BTreeMap;
use thiserror::Error;

use nebokrai_shared::protocol::LegacyReader;
use nebokrai_shared::values::TagTime;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AttackCityDecodeError {
    #[error("поле {field} с offset {offset} требует {needed} байт, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug)]
pub struct AttackCityTime {
    pub time: i32,
    pub city_region_id: i32,
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
    pub mass_event_id: u32,
    pub mass_time: TagTime,
    pub clear_event_id: u32,
    pub clear_player_time: TagTime,
    pub refresh_event_id: u32,
    pub refresh_region_time: TagTime,
    pub region_state: i32,
    pub declaring_factions: Vec<i32>,
}

#[derive(Default)]
pub struct CAttackCitySys {
    pub attacks: BTreeMap<i32, AttackCityTime>,
}

pub trait AttackCityRegionContext {
    type Region: Copy;

    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Вызывает virtual `ReSetWarState(long, eCityState)` найденного региона.
    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32);
}

pub trait AttackCityPhaseContext {
    type Region: Copy;

    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Ищет только non-null entry в `CGame::s_mapRegion`, без proxy fallback.
    fn find_server_region(&mut self, region_id: i32) -> Option<Self::Region>;

    fn on_war_declare(&mut self, region: Self::Region, war_number: i32);
    fn on_war_start(&mut self, region: Self::Region, war_number: i32);
    fn on_war_time_out(&mut self, region: Self::Region, war_number: i32);
    fn on_war_end(&mut self, region: Self::Region, war_number: i32);
    fn on_war_mass(&mut self, region: Self::Region, war_number: i32);
    fn on_clear_other_player(&mut self, region: Self::Region, war_number: i32);
    fn on_refresh_region(&mut self, region: Self::Region, war_number: i32);
}

pub trait AttackCityNameContext {
    /// Возвращает копию имени после lookup `s_mapRegion`, затем proxy fallback.
    fn region_name_then_proxy(&mut self, region_id: i32) -> Option<String>;
}

const CITY_STATE_DECLARE: i32 = 1;
const CITY_STATE_MASS: i32 = 2;
const CITY_STATE_FIGHT: i32 = 3;

impl CAttackCitySys {
    /// Очищает прежний map и принимает полный payload `SI_ATTACKCITYSYS_SETUP`.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), AttackCityDecodeError> {
        self.attacks.clear();
        let schedule_count = read_i32(source, cursor, "m_Attacks.size")?;
        for _ in 0..schedule_count.max(0) {
            let setup = decode_setup(source, cursor)?;
            self.attacks.insert(setup.time, setup);
        }
        tracing::trace!(
            schedules = self.attacks.len(),
            "расписание осады города декодировано"
        );
        Ok(())
    }

    /// Проецирует active schedule state в найденные server/proxy regions.
    pub fn init_city_region_state<Context: AttackCityRegionContext>(&self, context: &mut Context) {
        self.init_city_region_state_at(TagTime::local_now(), context)
    }

    pub fn init_city_region_state_at<Context: AttackCityRegionContext>(
        &self,
        now: TagTime,
        context: &mut Context,
    ) {
        let mut active_schedules = 0usize;
        let mut projected_regions = 0usize;
        for (&war_number, setup) in &self.attacks {
            if !now.legacy_ge(setup.declare_time) || !now.legacy_le(setup.end_time) {
                continue;
            }
            active_schedules += 1;

            if let Some(region) = context.find_region_then_proxy(setup.city_region_id) {
                context.reset_war_state(region, war_number, setup.region_state);
                projected_regions += 1;
            }
        }
        tracing::trace!(
            schedules = self.attacks.len(),
            active_schedules,
            projected_regions,
            "состояние регионов осады города инициализировано"
        );
    }

    /// Заменяет ordered faction list одного schedule и обновляет contenders региона.
    pub fn update_apply_war_factions(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<Option<i32>, AttackCityDecodeError> {
        let war_number = read_i32(source, cursor, "UpdateApplyWarFacs.war_number")?;
        let Some(setup) = self.attacks.get_mut(&war_number) else {
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
        Ok(Some(setup.city_region_id))
    }

    /// Проверяет authoritative ordered faction-list текущего schedule.
    pub fn is_already_declar_for_war(&self, war_number: i32, faction_id: i32) -> bool {
        let Some(setup) = self.attacks.get(&war_number) else {
            return false;
        };
        setup.declaring_factions.contains(&faction_id)
    }

    /// Возвращает legacy packed start-time первого ещё не завершённого schedule.
    pub fn get_war_start_time(&self, city_region_id: i32) -> i32 {
        self.get_war_start_time_at(city_region_id, TagTime::local_now())
    }

    pub fn get_war_start_time_at(&self, city_region_id: i32, now: TagTime) -> i32 {
        self.attacks
            .values()
            .find(|setup| setup.city_region_id == city_region_id && now.legacy_lt(setup.end_time))
            .map_or(0, |setup| legacy_packed_war_start(setup.start_time))
    }

    /// Возвращает имя первого schedule, содержащего faction ID.
    pub fn get_war_name_for_declar<Context: AttackCityNameContext>(
        &self,
        faction_id: i32,
        context: &mut Context,
    ) -> Option<String> {
        self.attacks.values().find_map(|setup| {
            if !setup.declaring_factions.contains(&faction_id) {
                return None;
            }
            context.region_name_then_proxy(setup.city_region_id)
        })
    }

    pub fn on_declar_war<Context: AttackCityPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get_mut(&war_number) else {
            return;
        };
        setup.region_state = CITY_STATE_DECLARE;
        let city_region_id = setup.city_region_id;

        if let Some(region) = context.find_region_then_proxy(city_region_id) {
            context.on_war_declare(region, war_number);
        }
    }

    pub fn on_attack_city_start<Context: AttackCityPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get_mut(&war_number) else {
            return;
        };
        setup.region_state = CITY_STATE_FIGHT;
        let city_region_id = setup.city_region_id;

        if let Some(region) = context.find_region_then_proxy(city_region_id) {
            context.on_war_start(region, war_number);
        }
    }

    pub fn on_attack_city_time_out<Context: AttackCityPhaseContext>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get(&war_number) else {
            return;
        };
        if setup.region_state != CITY_STATE_FIGHT {
            return;
        }

        if let Some(region) = context.find_region_then_proxy(setup.city_region_id) {
            context.on_war_time_out(region, war_number);
        }
    }

    pub fn on_attack_city_end<Context: AttackCityPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get_mut(&war_number) else {
            return;
        };
        setup.declaring_factions.clear();
        let city_region_id = setup.city_region_id;

        if let Some(region) = context.find_region_then_proxy(city_region_id) {
            context.on_war_end(region, war_number);
        }
    }

    pub fn on_mass<Context: AttackCityPhaseContext>(
        &mut self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get_mut(&war_number) else {
            return;
        };
        setup.region_state = CITY_STATE_MASS;
        let city_region_id = setup.city_region_id;

        if let Some(region) = context.find_region_then_proxy(city_region_id) {
            context.on_war_mass(region, war_number);
        }
    }

    pub fn on_clear_other_player<Context: AttackCityPhaseContext>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get(&war_number) else {
            return;
        };
        if setup.region_state != CITY_STATE_MASS {
            return;
        }

        if let Some(region) = context.find_server_region(setup.city_region_id) {
            context.on_clear_other_player(region, war_number);
        }
    }

    pub fn on_refresh_region<Context: AttackCityPhaseContext>(
        &self,
        war_number: i32,
        context: &mut Context,
    ) {
        let Some(setup) = self.attacks.get(&war_number) else {
            return;
        };

        if let Some(region) = context.find_server_region(setup.city_region_id) {
            context.on_refresh_region(region, war_number);
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
) -> Result<AttackCityTime, AttackCityDecodeError> {
    let time = read_i32(source, cursor, "tagAttackCityTime.lTime")?;
    let city_region_id = read_i32(source, cursor, "tagAttackCityTime.lCityRegionID")?;
    let declare_event_id = read_u32(source, cursor, "tagAttackCityTime.lDeclarWarEventID")?;
    let declare_time = read_tag_time(source, cursor, "tagAttackCityTime.DeclarWarTime")?;
    let start_info_event_id = read_u32(source, cursor, "tagAttackCityTime.lStartInfoEventID")?;
    let start_info_time =
        read_tag_time(source, cursor, "tagAttackCityTime.AttackCityStartInfoTime")?;
    let start_event_id = read_u32(source, cursor, "tagAttackCityTime.lStartEventID")?;
    let start_time = read_tag_time(source, cursor, "tagAttackCityTime.AttackCityStartTime")?;
    let end_info_event_id = read_u32(source, cursor, "tagAttackCityTime.lEndInfoEventID")?;
    let end_info_time = read_tag_time(source, cursor, "tagAttackCityTime.AttackCityEndInfoTime")?;
    let end_event_id = read_u32(source, cursor, "tagAttackCityTime.lEndEventID")?;
    let end_time = read_tag_time(source, cursor, "tagAttackCityTime.AttackCityEndTime")?;
    let mass_event_id = read_u32(source, cursor, "tagAttackCityTime.lMassEventID")?;
    let mass_time = read_tag_time(source, cursor, "tagAttackCityTime.MassTime")?;
    let clear_event_id = read_u32(source, cursor, "tagAttackCityTime.lClearEventID")?;
    let clear_player_time = read_tag_time(source, cursor, "tagAttackCityTime.ClearPlayerTime")?;
    let refresh_event_id = read_u32(source, cursor, "tagAttackCityTime.lRefreshEventID")?;
    let refresh_region_time = read_tag_time(source, cursor, "tagAttackCityTime.RefreshRegionTime")?;
    let region_state = read_i32(source, cursor, "tagAttackCityTime.RegionState")?;
    let _allocator_base = read_exact::<4>(
        source,
        cursor,
        "tagAttackCityTime.DecWarFactions allocator-base",
    )?;
    let faction_count = read_i32(source, cursor, "tagAttackCityTime.DecWarFactions.size")?;
    let mut declaring_factions = Vec::new();
    for _ in 0..faction_count.max(0) {
        declaring_factions.push(read_i32(
            source,
            cursor,
            "tagAttackCityTime.DecWarFactions.value",
        )?);
    }

    Ok(AttackCityTime {
        time,
        city_region_id,
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
        mass_event_id,
        mass_time,
        clear_event_id,
        clear_player_time,
        refresh_event_id,
        refresh_region_time,
        region_state,
        declaring_factions,
    })
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, AttackCityDecodeError> {
    let mut reader = attack_city_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| attack_city_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, AttackCityDecodeError> {
    let mut reader = attack_city_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| attack_city_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_tag_time(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<TagTime, AttackCityDecodeError> {
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
) -> Result<[u8; N], AttackCityDecodeError> {
    let mut reader = attack_city_reader(source, *cursor, field, N)?;
    let bytes = reader
        .read_bytes(N)
        .map_err(|block| attack_city_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn attack_city_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, AttackCityDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| AttackCityDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn attack_city_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> AttackCityDecodeError {
    AttackCityDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
