//! Технические функции process-owner-а исторического WorldServer в составе
//! Realm `app/`.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`. Файл хранит
//! operator-log адаптеры, имя/состояние процесса и узкие lifecycle helpers,
//! используемые `CGame`; доменный `Init/MainLoop/Release` живёт в
//! `world_game_init`/`world_main_loop`.
//!
//! Windows MFC/console side effects заменены структурированными результатами и
//! stderr process-оболочки. Byte-exact format keys, порядок публикации и
//! различие штатной ошибки, retained owner и безопасной остановки сохраняются.
//! Rust не вводит второй singleton либо дополнительный process lifecycle.
//! Здесь же опубликованы resource/reload context-границы и типовой блок
//! перезагрузки world-сервера; их владельцы-железо реализуют процесс.
//!
//! В конце файла собраны data-контракты бывшего `CGame`, цитируемые полями
//! диспетчерских outcome/report-типов `app::servermessage`: CD-key snapshot,
//! reconnect/region transition/save response записи, ping/region decode итоги,
//! запуск save-thread и отчёт offline-миграции терминала ветви `0x3FC02`
//! (`WorldGameServerLostReport`). Их fn-владельцы и trait
//! `WorldSaveRuntimeContext` остаются у process-owner-а (`world_game*`).
//!
//! Там же свободный monitoring-owner `SendErrLog` (`send_err_log_to_login` +
//! `WorldErrorLogDelivery`): исходная cdecl-функция принадлежит коду процесса
//! WorldServer, а не nets-классу `CMessage` (S_PUB32 `?SendErrLog@@YAXDJJPBD@Z`
//! `1:00000f30` той же пары `Nworldserver.exe`/`WorldServer.pdb`, RSDS
//! совпадает), и публикует Login wire
//! `0x0001_FE08` средствами [`crate::app::world_message::CMessage`]. Поэтому
//! её место у process-owner-а Realm `app/`, а не в `app::world_message`,
//! который воспроизводит только сам nets-класс.
//!
//! Процессный environment-helper `resolve_first_local_ipv4` (первый локальный
//! IPv4 через nodename-lookup системного resolver-а) имеет здесь единственного
//! мирового владельца; приватные per-runtime копии той же формы у
//! Auth/Billing/Login не сводились.

use std::error::Error;
use std::fmt;
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nebokrai_shared::network::ClientSendQueue;
use nebokrai_shared::resources::{
    BattleFairyExpSerializeError, CBattleFairyExpConfig, CChangeBodyConf, CDaKongXiangQian,
    CFairyExpConf, CGMList, CLingBaoSetup, CLogSystem, CPlayerList, CRegionSetup, CSynthesis,
    CThingSetup, ChangeBodySerializeError, CiQingSerializationBlock, ContributeSetupFormatError,
    ContributeSetupSerializeError, DaKongSerializeError, EmotionFormatError, EmotionSerializeError,
    EquipmentComposeSerializeError, GlobeSetupLoadError, GlobeSetupSnapshot, GmListLoadError,
    GmListSerializationBlock, GodsBattleSerializeError, GoodsDestroyFormatError,
    GoodsDestroySerializeError, GoodsDestroySetup, HitLevelFormatError, HitLevelSerializeError,
    HonorElimilateConfig, IncrementShopSerializeError, LingBaoSerializationBlock,
    LogSystemLoadError, LogSystemSerializeError, MonsterDropRegistry, MonsterListLoadError,
    MonsterListSerializeError, MonsterRegistry, NewSkillMonsterConf,
    NewSkillMonsterSerializeError, PlayerListFormatError, PlayerListSerializeError, PreciousBoxConf,
    PreciousBoxSerializeError, PrisonConfFormatError, PrisonConfSerializeError,
    QuestSystemSerializationBlock, RegionRouter, RegionRouterLoadError, RegionRouterSerializeError,
    RegionSetupLoadError, RegionSetupSerializeError, SynthesisSerializeError,
    TaoZhuangSerializationBlock, ThingSetupCodecError, TradeListFormatError,
    TradeListSerializeError,
};
use parking_lot::Mutex;
use rustix::system::uname;

