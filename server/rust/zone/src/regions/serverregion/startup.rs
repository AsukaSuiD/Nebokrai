//! Startup decode-семейство `CServerRegion` исторического GameServer (порция
//! 6): полный World -> Game snapshot decoder `DecordFromByteArray`, setup
//! decoder `DecordSetupFromByteArray` с return-setup prefix, exact lookup
//! `FindForbidGood` и resource-обёртки `Save/New/Load`. Исходный владелец —
//! `appserver/serverregion.h/.cpp`. Переходный агрегат `CServerRegion` и
//! доменные `CNpc`/`CMonster` остаются в старом пакете: decode-ядра ниже
//! работают над единым store-швом `ServerRegionDecodeStore` (реализация в
//! `appserver/serverregion.rs` рядом с store-швами spawn setup) и повторно
//! используют spawn store-трейты порции 2. Pub(crate)-обёртки агрегата и
//! оркестр `servermessage`/`game` не правятся: обёртка делегирует ядру,
//! log/send/guard остаются closure-обвязками исходных context-трейтов,
//! trace-обвязки — у переходного владельца batch-ядер. Имя региона передаётся
//! шву byte-exact (`name_bytes`): ANSI-преобразование WINDOWS_1251 остаётся
//! у переходного владельца, где уже есть `encoding_rs`; zone не заводит
//! новой зависимости, а `CRegion` по-прежнему владеет byte-exact именем.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Pub `DecordFromByteArray@CServerRegion` — RVA `0x000858F0`,
//! `DecordSetupFromByteArray` — RVA `0x0007EAC0`, `FindForbidGood` — RVA
//! `0x0007D6A0` из карты стены; машинная сверка тел этой порцией не
//! выполняется, статусы не повышаются. Перенесённые тела сверены
//! statement-в-statement с телами переходного владельца на момент переноса:
//! различия только в pub-видимости, путях модулей и объявленном store-шве.
//!
//! Этой порцией не переносятся: context-трейты spawn-эффектов и
//! `ServerRegionDecodeContext` старого пакета, AI refresh-фасады
//! `refresh_monster_groups*` и weather AI-fragment этого агрегата,
//! lookup-семья NPC (`find_npc_by_name`, accessors, owned-remove/detach),
//! subtype wire-tail декодеры War/City/Country и сериализаторы
//! `AddToByteArray` (`0x00019120`)/`AddRegionParamToByteArray`.

use super::areagrid::AreaGridBlock;
use super::queries::legacy_msvc_npc_hash_traversal;
use super::returnsetup::ServerReturnSetup;
use super::spawnsetup::{
    ServerRegionMonsterRectBlock, ServerRegionMonsterSetup, ServerRegionMonsterSpawnStore,
    ServerRegionMonsterVariant, ServerRegionNpcSetup, ServerRegionNpcSpawnBlock,
    ServerRegionNpcSpawnStore, add_monster_rect_batch, add_npc_batch,
};
use super::weather::{ServerRegionWeather, ServerRegionWeatherOption, ServerRegionWeatherTime};
use crate::regions::region::{
    CRegion, RegionDecodeError, RegionRandomContext, RegionResourceWrite, RegionStorageBlock,
};
use crate::regions::regionparam::RegionParamState;
use crate::skills::skillfactory::CSkillFactory;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use nebokrai_shared::resources::MonsterRegistry;

