//! Владелец GameServer-снимка городских войн `CAttackCitySys`.
//!
//! Snapshot/setup (`0x00060E40`), initial state (`0x000600F0`), faction update
//! (`0x000601F0`) и фазовые callbacks `0x00060320`,
//! `0x00061040..0x00061420` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; schedule queries `0x0005FAD0..0x0005FBC0` и
//! `0x000603C0` — `IMPLEMENTED`, с локальным membership-блоком ниже. Остальные
//! функции owner-а остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Исходник PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp`;
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//!
//! Wire начинается с signed `long` количества. Для каждого положительного
//! элемента исходник копирует prefix `tagAttackCityTime` длиной `0xB0`, затем
//! читает signed count и ordered faction IDs. PDB подтверждает восемь пар
//! `event ID + tagTime`, signed schedule/city/state и `std::list` с offset
//! `0xAC`; weekly flag по offset `0xB8` в snapshot не входит. `BTreeMap` и
//! `Vec` заменяют STL с тем же key/order-контрактом; повторный `lTime` заменяет
//! ранее декодированную запись, как `map::operator[]` с присваиванием.
//!
//! Четыре bytes stateless allocator-base внутри prefix сознательно читаются и
//! отбрасываются. Event IDs сохраняются как wire `u32`, но в этом GameServer
//! owner-е ни один достигнутый consumer их не читает. Недоставленный weekly
//! flag выражен `None`, а не выдуманным значением. Exact EXE показывает внешний
//! цикл `0x00461009..0x00461015` и общий `mov al,1` по `0x00461025`.
//! Безразмерный C++ pointer на short buffer уходил в неизвестный UB; safe Rust
//! возвращает локальный `UnexpectedEnd`, сохраняя уже пройденный cursor и уже
//! вставленные полные записи. Static `m_Attacks` остаётся у единственного
//! Rust-owner-а; STL tree/list allocation, SEH и cleanup не восстанавливаются.
//!
//! Initial-state owner в map-order выбирает только closed interval
//! `DeclarWarTime <= now <= AttackCityEndTime`. City ID ищется сначала в
//! `CGame::s_mapRegion`, а при miss/null — через `FindProxyRegion`; найденный
//! region получает virtual `ReSetWarState(war number, state)`. Exact EXE
//! подтверждает два stack-аргумента и vtable slot `+0xE8`. Сырой
//! `CGame/CServerRegion` остаётся внешним context-контрактом до своего прохода.
//! Faction-update всегда читает war ID, но при miss сразу возвращает `false`,
//! не читая count. Hit очищает прежний list, принимает signed count и ordered
//! IDs, затем ищет city region только в `s_mapRegion` без proxy fallback и при
//! non-null вызывает virtual `UpdateContendPlayer()` slot `+0x104`. Safe short
//! buffer сохраняет уже выполненные clear/append и cursor, не назначая старому
//! UB дополнительный результат.
//!
//! Фазы сохраняют значения `DUTH=1`, `Mass=2`, `Fight=3`, порядок мутации
//! schedule перед lookup и точные различия поиска: declare/start/timeout/end/mass
//! допускают proxy, а clear/refresh — только основной map. Timeout действует
//! лишь в `Fight`, clear лишь в `Mass`; end очищает faction list, но не меняет
//! schedule state. Vtable slots `+0x78..+0x90` сверены с точными сигнатурами
//! `CServerRegion` из PDB; context выражает ещё не реализованный region-owner.
//!
//! Query-поверхность сохраняет map-order, strict `now < end`, старую packed
//! minute-формулу с x86 wrapping и main-then-proxy lookup имени. Выходной
//! `std::string& + bool` представлен `Option<String>`. Оба нативных membership
//! query проверяли `bIsEveryWeek`, но setup-кодирует только `0xB0`-префикс и
//! ordered faction-list: конструктор не задаёт DWORD `+0xB8`, а assignment
//! копирует туда неинициализированный stack. Без доказанного внешнего эффекта
//! этот UB не воспроизводится: оба query используют переданный список фракций.

use std::collections::BTreeMap;
use thiserror::Error;

use crate::public::date::TagTime;
use crate::gameserver::appserver::legacycodec::LegacyReader;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum AttackCityDecodeError {
    #[error("поле {field} с offset {offset} требует {needed} байт, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct AttackCityTime {
    pub(crate) time: i32,
    pub(crate) city_region_id: i32,
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
    pub(crate) mass_event_id: u32,
    pub(crate) mass_time: TagTime,
    pub(crate) clear_event_id: u32,
    pub(crate) clear_player_time: TagTime,
    pub(crate) refresh_event_id: u32,
    pub(crate) refresh_region_time: TagTime,
    pub(crate) region_state: i32,
    pub(crate) declaring_factions: Vec<i32>,
}