use crate::activities::attackcitysys::AttackCityReloadBlock;
use crate::activities::countrywarsys::CountryWarReloadBlock;
use crate::activities::fournationwarsys::FourNationWarSerializationBlock;
use crate::activities::villagewarsys::VillageWarReloadBlock;
use crate::app::organsysmessage::OrganizingCityWarResultContextBlock;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::characters::player::PlayerCodecError;
use crate::content::battlefairyproperty::{BattleFairyComposeWireError, CBattleFairyProperty};
use crate::content::cgoodsfactory::{
    GoodsNameIndex, GoodsOriginalNameIndex, GoodsRegistryLoadError, GoodsRegistrySerializeError,
};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::skillfactory::SkillFactorySerializeError;
use crate::content::TimeToReturnLoadError;
use crate::content::{DefaultClientResourceOwner, find_script_files};
use crate::organizations::faction::FactionReinitializationBlock;
use crate::organizations::organizingctrl::{
    OrganizingSaveDataBlock, OrganizingSaveDataReport, PlayerEnterGameOutcome, PlayerExitGameOutcome,
};
use crate::persistence::writelogqueue::WorldWriteLogQueue;
use crate::regions::region::RegionSerializationBlock;
use crate::regions::worldcityregion::{WorldCityRegionLoadError, WorldCityRegionSerializationBlock};
use crate::regions::worldcountrywarregion::{
    WorldCountryWarRegionLoadError, WorldCountryWarRegionSerializationBlock,
};
use crate::regions::worldregion::{
    WorldRegionLoadError, WorldRegionParamDecodeError, WorldRegionSerializationBlock,
};
use crate::regions::worldwarregion::WorldWarRegionSerializationBlock;

const LEGACY_LOG_BUFFER_CAPACITY: usize = 64_000;
const LEGACY_WINDOW_TEXT_CAPACITY: usize = 64_000;
const SAVE_LOG_HEADER: &[u8] =
    b"\r\n=============================== Start Save Log ===============================\r\n";