/// BLOCKED_MISSING_FACT: исходный setup decoder не принимает buffer length;
/// truncated `m_stSetup` в safe Rust останавливается на точной границе.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerReturnSetupInputBlock {
    pub offset: usize,
    pub required: usize,
    pub available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerRegionSetupDecodeError {
    Setup(ServerReturnSetupInputBlock),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerRegionLoadError {
    Region(RegionDecodeError),
    AreaGrid(AreaGridBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerRegionDecodeInputBlock {
    pub field: &'static str,
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerRegionDecodeError {
    Region(RegionDecodeError),
    AreaGrid(AreaGridBlock),
    Setup(ServerRegionSetupDecodeError),
    Input(ServerRegionDecodeInputBlock),
    Npc(ServerRegionNpcSpawnBlock),
    Monster(ServerRegionMonsterRectBlock),
}

/// Read-проекция owned NPC для startup name-cache: поля одной успешно
/// зарегистрированной записи в порядке hash traversal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionNpcNameEntry {
    pub name: Vec<u8>,
    pub tile_x: i32,
    pub tile_y: i32,
}

/// Единый store-шов startup decode-семейства над переходным агрегатом
/// `CServerRegion`: base `CRegion`, identity/flags колонки, area-grid
/// пересоздание, NPC setups и name-cache хранилище, monster setups, weather
/// setup и current counter, return-setup с forbidden goods и param-проекция.
/// Spawn-хранилища повторно используют store-трейты порции 2 как supertrait-ы;
/// реализация живёт в `appserver/serverregion.rs` и не меняет сигнатур методов
/// агрегата.
pub trait ServerRegionDecodeStore:
    ServerRegionNpcSpawnStore + ServerRegionMonsterSpawnStore
{
    fn base_region(&self) -> &CRegion;

    fn base_region_mut(&mut self) -> &mut CRegion;

    fn server_region_id(&self) -> i32;

    fn assign_identity(&mut self, id: i32, country: u8, name_bytes: &[u8]);

    fn assign_region_flags(&mut self, war_region_type: i32, no_pk: bool, no_contribute: bool);

    fn rebuild_area_grid(&mut self, area_width: i32, area_height: i32) -> Result<(), AreaGridBlock>;

    fn clear_npc_setups(&mut self);

    fn owned_npc_ids(&self) -> Vec<i32>;

    fn reset_npc_name_cache(&mut self);

    fn owned_npc_show_list(&self, id: i32) -> Option<bool>;

    fn owned_npc_name_entry(&self, id: i32) -> Option<ServerRegionNpcNameEntry>;

    fn append_npc_name_entry(&mut self, name: &[u8], tile_x: i32, tile_y: i32);

    fn clear_monster_setups(&mut self);

    fn clear_weather_setup(&mut self);

    fn push_weather_time(&mut self, time: ServerRegionWeatherTime);

    fn first_weather_setup_time(&self) -> Option<i32>;

    fn set_current_weather_count(&mut self, count: i32);

    fn assign_return_setup(&mut self, setup: ServerReturnSetup);

    fn clear_forbid_goods(&mut self);

    fn insert_forbid_good(&mut self, value: Vec<u8>);

    fn contains_forbid_good(&self, name: &[u8]) -> bool;

    fn assign_param(&mut self, param: RegionParamState);
}

/// Ядро полного snapshot decoder-а `DecordFromByteArray` (`0x000858F0`) в
/// исходном порядке: base region, war-region-type/no_pk/no_contribute, имя,
/// area-grid, NPC, NPC-name cache, monster rectangles, weather, setup и
/// `tagRegionParam`. `after_npc_entry` вызывается на исходном месте
/// virtual-хвоста `AddObject` каждого NPC; часовой расход NPC принудительно
/// `|_| 0` (wire lTime всегда 0, RNG/clock не расходуется), а monster setup
/// получает одно чтение `now_millis` до `AddMonsterRect` с
/// `last_reset_time_ms = now - start_time`. Log/send/guard-обвязки приходят
/// closure-параметрами caller-а и пробрасываются в batch-ядра без смены порядка.
#[allow(
    clippy::too_many_arguments,
    reason = "literal startup decoder сохраняет исходные аргументы и объявленные context-швы переходного агрегата"
)]
pub fn decord_server_region_from_byte_array_with_npc_entry<Store, Context>(
    store: &mut Store,
    source: &[u8],
    cursor: &mut usize,
    include_child: bool,
    area_width: i32,
    area_height: i32,
    monster_registry: &MonsterRegistry,
    skill_factory: &CSkillFactory,
    context: &mut Context,
    mut now_millis: impl FnMut(&mut Context) -> u32,
    mut after_npc_entry: impl FnMut(&mut Store, i32, &mut Context),
    mut on_npc_position_failure: impl FnMut(&[u8], &mut Context),
    mut on_monster_variant_failure: impl FnMut(i32, i32, &mut Context),
    mut on_monster_position_failure: impl FnMut(&[u8], &mut Context),
    mut on_guard_monster: impl FnMut(i32, &mut Context),
    mut on_guard_index: impl FnMut(i32, &mut Context),
    mut on_monster_entry: impl FnMut(&Store, &Store::Monster, &mut Context),
) -> Result<bool, ServerRegionDecodeError>
where
    Store: ServerRegionDecodeStore,
    Context: RegionRandomContext,
{
    store
        .base_region_mut()
        .decord_from_byte_array(source, cursor, include_child)
        .map_err(ServerRegionDecodeError::Region)?;
    let (id, country, name) = {
        let region = store.base_region();
        (
            region.get_id(),
            region
                .country()
                .expect("успешный CRegion decoder назначает country"),
            region.get_name().to_vec(),
        )
    };
    store.assign_identity(id, country, &name);

    let war_region_type = read_server_region_i32(source, cursor, "m_WarRegionType")
        .map_err(ServerRegionDecodeError::Input)?;
    let no_pk = read_server_region_u8(source, cursor, "m_bNoPk")
        .map_err(ServerRegionDecodeError::Input)?
        != 0;
    let no_contribute = read_server_region_u8(source, cursor, "m_bNoContribute")
        .map_err(ServerRegionDecodeError::Input)?
        != 0;
    store.assign_region_flags(war_region_type, no_pk, no_contribute);

    store
        .rebuild_area_grid(area_width, area_height)
        .map_err(ServerRegionDecodeError::AreaGrid)?;

    store.clear_npc_setups();
    let npc_count = read_server_region_i32(source, cursor, "m_listNpc.size")
        .map_err(ServerRegionDecodeError::Input)?;
    for _ in 0..npc_count.max(0) {
        let header = read_server_region_bytes(source, cursor, 0x24, "m_listNpc[]")
            .map_err(ServerRegionDecodeError::Input)?;
        let name = read_server_region_c_string(source, cursor, "tagNpc.strName")
            .map_err(ServerRegionDecodeError::Input)?;
        let script = read_server_region_c_string(source, cursor, "tagNpc.strScript")
            .map_err(ServerRegionDecodeError::Input)?;
        let setup = ServerRegionNpcSetup {
            show_list: header[0] != 0,
            picture_id: read_server_region_i32_at(header, 0x04),
            left: read_server_region_i32_at(header, 0x08),
            top: read_server_region_i32_at(header, 0x0C),
            right: read_server_region_i32_at(header, 0x10),
            bottom: read_server_region_i32_at(header, 0x14),
            count: read_server_region_i32_at(header, 0x18),
            direction: read_server_region_i32_at(header, 0x1C),
            // Оригинал безусловно обнуляет wire `lTime` до `AddNpc`.
            time: 0,
            name,
            script,
        };
        // Wire lTime принудительно равен нулю, поэтому AddNpc не читает
        // clock; `now_ms` не участвует ни в одном NPC side effect.
        add_npc_batch(
            store,
            &setup,
            true,
            false,
            area_width,
            area_height,
            context,
            |_| 0,
            &mut after_npc_entry,
            |_, _| {},
            &mut on_npc_position_failure,
        )
        .map_err(ServerRegionDecodeError::Npc)?;
    }

    // Compatibility quirk: decoder сбрасывает count, но не очищает bytes
    // прежнего cache-vector перед новым append.
    store.reset_npc_name_cache();
    for npc_id in legacy_msvc_npc_hash_traversal(store.owned_npc_ids()) {
        if !store
            .owned_npc_show_list(npc_id)
            .expect("hash traversal построен из owned NPC keys")
        {
            continue;
        }
        let entry = store
            .owned_npc_name_entry(npc_id)
            .expect("успешно зарегистрированный startup NPC имеет координаты");
        store.append_npc_name_entry(&entry.name, entry.tile_x, entry.tile_y);
    }

    store.clear_monster_setups();
    let monster_count = read_server_region_i32(source, cursor, "m_listMonster.size")
        .map_err(ServerRegionDecodeError::Input)?;
    for _ in 0..monster_count.max(0) {
        let header = read_server_region_bytes(source, cursor, 0x24, "m_listMonster[]")
            .map_err(ServerRegionDecodeError::Input)?;
        let variant_count =
            read_server_region_i32(source, cursor, "tagMonster.vectorMonsterList.size")
                .map_err(ServerRegionDecodeError::Input)?;
        let mut variants = Vec::new();
        for _ in 0..variant_count.max(0) {
            let prefix =
                read_server_region_bytes(source, cursor, 0x22, "tagMonster.vectorMonsterList[]")
                    .map_err(ServerRegionDecodeError::Input)?;
            variants.push(ServerRegionMonsterVariant {
                cumulative_odds: read_server_region_u16_at(prefix, 0x00),
                sign: read_server_region_u16_at(prefix, 0x02),
                leader_sign: read_server_region_u16_at(prefix, 0x04),
                leader_distance: read_server_region_u16_at(prefix, 0x06),
                name: read_server_region_c_string(source, cursor, "tagMonsterList.strName")
                    .map_err(ServerRegionDecodeError::Input)?,
                script: read_server_region_c_string(source, cursor, "tagMonsterList.strScript")
                    .map_err(ServerRegionDecodeError::Input)?,
            });
        }
        let start_time = read_server_region_i32_at(header, 0x1C);
        let now_ms = now_millis(context);
        let setup = ServerRegionMonsterSetup {
            index: read_server_region_i32_at(header, 0x00),
            left: read_server_region_i32_at(header, 0x04),
            top: read_server_region_i32_at(header, 0x08),
            right: read_server_region_i32_at(header, 0x0C),
            bottom: read_server_region_i32_at(header, 0x10),
            count: read_server_region_i32_at(header, 0x14),
            reset_time: read_server_region_i32_at(header, 0x18),
            start_time,
            direction: read_server_region_i32_at(header, 0x20),
            living_count: 0,
            last_reset_time_ms: now_ms.wrapping_sub(start_time as u32),
            variants,
        };
        let amount = setup.count;
        let region_id = store.server_region_id();
        add_monster_rect_batch(
            store,
            region_id,
            &setup,
            amount,
            true,
            false,
            now_ms,
            area_width,
            area_height,
            monster_registry,
            skill_factory,
            context,
            &mut on_monster_variant_failure,
            &mut on_monster_position_failure,
            &mut on_guard_monster,
            &mut on_guard_index,
            &mut on_monster_entry,
        )
        .map_err(ServerRegionDecodeError::Monster)?;
    }

    store.clear_weather_setup();
    let weather_time_count = read_server_region_i32(source, cursor, "m_vectorWeatherSetup.size")
        .map_err(ServerRegionDecodeError::Input)?;
    for _ in 0..weather_time_count.max(0) {
        let time = read_server_region_i32(source, cursor, "tagWeatherTime.lTime")
            .map_err(ServerRegionDecodeError::Input)?;
        let option_count = read_server_region_i32(source, cursor, "tagWeatherTime.vectorOption.size")
            .map_err(ServerRegionDecodeError::Input)?;
        let mut options = Vec::new();
        for _ in 0..option_count.max(0) {
            let cumulative_odds =
                read_server_region_i32(source, cursor, "tagWeatherTime.tagOption.lOdds")
                    .map_err(ServerRegionDecodeError::Input)?;
            let weather_count = read_server_region_i32(
                source,
                cursor,
                "tagWeatherTime.tagOption.vectorWeather.size",
            )
            .map_err(ServerRegionDecodeError::Input)?;
            let mut weather = Vec::new();
            for _ in 0..weather_count.max(0) {
                weather.push(ServerRegionWeather {
                    weather_index: read_server_region_i32(source, cursor, "tagWeather.lWeatherIndex")
                        .map_err(ServerRegionDecodeError::Input)?,
                    fog_color: read_server_region_u32(source, cursor, "tagWeather.dwFogColor")
                        .map_err(ServerRegionDecodeError::Input)?,
                });
            }
            options.push(ServerRegionWeatherOption {
                cumulative_odds,
                weather,
            });
        }
        store.push_weather_time(ServerRegionWeatherTime { time, options });
        let first_time = store
            .first_weather_setup_time()
            .expect("weather time только что добавлен");
        store.set_current_weather_count(first_time.wrapping_mul(60).wrapping_sub(1));
    }

    decord_server_region_setup_from_byte_array(store, source, cursor, include_child)
        .map_err(ServerRegionDecodeError::Setup)?;
    let param = read_server_region_bytes(source, cursor, 0x24, "m_Param")
        .map_err(ServerRegionDecodeError::Input)?;
    store.assign_param(RegionParamState {
        region_id: read_server_region_i32_at(param, 0x00),
        max_tax_rate: read_server_region_i32_at(param, 0x04),
        current_tax_rate: read_server_region_i32_at(param, 0x08),
        total_tax: read_server_region_u32_at(param, 0x0C),
        today_total_tax: read_server_region_u32_at(param, 0x10),
        superior_region_id: read_server_region_i32_at(param, 0x14),
        turn_in_tax_rate: read_server_region_i32_at(param, 0x18),
        owned_faction_id: read_server_region_i32_at(param, 0x1C),
        owned_union_id: read_server_region_i32_at(param, 0x20),
    });
    Ok(true)
}

/// Читает полный startup/reload snapshot `regions/{id}.rs`: восемь
/// signed DWORD setup-а, count и C-string set запрещённых товаров.
pub fn decord_server_region_setup_from_byte_array<Store: ServerRegionDecodeStore>(
    store: &mut Store,
    source: &[u8],
    cursor: &mut usize,
    _include_child: bool,
) -> Result<bool, ServerRegionSetupDecodeError> {
    decode_server_return_setup_prefix(store, source, cursor)
        .map_err(ServerRegionSetupDecodeError::Setup)?;

    // RVA 0x0007EAC0 очищает tree до чтения count и повторяет clear после
    // него. Второй clear не наблюдаем для owned BTreeSet.
    store.clear_forbid_goods();
    let count = read_setup_i32(source, cursor, "m_ForbidMakeGoods.size")?;
    for _ in 0..count.max(0) {
        let value = read_setup_c_string(source, cursor, "m_ForbidMakeGoods[]")?;
        store.insert_forbid_good(value);
    }
    Ok(true)
}

/// Return-setup prefix `DecordSetupFromByteArray` (`0x0007EAC0`): точные
/// `0x20` байт `m_stSetup` материализуются до set запрещённых товаров.
pub fn decode_server_return_setup_prefix<Store: ServerRegionDecodeStore>(
    store: &mut Store,
    source: &[u8],
    cursor: &mut usize,
) -> Result<(), ServerReturnSetupInputBlock> {
    let offset = *cursor;
    let required = 0x20usize;
    let Some(end) = offset.checked_add(required) else {
        return Err(ServerReturnSetupInputBlock {
            offset,
            required,
            available: source.len().saturating_sub(offset),
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(ServerReturnSetupInputBlock {
            offset,
            required,
            available: source.len().saturating_sub(offset),
        });
    };
    let read_i32 = |at: usize| {
        LegacyReader::at(bytes, at)
            .and_then(|mut reader| reader.read_i32())
            .expect("проверенный 0x20-байтовый setup")
    };
    store.assign_return_setup(ServerReturnSetup {
        region_id: read_i32(0x00),
        left: read_i32(0x04),
        top: read_i32(0x08),
        right: read_i32(0x0C),
        bottom: read_i32(0x10),
        does_recall_when_lost: read_i32(0x14),
        move_monster_when_refeash: read_i32(0x18),
        use_return: read_i32(0x1C),
    });
    *cursor = end;
    Ok(())
}

/// Сохраняет `std::set<std::string>::find`: вход рассматривается как
/// C-string, поэтому байты после первого NUL не участвуют в lookup.
pub fn find_forbid_good<Store: ServerRegionDecodeStore>(store: &Store, name: &[u8]) -> bool {
    let end = name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len());
    store.contains_forbid_good(&name[..end])
}

/// Делегирует exact `CServerRegion::New` в базовый `CRegion::New`.
pub fn new_region<Store: ServerRegionDecodeStore>(
    store: &mut Store,
) -> Result<i32, RegionStorageBlock> {
    store.base_region_mut().new_region()
}

/// Делегирует exact `CServerRegion::Save` в базовый resource serializer.
pub fn save_region_resource<Store: ServerRegionDecodeStore>(
    store: &mut Store,
) -> Result<RegionResourceWrite, RegionStorageBlock> {
    let id = store.server_region_id();
    store.base_region_mut().set_id(id);
    store.base_region_mut().save_resource()
}

/// Сохраняет `Load -> success gate -> CreateAreaArray` без domain effects.
pub fn load_region_resource<Store: ServerRegionDecodeStore>(
    store: &mut Store,
    source: Option<&[u8]>,
    area_width: i32,
    area_height: i32,
) -> Result<bool, ServerRegionLoadError> {
    let id = store.server_region_id();
    store.base_region_mut().set_id(id);
    let loaded = store
        .base_region_mut()
        .load_resource(source)
        .map_err(ServerRegionLoadError::Region)?;
    if !loaded {
        return Ok(false);
    }
    store
        .rebuild_area_grid(area_width, area_height)
        .map_err(ServerRegionLoadError::AreaGrid)?;
    Ok(true)
}

/// Дописывает C-string (завершающий NUL включён) в name-cache vector; ядро
/// `LegacyWriter` принадлежит `nebokrai_shared::protocol`.
pub fn append_server_region_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    LegacyWriter::new(destination).write_c_string(value);
}

fn read_setup_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ServerRegionSetupDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        ServerRegionSetupDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader.read_i32().map_err(|block| {
        ServerRegionSetupDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        }
    })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_setup_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, ServerRegionSetupDecodeError> {
    let mut value = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = LegacyReader::at(source, offset).map_err(|block| {
            ServerRegionSetupDecodeError::UnexpectedEnd {
                field,
                offset: block.offset,
                needed: 1,
                available: block.available,
            }
        })?;
        let byte = reader.read_u8().map_err(|block| {
            ServerRegionSetupDecodeError::UnexpectedEnd {
                field,
                offset: block.offset,
                needed: block.needed,
                available: block.available,
            }
        })?;
        *cursor = reader.position();
        if byte == 0 {
            return Ok(value);
        }
        value.push(byte);
    }
}