#[derive(Default)]
pub(crate) struct CAttackCitySys {
    pub(crate) attacks: BTreeMap<i32, AttackCityTime>,
}

pub(crate) trait AttackCityRegionContext {
    type Region: Copy;

    /// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`.
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;

    /// Вызывает virtual `ReSetWarState(long, eCityState)` найденного региона.
    fn reset_war_state(&mut self, region: Self::Region, war_number: i32, state: i32);
}

pub(crate) trait AttackCityPhaseContext {
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

pub(crate) trait AttackCityNameContext {
    /// Возвращает копию имени после lookup `s_mapRegion`, затем proxy fallback.
    fn region_name_then_proxy(&mut self, region_id: i32) -> Option<String>;
}

const CITY_STATE_DECLARE: i32 = 1;
const CITY_STATE_MASS: i32 = 2;
const CITY_STATE_FIGHT: i32 = 3;

impl CAttackCitySys {
    /// Очищает прежний map и принимает полный payload `SI_ATTACKCITYSYS_SETUP`.
    pub(crate) fn decord_from_byte_array(
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
        tracing::trace!(schedules = self.attacks.len(), "расписание осады города декодировано");
        Ok(())
    }

    /// Проецирует active schedule state в найденные server/proxy regions.
    pub(crate) fn init_city_region_state<Context: AttackCityRegionContext>(
        &self,
        context: &mut Context,
    ) {
        self.init_city_region_state_at(TagTime::local_now(), context)
    }

    pub(crate) fn init_city_region_state_at<Context: AttackCityRegionContext>(
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
        tracing::trace!(schedules = self.attacks.len(), active_schedules, projected_regions, "состояние регионов осады города инициализировано");
    }