const SAVE_LOG_FOOTER: &[u8] =
    b"================================ End Save Log ================================\r\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLogLocalTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveLogTextDisposition {
    Retained,
    Flushed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddLogTextBlock {
    MissingNoArgumentValue { percent_offset: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldLogLine {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddLogTextDisposition {
    Written {
        rotation: SaveLogTextDisposition,
        line: WorldLogLine,
    },
    BlockedMissingFact {
        rotation: SaveLogTextDisposition,
        local_time: WorldLogLocalTime,
        block: AddLogTextBlock,
    },
}

#[derive(Clone, Default)]
pub struct WorldLogTextOwner {
    state: Arc<Mutex<WorldLogTextState>>,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct WorldLogTextState {
    initialized: bool,
    last_save_tick_ms: u32,
    info_text: Vec<u8>,
    log_text: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRefreshInfoCurrent {
    pub connections: i32,
    pub map_players: i32,
    pub online_players: u32,
    pub offline_players: u32,
    pub login_players: u32,
    pub creation_players: u32,
    pub deletion_players: u32,
    pub restore_players: u32,
    pub saving_players: i32,
    pub team_sessions: i32,
    pub largess_entries: u32,
    pub write_log_queue: u32,
    pub player_load_queue: u32,
    pub reback_messages: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldRefreshInfoHighWater {
    pub connections: i32,
    pub map_players: i32,
    pub online_players: u32,
    pub offline_players: u32,
    pub login_players: u32,
    pub creation_players: u32,
    pub deletion_players: u32,
    pub restore_players: u32,
    pub saving_players: i32,
    pub team_sessions: i32,
    pub largess_entries: u32,
    pub write_log_queue: u32,
    pub player_load_queue: u32,
    pub reback_messages: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRefreshSaveState {
    pub last_save_time: WorldLogLocalTime,
    pub save_point_time_ms: u32,
    pub last_save_tick_ms: u32,
    pub this_save_start_tick_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRefreshSaveStatus {
    Normal,
    Abnormal,
}

impl WorldRefreshSaveStatus {
    const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Normal => b"(Normal)",
            Self::Abnormal => b"(Abnormal!!!!)",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRefreshInfoReport {
    pub current: WorldRefreshInfoCurrent,
    pub high_water: WorldRefreshInfoHighWater,
    pub save_status: WorldRefreshSaveStatus,
    pub this_save_seconds: u32,
    pub text: Vec<u8>,
}

pub fn refresh_info_text<GetTick>(
    owner: &mut WorldLogTextOwner,
    current: WorldRefreshInfoCurrent,
    high_water: &mut WorldRefreshInfoHighWater,
    save: WorldRefreshSaveState,
    mut get_tick: GetTick,
) -> WorldRefreshInfoReport
where
    GetTick: FnMut() -> u32,
{
    high_water.connections = high_water.connections.max(current.connections);
    high_water.map_players = high_water.map_players.max(current.map_players);
    update_signed_u32_max(&mut high_water.online_players, current.online_players);
    update_signed_u32_max(&mut high_water.offline_players, current.offline_players);
    update_signed_u32_max(&mut high_water.login_players, current.login_players);
    update_signed_u32_max(&mut high_water.creation_players, current.creation_players);
    update_signed_u32_max(&mut high_water.deletion_players, current.deletion_players);
    update_signed_u32_max(&mut high_water.restore_players, current.restore_players);
    high_water.saving_players = high_water.saving_players.max(current.saving_players);
    high_water.team_sessions = high_water.team_sessions.max(current.team_sessions);
    update_signed_u32_max(&mut high_water.largess_entries, current.largess_entries);
    update_signed_u32_max(&mut high_water.write_log_queue, current.write_log_queue);
    update_signed_u32_max(&mut high_water.player_load_queue, current.player_load_queue);
    high_water.reback_messages = high_water.reback_messages.max(current.reback_messages);

    let status_tick_ms = get_tick();
    let save_status =
        if save.save_point_time_ms < status_tick_ms.wrapping_sub(save.last_save_tick_ms) {
            WorldRefreshSaveStatus::Abnormal
        } else {
            WorldRefreshSaveStatus::Normal
        };
    let this_save_seconds = if save.this_save_start_tick_ms == 0 {
        0
    } else {
        get_tick()
            .wrapping_sub(save.this_save_start_tick_ms)
            .wrapping_div(1_000)
    };

    let text = format!(
        "Last Save Time : {:04}-{:02}-{:02} {:02}:{:02}:{:02} {}\r\nThis Save Time : {} sec  Saving = {}/{}\r\nConnects = {}/{}  Map = {}/{}  Online = {}/{}  Offline = {}/{}  Login = {}/{}  Create = {}/{}  Delete = {}/{}  Resume = {}/{}\r\nTeams = {}/{}  Largess = {}/{}  WriteLog = {}/{}  LoadPlayer = {}/{}  ReMsg = {}/{}\r\n",
        save.last_save_time.year,
        save.last_save_time.month,
        save.last_save_time.day,
        save.last_save_time.hour,
        save.last_save_time.minute,
        save.last_save_time.second,
        String::from_utf8_lossy(save_status.as_bytes()),
        this_save_seconds as i32,
        current.saving_players,
        high_water.saving_players,
        current.connections,
        high_water.connections,
        current.map_players,
        high_water.map_players,
        current.online_players as i32,
        high_water.online_players as i32,
        current.offline_players as i32,
        high_water.offline_players as i32,
        current.login_players as i32,
        high_water.login_players as i32,
        current.creation_players as i32,
        high_water.creation_players as i32,
        current.deletion_players as i32,
        high_water.deletion_players as i32,
        current.restore_players as i32,
        high_water.restore_players as i32,
        current.team_sessions,
        high_water.team_sessions,
        current.largess_entries as i32,
        high_water.largess_entries as i32,
        current.write_log_queue as i32,
        high_water.write_log_queue as i32,
        current.player_load_queue as i32,
        high_water.player_load_queue as i32,
        current.reback_messages,
        high_water.reback_messages,
    )
    .into_bytes();
    owner.set_info_text(&text);
    WorldRefreshInfoReport {
        current,
        high_water: *high_water,
        save_status,
        this_save_seconds,
        text,
    }
}

fn update_signed_u32_max(high_water: &mut u32, current: u32) {
    if (*high_water as i32) < current as i32 {
        *high_water = current;
    }
}

impl WorldLogTextOwner {
    pub fn set_info_text(&self, text: &[u8]) {
        let mut state = self.state.lock();
        state.info_text.clear();
        state.info_text
            .extend_from_slice(legacy_c_string_prefix(text));
    }

    pub fn log_text(&self) -> Vec<u8> {
        self.state.lock().log_text.clone()
    }

    pub fn save_log_text<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        force: bool,
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> SaveLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().save_log_text_with(
            force,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }

    pub fn add_log_text<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        message: &[u8],
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().add_log_text_with(
            message,
            false,
            false,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }

    pub fn add_error_log_text<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        message: &[u8],
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().add_log_text_with(
            message,
            false,
            true,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }

    pub fn add_log_text_no_arguments<GetTick, GetLocalTime, PutLogInfo>(
        &self,
        format: &[u8],
        save_info_time_ms: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut put_log_info: PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        self.state.lock().add_log_text_with(
            format,
            true,
            false,
            save_info_time_ms,
            &mut get_tick,
            &mut get_local_time,
            &mut put_log_info,
        )
    }
}

impl WorldLogTextState {
    fn add_log_text_with<GetTick, GetLocalTime, PutLogInfo>(
        &mut self,
        message: &[u8],
        no_arguments_format: bool,
        error_marker: bool,
        save_info_time_ms: u32,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> AddLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        let rotation = self.save_log_text_with(
            false,
            save_info_time_ms,
            get_tick,
            get_local_time,
            put_log_info,
        );
        let local_time = get_local_time();
        let message = legacy_c_string_prefix(message);
        let message = if no_arguments_format {
            match format_without_arguments(message) {
                Ok(message) => message,
                Err(block) => {
                    return AddLogTextDisposition::BlockedMissingFact {
                        rotation,
                        local_time,
                        block,
                    };
                }
            }
        } else {
            message.to_vec()
        };
        let prefix = if error_marker {
            format!(
                "[{:02}-{:02} {:02}:{:02}:{:02}] <error> ",
                local_time.month,
                local_time.day,
                local_time.hour,
                local_time.minute,
                local_time.second,
            )
        } else {
            format!(
                "[{:02}-{:02} {:02}:{:02}:{:02}] ",
                local_time.month,
                local_time.day,
                local_time.hour,
                local_time.minute,
                local_time.second,
            )
        }
        .into_bytes();
        let required_bytes_with_nul = prefix.len() + message.len() + 2 + 1;
        let mut bytes = Vec::with_capacity(required_bytes_with_nul - 1);
        bytes.extend_from_slice(&prefix);
        bytes.extend_from_slice(&message);
        bytes.extend_from_slice(b"\r\n");
        put_log_info(&bytes);
        self.log_text.extend_from_slice(&bytes);

        AddLogTextDisposition::Written {
            rotation,
            line: WorldLogLine { bytes },
        }
    }

    fn save_log_text_with<GetTick, GetLocalTime, PutLogInfo>(
        &mut self,
        force: bool,
        save_info_time_ms: u32,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
        put_log_info: &mut PutLogInfo,
    ) -> SaveLogTextDisposition
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> WorldLogLocalTime,
        PutLogInfo: FnMut(&[u8]),
    {
        if !self.initialized {
            self.initialized = true;
            self.last_save_tick_ms = get_tick();
        }
        if !force {
            let now = get_tick();
            if now.wrapping_sub(self.last_save_tick_ms) <= save_info_time_ms
                && self.log_text.len() < LEGACY_LOG_BUFFER_CAPACITY
            {
                return SaveLogTextDisposition::Retained;
            }
        }

        self.last_save_tick_ms = get_tick();
        put_log_info(SAVE_LOG_HEADER);
        let local_time = get_local_time();
        let dated_separator = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}\r\n\r\n",
            local_time.year,
            local_time.month,
            local_time.day,
            local_time.hour,
            local_time.minute,
            local_time.second,
        );
        put_log_info(dated_separator.as_bytes());
        let info_text = legacy_c_string_prefix(&self.info_text);
        let copied_len = info_text
            .len()
            .min(LEGACY_WINDOW_TEXT_CAPACITY.saturating_sub(1));
        put_log_info(&info_text[..copied_len]);
        put_log_info(b"\r\n");
        self.log_text.clear();
        put_log_info(SAVE_LOG_FOOTER);
        SaveLogTextDisposition::Flushed
    }
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn format_without_arguments(format: &[u8]) -> Result<Vec<u8>, AddLogTextBlock> {
    let mut output = Vec::with_capacity(format.len());
    let mut offset = 0;
    while offset < format.len() {
        if format[offset] != b'%' {
            output.push(format[offset]);
            offset += 1;
            continue;
        }
        if format.get(offset + 1) == Some(&b'%') {
            output.push(b'%');
            offset += 2;
            continue;
        }
        return Err(AddLogTextBlock::MissingNoArgumentValue {
            percent_offset: offset,
        });
    }
    Ok(output)
}

/// Resource/string граница, которую `CWorldRegion::Load` вызывает
/// последовательно и потому не разрешает caller-у заранее читать весь набор.
pub trait WorldRegionResourceContext {
 /// Единственный опубликованный World resource-owner, общий для reload и
 /// всех последующих `rfOpen`-эквивалентов этого context-а.
    fn default_client_resource(&mut self) -> &mut DefaultClientResourceOwner;

    fn read_resource(&mut self, path: &[u8]) -> Option<Vec<u8>> {
        self.default_client_resource().read_resource(path)
    }
    fn region_monster_num_scale(&mut self) -> f32;
}

pub trait WorldReloadContext: WorldRegionResourceContext {
    fn runtime_directory(&self) -> &Path;
 /// Три карты единственного World `CGoodsFactory`; loader, lookup и wire
 /// работают с одним опубликованным состоянием.
    fn goods_registries(
        &mut self,
    ) -> (
        &mut GoodsBasePropertiesRegistry,
        &mut GoodsOriginalNameIndex,
        &mut GoodsNameIndex,
    );
    fn monster_registries(&mut self) -> (&mut MonsterRegistry, &mut MonsterDropRegistry);
    fn log_system(&mut self) -> &mut CLogSystem;
    fn region_setup(&mut self) -> &mut CRegionSetup;
    fn gm_list(&mut self) -> &mut CGMList;
    fn globe_setup(&mut self) -> &mut GlobeSetupSnapshot;
    fn region_router(&mut self) -> &mut RegionRouter;
    fn globe_setup_and_router(&mut self) -> (&GlobeSetupSnapshot, &RegionRouter);
 /// Отдельный mutable owner исторических static `CPlayerList` data.
 ///
 /// Он остаётся вне `CGame`, поскольку тот же экземпляр участвует в
 /// create-role и DB-load runtime; это исключает расходящиеся config копии.
    fn player_list(&mut self) -> &mut CPlayerList;
    fn goods_destroy_setup(&mut self) -> &mut GoodsDestroySetup;
    fn new_skill_monster_conf(&mut self) -> &mut NewSkillMonsterConf;
    fn battle_fairy_exp_config(&mut self) -> &mut CBattleFairyExpConfig;
    fn battle_fairy_property(&mut self) -> &mut CBattleFairyProperty;
    fn synthesis(&mut self) -> &mut CSynthesis;
    fn honor_eliminate_config(&mut self) -> &mut HonorElimilateConfig;
    fn fairy_exp_conf(&mut self) -> &mut CFairyExpConf;
    fn da_kong_xiang_qian(&mut self) -> &mut CDaKongXiangQian;
    fn change_body_conf(&mut self) -> &mut CChangeBodyConf;
    fn precious_box_conf(&mut self) -> &mut PreciousBoxConf;
    fn ling_bao_setup(&mut self) -> &mut CLingBaoSetup;
 /// Публикует immutable snapshot фоновой DB-load очереди после изменения
 /// любого входящего setup-owner-а.
    fn publish_player_load_snapshot(
        &mut self,
        thing_setup: &CThingSetup,
        gold_coin_index: u32,
        gold_coin_limit: u32,
        use_log_system: bool,
        write_log_queue: WorldWriteLogQueue,
    );
    fn query_goods_id_by_original_name(&mut self, original_name: &[u8]) -> u32;
    fn query_goods_name(&mut self, goods_id: u32) -> Option<Vec<u8>>;
    fn four_nation_country_names(&mut self) -> [Vec<u8>; 5];
    fn add_log_text(&mut self, payload: &[u8]);
    fn notify_reload_operator(&mut self, title: &[u8], message: &[u8]);

    /// Собирает дисковые сценарии относительно того же корня, что и read_resource.
    fn script_files(&mut self, pattern: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        let root = self
            .default_client_resource()
            .root_directory()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        let report = find_script_files(&root, pattern, extension);
        for error in &report.errors {
            tracing::warn!(root = %root.display(), ?error,
                "Ошибка поиска файлов сценариев");
        }
        if !report.errors.is_empty() {
            tracing::warn!(files = report.files.len(), errors = report.errors.len(),
                "Список файлов сценариев получен с ошибками");
        }
        report.files
    }
    fn add_region_object_counts(&mut self, monsters: i32, npcs: i32) -> (i32, i32);
    fn region_object_counts(&mut self) -> (i32, i32);
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldReloadBlock {
    RegionList(WorldRegionListBlock),
    RegionSnapshot(WorldReloadRegionSnapshotBlock),
    GlobeSetup(GlobeSetupLoadError),
    RegionRouter(RegionRouterLoadError),
    RegionRouterSerialization(RegionRouterSerializeError),
    LogSystem(LogSystemLoadError),
    GmList(GmListLoadError),
    GmListSerialization(GmListSerializationBlock),
    RegionSetup(RegionSetupLoadError),
    RegionSetupSerialization(RegionSetupSerializeError),
    MonsterList(MonsterListLoadError),
    MonsterListSerialization(MonsterListSerializeError),
    GoodsList(GoodsRegistryLoadError),
    GoodsListSerialization(GoodsRegistrySerializeError),
    LogSystemSerialization(LogSystemSerializeError),
    ThingSetupCodec(ThingSetupCodecError),
    EmotionFormat(EmotionFormatError),
    EmotionSerialization(EmotionSerializeError),
    PlayerListFormat(PlayerListFormatError),
    PlayerListSerialization(PlayerListSerializeError),
    GoodsDestroyFormat(GoodsDestroyFormatError),
    GoodsDestroySerialization(GoodsDestroySerializeError),
    NewSkillMonsterSerialization(NewSkillMonsterSerializeError),
    BattleFairyExpSerialization(BattleFairyExpSerializeError),
    FairyExpSerialization(BattleFairyExpSerializeError),
    DaKongSerialization(DaKongSerializeError),
    ChangeBodySerialization(ChangeBodySerializeError),
    PreciousBoxSerialization(PreciousBoxSerializeError),
    LingBaoSerialization(LingBaoSerializationBlock),
    BattleFairyCombineSerialization(BattleFairyComposeWireError),
    SynthesisSerialization(SynthesisSerializeError),
    EquipmentComposeSerialization(EquipmentComposeSerializeError),
    CiQingSerialization(CiQingSerializationBlock),
    TaoZhuangSerialization(TaoZhuangSerializationBlock),
    GodsBattleDatabaseOwnerRequired,
    GodsBattleSerialization(GodsBattleSerializeError),
    HitLevelFormat(HitLevelFormatError),
    HitLevelSerialization(HitLevelSerializeError),
    TradeListFormat(TradeListFormatError),
    TradeListSerialization(TradeListSerializeError),
    QuestSerialization(QuestSystemSerializationBlock),
    FactionReinitialization(FactionReinitializationBlock),
    SkillListSerialization(SkillFactorySerializeError),
    IncrementShopSerialization(IncrementShopSerializeError),
    PrisonFormat(PrisonConfFormatError),
    PrisonSerialization(PrisonConfSerializeError),
    ContributeFormat(ContributeSetupFormatError),
    ContributeSerialization(ContributeSetupSerializeError),
    CountryWar(CountryWarReloadBlock),
    FourNationWarSerialization(FourNationWarSerializationBlock),
    TimeToReturnLoad(TimeToReturnLoadError),
    VillageWar(VillageWarReloadBlock),
    AttackCity(AttackCityReloadBlock<OrganizingCityWarResultContextBlock>),
}

pub type WorldReloadResult = Result<i32, WorldReloadBlock>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldRegionOwnerLoadBlock {
    Base(WorldRegionLoadError),
    Village(WorldRegionLoadError),
    City(WorldCityRegionLoadError),
    Country(WorldCountryWarRegionLoadError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldRegionOwnerSerializationBlock {
    Base(WorldRegionSerializationBlock),
    Village(WorldWarRegionSerializationBlock),
    City(WorldCityRegionSerializationBlock),
    Country(WorldCountryWarRegionSerializationBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRegionListBlock {
    pub region_id: i32,
    pub source: WorldRegionOwnerLoadBlock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldReloadRegionSnapshotBlock {
    pub region_id: i32,
    pub source: WorldRegionOwnerSerializationBlock,
}

// Data-контракты бывшего `CGame` для полей диспетчерских типов
// `nebokrai_realm::app::servermessage`. Перенесены из
// `worldserver/worldserver/game.rs`, но его trait `WorldSaveRuntimeContext`
// цитирует `WorldSaveThreadJob` и поэтому остаётся у process-owner-а.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameServerLookupError {
    PortUnavailable { index: u32 },
}

impl fmt::Display for WorldGameServerLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortUnavailable { index } => write!(
                formatter,
                "у GameServer {index} не назначен port для точного сравнения"
            ),
        }
    }
}

impl Error for WorldGameServerLookupError {}

#[derive(Debug)]
pub struct WorldCdkeySnapshot {
    pub declared_online_players: u32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldOnlinePlayerAppendOutcome {
    pub inserted: bool,
    pub organizing: PlayerEnterGameOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReconnectedPlayerOwner {
    Existing,
    Created {
        replaced_existing_decoded_id: bool,
        offline_inserted: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldReconnectedPlayerDecode {
    pub requested_player_id: u32,
    pub decoded_player_id: i32,
    pub owner: WorldReconnectedPlayerOwner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldServerSnapshotPlayerOwner {
    Existing,
    Created {
        replaced_existing_decoded_id: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldServerSnapshotPlayerDecode {
    pub requested_player_id: u32,
    pub decoded_player_id: i32,
    pub owner: WorldServerSnapshotPlayerOwner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerSaveResponseProgress {
    pub previous_responses: i32,
    pub completion_counted: bool,
    pub responses_before_reset: i32,
    pub connected_game_servers: i32,
    pub save_triggered: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldOnlinePlayerRemoveOutcome {
    pub removed_occurrences: usize,
    pub organizing: PlayerExitGameOutcome,
}

/// Игрок потерянного GameServer терминала ветви `0x3FC02`: полный набор
/// side effects снятия online/login записи и постановки offline.
/// Тип перевезён из `game.rs` вместе со швом `on_game_server_lost`; старый
/// пакет реэкспортирует его для inherent-метода.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldLostGameServerPlayer {
    pub player_id: u32,
    pub player_name: Vec<u8>,
    pub online_removal: WorldOnlinePlayerRemoveOutcome,
    pub login_removed: bool,
    pub offline_inserted: bool,
}

/// Отчёт терминала `OnGameServerLost` ветви `0x3FC02`: affected region ID в
/// signed map-order, побочные эффекты по каждому затронутому игроку и итог
/// login-нотификации `0x1FE03`. Тип перевезён из `game.rs` вместе со швом;
/// старый пакет реэкспортирует.
#[derive(Debug)]
pub struct WorldGameServerLostReport {
    pub game_server_index: u32,
    pub affected_region_ids: Vec<i32>,
    pub skipped_null_region_owners: usize,
    pub players: Vec<WorldLostGameServerPlayer>,
    pub login_notice_type: i32,
    pub login_notice_delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRegionChangePlayerTransition {
    pub requested_player_id: u32,
    pub decoded_player_id: u32,
    pub target_region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub direction: i32,
    pub direction_applied: bool,
    pub team_id: i32,
    pub owner_type: i32,
    pub owner_id: i32,
    pub offline_removal_completed: bool,
    pub online_removal: WorldOnlinePlayerRemoveOutcome,
    pub login_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCdkeySnapshotError {
    MissingWorldNumber,
    OnlinePlayerCountOutsideLegacyRange { count: usize },
    MissingPlayerOwner { player_id: u32 },
}

impl fmt::Display for WorldCdkeySnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWorldNumber => {
                formatter.write_str("World setup не назначил поле dwNumber")
            }
            Self::OnlinePlayerCountOutsideLegacyRange { count } => write!(
                formatter,
                "online-list содержит {count} записей вне 32-битного диапазона оригинала"
            ),
            Self::MissingPlayerOwner { player_id } => write!(
                formatter,
                "online player {player_id} отсутствует в owning m_mPlayer"
            ),
        }
    }
}

impl Error for WorldCdkeySnapshotError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldRegionChangeTeamUpdate {
    SessionMissingOrNotTeam,
    PlugMissing,
    Updated,
}

/// Семантическая замена старого 36-байтового `tagPingGameServerInfo`.
///
/// Ветка `0x5FA0A` подтверждает `std::string strIP` и два signed `long`:
/// map ID из metadata сообщения и число игроков из payload. Rust-layout не
/// выдаётся за Windows ABI; owned bytes и `Vec` заменяют только STL-владение.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPingGameServerInfo {
    pub ip: Vec<u8>,
    pub map_id: i32,
    pub player_count: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldInitialRegionSnapshotKind {
    Assigned { region_type: i32 },
    Proxy,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldInitialRegionSnapshot {
    pub map_key: i32,
    pub region_id: i32,
    pub kind: WorldInitialRegionSnapshotKind,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldInitialRegionSnapshotSource {
    MissingRegionOwner,
    UninitializedRegionType,
    Full(WorldRegionOwnerSerializationBlock),
    Proxy(RegionSerializationBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldInitialRegionSnapshotBlock {
    pub map_key: i32,
    pub source: WorldInitialRegionSnapshotSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldRegionParamDecodeOutcome {
    RegionNotFound,
    NullRegionPointer,
    Decoded(Result<bool, WorldRegionParamDecodeError>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReceivedPlayerDataUpdate {
    GameServerNotFound,
    Uninitialized,
    Updated {
        previous: Option<i32>,
        current: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReceivedPlayerDataRead {
    GameServerNotFound { legacy_value: i32 },
    Uninitialized,
    Value(i32),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldGlobeVariables {
    pub world_cap_team_1: i32,
    pub world_cap_team_2: i32,
    pub world_cap_team_3: i32,
    pub world_cap_team_4: i32,
}

impl WorldGlobeVariables {
    pub fn values(self) -> [i32; 4] {
        [
            self.world_cap_team_1,
            self.world_cap_team_2,
            self.world_cap_team_3,
            self.world_cap_team_4,
        ]
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldGlobeVariablesDelivery {
    pub socket_id: i32,
    pub variables: WorldGlobeVariables,
    pub delivery: Result<i32, SendMessageError>,
}

/// Наблюдаемый результат свободного owner-а `SendErrLog`.
///
/// Исходная функция возвращала `void` и игнорировала результат `CMessage::Send`;
/// он сохранён здесь только для вызывающего Rust owner-а и не меняет её порядок
/// или внешний wire-контракт.
#[derive(Debug)]
pub enum WorldErrorLogDelivery {
    SkippedNullText,
    Sent {
        message_type: i8,
        server_ip: i32,
        world_id: i32,
        text: Vec<u8>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Строит и ставит `SendErrLog` packet указанному Login transport.
///
/// Выделен из `CGame` только для save-owner-а, который эксклюзивно держит
/// `m_DBData`, но заимствует независимый Login FIFO до async DB traversal.
/// Машинная привязка: S_PUB32 `.exe/Nworldserver.exe`
/// `?SendErrLog@@YAXDJJPBD@Z` `1:00000f30` (пара с `WorldServer.pdb`, RSDS
/// совпадает) зафиксирована ранее; новых machine-проверок не потребовалось.
pub fn send_err_log_to_login(
    sender: Option<&ClientSendQueue>,
    message_type: i8,
    server_ip: i32,
    world_id: i32,
    text: Option<&[u8]>,
) -> WorldErrorLogDelivery {
    let Some(text) = text else {
        return WorldErrorLogDelivery::SkippedNullText;
    };
    let text = &text[..text.iter().position(|byte| *byte == 0).unwrap_or(text.len())];

    let mut message = CMessage::new(0x0001_FE08);
    message.base_mut().add_char(message_type);
    message.base_mut().add_long(server_ip);
    message.base_mut().add_long(world_id);
    message.base_mut().add(text);
    message.base_mut().add_char(0);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send(sender, false);

    WorldErrorLogDelivery::Sent {
        message_type,
        server_ip,
        world_id,
        text: text.to_vec(),
        wire,
        delivery,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGenerateDbDataBlock {
    PlayerCodec(PlayerCodecError),
    Organizing(OrganizingSaveDataBlock),
}

impl From<PlayerCodecError> for WorldGenerateDbDataBlock {
    fn from(error: PlayerCodecError) -> Self {
        Self::PlayerCodec(error)
    }
}

impl From<OrganizingSaveDataBlock> for WorldGenerateDbDataBlock {
    fn from(error: OrganizingSaveDataBlock) -> Self {
        Self::Organizing(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldGenerateDbDataReport {
    pub organizing: OrganizingSaveDataReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldSaveThreadHandleState {
    Empty,
    Open,
}

/// Одноразовая обязанность действующего `__beginthreadex(SaveThreadFunc)`.
///
/// Сам request не угадывает handle: process save-owner возвращает наблюдаемое
/// `Open/Empty` состояние после фактической попытки запуска.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldSaveThreadLaunchRequest {
    pub previous_handle_closed: bool,
    pub security_attributes_is_null: bool,
    pub stack_size: u32,
    pub argument_is_null: bool,
    pub creation_flags: u32,
    pub thread_id_output_requested: bool,
}

pub fn prepare_save_thread_launch(
    handle: &mut WorldSaveThreadHandleState,
) -> WorldSaveThreadLaunchRequest {
    let previous_handle_closed = matches!(*handle, WorldSaveThreadHandleState::Open);
    *handle = WorldSaveThreadHandleState::Empty;
    WorldSaveThreadLaunchRequest {
        previous_handle_closed,
        security_attributes_is_null: true,
        stack_size: 0,
        argument_is_null: true,
        creation_flags: 0,
        thread_id_output_requested: true,
    }
}

/// Первый локальный IPv4 процесса через nodename-lookup системного resolver-а.
pub fn resolve_first_local_ipv4() -> Option<Ipv4Addr> {
    let hostname = uname();
    let hostname = hostname.nodename().to_str().ok()?;
    (hostname, 0)
        .to_socket_addrs()
        .ok()?
        .find_map(|address| match address {
            SocketAddr::V4(address) => Some(*address.ip()),
            SocketAddr::V6(_) => None,
        })
}