fn read_server_region_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, 1)?;
    let value = reader
        .read_u8()
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_server_region_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_server_region_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_server_region_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], ServerRegionDecodeInputBlock> {
    let mut reader = server_region_reader(source, *cursor, field, needed)?;
    let bytes = reader
        .read_bytes(needed)
        .map_err(|block| server_region_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn read_server_region_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, ServerRegionDecodeInputBlock> {
    let mut value = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = server_region_reader(source, offset, field, 1)?;
        let byte = reader
            .read_u8()
            .map_err(|block| server_region_error(field, block))?;
        *cursor = reader.position();
        if byte == 0 {
            return Ok(value);
        }
        value.push(byte);
    }
}

fn read_server_region_i32_at(bytes: &[u8], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("проверенный region DWORD")
}

fn read_server_region_u32_at(bytes: &[u8], offset: usize) -> u32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("проверенный region DWORD")
}

fn read_server_region_u16_at(bytes: &[u8], offset: usize) -> u16 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u16())
        .expect("проверенное region WORD")
}

fn server_region_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, ServerRegionDecodeInputBlock> {
    LegacyReader::at(source, cursor).map_err(|block| ServerRegionDecodeInputBlock {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn server_region_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> ServerRegionDecodeInputBlock {
    ServerRegionDecodeInputBlock {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