    /// Заменяет ordered faction list одного schedule и обновляет contenders региона.
    pub(crate) fn update_apply_war_factions(
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
    pub(crate) fn is_already_declar_for_war(&self, war_number: i32, faction_id: i32) -> bool {
        let Some(setup) = self.attacks.get(&war_number) else {
            return false;
        };
        setup.declaring_factions.contains(&faction_id)
    }

    /// Возвращает legacy packed start-time первого ещё не завершённого schedule.
    pub(crate) fn get_war_start_time(&self, city_region_id: i32) -> i32 {
        self.get_war_start_time_at(city_region_id, TagTime::local_now())
    }

    pub(crate) fn get_war_start_time_at(&self, city_region_id: i32, now: TagTime) -> i32 {
        self.attacks
            .values()
            .find(|setup| setup.city_region_id == city_region_id && now.legacy_lt(setup.end_time))
            .map_or(0, |setup| legacy_packed_war_start(setup.start_time))
    }

    /// Возвращает имя первого schedule, содержащего faction ID.
    pub(crate) fn get_war_name_for_declar<Context: AttackCityNameContext>(
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

    pub(crate) fn on_declar_war<Context: AttackCityPhaseContext>(
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

    pub(crate) fn on_attack_city_start<Context: AttackCityPhaseContext>(
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

    pub(crate) fn on_attack_city_time_out<Context: AttackCityPhaseContext>(
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

    pub(crate) fn on_attack_city_end<Context: AttackCityPhaseContext>(
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

    pub(crate) fn on_mass<Context: AttackCityPhaseContext>(
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

    pub(crate) fn on_clear_other_player<Context: AttackCityPhaseContext>(
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

    pub(crate) fn on_refresh_region<Context: AttackCityPhaseContext>(
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
    let value = reader.read_i32().map_err(|block| attack_city_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, AttackCityDecodeError> {
    let mut reader = attack_city_reader(source, *cursor, field, 4)?;
    let value = reader.read_u32().map_err(|block| attack_city_error(field, block))?;
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

fn attack_city_reader<'source>(source: &'source [u8], cursor: usize, field: &'static str, needed: usize) -> Result<LegacyReader<'source>, AttackCityDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| AttackCityDecodeError::UnexpectedEnd { field, offset: block.offset, needed, available: block.available })
}

fn attack_city_error(field: &'static str, block: crate::gameserver::appserver::legacycodec::LegacyReadBlock) -> AttackCityDecodeError {
    AttackCityDecodeError::UnexpectedEnd { field, offset: block.offset, needed: block.needed, available: block.available }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp

// ============================================================================
// FUNCTION: CAttackCitySys::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:33
// RVA: 0x0005F7F0
// ADDRESS: 0045f7f0
// PROTOTYPE: CAttackCitySys * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetAttackCitySys
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:336
// RVA: 0x0005F810
// ADDRESS: 0045f810
// PROTOTYPE: CAttackCitySys * __cdecl GetAttackCitySys(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAttackCitySys::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:43
// RVA: 0x0005F8D0
// ADDRESS: 0045f8d0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAttackCitySys::IsAlreadyDeclarForWar
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:233
// RVA: 0x0005FAD0
//
// IMPLEMENTED выше: exact map lookup и ordered membership-list. Нативный gate
// читает неинициализированный `bIsEveryWeek +0xB8`: DecordFromByteArray
// принимает только `0xB0` prefix, ctor поле не задаёт, operator= лишь копирует
// stack-мусор. Rust безопасно не воспроизводит этот внутренний UB и использует
// authoritative список, который WorldServer передаёт startup/update путями.
//

// ============================================================================
// FUNCTION: CAttackCitySys::GetWarStartTime
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:291
// RVA: 0x0005FBC0
//
// IMPLEMENTED выше: первый map-order city schedule со strict now < end,
// исходный 0-sentinel и packed minute-формула с 32-bit wrapping.
//

// ============================================================================
// FUNCTION: CAttackCitySys::InitiCityRegionState
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:67
// RVA: 0x000600F0
//
// IMPLEMENTED выше: map-order, closed declare/end interval, region-then-proxy
// lookup city ID и virtual ReSetWarState. Exact EXE подтверждает key/state
// pushes и vcall `0x004601B1..0x004601C0` через slot `+0xE8`.
//

// ============================================================================
// FUNCTION: CAttackCitySys::UpdateApplyWarFacs
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:90
// RVA: 0x000601F0
//
// IMPLEMENTED выше: signed war ID/count, hit-only clear/append, main-region-only
// lookup и virtual UpdateContendPlayer. Exact EXE задаёт miss `false` по
// `0x0046022D`, hit `true` по `0x0046030D` и vcall `0x00460305` slot `+0x104`.
//

// ============================================================================
// FUNCTION: CAttackCitySys::OnRefreshRegion
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00060320
//
// IMPLEMENTED выше: main-region-only lookup и virtual OnRefreshRegion. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CAttackCitySys::GetWarNameForDeclar
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:247
// RVA: 0x000603C0
//
// IMPLEMENTED выше: faction gate в map-order, main-then-proxy lookup
// city-region и продолжение после miss. Нативный `bIsEveryWeek` gate здесь
// читает тот же недоставленный неинициализированный DWORD и безопасно заменён
// authoritative faction-list; state gate в точном EXE отсутствует. bool +
// output string заменены на Option<String>.
//

// ============================================================================
// FUNCTION: CAttackCitySys::DecordFromByteArray
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp:48
// RVA: 0x00060E40
//
// IMPLEMENTED выше: clear, signed schedule/list counts, полный `0xB0` prefix,
// ordered factions и map assignment для каждой записи. Exact EXE подтверждает
// внешний цикл `0x00461009..0x00461015` и безусловный `true` `0x00461025`.
//

// ============================================================================
// FUNCTION: CAttackCitySys::OnAttackCityStart
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00061040
//
// IMPLEMENTED выше: state Fight, region-then-proxy lookup и virtual OnWarStart. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CAttackCitySys::OnAttackCityTimeOut
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00061110
//
// IMPLEMENTED выше: только state Fight, region-then-proxy lookup и virtual OnWarTimeOut. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CAttackCitySys::OnAttackCityEnd
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x000611B0
//
// IMPLEMENTED выше: clear faction list, region-then-proxy lookup и virtual OnWarEnd. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CAttackCitySys::OnDeclarWar
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00061280
//
// IMPLEMENTED выше: state DUTH, region-then-proxy lookup и virtual OnWarDeclare. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CAttackCitySys::OnMass
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00061350
//
// IMPLEMENTED выше: state Mass, region-then-proxy lookup и virtual OnWarMass. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: CAttackCitySys::OnClearOtherPlayer
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x00061420
//
// IMPLEMENTED выше: только state Mass, main-region-only lookup и virtual OnClearOtherPlayer. STL lookup/cleanup не имеют отдельной
// Rust-семантики; miss/null сохраняют исходный no-op.

// ============================================================================
// FUNCTION: Catch@005be3c9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp
// RVA: 0x001BE3C9
// ADDRESS: 005be3c9
// PROTOTYPE: undefined Catch@005be3c9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005be639
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp
// RVA: 0x001BE639
// ADDRESS: 005be639
// PROTOTYPE: undefined Catch@005be639()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005be7a6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\organizingsystem\attackcitysys.cpp
// RVA: 0x001BE7A6
// ADDRESS: 005be7a6
// PROTOTYPE: undefined Catch@005be7a6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
