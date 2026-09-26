//! `ReLoad` и связанные reload-стадии `CGame` из `worldserver/game.cpp/.h`,
//! перенесённые в Realm волной C5-C (см. [`crate::app::world_game`]).
//!
//! Статус: тело `reload` (`1:14740`) и семейство reload-стадий (war/time-to-
//! return/city/string-table/initial config/`reload_conf_log`) перенесены
//! буквально. По полной машинной досверке C5-C (та же точная пара
//! `Nworldserver.exe` + `WorldServer.pdb`, RSDS `289F1FB3-…` age 1; дампы
//! `.local/verify-c5c/`, `dis_reload.txt`/`dis_reload_profiles.txt`) закрыты
//! наблюдаемые расхождения диспетчера `reload_profiles` и таблицы
//! [`WORLD_RELOAD_ACTIONS`]: DIFF-3 (обработка любого low-бита выполняет
//! `low &= ~mask; high = 0` — первый обработанный low-профиль гасит все
//! pending high-флаги), DIFF-4 (ChangeBodyConf, lo 0x80000000;
//! SynthesisList, lo 0x50000000; Allthing, lo 0x100 — машинный `ReLoad`
//! вызывается с `(send=1, resources=1)`), DIFF-5 (Broadcast: при отсутствии
//! `setup/sysboardcast.ini` `return 0` без записи conf-log; машинный
//! MessageBox — единственный UI-side-effect ветки, без wire/DB-эффекта — в
//! headless-сервисе осознанно не воспроизводится; при успехе лог
//! `Load System Broadcast List...OK!`, result остаётся 0) и DIFF-6
//! (conf-log пишет профиль машинным написанием `godsBattle`). Покрытие
//! статических `Load*`-внутренностей досверкой не проверялось — статус
//! тела `reload` остаётся PARTIAL, не VERIFIED.
//!
//! DIFF-1/DIFF-2 сознательно оставлены как есть: расхождение касается
//! только внутреннего `legacy_result`, значение которого ни один машинный
//! caller не читает, — наблюдаемости нет.
//!
//! Делегировано владельцам доменов: wwar-ветки (village/city/country/
//! four-nation reload) и внутренности статических `Load*` — их статусы
//! ведутся у соответствующих owner-ов, а не здесь.
//!
//! Нормализации — общие для волны (см. `crate::app::world_game`).

use crate::activities::attackcitysys::{AttackCityCallbacks, AttackCityReloadBlock, CAttackCitySys};
use crate::activities::countrywarsys::{CountryWarCallbacks, CountryWarSys};
use crate::activities::factionwarsys::CFactionWarSys;
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationWarCallbacks, FourNationWarReloadDisposition};
use crate::activities::jjcsystem::CJJcSystem;
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::villagewarsys::{CVillageWarSys, VillageWarCallbacks};
use crate::app::servermessage::{WorldInitialConfigurationRunCompletion, WorldInitialConfigurationRunReport};
use crate::app::world_client::CMyNetClient;
use crate::app::world_dispatch::{CityWarCountryGateBridge, WorldUnionApplicationEffectCallbacks, add_legacy_c_string, format_legacy_percent_s, legacy_c_string_prefix, reload_attack_city};
use crate::app::world_game::CGame;
use crate::app::world_hub_data::{WorldStringTableLoadReport, WorldStringTableUpdateCompletion, WorldStringTableUpdateReport};
use crate::app::world_hub_entries::{WorldRegionAssignment, WorldSystemBroadcast};
use crate::app::world_message::CMessage;
use crate::app::world_reload_profiles::{WORLD_RELOAD_ACTIONS, WorldRegionLoadSpec, WorldReloadActionKind, WorldReloadConfLogBlock, WorldReloadConfLogDisposition, WorldReloadOneScriptBlock, WorldReloadOneScriptResult, WorldReloadProfile, WorldReloadProfileEvent, WorldReloadProfileFlags, WorldReloadProfilesReport, WorldReloadRegionSetupBlock, WorldScriptLoadContext};
use crate::app::world_runtime::{WorldRegionOwner, WorldStringTableEncodingBlock};
use crate::app::world_setup::WorldServerSetupTokens;
use crate::app::worldserver::{WorldLogLocalTime, WorldRegionListBlock, WorldRegionOwnerLoadBlock, WorldRegionOwnerSerializationBlock, WorldReloadBlock, WorldReloadContext, WorldReloadRegionSnapshotBlock, WorldReloadResult};
use crate::characters::honorranks::CHonorRanks;
use crate::characters::playerranks::CPlayerRanks;
use crate::content::{QUEST_EX_PATH, QUEST_PATH, normalize_script_path};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::cgoodsfactory::{load_goods_registry, serialize_goods_registry};
use crate::content::{DefaultClientResourceOwner, DefaultClientResourceReplacement, LOAD_SERVER_RESOURCE_SUCCESS_LOG};
use crate::content::countryparam::CCountryParam;
use crate::content::skillfactory::{CSkillFactory, SkillFactoryCacheLoadReport, SkillFactoryCacheResource};
use crate::content::{TimeToReturn, TimeToReturnCallbacks, TimeToReturnLoadError};
use crate::content::variablelist::CVariableList;
use crate::organizations::countryhandler::CCountryHandler;
use crate::organizations::organizingctrl::COrganizingCtrl;
use crate::organizations::organizingparam::COrganizingParam;
use crate::regions::worldcityregion::CWorldCityRegion;
use crate::regions::worldcountrywarregion::WorldCountryWarRegion;
use crate::regions::worldregion::{CWorldRegion, WorldRegionLoadedCounts};
use crate::regions::worldvillageregion::CWorldVillageRegion;
use nebokrai_shared::resources::{CGodsBattleConf, CPlayerList, GlobeSetupSnapshot, GmListCollection, GodsBattleLoadError, IncrementShopGoodsQuery, IncrementShopGoodsResult, QuestSystemLoadCompletion, QuestSystemLoadReport, RegionRouter, load_drop_goods_list, load_monster_list, serialize_monster_list};
use nebokrai_shared::runtime::CTimer;
use nebokrai_shared::values::TagTime;
use std::collections::BTreeMap;
use std::time::Duration;
use crate::app::servermessage as servermessage;
use crate::activities::rsgodsbattle::RsGodsBattleOwner;

pub(crate) enum WorldRegionMaterialization {
    Direct {
        owner: WorldRegionOwner,
        counts: WorldRegionLoadedCounts,
        loaded: bool,
    },
    MissingSubtype,
}

pub(crate) fn reload_conf_log<GetLocalTime>(
    game: &CGame,
    profile: Option<&[u8]>,
    _reload_result: i32,
    get_local_time: &mut GetLocalTime,
) -> Result<WorldReloadConfLogDisposition, WorldReloadConfLogBlock>
where
    GetLocalTime: FnMut() -> WorldLogLocalTime,
{
    let Some(profile) = profile else {
        return Ok(WorldReloadConfLogDisposition::SuppressedEmptyProfile);
    };
    let profile = legacy_c_string_prefix(profile);
    if profile.is_empty() {
        return Ok(WorldReloadConfLogDisposition::SuppressedEmptyProfile);
    }

    let date = get_local_time();
    let time = get_local_time();
    let date_and_time = format!(
        "{:02}/{:02}/{:02} {:02}:{:02}:{:02}",
        date.month,
        date.day,
        date.year % 100,
        time.hour,
        time.minute,
        time.second,
    );
    let mut text = Vec::with_capacity(32 + profile.len());
    text.extend_from_slice(b"WS On ");
    text.extend_from_slice(date_and_time.as_bytes());
    text.extend_from_slice(b" Reload ");
    text.extend_from_slice(profile);
    text.push(b'.');

    let net_server = game
        .net_server
        .as_ref()
        .ok_or(WorldReloadConfLogBlock::MissingNetworkServerOwner)?;
    let world_number = game
        .setup
        .world_number
        .ok_or(WorldReloadConfLogBlock::MissingWorldNumber)?;

    let mut message = CMessage::new(0x0001_FE06);
    message.base_mut().add_ulong(net_server.local_ipv4_word());
    message.base_mut().add_ulong(world_number);
    add_legacy_c_string(message.base_mut(), &text);
    let delivery = message.send(
        game.net_client.as_ref().map(CMyNetClient::send_queue),
        false,
    );

    Ok(WorldReloadConfLogDisposition::Published { text, delivery })
}

pub(crate) async fn reload_profiles<Context, GetLocalTime, GetTimerLocalTime, TimerCallback>(
    game: &mut CGame,
    flags: &WorldReloadProfileFlags,
    context: &mut Context,
    jjc: &mut CJJcSystem,
    gods_battle: &mut CGodsBattleConf,
    skills: &mut CSkillFactory,
    mut rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
    mut get_local_time: GetLocalTime,
    country_war: &mut CountryWarSys,
    timer: &mut CTimer<TimerCallback>,
    country_war_callbacks: CountryWarCallbacks<TimerCallback>,
    four_nation_war: &mut CFourNationWarSys,
    four_nation_war_callbacks: FourNationWarCallbacks<TimerCallback>,
    time_to_return: &mut TimeToReturn,
    time_to_return_callbacks: TimeToReturnCallbacks<TimerCallback>,
    village_war: &mut CVillageWarSys,
    village_war_callbacks: VillageWarCallbacks<TimerCallback>,
    attack_city: &mut CAttackCitySys,
    attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
    organizing: &mut COrganizingCtrl,
    faction_war: &mut CFactionWarSys,
    country_handler: &mut CCountryHandler,
    country_parameters: &mut CCountryParam,
    organizing_parameters: &mut COrganizingParam,
    organizing_tax_callback: TimerCallback,
    globe_setup: &GlobeSetupSnapshot,
    get_tick: &mut dyn FnMut() -> u32,
    application_callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    mut get_timer_local_time: GetTimerLocalTime,
) -> WorldReloadProfilesReport
where
    Context: WorldReloadContext + ?Sized,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    GetTimerLocalTime: FnMut() -> TagTime,
    TimerCallback: Copy,
{
    let mut events = Vec::new();
    if !flags.has_pending() {
        return WorldReloadProfilesReport::Complete {
            events,
            remaining_flags: flags.snapshot(),
        };
    }

    for &action in WORLD_RELOAD_ACTIONS {
        if !flags.contains(action.half, action.mask) {
            continue;
        }

        flags.consume(action);
        let flags_after_clear = flags.snapshot();
        if action.reload_profile == b"AttackCitySys"
            || action.reload_profile == b"CityWarPara"
        {
            let log_system = context.log_system();
            application_callbacks.faction_level_log_enabled =
                log_system.faction_level_enabled();
            application_callbacks.faction_experience_log_enabled =
                log_system.faction_experience_enabled();
        }
        let reload_result = match action.kind {
            WorldReloadActionKind::Reload => match if action.reload_profile == b"CountryWar" {
                game.reload_country_war(
                    context,
                    country_war,
                    timer,
                    country_war_callbacks,
                    get_timer_local_time(),
                    action.second_option,
                )
            } else if action.reload_profile == b"FourNationWar" {
                game.reload_four_nation_war(
                    context,
                    four_nation_war,
                    timer,
                    four_nation_war_callbacks,
                    get_timer_local_time(),
                    action.first_option,
                    action.second_option,
                )
            } else if action.reload_profile == b"TimeToReturn" {
                game.reload_time_to_return(
                    context,
                    time_to_return,
                    timer,
                    time_to_return_callbacks,
                    get_timer_local_time(),
                    action.second_option,
                )
            } else if action.reload_profile == b"VilWarPara" {
                game.reload_village_war(
                    context,
                    village_war,
                    timer,
                    village_war_callbacks,
                    get_timer_local_time(),
                    action.second_option,
                )
            } else if action.reload_profile == b"AttackCitySys" {
                game.reload_attack_city(
                    context,
                    attack_city,
                    timer,
                    attack_city_callbacks,
                    organizing,
                    country_handler,
                    country_parameters,
                    organizing_parameters,
                    globe_setup,
                    application_callbacks,
                    update_player,
                    get_timer_local_time(),
                    action.second_option,
                )
            } else if action.reload_profile == b"CityWarPara" {
                game.reload_city_war_parameters(
                    context,
                    attack_city,
                    timer,
                    attack_city_callbacks,
                    organizing,
                    country_handler,
                    country_parameters,
                    organizing_parameters,
                    globe_setup,
                    application_callbacks,
                    update_player,
                    get_timer_local_time(),
                    action.second_option,
                )
            } else if action.reload_profile == b"FactionPara" {
                let runtime_directory = context.runtime_directory().to_path_buf();
                let _legacy_result = organizing_parameters.load(
                    &runtime_directory,
                    get_timer_local_time(),
                    timer,
                    organizing_tax_callback,
                );
                match organizing.reinitialize_factions_by_level(game, organizing_parameters) {
                    Ok(_) => {
                        context.add_log_text(b"Load FactionPara...OK!");
                        Ok(0)
                    }
                    Err(source) => Err(WorldReloadBlock::FactionReinitialization(source)),
                }
            } else if action.reload_profile == b"FactionWarPara" {
                let source = context.read_resource(b"data/FactionWarSys.ini");
                let _ = faction_war.load_ini_from_resource(source.as_deref());
                context.add_log_text(b"Load FactionWarPara...OK!");
                Ok(0)
            } else if action.reload_profile == b"CountryParam"
                || action.reload_profile == b"CountryPara"
            {
                let source = context.read_resource(b"data/CountryParam.ini");
                let _legacy_result = country_parameters.load(source.as_deref());
                context.add_log_text(if action.reload_profile == b"CountryParam" {
                    b"Load CountryParam...OK!"
                } else {
                    b"Load CountryPara...OK!"
                });
                Ok(0)
            } else if action.reload_profile == b"Broadcast" {
 // DIFF-5 (машинная досверка C5-C по точной паре, дамп
 // `.local/verify-c5c/dis_reload_profiles.txt`): при отсутствии
 // `setup/sysboardcast.ini` оригинал формирует
 // "file '%s' can't found!" и показывает MessageBox, после чего
 // возвращает 0 без записи в conf-log. Этот MessageBox — единственный
 // UI-side-effect ветки, без wire/DB-эффекта, поэтому в headless-сервисе
 // он осознанно не воспроизводится (ни диалогом, ни operator-notice).
 // При наличии файла оригинал пишет `Load System Broadcast List...OK!`,
 // result остаётся 0 в обеих ветвях.
                let Some(source) = context.read_resource(b"setup/sysboardcast.ini") else {
                    events.push(WorldReloadProfileEvent {
                        half: action.half,
                        mask: action.mask,
                        reload_profile: action.reload_profile,
                        log_profile: action.log_profile,
                        flags_after_clear,
                        reload_result: 0,
                        log: WorldReloadConfLogDisposition::SuppressedBroadcastMissingFile,
                    });
                    continue;
                };
                game.reload_system_broadcasts(
                    Some(&source),
                    &mut *application_callbacks.random,
                    &mut *get_tick,
                );
                context.add_log_text(b"Load System Broadcast List...OK!");
                Ok(0)
            } else {
                game.reload(
                    context,
                    jjc,
                    gods_battle,
                    skills,
                    rs_gods_battle.as_deref_mut(),
                    action.reload_profile,
                    action.first_option,
                    action.second_option,
                )
                .await
            } {
                Ok(result) => result,
                Err(block) => {
                    return WorldReloadProfilesReport::BlockedReloadOwner {
                        completed_events: events,
                        half: action.half,
                        mask: action.mask,
                        reload_profile: action.reload_profile,
                        log_profile: action.log_profile,
                        flags_after_clear,
                        block,
                    };
                }
            },
            WorldReloadActionKind::ReloadAllRegions => {
                match game.reload_all_region_setup(context) {
                    Ok(true) => 1,
                    Ok(false) => 0,
                    Err(block) => {
                        return WorldReloadProfilesReport::BlockedRegionSetup {
                            completed_events: events,
                            half: action.half,
                            mask: action.mask,
                            flags_after_clear,
                            block,
                        };
                    }
                }
            }
        };
        let log = match reload_conf_log(
            game,
            Some(action.log_profile),
            reload_result,
            &mut get_local_time,
        ) {
            Ok(log) => log,
            Err(block) => {
                return WorldReloadProfilesReport::BlockedMissingFact {
                    completed_events: events,
                    half: action.half,
                    mask: action.mask,
                    reload_profile: action.reload_profile,
                    log_profile: action.log_profile,
                    flags_after_clear,
                    reload_result,
                    block,
                };
            }
        };
        events.push(WorldReloadProfileEvent {
            half: action.half,
            mask: action.mask,
            reload_profile: action.reload_profile,
            log_profile: action.log_profile,
            flags_after_clear,
            reload_result,
            log,
        });
    }

    WorldReloadProfilesReport::Complete {
        events,
        remaining_flags: flags.snapshot(),
    }
}

impl CGame {
 /// Выполняет полный observable путь `CGame::LoadServerResource`.
 ///
 /// Стандартный `current_dir` заменяет `GetCurrentDirectoryA` без его
 /// внутреннего 260-byte лимита. Если ОС не даёт cwd, пустой `PathBuf`
 /// сохраняет безопасную попытку relative resource owner-а. Независимо от
 /// результата `LoadEx` код пишет success-log и возвращает `true`.
    pub fn load_server_resource<Log>(
        &mut self,
        default_resource: &mut DefaultClientResourceOwner,
        add_log_text: &mut Log,
    ) -> DefaultClientResourceReplacement
    where
        Log: FnMut(&[u8]),
    {
        let root = std::env::current_dir().unwrap_or_default();
        let report = default_resource.replace_from_world_directory(&root);
        add_log_text(LOAD_SERVER_RESOURCE_SUCCESS_LOG);
        report
    }

 /// Связывает resource replacement с тем же owner-ом, из которого
 /// `WorldRegionResourceContext::read_resource` обслуживает дальнейшие
 /// загрузки. Отдельная публикация log после освобождения mutable borrow
 /// меняет только Rust-заимствование, но не исходный порядок side effects.
    pub(crate) fn load_server_resource_from_context<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
    ) -> DefaultClientResourceReplacement {
        let report = self.load_server_resource(
            context.default_client_resource(),
            &mut |_payload: &[u8]| {},
        );
        context.add_log_text(LOAD_SERVER_RESOURCE_SUCCESS_LOG);
        report
    }

    pub fn clear_string_table(&mut self) {
        self.string_table.table_mut().free();
        self.string_table_array.clear();
    }

    pub fn load_string_table_resource(
        &mut self,
        package: &[u8],
        source: Option<&[u8]>,
    ) -> WorldStringTableLoadReport {
        let package = legacy_c_string_prefix(package);
        let succeeded = if package.is_empty() {
            self.string_table
                .table_mut()
                .reject_empty_resource_name();
            false
        } else if let Some(source) = source {
            match self.string_table.table_mut().load_bytes(source) {
                Ok(()) => true,
                Err(error) => {
                    tracing::warn!(
                        path = %String::from_utf8_lossy(package),
                        offset = error.offset,
                        record_offset = error.record_offset,
                        kind = ?error.kind,
                        id = %String::from_utf8_lossy(&error.id),
                        retained_entries = self.string_table.table().entries().len(),
                        "Ошибка разбора таблицы текстов; ранее применённые записи сохранены"
                    );
                    false
                }
            }
        } else {
            self.string_table
                .table_mut()
                .reject_missing_resource(package);
            false
        };

        let mut log_payload = b"Load language packet [".to_vec();
        log_payload.extend_from_slice(package);
        if succeeded {
            log_payload.extend_from_slice(b"]...OK!");
        } else {
            log_payload.extend_from_slice(b"]...FAILED! : ");
            log_payload.extend_from_slice(self.string_table.table().last_error());
        }

        WorldStringTableLoadReport {
            package: package.to_vec(),
            succeeded,
            log_payload,
        }
    }

    pub fn code_string_table(
        &mut self,
    ) -> Result<(), WorldStringTableEncodingBlock> {
        self.string_table
            .to_byte_array(&mut self.string_table_array)
            .map_err(|entry_count| WorldStringTableEncodingBlock { entry_count })
    }

    pub fn get_string_table_byte_array(&self) -> &[u8] {
        &self.string_table_array
    }

 /// Выполняет `CQuestSystem::Initialize` в его точной позиции World init.
 /// `Initialize` всегда возвращал true после void `Load`, поэтому report не
 /// превращается в init-block и сохраняет уже сделанные in-place изменения.
    pub(crate) fn initialize_quest_system<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
    ) -> QuestSystemLoadReport {
        let string_table = self.string_table.table();
        let report = self.quest_system.load(
            |path| context.read_resource(path),
            &mut |string_id| string_table.get_string_by_id(string_id).map(ToOwned::to_owned),
        );
        for (path, error) in [
            (QUEST_PATH, report.primary_error),
            (QUEST_EX_PATH, report.extension_error),
        ] {
            if let Some(error) = error {
                tracing::warn!(
                    path = %String::from_utf8_lossy(path),
                    field = error.field,
                    offset = error.offset,
                    record_offset = error.record_offset,
                    kind = ?error.kind,
                    "Разбор каталога заданий остановлен; ранее применённые данные сохранены"
                );
            }
        }
        match report.completion {
            QuestSystemLoadCompletion::QuestFileMissing => {
                context.add_log_text(b"Data/Quest.ini can't found!");
            }
            QuestSystemLoadCompletion::QuestExFileMissing => {
                tracing::warn!(path = %String::from_utf8_lossy(QUEST_EX_PATH),
                    "Расширение каталога заданий недоступно; основной список сохранён");
            }
            QuestSystemLoadCompletion::PrimaryFormatStopped => {}
            QuestSystemLoadCompletion::Partial => {
                tracing::warn!(primary_records = report.primary_records,
                    extension_records = report.extension_records,
                    "Каталог заданий загружен частично");
            }
            QuestSystemLoadCompletion::Loaded => {
                context.add_log_text(b"Load Quest List Data/Quest.ini, OK!");
                context.add_log_text(b"Load Quest List Data/QuestEx.ini, OK!");
            }
        }
        report
    }

 /// reload всегда игнорирует переданное имя и перечитывает default и
 /// настроенный packages. Resource I/O остаётся инфраструктурным callback-ом.
    pub fn update_string_table<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        requested_package: &[u8],
    ) -> WorldStringTableUpdateReport {
        const DEFAULT_LANGUAGE: &[u8] = b"data/Language.lag";

        self.clear_string_table();
        let source = context.read_resource(DEFAULT_LANGUAGE);
        let default = self.load_string_table_resource(DEFAULT_LANGUAGE, source.as_deref());
        context.add_log_text(&default.log_payload);
        if !default.succeeded {
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::DefaultLanguageFailed,
            };
        }

        let configured_package = self.setup.language_package.clone();
        let source = context.read_resource(&configured_package);
        let configured =
            self.load_string_table_resource(&configured_package, source.as_deref());
        context.add_log_text(&configured.log_payload);
        if !configured.succeeded {
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::ConfiguredLanguageFailed,
            };
        }

        if let Err(block) = self.code_string_table() {
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::EncodingBlocked(block),
            };
        }
        if self.string_table_array.is_empty() {
            context.add_log_text(
                b"WARNING : Language packet is NULL, will NOT send to WorldServer.",
            );
            return WorldStringTableUpdateReport {
                requested_package: requested_package.to_vec(),
                completion: WorldStringTableUpdateCompletion::Empty,
            };
        }

        let mut message = CMessage::new(0x0007_F807);
        message.base_mut().add(&self.string_table_array);
        let delivery = message.send_all(self.current_game_server_sender().as_ref());
        context.add_log_text(b"Send the new language packet to all the GameServers.");
        WorldStringTableUpdateReport {
            requested_package: requested_package.to_vec(),
            completion: WorldStringTableUpdateCompletion::Broadcast {
                message_type: 0x0007_F807,
                payload_length: self.string_table_array.len(),
                delivery,
            },
        }
    }

    pub(crate) fn publish_player_load_snapshot(
        &self,
        context: &mut (impl WorldReloadContext + ?Sized),
    ) {
        let gold_coin_name = self.get_string_by_id(b"WS0108").to_vec();
        let gold_coin_index = context.query_goods_id_by_original_name(&gold_coin_name);
        let gold_coin_limit = context.globe_setup().gold_coin_limit();
        context.publish_player_load_snapshot(
            &self.thing_setup,
            gold_coin_index,
            gold_coin_limit,
            self.setup.use_log_system,
            self.write_log_queue.clone(),
        );
    }

    pub fn load_one_script<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        path: &[u8],
    ) -> bool {
        self.script_resources
            .load_one(&mut WorldScriptLoadContext(context), path)
    }

    pub fn load_script_file_data<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        _script_directory: &[u8],
        function_file: &[u8],
        variable_file: &[u8],
        _general_variable_data_file: &[u8],
    ) -> bool {
        match self.script_resources.load(
            &mut WorldScriptLoadContext(context),
            function_file,
            variable_file,
        ) {
            Ok(report) => {
                if report.failed != 0 {
                    tracing::warn!(?report, "Сценарии загружены частично");
                } else {
                    tracing::info!(?report, "Загрузка ресурсов сценариев завершена");
                }
                true
            }
            Err(file) => {
                tracing::warn!(?file, "Загрузка сценариев остановлена на обязательном файле");
                false
            }
        }
    }

    pub fn reload_one_script<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        path: &[u8],
    ) -> WorldReloadOneScriptResult {
        let path = legacy_c_string_prefix(path);
        if !self.load_one_script(context, path) {
            let mut log = b"Reload Script(".to_vec();
            log.extend_from_slice(path);
            log.extend_from_slice(b")...FAILED!");
            context.add_log_text(&log);
            return Ok(false);
        }

        let mut log = b"Reload Script(".to_vec();
        log.extend_from_slice(path);
        log.extend_from_slice(b")...OK!");
        context.add_log_text(&log);

        let Some(data) = self.get_script_file_data(path).map(legacy_c_string_prefix) else {
 // Исходный владелец передаёт исходный path в
 // `GetScriptFileData` после того, как `LoadOneScript` нормализовал
 // только map-key. При несовпадении оригинал вызывает lstrlen(NULL).
            return Err(WorldReloadOneScriptBlock {
                requested_path: path.to_vec(),
                normalized_map_key: normalize_script_path(path),
            });
        };
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x0D);
        add_legacy_c_string(message.base_mut(), path);
        message.base_mut().add_long(data.len() as u32 as i32);
        add_legacy_c_string(message.base_mut(), data);
        let sender = self.current_game_server_sender();
        let _ = message.send_all(sender.as_ref());
        Ok(true)
    }

    pub(crate) fn configure_region_owner(region: &mut CWorldRegion, spec: &WorldRegionLoadSpec) {
        region.set_region_identity(spec.region_id, &spec.name);
        region.set_region_list_base_fields(
            spec.resource_id,
            spec.exp_scale,
            spec.country,
            spec.notify,
        );
        region.set_world_region_list_fields(spec.region_type, spec.no_pk, spec.no_contribute);
    }

    pub(crate) fn materialize_region_owner<Context, ResolveName>(
        context: &mut Context,
        spec: &WorldRegionLoadSpec,
        resolve_name: &mut ResolveName,
    ) -> Result<WorldRegionMaterialization, WorldRegionListBlock>
    where
        Context: WorldReloadContext + ?Sized,
        ResolveName: FnMut(&[u8]) -> Vec<u8> + ?Sized,
    {
        if !matches!(spec.region_type, 0..=5) {
            return Ok(WorldRegionMaterialization::MissingSubtype);
        }

        match spec.region_type {
            0 | 4 | 5 => {
                let mut region = Box::new(CWorldRegion::with_constructor_region_base());
                Self::configure_region_owner(&mut region, spec);
                let counts =
                    region
                        .load_from_context(context, resolve_name)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::Base(source),
                        })?;
                let loaded = counts.base_failure.is_none();
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::Base(region),
                    counts,
                    loaded,
                })
            }
            1 => {
                let mut region = Box::new(CWorldVillageRegion::with_constructor_state());
                Self::configure_region_owner(region.war_mut().base_mut(), spec);
                let counts =
                    region
                        .load_from_context(context, resolve_name)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::Village(source),
                        })?;
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::Village(region),
                    counts,
 // игнорирует base-result и возвращает `1`.
                    loaded: true,
                })
            }
            2 => {
                let mut region = Box::new(CWorldCityRegion::with_constructor_state());
                Self::configure_region_owner(region.war_mut().base_mut(), spec);
                let outcome =
                    region
                        .load_from_context(context, resolve_name)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::City(source),
                        })?;
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::City(region),
                    counts: outcome.counts,
                    loaded: outcome.loaded,
                })
            }
            3 => {
                let mut region = Box::new(WorldCountryWarRegion::with_constructor_state());
                Self::configure_region_owner(region.base_mut(), spec);
                let outcome =
                    region
                        .load_from_context(context, resolve_name)
                        .map_err(|source| WorldRegionListBlock {
                            region_id: spec.region_id,
                            source: WorldRegionOwnerLoadBlock::Country(source),
                        })?;
                Ok(WorldRegionMaterialization::Direct {
                    owner: WorldRegionOwner::Country(region),
                    counts: outcome.counts,
                    loaded: outcome.loaded,
                })
            }
            _ => unreachable!("region type отфильтрован выше"),
        }
    }

    pub fn load_region_list<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        path: &[u8],
    ) -> Result<bool, WorldRegionListBlock> {
        let Some(data) = context.read_resource(path) else {
            let mut message = b"Can't find file ".to_vec();
            message.extend_from_slice(legacy_c_string_prefix(path));
            context.notify_reload_operator(b"message", &message);
            return Ok(false);
        };

        let mut tokens = WorldServerSetupTokens::new(&data);
        let mut previous_monsters = 0i32;
        let mut previous_npcs = 0i32;

        while tokens.seek_to(b"#") {
            let region_id = tokens.next_ascii().unwrap_or(0);
            let resource_id = tokens.next_ascii().unwrap_or(0);
            let exp_scale = tokens.next_ascii().unwrap_or(1.0);
            let region_type = tokens.next_ascii().unwrap_or(0);
            let no_pk = tokens.next_ascii::<i32>().unwrap_or(0) != 0;
            let no_contribute = tokens.next_ascii::<i32>().unwrap_or(0) != 0;
            let string_id = tokens.next_bytes().unwrap_or_default().to_vec();
            let game_server_index = tokens.next_ascii().unwrap_or(0);
            let country = tokens.next_ascii::<i32>().unwrap_or(0) as u8;
            let notify = tokens.next_ascii().unwrap_or(0);
            let name = self
                .string_table
                .table()
                .get_string_by_id(&string_id)
                .map_or_else(Vec::new, ToOwned::to_owned);
            let spec = WorldRegionLoadSpec {
                region_id,
                resource_id,
                exp_scale,
                region_type,
                no_pk,
                no_contribute,
                name,
                game_server_index,
                country,
                notify,
            };

            let (owner, total_monster_count, total_npc_count, loaded) =
                match Self::materialize_region_owner(context, &spec, &mut |string_id| {
                    self.string_table
                        .table()
                        .get_string_by_id(string_id)
                        .map_or_else(Vec::new, ToOwned::to_owned)
                })? {
                    WorldRegionMaterialization::MissingSubtype => {
                        let mut log = format!("Region ({region_id}) ").into_bytes();
                        log.extend_from_slice(&spec.name);
                        log.extend_from_slice(b"... Read Setup FAILED!");
                        context.add_log_text(&log);
                        continue;
                    }
                    WorldRegionMaterialization::Direct {
                        owner,
                        counts,
                        loaded,
                    } => {
                        let (total_monster_count, total_npc_count) =
                            context.add_region_object_counts(counts.monsters, counts.npcs);
                        (owner, total_monster_count, total_npc_count, loaded)
                    }
                };
            if !loaded {
                let mut log = format!("Region ({region_id}) ").into_bytes();
                log.extend_from_slice(&spec.name);
                log.extend_from_slice(b"...Load FAILED!");
                context.add_log_text(&log);
                continue;
            }
            let mut log = format!("Region ({region_id}) ").into_bytes();
            log.extend_from_slice(&spec.name);
            log.extend_from_slice(
                format!(
                    " [m={} n={}]...OK!",
                    total_monster_count.wrapping_sub(previous_monsters),
                    total_npc_count.wrapping_sub(previous_npcs),
                )
                .as_bytes(),
            );
            context.add_log_text(&log);
            previous_monsters = total_monster_count;
            previous_npcs = total_npc_count;
 // содержит 549 записей и 549 уникальных ID; поэтому Rust Drop
 // заменённого Box не достигается в baseline и не подменяет утечку.
            self.regions.insert(
                region_id,
                WorldRegionAssignment {
                    region: Some(owner),
                    game_server_index,
                    region_type: Some(region_type),
                },
            );
        }
        let (final_monsters, final_npcs) = context.region_object_counts();
        context.add_log_text(format!("Monster={final_monsters} Npc={final_npcs}!").as_bytes());
        Ok(true)
    }

    pub fn reload_one_region_setup<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        region_id: i32,
    ) -> Result<bool, WorldReloadRegionSetupBlock> {
        let Some(assignment) = self.regions.get_mut(&region_id) else {
            return Ok(false);
        };
        let Some(region) = assignment.region.as_mut() else {
            return Ok(false);
        };
        let region = region.base_mut();
        Self::reload_region_setup_owner(context, region);
        let bytes = region
            .add_setup_to_byte_array()
            .map_err(|source| WorldReloadRegionSetupBlock { region_id, source })?;
        let map_id = assignment.game_server_index as i32;
        let sender = self.current_game_server_sender();
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x10);
        message.base_mut().add_long(region_id);
        message.base_mut().add(&bytes);
        let _ = message.send_to_map_id(sender.as_ref(), map_id);
        Ok(true)
    }

    pub fn reload_all_region_setup<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
    ) -> Result<bool, WorldReloadRegionSetupBlock> {
        let sender = self.current_game_server_sender();
        for assignment in self.regions.values_mut() {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            let region = region.base_mut();
            Self::reload_region_setup_owner(context, region);
            let region_id = region.get_id();
            let bytes = region
                .add_setup_to_byte_array()
                .map_err(|source| WorldReloadRegionSetupBlock { region_id, source })?;
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(0x10);
            message.base_mut().add_long(region_id);
            message.base_mut().add(&bytes);
            let _ = message.send_to_map_id(sender.as_ref(), assignment.game_server_index as i32);
        }
        Ok(true)
    }

    pub(crate) fn reload_region_setup_owner<Context: WorldReloadContext + ?Sized>(
        context: &mut Context,
        region: &mut CWorldRegion,
    ) {
        let path = format!("regions/{}.rs", region.get_id()).into_bytes();
        if let Some(bytes) = context.read_resource(&path) {
            region.load_setup_bytes(&bytes);
            return;
        }
        let mut message = b"file '".to_vec();
        message.extend_from_slice(&path);
        message.extend_from_slice(b"' can't found!");
        context.notify_reload_operator(b"ERROR", &message);
    }

    pub(crate) fn send_reload_payload(&self, subcode: i32, bytes: &[u8]) {
        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(subcode);
        message.base_mut().add(bytes);
        let sender = self.current_game_server_sender();
        let _ = message.send_all(sender.as_ref());
    }

    pub(crate) fn reload_country_war<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        country_war: &mut CountryWarSys,
        timer: &mut CTimer<TimerCallback>,
        callbacks: CountryWarCallbacks<TimerCallback>,
        now: TagTime,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let source = context.read_resource(b"setup/CountryWarSys.ini");
        let sender = self.current_game_server_sender();
        let report = country_war
            .reload(
                source.as_deref(),
                now,
                timer,
                callbacks,
                |payload| context.add_log_text(payload),
                |message| message.send_all(sender.as_ref()),
            )
            .map_err(WorldReloadBlock::CountryWar)?;
        let succeeded = report.load.legacy_result;
        context.add_log_text(if succeeded {
            b"Load CountryWar...OK!"
        } else {
            b"Load CountryWar...FAILED!"
        });
        Ok(i32::from(succeeded))
    }

    pub(crate) fn reload_four_nation_war<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        four_nation_war: &mut CFourNationWarSys,
        timer: &mut CTimer<TimerCallback>,
        callbacks: FourNationWarCallbacks<TimerCallback>,
        now: TagTime,
        send_to_game_servers: bool,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let country_names = context.four_nation_country_names();
        let source = context.read_resource(b"setup/FourNationWarSys.ini");
        let context_cell = std::cell::RefCell::new(&mut *context);
        let disposition = four_nation_war.reload(
            country_names,
            source.as_deref(),
            now,
            timer,
            callbacks,
            |region_id| {
                let path = format!("regions/{region_id}.nation");
                context_cell.borrow_mut().read_resource(path.as_bytes())
            },
            |payload| context_cell.borrow_mut().add_log_text(payload),
        );
        if !matches!(disposition, FourNationWarReloadDisposition::Reloaded(_))
            || !send_to_game_servers
        {
            context.add_log_text(b"Reload the time of FourNationWar...Fail! Please examine wheather did reloading operation when the war was still on!!");
            return Ok(0);
        }

        let mut payload = Vec::new();
        four_nation_war
            .add_to_byte_array(&mut payload)
            .map_err(WorldReloadBlock::FourNationWarSerialization)?;
        let legacy_result = payload.len() as u32 as i32;
        self.send_reload_payload(0x25, &payload);
        context.add_log_text(b"Reload file FourNationWarSys.ini...ok!");
        Ok(legacy_result)
    }

 /// Выполняет concrete `TimeToReturn::reload` для main-loop профиля.
 ///
 /// Его bool в старом dispatcher-е не записывался в общий return-slot:
 /// результат определяет только success/failure log.
    pub(crate) fn reload_time_to_return<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        time_to_return: &mut TimeToReturn,
        timer: &mut CTimer<TimerCallback>,
        callbacks: TimeToReturnCallbacks<TimerCallback>,
        now: TagTime,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let source = context.read_resource(b"setup/TimeToReturn.ini");
        match time_to_return.reload(source.as_deref(), now, timer, callbacks) {
            Ok(_) => {}
            Err(TimeToReturnLoadError::ResourceMissing) => {
                context.add_log_text(b"setup/TimeToReturn.ini can't found!");
            }
            Err(source) => return Err(WorldReloadBlock::TimeToReturnLoad(source)),
        }
 // и caller:
 // missing-файл тоже даёт bool `1`, который становится ReLoad result.
        context.add_log_text(b"Load TimeToReturn...OK!");
        Ok(1)
    }

    pub(crate) fn reload_village_war<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        village_war: &mut CVillageWarSys,
        timer: &mut CTimer<TimerCallback>,
        callbacks: VillageWarCallbacks<TimerCallback>,
        now: TagTime,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let source = context.read_resource(b"setup/villageWarSys.ini");
        let sender = self.current_game_server_sender();
        village_war
            .reload(source.as_deref(), now, timer, callbacks, |message| {
                message.send_all(sender.as_ref()).unwrap_or(0)
            })
            .map_err(WorldReloadBlock::VillageWar)?;
        let mut payload = Vec::new();
        let _ = village_war.add_to_byte_array(&mut payload);
        let legacy_result = payload.len() as u32 as i32;
        self.send_reload_payload(0x1C, &payload);
        context.add_log_text(b"Load VilWarPara...OK!");
        Ok(legacy_result)
    }

 /// Выполняет concrete `CAttackCitySys::Reload` для main-loop профиля.
 ///
 /// Ложный результат `Initialize` остаётся обычным legacy `0`: прежние
 /// таймеры и активные войны уже обработаны Reload и не откатываются.
    #[allow(
        clippy::too_many_arguments,
        reason = "Reload объединяет доказанные singleton-owner-ы city-war lifecycle"
    )]

    pub(crate) fn reload_attack_city<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        attack_city: &mut CAttackCitySys,
        timer: &mut CTimer<TimerCallback>,
        attack_callbacks: AttackCityCallbacks<TimerCallback>,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        country_parameters: &CCountryParam,
        organizing_parameters: &COrganizingParam,
        globe_setup: &GlobeSetupSnapshot,
        effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
        update_player: &mut dyn FnMut(i32),
        now: TagTime,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let source = context.read_resource(b"setup/CityWarSys.ini");
        let country_gate = &mut CityWarCountryGateBridge {
            country_handler,
            country_parameters,
        };
        match reload_attack_city(
            self,
            attack_city,
            source.as_deref(),
            now,
            timer,
            attack_callbacks,
            organizing_parameters.latest_tax_event_id(),
            organizing,
            country_gate,
            globe_setup,
            effects,
            update_player,
        ) {
            Ok(_) => {}
 // `CAttackCitySys::Reload` буквально возвращает Initialize bool.
 // Ошибка parse/open представляет его ложный результат, не block.
            Err(AttackCityReloadBlock::Load(_)) => return Ok(0),
            Err(block) => return Err(WorldReloadBlock::AttackCity(block)),
        };
        let mut payload = Vec::new();
        let _ = attack_city.add_to_byte_array(&mut payload);
        let legacy_result = payload.len() as u32 as i32;
        self.send_reload_payload(0x1B, &payload);
        context.add_log_text(b"Load AttackCitySys List...OK!");
        Ok(legacy_result)
    }

 /// Выполняет отдельный профиль `CityWarPara` из `CGame::ReLoad`.
 ///
 /// вызывает тот же статический
 /// `CAttackCitySys::Reload`, но намеренно не проверяет его bool: сразу
 /// после вызова сериализует live owner в subcode `0x1B`, рассылает
 /// `0x7F801` и пишет success-log. Это не тот же контракт, что
 /// `AttackCitySys`, где ложный `Reload` прекращает публикацию.
    #[allow(
        clippy::too_many_arguments,
        reason = "CityWarPara сохраняет тот же concrete lifecycle, но отдельный return/log contract"
    )]

    pub(crate) fn reload_city_war_parameters<Context, TimerCallback>(
        &mut self,
        context: &mut Context,
        attack_city: &mut CAttackCitySys,
        timer: &mut CTimer<TimerCallback>,
        attack_callbacks: AttackCityCallbacks<TimerCallback>,
        organizing: &mut COrganizingCtrl,
        country_handler: &mut CCountryHandler,
        country_parameters: &CCountryParam,
        organizing_parameters: &COrganizingParam,
        globe_setup: &GlobeSetupSnapshot,
        effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
        update_player: &mut dyn FnMut(i32),
        now: TagTime,
        reload_server_resources: bool,
    ) -> WorldReloadResult
    where
        Context: WorldReloadContext + ?Sized,
        TimerCallback: Copy,
    {
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let source = context.read_resource(b"setup/CityWarSys.ini");
        let country_gate = &mut CityWarCountryGateBridge {
            country_handler,
            country_parameters,
        };
        match reload_attack_city(
            self,
            attack_city,
            source.as_deref(),
            now,
            timer,
            attack_callbacks,
            organizing_parameters.latest_tax_event_id(),
            organizing,
            country_gate,
            globe_setup,
            effects,
            update_player,
        ) {
            Ok(_) | Err(AttackCityReloadBlock::Load(_)) => {}
            Err(block) => return Err(WorldReloadBlock::AttackCity(block)),
        }

        let mut payload = Vec::new();
        let _ = attack_city.add_to_byte_array(&mut payload);
        let legacy_result = payload.len() as u32 as i32;
        self.send_reload_payload(0x1B, &payload);
        context.add_log_text(b"Load CityWarPara...OK!");
        Ok(legacy_result)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "initial-config использует те же live owners, что Init/reload/MainLoop"
    )]

    pub(crate) fn send_initial_game_server_configuration<Context: WorldReloadContext + ?Sized>(
        &self,
        context: &mut Context,
        socket_id: i32,
        game_server_index: u32,
        registry: &GoodsBasePropertiesRegistry,
        player_list: &CPlayerList,
        skills: &CSkillFactory,
        globe_setup: &GlobeSetupSnapshot,
        region_router: &RegionRouter,
        country_parameters: &CCountryParam,
        country_handler: &CCountryHandler,
        gods_battle: &CGodsBattleConf,
        four_nation_war: &CFourNationWarSys,
        honor_ranks: &CHonorRanks,
        player_ranks: &CPlayerRanks,
        general_variables: Option<&CVariableList>,
        attack_city: &CAttackCitySys,
        village_war: &CVillageWarSys,
        country_war: &CountryWarSys,
    ) -> WorldInitialConfigurationRunReport {
        let mut deliveries = Vec::new();
        macro_rules! blocked {
            ($owner:literal) => {
                return WorldInitialConfigurationRunReport {
                    deliveries,
                    completion: WorldInitialConfigurationRunCompletion::Blocked {
                        owner: $owner,
                    },
                }
            };
        }
        macro_rules! optional_step {
            ($report:expr, $pending:pat, $owner:literal) => {{
                let report = $report;
                if let Some(delivery) = report.delivery {
                    deliveries.push(delivery);
                }
                if !matches!(report.completion, $pending) {
                    blocked!($owner);
                }
            }};
        }

        let mut da_kong = Vec::new();
        if context
            .da_kong_xiang_qian()
            .add_to_byte_array(&mut da_kong)
            .is_err()
        {
            blocked!("CDaKongXiangQian");
        }
        macro_rules! game_sender {
            () => {
                self.current_game_server_sender()
            };
        }

        let prefix = servermessage::continue_game_server_initial_configuration_prefix(
            game_sender!().as_ref(),
            socket_id,
            servermessage::WorldGameServerInitialConfigurationPrefix {
                da_kong_xiang_qian: &da_kong,
                goods_registry: registry,
                string_table: self.get_string_table_byte_array(),
                words_filter: self.words_filter(),
                thing_setup: self.thing_setup(),
            },
        );
        deliveries.extend(prefix.deliveries);
        if !matches!(
            prefix.completion,
            servermessage::WorldInitialConfigurationPrefixCompletion::MonsterListPending { .. }
        ) {
            blocked!("initial prefix");
        }

        let (monster_registry, monster_drop_registry) = context.monster_registries();
        let monsters = servermessage::continue_game_server_monster_configuration(
            game_sender!().as_ref(),
            socket_id,
            monster_registry,
            monster_drop_registry,
        );
        if let Some(delivery) = monsters.delivery {
            deliveries.push(delivery);
        }
        if !matches!(
            monsters.completion,
            servermessage::WorldMonsterConfigurationCompletion::HitLevelSetupPending { .. }
        ) {
            blocked!("CMonsterList");
        }
        optional_step!(servermessage::continue_game_server_hit_level_configuration(game_sender!().as_ref(), socket_id, self.hit_level_setup()), servermessage::WorldHitLevelConfigurationCompletion::PlayerListPending { .. }, "CHitLevelSetup");
        optional_step!(servermessage::continue_game_server_player_list_configuration(game_sender!().as_ref(), socket_id, player_list), servermessage::WorldPlayerListConfigurationCompletion::EmotionPending { .. }, "CPlayerList");
        optional_step!(servermessage::continue_game_server_emotion_configuration(game_sender!().as_ref(), socket_id, self.emotion()), servermessage::WorldEmotionConfigurationCompletion::SkillFactoryPending { .. }, "CEmotion");
        optional_step!(servermessage::continue_game_server_skill_configuration(game_sender!().as_ref(), socket_id, skills), servermessage::WorldSkillConfigurationCompletion::TradeListPending { .. }, "CSkillFactory");
        optional_step!(servermessage::continue_game_server_trade_list_configuration(game_sender!().as_ref(), socket_id, self.trade_list()), servermessage::WorldTradeListConfigurationCompletion::IncrementShopListPending { .. }, "CTradeList");
        optional_step!(servermessage::continue_game_server_increment_shop_configuration(game_sender!().as_ref(), socket_id, self.increment_shop_list()), servermessage::WorldIncrementShopConfigurationCompletion::ContributeSetupPending { .. }, "CIncrementShopList");
        optional_step!(servermessage::continue_game_server_contribute_configuration(game_sender!().as_ref(), socket_id, self.contribute_setup()), servermessage::WorldContributeConfigurationCompletion::PrisonConfigurationPending { .. }, "CContributeSetup");
        optional_step!(servermessage::continue_game_server_prison_configuration(game_sender!().as_ref(), socket_id, self.prison_conf()), servermessage::WorldPrisonConfigurationCompletion::PreciousBoxConfigurationPending { .. }, "PrisonConf");
        optional_step!(servermessage::continue_game_server_precious_box_configuration(game_sender!().as_ref(), socket_id, context.precious_box_conf()), servermessage::WorldPreciousBoxConfigurationCompletion::FairyExpConfigurationPending { .. }, "PreciousBoxConf");
        optional_step!(servermessage::continue_game_server_fairy_exp_configuration(game_sender!().as_ref(), socket_id, context.fairy_exp_conf()), servermessage::WorldFairyExpConfigurationCompletion::SynthesisConfigurationPending { .. }, "CFairyExpConf");
        optional_step!(servermessage::continue_game_server_synthesis_configuration(game_sender!().as_ref(), socket_id, context.synthesis()), servermessage::WorldSynthesisConfigurationCompletion::EquipmentComposeConfigurationPending { .. }, "CSynthesis");
        optional_step!(servermessage::continue_game_server_equipment_compose_configuration(game_sender!().as_ref(), socket_id, self.equipment_compose_list()), servermessage::WorldEquipmentComposeConfigurationCompletion::NewSkillMonsterConfigurationPending { .. }, "EquipmentComposeList");
        optional_step!(servermessage::continue_game_server_new_skill_monster_configuration(game_sender!().as_ref(), socket_id, context.new_skill_monster_conf()), servermessage::WorldNewSkillMonsterConfigurationCompletion::GoodsDestroyConfigurationPending { .. }, "CNewSkillMonsterConf");
        optional_step!(servermessage::continue_game_server_goods_destroy_configuration(game_sender!().as_ref(), socket_id, context.goods_destroy_setup()), servermessage::WorldGoodsDestroyConfigurationCompletion::GlobeSetupConfigurationPending { .. }, "CGoodsDestroySetup");
        optional_step!(servermessage::continue_game_server_globe_setup_configuration(game_sender!().as_ref(), socket_id, globe_setup, region_router), servermessage::WorldGlobeSetupConfigurationCompletion::LogSystemConfigurationPending { .. }, "CGlobeSetup");
        optional_step!(servermessage::continue_game_server_log_system_configuration(game_sender!().as_ref(), socket_id, context.log_system()), servermessage::WorldLogSystemConfigurationCompletion::CountryParamConfigurationPending { .. }, "CLogSystem");
        optional_step!(servermessage::continue_game_server_country_param_configuration(game_sender!().as_ref(), socket_id, country_parameters), servermessage::WorldCountryParamConfigurationCompletion::CountryHandlerConfigurationPending { .. }, "CCountryParam");

        let mut country_handler_payload = Vec::new();
        let country_handler_payload = country_handler
            .add_to_byte_array(&mut country_handler_payload)
            .map(|()| country_handler_payload);
        optional_step!(servermessage::continue_game_server_country_handler_configuration(game_sender!().as_ref(), socket_id, country_handler_payload), servermessage::WorldCountryHandlerConfigurationCompletion::GodsBattleConfigurationPending { .. }, "CCountryHandler");
        optional_step!(servermessage::continue_game_server_gods_battle_configuration(game_sender!().as_ref(), socket_id, gods_battle), servermessage::WorldGodsBattleConfigurationCompletion::RegionSnapshotsPending { .. }, "CGodsBattleConf");

        let regions = servermessage::continue_game_server_region_configurations(
            game_sender!().as_ref(),
            socket_id,
            |visit| self.visit_initial_region_snapshots(game_server_index, visit),
            |milliseconds| std::thread::sleep(Duration::from_millis(u64::from(milliseconds))),
        );
        deliveries.extend(regions.deliveries.into_iter().map(|entry| entry.delivery));
        if !matches!(regions.completion, servermessage::WorldRegionConfigurationCompletion::RegionSetupConfigurationPending { .. }) {
            blocked!("CWorldRegion");
        }
        optional_step!(servermessage::continue_game_server_region_setup_configuration(game_sender!().as_ref(), socket_id, context.region_setup()), servermessage::WorldRegionSetupConfigurationCompletion::DupliRegionSetupPending { .. }, "CRegionSetup");
        optional_step!(servermessage::continue_game_server_dupli_region_configuration(game_sender!().as_ref(), socket_id, self.dupli_region_setup()), servermessage::WorldDupliRegionConfigurationCompletion::HonorEliminateConfigurationPending { .. }, "CDupliRegionSetup");

        let honor_eliminate = servermessage::continue_game_server_honor_eliminate_configuration(game_sender!().as_ref(), socket_id, *context.honor_eliminate_config());
        deliveries.push(honor_eliminate.delivery);
        let honor = servermessage::continue_game_server_honor_ranks_configuration(game_sender!().as_ref(), socket_id, honor_ranks);
        deliveries.extend(honor.deliveries.into_iter().map(|entry| entry.delivery));
        if !matches!(honor.completion, servermessage::WorldHonorRanksConfigurationCompletion::FunctionListPending { .. }) {
            blocked!("CHonorRanks");
        }
        let raw_scripts = servermessage::continue_game_server_raw_script_lists_configuration(game_sender!().as_ref(), socket_id, self.function_list_file_data(), self.variable_list_file_data());
        deliveries.extend(raw_scripts.deliveries.into_iter().map(|entry| entry.delivery));
        if !matches!(raw_scripts.completion, servermessage::WorldRawScriptListsConfigurationCompletion::GeneralVariableListPending { .. }) {
            blocked!("raw script lists");
        }
        optional_step!(servermessage::continue_game_server_general_variable_configuration(game_sender!().as_ref(), socket_id, general_variables), servermessage::WorldGeneralVariableConfigurationCompletion::ScriptFilesPending { .. }, "CVariableList");
        let script_files: Vec<(&[u8], &[u8])> = self.initial_script_files().collect();
        let scripts = servermessage::continue_game_server_script_files_configuration(game_sender!().as_ref(), socket_id, &script_files);
        deliveries.extend(scripts.deliveries.into_iter().map(|entry| entry.delivery));
        if !matches!(scripts.completion, servermessage::WorldScriptFilesConfigurationCompletion::QuestSystemPending { .. }) {
            blocked!("script files");
        }
        optional_step!(servermessage::continue_game_server_quest_configuration(game_sender!().as_ref(), socket_id, self.quest_system()), servermessage::WorldQuestConfigurationCompletion::PlayerRanksPending { .. }, "CQuestSystem");
        optional_step!(servermessage::continue_game_server_player_ranks_configuration(game_sender!().as_ref(), socket_id, player_ranks), servermessage::WorldPlayerRanksConfigurationCompletion::GmListPending { .. }, "CPlayerRanks");
        optional_step!(servermessage::continue_game_server_gm_list_configuration(game_sender!().as_ref(), socket_id, game_server_index, context.gm_list()), servermessage::WorldGmListConfigurationCompletion::GameServerIndexPending { .. }, "CGMList");

        let index = servermessage::continue_game_server_index_configuration(game_sender!().as_ref(), socket_id, game_server_index);
        deliveries.push(index.delivery);
        optional_step!(servermessage::continue_game_server_four_nation_war_configuration(game_sender!().as_ref(), socket_id, four_nation_war), servermessage::WorldFourNationWarConfigurationCompletion::BattleFairyExpConfigurationPending { .. }, "CFourNationWarSys");
        optional_step!(servermessage::continue_game_server_battle_fairy_exp_configuration(game_sender!().as_ref(), socket_id, context.battle_fairy_exp_config()), servermessage::WorldBattleFairyExpConfigurationCompletion::BattleFairyPropertyPending { .. }, "CBattleFairyExpConfig");
        optional_step!(servermessage::continue_game_server_battle_fairy_property_configuration(game_sender!().as_ref(), socket_id, context.battle_fairy_property()), servermessage::WorldBattleFairyPropertyConfigurationCompletion::CiQingLingBaoConfigurationPending { .. }, "CBattleFairyProperty");
        optional_step!(servermessage::continue_game_server_ciqing_ling_bao_configuration(game_sender!().as_ref(), socket_id, self.ci_qing_setup(), context.ling_bao_setup()), servermessage::WorldCiQingLingBaoConfigurationCompletion::TaoZhuangConfigurationPending { .. }, "CCiQingSetup/CLingBaoSetup");
        optional_step!(servermessage::continue_game_server_tao_zhuang_configuration(game_sender!().as_ref(), socket_id, self.tao_zhuang_setup()), servermessage::WorldTaoZhuangConfigurationCompletion::AttackCityConfigurationPending { .. }, "CTaoZhuangSetup");

        let attack = servermessage::continue_game_server_attack_city_configuration(game_sender!().as_ref(), socket_id, attack_city);
        deliveries.push(attack.delivery);
        let village = servermessage::continue_game_server_village_war_configuration(game_sender!().as_ref(), socket_id, village_war);
        deliveries.push(village.delivery);
        let mut country_war_payload = Vec::new();
        let _legacy_success = country_war.add_to_byte_array(&mut country_war_payload);
        let country = servermessage::continue_game_server_country_war_configuration(game_sender!().as_ref(), socket_id, &country_war_payload);
        deliveries.push(country.delivery);
        let identity = servermessage::finish_game_server_initial_configuration(game_sender!().as_ref(), socket_id, self.configured_world_number(), self.login_server_id());
        if let Some(delivery) = identity.delivery {
            deliveries.push(delivery);
        }
        if !matches!(identity.completion, servermessage::WorldGameServerIdentityCompletion::InitialConfigurationComplete { .. }) {
            blocked!("GameServer identity");
        }
        WorldInitialConfigurationRunReport {
            deliveries,
            completion: WorldInitialConfigurationRunCompletion::Complete,
        }
    }

    pub(crate) fn load_skill_factory_cache<Context: WorldReloadContext + ?Sized>(
        context: &mut Context,
        skills: &mut CSkillFactory,
        extension: &[u8],
        skill_cache: bool,
    ) -> SkillFactoryCacheLoadReport {
        let paths = context
            .default_client_resource()
            .find_cache_file_list(extension);
        let resources = paths
            .into_iter()
            .map(|path| {
                let contents = context.default_client_resource().read_resource(&path);
                (path, contents)
            })
            .collect::<Vec<_>>();
        let resources = resources.iter().map(|(path, contents)| SkillFactoryCacheResource {
            path,
            contents: contents.as_deref(),
        });
        if skill_cache {
            skills.load_skill_cache(resources)
        } else {
            skills.load_usage_cache(resources)
        }
    }

    pub async fn reload<Context: WorldReloadContext + ?Sized>(
        &mut self,
        context: &mut Context,
        jjc: &mut CJJcSystem,
        gods_battle: &mut CGodsBattleConf,
        skills: &mut CSkillFactory,
        rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
        profile: &[u8],
        send_to_game_servers: bool,
        reload_server_resources: bool,
    ) -> WorldReloadResult {
        let mut legacy_result = 0i32;
        if reload_server_resources {
            let _ = self.load_server_resource_from_context(context);
        }
        let profile_name = legacy_c_string_prefix(profile);
        let Some(profile) = WorldReloadProfile::parse(profile_name) else {
            return Ok(legacy_result);
        };

        match profile {
            WorldReloadProfile::PlayerList => {
                const PLAYER_LIST_PATH: &[u8] = b"data/playerlist.ini";
                const ORIGIN_EQUIPMENT_PATH: &[u8] = b"data/playerOrginEquip.ini";
                const EXPERIENCE_PATH: &[u8] = b"data/playerExp.ini";
                const UPGRADES_PATH: &[u8] = b"data/playerPropertiesUpgrade.ini";

 // `LoadPlayerList` открывает второй файл только после успешного
 // первого. Каждая missing-file ветвь сохраняет clear scope.
                let player = match context.read_resource(PLAYER_LIST_PATH) {
                    Some(source) => {
                        context
                            .player_list()
                            .load_player_properties_from_bytes(&source)
                            .map_err(WorldReloadBlock::PlayerListFormat)?;
                        match context.read_resource(ORIGIN_EQUIPMENT_PATH) {
                            Some(source) => {
                                context
                                    .player_list()
                                    .load_origin_equipment_from_bytes(&source)
                                    .map_err(WorldReloadBlock::PlayerListFormat)?;
                                true
                            }
                            None => {
                                context.player_list().clear_origin_equipment();
                                false
                            }
                        }
                    }
                    None => {
                        context.player_list().clear_player_properties();
                        false
                    }
                };
                context.add_log_text(if player {
                    b"Load PlayerList playerOrginEquip.ini...OK!"
                } else {
                    b"Load PlayerList playerOrginEquip.ini...FAILED!"
                });
                let experience = match context.read_resource(EXPERIENCE_PATH) {
                    Some(source) => {
                        context
                            .player_list()
                            .load_player_experience_from_bytes(&source)
                            .map_err(WorldReloadBlock::PlayerListFormat)?;
                        true
                    }
                    None => {
                        context.player_list().clear_player_experience();
                        false
                    }
                };
                context.add_log_text(if player & experience {
                    b"Load PlayerExpList playerExp.ini...OK!"
                } else {
                    b"Load PlayerExpList playerExp.ini...FAILED!"
                });
                let properties = match context.read_resource(UPGRADES_PATH) {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        context
                            .player_list()
                            .load_properties_upgrades_from_bytes(&source, &mut |key| {
                                string_table.get_string_by_id(key).map(ToOwned::to_owned)
                            })
                            .map_err(WorldReloadBlock::PlayerListFormat)?;
                        true
                    }
                    None => {
                        context.player_list().clear_properties_upgrades();
                        false
                    }
                };
                let player_complete = player & experience & properties;
                context.add_log_text(if player_complete {
                    b"Load playerPropertiesUpgrade.ini...OK!"
                } else {
                    b"Load Player Property Upgrade List playerPropertiesUpgrade.ini...FAILED!"
                });
                if player_complete && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .player_list()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::PlayerListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(1, &payload);
                }
                let emotion = match context.read_resource(b"data/Emotions.ini") {
                    Some(source) => self
                        .emotion
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::EmotionFormat)?,
                    None => false,
                };
                context.add_log_text(if emotion {
                    b"Load Emotins.ini...OK!"
                } else {
                    b"Load Emotins.ini...FAILED!"
                });
                if emotion && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.emotion
                        .serialize(&mut payload)
                        .map_err(WorldReloadBlock::EmotionSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x15, &payload);
                }
                self.publish_player_load_snapshot(context);
            }
            WorldReloadProfile::GoodsList => {
                let loaded = match context.read_resource(b"data/goodslist.dat") {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        let (registry, original_name_index, name_index) =
                            context.goods_registries();
                        load_goods_registry(
                            &source,
                            registry,
                            original_name_index,
                            name_index,
                            &mut |key| string_table.get_string_by_id(key).map(ToOwned::to_owned),
                        )
                        .map_err(WorldReloadBlock::GoodsList)?;
                        true
                    }
                    None => {
                        let (registry, original_name_index, name_index) =
                            context.goods_registries();
                        registry.clear();
                        original_name_index.clear();
                        name_index.clear();
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load goodslist.dat...OK!"
                } else {
                    b"Load goodslist.dat...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    let (registry, _, _) = context.goods_registries();
                    serialize_goods_registry(registry, &mut payload)
                        .map_err(WorldReloadBlock::GoodsListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0, &payload);
                }
                self.publish_player_load_snapshot(context);
            }
            WorldReloadProfile::MonsterList => {
                let monsters = match context.read_resource(b"data/monsterlist.ini") {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        let (monsters, _) = context.monster_registries();
                        load_monster_list(monsters, &source, |id| {
                            string_table.get_string_by_id(id).map(ToOwned::to_owned)
                        })
                        .map_err(WorldReloadBlock::MonsterList)?;
                        true
                    }
                    None => false,
                };
                context.add_log_text(if monsters {
                    b"Load monsterlist.ini...OK!"
                } else {
                    b"Load monsterlist.ini...FAILED!"
                });
                let drops = match context.read_resource(b"data/dropgoodslist.ini") {
                    Some(source) => {
                        let goods_names: Vec<Vec<u8>> = source
                            .split(|byte| *byte == b'\n')
                            .filter_map(|line| {
                                let first = line
                                    .split(|byte| byte.is_ascii_whitespace())
                                    .find(|token| !token.is_empty())?;
                                (first != b">").then(|| first.to_vec())
                            })
                            .collect();
                        let goods_ids: BTreeMap<Vec<u8>, u32> = goods_names
                            .into_iter()
                            .map(|name| {
                                let goods_id = context.query_goods_id_by_original_name(&name);
                                (name, goods_id)
                            })
                            .collect();
                        let (_, drops) = context.monster_registries();
                        load_drop_goods_list(drops, &source, |name| {
                            goods_ids.get(name).copied().unwrap_or(0)
                        })
                        .map_err(WorldReloadBlock::MonsterList)?;
                        true
                    }
                    None => false,
                };
                context.add_log_text(if monsters && drops {
                    b"Load dropgoodslist.ini...OK!"
                } else {
                    b"Load dropgoodslist.ini...FAILED!"
                });
                if monsters && drops && send_to_game_servers {
                    let mut payload = Vec::new();
                    let (monsters, drops) = context.monster_registries();
                    serialize_monster_list(monsters, drops, &mut payload)
                        .map_err(WorldReloadBlock::MonsterListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(2, &payload);
                }
            }
            WorldReloadProfile::TradeList => {
                const PATH: &[u8] = b"data/tradelist.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        self.trade_list
                            .load_from_bytes(
                                &source,
                                &mut |id| {
                                    string_table
                                        .get_string_by_id(id)
                                        .map(ToOwned::to_owned)
                                },
                                &mut |original_name| {
                                    context.query_goods_id_by_original_name(original_name)
                                },
                            )
                            .map(|_| true)
                            .map_err(WorldReloadBlock::TradeListFormat)?
                    }
                    None => {
                        self.trade_list.clear();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load tradelist.ini...OK!"
                } else {
                    b"Load tradelist.ini...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.trade_list
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::TradeListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(3, &payload);
                }
            }
            WorldReloadProfile::SkillList => {
                let usage = Self::load_skill_factory_cache(context, skills, b".usage", false);
                if usage.failure().is_some() {
                    context.add_log_text(b"Load Skill Usage List...FAILED!");
                    return Ok(legacy_result);
                }
                let loaded = Self::load_skill_factory_cache(context, skills, b".skill", true);
                context.add_log_text(if loaded.failure().is_none() {
                    b"Load Skillist...OK!"
                } else {
                    b"Load Skillist...FAILED!"
                });
                if loaded.failure().is_none() && send_to_game_servers {
                    let mut payload = Vec::new();
                    skills
                        .serialize(&mut payload)
                        .map_err(WorldReloadBlock::SkillListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(6, &payload);
                }
            }
            WorldReloadProfile::NewSkillMonsterList => {
                const PATH: &[u8] = b"data/NewSkillMonsterList.xml";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        let string_table = self.string_table.table();
                        match context.new_skill_monster_conf().load_from_bytes(
                            &source,
                            &mut |key| string_table.get_string_by_id(key).map(ToOwned::to_owned),
                        ) {
                            Ok(report) => {
                                for count in report.read_monster_counts {
                                    let count = count as u32 as i32;
                                    context.add_log_text(
                                        format!("read monster num: {count}").as_bytes(),
                                    );
                                }
                                true
                            }
                            Err(error) => {
                                context.add_log_text(error.log_payload());
                                false
                            }
                        }
                    }
                    None => {
                        context.new_skill_monster_conf().clear();
                        context.add_log_text(
                            b"error: original name in file [NewSkillMonsterList.xml] not exist!!",
                        );
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load NewSkillMonsterList.xml...ok!"
                } else {
                    b"Load NewSkillMonsterList.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .new_skill_monster_conf()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::NewSkillMonsterSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x22, &payload);
                }
            }
            WorldReloadProfile::GlobeSetup | WorldReloadProfile::GameSetup => {
                let globe_profile = profile == WorldReloadProfile::GlobeSetup;
                let (path, ok, failed) = if globe_profile {
                    (
                        b"setup/globesetup.ini".as_slice(),
                        b"Load globesetup.ini...OK!".as_slice(),
                        b"Load globesetup.ini...FAILED!".as_slice(),
                    )
                } else {
                    (
                        b"setup/gamesetup.ini".as_slice(),
                        b"Load gamesetup.ini...OK!".as_slice(),
                        b"Load gamesetup.ini...FAILED!".as_slice(),
                    )
                };
                let succeeded = match context.read_resource(path) {
                    Some(source) => {
                        if globe_profile {
                            context
                                .globe_setup()
                                .load_globe_setup(&source)
                                .map_err(WorldReloadBlock::GlobeSetup)?;
                            let string_table = self.string_table.table();
                            context
                                .globe_setup()
                                .resolve_country_text(|id| {
                                    string_table.get_string_by_id(id).map(ToOwned::to_owned)
                                })
                                .map_err(WorldReloadBlock::GlobeSetup)?;
                        } else {
                            context
                                .globe_setup()
                                .load_game_setup(&source)
                                .map_err(WorldReloadBlock::GlobeSetup)?;
                        }
                        true
                    }
                    None => {
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(path);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                context.add_log_text(if succeeded { ok } else { failed });

                let game_setup_loaded = if succeeded && globe_profile {
                    match context.read_resource(b"setup/gamesetup.ini") {
                        Some(source) => {
                            context
                                .globe_setup()
                                .load_game_setup(&source)
                                .map_err(WorldReloadBlock::GlobeSetup)?;
                            true
                        }
                        None => false,
                    }
                } else {
                    succeeded
                };
                let auction_loaded = if game_setup_loaded {
                    match context.read_resource(b"setup/AuctionList.ini") {
                        Some(source) => {
                            context
                                .globe_setup()
                                .load_auction_goods(&source)
                                .map_err(WorldReloadBlock::GlobeSetup)?;
                            true
                        }
                        None => false,
                    }
                } else {
                    false
                };
                context.add_log_text(if auction_loaded {
                    b"Load AuctionList.ini...OK!"
                } else {
                    b"Load AuctionList.ini...FAILED!"
                });

                if globe_profile && succeeded {
                    context
                        .globe_setup()
                        .reset_total_jing_li_dan_count();
                    match context.read_resource(b"data/RegionRouter.ini") {
                        Some(source) => {
                            context
                                .region_router()
                                .load_router_setup_bytes(&source)
                                .map_err(WorldReloadBlock::RegionRouter)?;
                        }
                        None => context.region_router().clear(),
                    }
                }
                let complete = succeeded && game_setup_loaded && auction_loaded;
                if complete && send_to_game_servers {
                    let mut payload = Vec::new();
                    let (globe_setup, region_router) = context.globe_setup_and_router();
                    globe_setup
                        .add_to_byte_array(region_router, &mut payload)
                        .map_err(WorldReloadBlock::RegionRouterSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(7, &payload);
                }
                self.publish_player_load_snapshot(context);
            }
            WorldReloadProfile::StringTable => {
                let _ = self.update_string_table(context, profile_name);
            }
            WorldReloadProfile::LogSystem => {
                let loaded = match context.read_resource(b"setup/logsystem.ini") {
                    Some(source) => {
                        let original_names: Vec<Vec<u8>> = source
                            .split(|byte| *byte == b'\n')
                            .filter_map(|line| {
                                let mut tokens = line
                                    .split(|byte| byte.is_ascii_whitespace())
                                    .filter(|token| !token.is_empty());
                                (tokens.next()? == b"*").then(|| tokens.next().map(ToOwned::to_owned)).flatten()
                            })
                            .collect();
                        let goods_ids: BTreeMap<Vec<u8>, u32> = original_names
                            .into_iter()
                            .map(|name| {
                                let goods_id = context.query_goods_id_by_original_name(&name);
                                (name, goods_id)
                            })
                            .collect();
                        context
                            .log_system()
                            .load_from_bytes(&source, |name| goods_ids.get(name).copied().unwrap_or(0))
                            .map_err(WorldReloadBlock::LogSystem)?;
                        true
                    }
                    None => false,
                };
                context.add_log_text(if loaded {
                    b"Load LogSystem.ini...OK!"
                } else {
                    b"Load LogSystem.ini...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .log_system()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::LogSystemSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(8, &payload);
                }
            }
            WorldReloadProfile::GmList => {
                let passport = context.read_resource(b"data/temp.ini");
                let gm = match context.read_resource(b"setup/gmlist.ini") {
                    Some(source) => {
                        context
                            .gm_list()
                            .load_from_bytes(
                                &source,
                                GmListCollection::Gm,
                                passport.as_deref(),
                            )
                            .map_err(WorldReloadBlock::GmList)?;
                        true
                    }
                    None => false,
                };
                context.add_log_text(if gm {
                    b"Load GMList.ini...OK!"
                } else {
                    b"Load gmlist.ini...FAILED!"
                });
                let player_gm = match context.read_resource(b"setup/playergmlist.ini") {
                    Some(source) => {
                        context
                            .gm_list()
                            .load_from_bytes(
                                &source,
                                GmListCollection::PlayerGm,
                                passport.as_deref(),
                            )
                            .map_err(WorldReloadBlock::GmList)?;
                        true
                    }
                    None => false,
                };
                context.add_log_text(if player_gm {
                    b"Load playerGMList.ini...OK!"
                } else {
                    b"Load playerGMList.ini...FAILED!"
                });
                if gm && player_gm && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .gm_list()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::GmListSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(9, &payload);
                }
            }
            WorldReloadProfile::ScriptFile => {
                let succeeded = self.load_script_file_data(
                    context,
                    b"scripts/",
                    b"data/function.ini",
                    b"data/variable.ini",
                    b"data/general_variable_data.ini",
                );
                context.add_log_text(if succeeded {
                    b"Load function.ini...OK!"
                } else {
                    b"Load function.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    self.send_script_reload_data();
                }
            }
            WorldReloadProfile::RegionList => {
                let succeeded = self
                    .load_region_list(context, b"setup/regionlist.ini")
                    .map_err(WorldReloadBlock::RegionList)?;
                context.add_log_text(if succeeded {
                    b"Load regionlist.ini...OK!"
                } else {
                    b"Load regionlist.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    self.send_loaded_regions(&mut legacy_result)
                        .map_err(WorldReloadBlock::RegionSnapshot)?;
                }
            }
            WorldReloadProfile::RegionLevelSetup => {
                let loaded = match context.read_resource(b"data/regionlevelsetup.ini") {
                    Some(source) => {
                        context
                            .region_setup()
                            .load_from_bytes(&source)
                            .map_err(WorldReloadBlock::RegionSetup)?;
                        true
                    }
                    None => false,
                };
                context.add_log_text(if loaded {
                    b"Load regionlevelsetup.ini...OK!"
                } else {
                    b"Load regionlevelsetup.ini...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .region_setup()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::RegionSetupSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x11, &payload);
                }
            }
            WorldReloadProfile::HitLevelSetup => {
                const PATH: &[u8] = b"data/hitlevel.ini";
                let succeeded = match context.read_resource(PATH) {
                    Some(source) => self
                        .hit_level_setup
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::HitLevelFormat)?,
                    None => {
                        self.hit_level_setup.clear();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                legacy_result = i32::from(succeeded);
                context.add_log_text(if succeeded {
                    b"Load hitlevel.ini...OK!"
                } else {
                    b"Load hitlevel.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.hit_level_setup
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::HitLevelSerialization)?;
                    self.send_reload_payload(0x14, &payload);
                }
            }
            WorldReloadProfile::AttackCity
            | WorldReloadProfile::Broadcast
            | WorldReloadProfile::FactionParameters
            | WorldReloadProfile::VillageWar
            | WorldReloadProfile::FourNationWar
            | WorldReloadProfile::TimeToReturn
            | WorldReloadProfile::CountryWar
            | WorldReloadProfile::CityWar
            | WorldReloadProfile::FactionWar
            | WorldReloadProfile::CountryParameters => unreachable!(
                "reload_profiles направляет profile владельцу с его timer/domain context"
            ),
            WorldReloadProfile::InvalidStrings => {
                let filter_path = self.words_filter.filter_file_name().to_vec();
                let char_code_path = self.words_filter.char_code_file_name().to_vec();
                let filter_source = context.read_resource(&filter_path);
                let char_code_source = filter_source
                    .as_ref()
                    .and_then(|_| context.read_resource(&char_code_path));
                if self
                    .words_filter
                    .reload(filter_source.as_deref(), char_code_source.as_deref())
                {
                    context.add_log_text(b"Load InvalidStr...OK!");
                }
            }
            WorldReloadProfile::GeneralVariableList => {}
            WorldReloadProfile::Quest => {
                let string_table = self.string_table.table();
                let report = self.quest_system.load(
                    |path| context.read_resource(path),
                    &mut |key| string_table.get_string_by_id(key).map(ToOwned::to_owned),
                );
                for (path, error) in [
                    (QUEST_PATH, report.primary_error),
                    (QUEST_EX_PATH, report.extension_error),
                ] {
                    if let Some(error) = error {
                        tracing::warn!(
                            path = %String::from_utf8_lossy(path),
                            field = error.field,
                            offset = error.offset,
                            record_offset = error.record_offset,
                            kind = ?error.kind,
                            "Повторная загрузка каталога заданий остановлена; ранее применённые данные сохранены"
                        );
                    }
                }
                if report.completion != QuestSystemLoadCompletion::Loaded {
                    tracing::warn!(
                        completion = ?report.completion,
                        primary_records = report.primary_records,
                        extension_records = report.extension_records,
                        "Каталог заданий после повторной загрузки неполон"
                    );
                }
                context.add_log_text(b"Load QuestData...OK!");
                let mut payload = Vec::new();
                self.quest_system
                    .system()
                    .add_to_byte_array(&mut payload)
                    .map_err(WorldReloadBlock::QuestSerialization)?;
                legacy_result = payload.len() as u32 as i32;
                self.send_reload_payload(0x16, &payload);
            }
            WorldReloadProfile::IncrementShop => {
                const PATH: &[u8] = b"setup/incrementshoplist.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        let result = self.increment_shop_list.load_from_bytes(
                            &source,
                            &mut |query| match query {
                                IncrementShopGoodsQuery::OriginalName(name) => {
                                    IncrementShopGoodsResult::Id(
                                        context.query_goods_id_by_original_name(name),
                                    )
                                }
                                IncrementShopGoodsQuery::DisplayName(goods_id) => {
                                    IncrementShopGoodsResult::Name(
                                        context.query_goods_name(goods_id),
                                    )
                                }
                            },
                        );
                        match result {
                            Ok(report) => {
                                for warning in report.warnings {
                                    context.add_log_text(&warning);
                                }
                                true
                            }
                            Err(error) => {
                                let diagnostic = error.log_payload();
                                if !diagnostic.is_empty() {
                                    context.add_log_text(&diagnostic);
                                }
                                false
                            }
                        }
                    }
                    None => {
                        self.increment_shop_list.release();
                        let mut message = b"IncShopList : file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.add_log_text(&message);
                        false
                    }
                };
                legacy_result = i32::from(loaded);
                context.add_log_text(if loaded {
                    b"Load IncrementShopList...OK!"
                } else {
                    b"Load IncrementShopList...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.increment_shop_list
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::IncrementShopSerialization)?;
                    self.send_reload_payload(4, &payload);
                }
            }
            WorldReloadProfile::Contribute => {
                const PATH: &[u8] = b"data/ContributeSetup.ini";
                let succeeded = match context.read_resource(PATH) {
                    Some(source) => self
                        .contribute_setup
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::ContributeFormat)?,
                    None => {
                        self.contribute_setup.clear_items();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                legacy_result = i32::from(succeeded);
                context.add_log_text(if succeeded {
                    b"Load ContributeSetup.ini...OK!"
                } else {
                    b"Load ContributeSetup.ini...FAILED!"
                });
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.contribute_setup
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::ContributeSerialization)?;
                    self.send_reload_payload(5, &payload);
                }
            }
            WorldReloadProfile::Prison => {
                const PATH: &[u8] = b"data/PrisonConf.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => self
                        .prison_conf
                        .load_from_bytes(&source)
                        .map(|_| true)
                        .map_err(WorldReloadBlock::PrisonFormat)?,
                    None => {
                        self.prison_conf.clear_prison_params();
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"ERROR", &message);
                        false
                    }
                };
                legacy_result = i32::from(loaded);
                context.add_log_text(if loaded {
                    b"Load PrisonConf.ini...OK!"
                } else {
                    b"Load PrisonConf.ini...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.prison_conf
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::PrisonSerialization)?;
                    self.send_reload_payload(0x1D, &payload);
                }
            }
            WorldReloadProfile::PreciousBox => {
                const PATH: &[u8] = b"data/preciousboxconf.xml";
                let source = context.read_resource(PATH);
 // Убираем Rust-only mutable aliasing, не создавая копию owner state.
                let mut owner = std::mem::take(context.precious_box_conf());
                let load_result = owner.load_from_bytes(source.as_deref(), |original_name| {
                    context.query_goods_id_by_original_name(original_name)
                });
                *context.precious_box_conf() = owner;
                let loaded = match load_result {
                    Ok(report) => {
                        for diagnostic in report.diagnostics() {
                            context.add_log_text(&diagnostic.log_payload());
                        }
                        true
                    }
                    Err(error) => {
                        if let Some(diagnostic) = error.log_payload() {
                            context.add_log_text(diagnostic);
                        }
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load PreciousBoxConf.xml...OK!"
                } else {
                    b"Load PreciousBoxConf.xml...FAILED!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .precious_box_conf()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::PreciousBoxSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x1E, &payload);
                }
            }
            WorldReloadProfile::FairyExp => {
                const PATH: &[u8] = b"data/fairyexp.xml";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => match context.fairy_exp_conf().load_from_bytes(&source) {
                        Ok(()) => true,
                        Err(error) => {
                            context.add_log_text(error.log_payload());
                            false
                        }
                    },
                    None => {
                        context.fairy_exp_conf().clear();
                        context.add_log_text(b"file FairyExp  can't found ..  ...failed  !");
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load FairyExp ....OK!"
                } else {
                    b"Load FairyExp....failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .fairy_exp_conf()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::FairyExpSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x20, &payload);
                }
            }
            WorldReloadProfile::ChangeBody => {
                const PATH: &[u8] = b"data/CHBYRestrictionsGoods.xml";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => match context.change_body_conf().load_from_bytes(&source) {
                        Ok(()) => true,
                        Err(error) => {
                            let diagnostic = self
                                .get_string_by_id(error.string_id())
                                .to_vec();
                            context.add_log_text(&diagnostic);
                            false
                        }
                    },
                    None => {
                        context.change_body_conf().clear();
                        let diagnostic = format_legacy_percent_s(self.get_string_by_id(b"GS1148"), PATH);
                        context.add_log_text(&diagnostic);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load CHBYRestrictionsGoods.xml...ok!"
                } else {
                    b"Load CHBYRestrictionsGoods.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .change_body_conf()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::ChangeBodySerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x24, &payload);
                }
            }
            WorldReloadProfile::BattleFairyExp => {
                const PATH: &[u8] = b"BattleFairyReleate/BattleFairyExp.xml";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => match context.battle_fairy_exp_config().load_from_bytes(&source)
                    {
                        Ok(_) => true,
                        Err(error) => {
                            let diagnostic = self
                                .string_table
                                .table()
                                .get_string_by_id(error.string_id())
                                .map(ToOwned::to_owned)
                                .unwrap_or_default();
                            context.add_log_text(&diagnostic);
                            false
                        }
                    },
                    None => {
                        context.battle_fairy_exp_config().clear();
                        let diagnostic = self
                            .string_table
                            .table()
                            .get_string_by_id(b"ZHGS0029")
                            .map(ToOwned::to_owned)
                            .unwrap_or_default();
                        context.add_log_text(&diagnostic);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Add BattleFairyExpConfig.ini...ok!"
                } else {
                    b"Add BattleFairyExpConfig.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .battle_fairy_exp_config()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::BattleFairyExpSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x2C, &payload);
                }
            }
            WorldReloadProfile::BattleFairyCombine => {
                const PATH: &[u8] = b"BattleFairyReleate/CombineConfig.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        context.battle_fairy_property().load_combine_config(&source);
                        true
                    }
                    None => false,
                };
                context.add_log_text(if loaded {
                    b"Add BattleFairyCombineConfig.xml...ok!"
                } else {
                    b"Add BattleFairyCombineConfig.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .battle_fairy_property()
                        .serialize_combine(&mut payload)
                        .map_err(WorldReloadBlock::BattleFairyCombineSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x2D, &payload);
                }
            }
            WorldReloadProfile::Synthesis => {
                const PATH: &[u8] = b"data/synthesis.xml";
 // EXE очищает recipe-vector до rfOpen, но broadcast-map остаётся static.
                context.synthesis().clear_recipes();
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
 // На время goods lookup owner извлечён безопасно: lookup идёт в
 // тот же `WorldReloadContext`, а после точного loader-а state
 // возвращается в его единственный runtime slot.
                        let mut synthesis = std::mem::take(context.synthesis());
                        let result = synthesis.load_from_bytes(
                            &source,
                            |original_name| {
                                let goods_id =
                                    context.query_goods_id_by_original_name(original_name);
                                let goods_name = (goods_id != 0)
                                    .then(|| context.query_goods_name(goods_id))
                                    .flatten();
                                (goods_id, goods_name)
                            },
                        );
                        *context.synthesis() = synthesis;
                        match result {
                            Ok(_) => true,
                            Err(error) => {
                                context.add_log_text(error.log_payload());
                                false
                            }
                        }
                    }
                    None => {
                        context.add_log_text(b"error: compose file is not exist!");
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load synthesis.xml...ok!"
                } else {
                    b"Load synthesis.xml...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .synthesis()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::SynthesisSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x21, &payload);
                }
            }
            WorldReloadProfile::DaKongXiangQian => {
                const MAIN_PATH: &[u8] = b"data/dakongxiangqian.ini";
                const DELUX_PATH: &[u8] = b"data/DaKongDeluxModify.ini";
                let load_result = match context.read_resource(MAIN_PATH) {
                    Some(main) => {
                        let delux = context.read_resource(DELUX_PATH);
                        context
                            .da_kong_xiang_qian()
                            .load_from_resources(Some(&main), delux.as_deref())
                    }
                    None => context.da_kong_xiang_qian().load_from_resources(None, None),
                };
                let loaded = match load_result {
                    Ok(report) => {
                        if report.delux_modify_missing {
                            context.add_log_text(b"error:file DaKongDeluxModify.ini not exist!!");
                        }
                        true
                    }
                    Err(error) => {
                        context.add_log_text(error.log_payload());
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load DaKongXiangQian.ini...ok!"
                } else {
                    b"Load DaKongXiangQian.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .da_kong_xiang_qian()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::DaKongSerialization)?;
                    legacy_result = payload.len() as u32 as i32;
                    self.send_reload_payload(0x2B, &payload);
                }
            }
            WorldReloadProfile::EquipmentCompose => {
                const PATH: &[u8] = b"data/EquipmentCompose.ini";
                let source = context.read_resource(PATH);
                let loaded = self.equipment_compose_list.load_list(source.as_deref());
                legacy_result = i32::from(loaded);
                context.add_log_text(if loaded {
                    b"Load EquipmentCompose.ini...ok!"
                } else {
                    b"Load EquipmentCompose.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.equipment_compose_list
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::EquipmentComposeSerialization)?;
                    self.send_reload_payload(0x30, &payload);
                }
            }
            WorldReloadProfile::GoodsDestroy => {
                const PATH: &[u8] = b"data/GoodsDestroyConf.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        context
                            .goods_destroy_setup()
                            .load_from_bytes(&source)
                            .map_err(WorldReloadBlock::GoodsDestroyFormat)?;
                        true
                    }
                    None => {
                        context.goods_destroy_setup().clear_lists();
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load GoodsDestroyConf.ini...ok!"
                } else {
                    b"Load GoodsDestroyConf.ini...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut payload = Vec::new();
                    context
                        .goods_destroy_setup()
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::GoodsDestroySerialization)?;
 // Dispatcher не переписывал legacy result для GoodsDestroy.
                    self.send_reload_payload(0x23, &payload);
                }
            }
            WorldReloadProfile::HonorEliminate => {
                const PATH: &[u8] = b"data/honorelimilate.ini";
                let loaded = match context.read_resource(PATH) {
                    Some(source) => {
                        context.honor_eliminate_config().load_from_bytes(&source);
                        true
                    }
                    None => {
                        let mut message = b"file '".to_vec();
                        message.extend_from_slice(PATH);
                        message.extend_from_slice(b"' can't found!");
                        context.notify_reload_operator(b"error", &message);
                        false
                    }
                };
                context.add_log_text(if loaded {
                    b"Load HonorElimilate.ini Config...ok!"
                } else {
                    b"Load HonorElimilate.ini Config...failed!"
                });
            }
            WorldReloadProfile::TaoZhuang => {
                const PATH: &[u8] = b"data/taozhuang.ini";
                let source = context.read_resource(PATH);
                let succeeded = self
                    .tao_zhuang_setup
                    .read_file(source.as_deref(), |payload| context.add_log_text(payload));
                legacy_result = i32::from(succeeded);
                context.add_log_text(if succeeded {
                    b"Load TaoZhuang config...ok!"
                } else {
                    b"Load TaoZhuang config...failed!"
                });
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.tao_zhuang_setup
                        .add_byte_to_array(&mut payload)
                        .map_err(WorldReloadBlock::TaoZhuangSerialization)?;
                    self.send_reload_payload(0x34, &payload);
                }
            }
            WorldReloadProfile::CiQing => {
                const PATH: &[u8] = b"/data/ciqing.ini";
                let source = context.read_resource(PATH);
                let succeeded = self.ci_qing_setup.read_setup_file(
                    source.as_deref(),
                    |original_name| context.query_goods_id_by_original_name(original_name),
                );
                context.add_log_text(if succeeded {
                    b"Add ciqing.ini...ok!"
                } else {
                    b"Add ciqing.ini...failed!"
                });
                legacy_result = i32::from(succeeded);
                const LING_BAO_PATH: &[u8] = b"/data/lingbao.ini";
                let ling_bao_source = context.read_resource(LING_BAO_PATH);
                let ling_bao_report = context
                    .ling_bao_setup()
                    .load_from_bytes(ling_bao_source.as_deref());
                if ling_bao_report.missing_resource {
                    let mut message = b"file '".to_vec();
                    message.extend_from_slice(LING_BAO_PATH);
                    message.extend_from_slice(b"' can't found!");
                    context.notify_reload_operator(b"error", &message);
                }
                if succeeded && send_to_game_servers {
                    let mut payload = Vec::new();
                    self.ci_qing_setup
                        .add_byte_to_array(&mut payload)
                        .map_err(WorldReloadBlock::CiQingSerialization)?;
                    context
                        .ling_bao_setup()
                        .add_byte_ling_bao(&mut payload)
                        .map_err(WorldReloadBlock::LingBaoSerialization)?;
                    self.send_reload_payload(0x35, &payload);
                }
            }
            WorldReloadProfile::Jjc => {
                let report = Self::load_jjc_configuration_from_resources(jjc, context);
                if let Some(path) = report.missing_path() {
                    context.notify_reload_operator(b"file not found", path);
                }
                context.add_log_text(if report.legacy_result() {
                    b"Load JJcConfig.ini...ok!"
                } else {
                    b"Load JJcCoinfig.ini...failed!"
                });
            }
            WorldReloadProfile::AllThing => {
                const PATH: &[u8] = b"/data/LeitingAction.ini";
                let loaded = if let Some(source) = context.read_resource(PATH) {
                    self.thing_setup
                        .load_all_thing_list(&source, PATH, |payload| {
                            context.add_log_text(payload)
                        })
                        .is_ok()
                } else {
                    self.thing_setup.clear_all_things_for_load();
                    let mut message = b"file '".to_vec();
                    message.extend_from_slice(PATH);
                    message.extend_from_slice(b"' can't found!");
                    context.notify_reload_operator(b"error", &message);
                    false
                };
                context.add_log_text(if loaded {
                    b"Load LeitingAction.ini...ok!"
                } else {
                    b"Load...failed!"
                });
                if loaded && send_to_game_servers {
                    let mut bytes = Vec::new();
                    self.thing_setup
                        .add_to_byte_array(&mut bytes)
                        .map_err(WorldReloadBlock::ThingSetupCodec)?;
                    self.send_reload_payload(0x36, &bytes);
                }
                self.publish_player_load_snapshot(context);
            }
            WorldReloadProfile::GodsBattle => {
                let string_table = self.string_table.table();
                let load = gods_battle.load_from_resources(
                    &mut |section| context.read_resource(section.path()),
                    &mut |string_id| {
                        string_table
                            .get_string_by_id(string_id)
                            .map(ToOwned::to_owned)
                    },
                );
                if let Err(GodsBattleLoadError::MissingResource { section }) = &load {
                    let (title, message) = section.missing_notice();
                    context.notify_reload_operator(title, message);
                }
 //: LoadFile устанавливает EAX=1
 // и после missing-file notice тоже приходит в этот epilogue.
                legacy_result = 1;
                context.add_log_text(b"Load Gods-Battle...ok!");

                let Some(rs_gods_battle) = rs_gods_battle else {
                    return Err(WorldReloadBlock::GodsBattleDatabaseOwnerRequired);
                };
                let _legacy_result = rs_gods_battle.get_npc_faction(gods_battle).await;
                if send_to_game_servers {
                    let mut payload = Vec::new();
                    gods_battle
                        .add_to_byte_array(&mut payload)
                        .map_err(WorldReloadBlock::GodsBattleSerialization)?;
                    self.send_reload_payload(0x39, &payload);
                }
            }
        }
        Ok(legacy_result)
    }

    pub(crate) fn send_script_reload_data(&self) {
        let sender = self.current_game_server_sender();
        for (subcode, data) in [
            (0x0A, self.script_resources.functions()),
            (0x0B, self.script_resources.variables()),
        ] {
            let Some(data) = data else { continue };
            let data = legacy_c_string_prefix(data);
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(subcode);
            message.base_mut().add_long(data.len() as u32 as i32);
            add_legacy_c_string(message.base_mut(), data);
            let _ = message.send_all(sender.as_ref());
        }
        for (path, data) in self.script_resources.iter() {
            let data = legacy_c_string_prefix(data);
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(0x0D);
            add_legacy_c_string(message.base_mut(), path);
            message.base_mut().add_long(data.len() as u32 as i32);
            add_legacy_c_string(message.base_mut(), data);
            let _ = message.send_all(sender.as_ref());
        }
    }

    pub(crate) fn send_loaded_regions(
        &mut self,
        legacy_result: &mut i32,
    ) -> Result<(), WorldReloadRegionSnapshotBlock> {
        let sender = self.current_game_server_sender();
        for assignment in self.regions.values_mut() {
            let Some(region) = assignment.region.as_mut() else {
                continue;
            };
            let region_id = region.base().get_id();
            let mut bytes = Vec::new();
            match region {
                WorldRegionOwner::Base(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::Base(source),
                        })?;
                }
                WorldRegionOwner::Village(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::Village(source),
                        })?;
                }
                WorldRegionOwner::City(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::City(source),
                        })?;
                }
                WorldRegionOwner::Country(region) => {
                    let _ = region
                        .add_to_byte_array(&mut bytes, true)
                        .map_err(|source| WorldReloadRegionSnapshotBlock {
                            region_id,
                            source: WorldRegionOwnerSerializationBlock::Country(source),
                        })?;
                }
            }
            *legacy_result = bytes.len() as u32 as i32;
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(0x0E);
            message
                .base_mut()
                .add_long(assignment.region_type.unwrap_or_default());
            message.base_mut().add(&bytes);
            let _ = message.send_to_map_id(sender.as_ref(), assignment.game_server_index as i32);
        }
        Ok(())
    }

 /// Перечитывает `setup/sysboardcast.ini` в тот же live список, который
 /// обслуживает AI. Отсутствующий resource сохраняет прежний список;
 /// открытый источник очищает его до разбора и оставляет подтверждённый
 /// prefix при повреждённой записи.
    pub fn reload_system_broadcasts<Random, GetTick>(
        &mut self,
        source: Option<&[u8]>,
        random: &mut Random,
        get_tick: &mut GetTick,
    ) -> bool
    where
        Random: FnMut(i32) -> i32 + ?Sized,
        GetTick: FnMut() -> u32 + ?Sized,
    {
        let Some(source) = source else {
            return false;
        };
        self.system_broadcasts.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let parse_i32 = |token: &[u8]| {
            std::str::from_utf8(token).ok()?.parse::<i32>().ok()
        };
        while tokens.any(|token| token == b"#") {
            let Some(import_level) = tokens.next().and_then(parse_i32) else {
                return false;
            };
            let Some(region_id) = tokens.next().and_then(parse_i32) else {
                return false;
            };
            let Some(min_time_seconds) = tokens.next().and_then(parse_i32) else {
                return false;
            };
            let Some(max_time_seconds) = tokens.next().and_then(parse_i32) else {
                return false;
            };
            let Some(odds) = tokens.next().and_then(parse_i32) else {
                return false;
            };
            let mut colors = [0_i32; 8];
            for color in &mut colors {
                let Some(value) = tokens.next().and_then(parse_i32) else {
                    return false;
                };
                *color = value;
            }
            let Some(message_id) = tokens.next() else {
                return false;
            };
            let argb = |values: &[i32]| {
                ((values[0] as u32 & 0xff) << 24)
                    | ((values[1] as u32 & 0xff) << 16)
                    | ((values[2] as u32 & 0xff) << 8)
                    | (values[3] as u32 & 0xff)
            };
            let random_range = max_time_seconds.wrapping_sub(min_time_seconds);
            self.system_broadcasts.push_back(WorldSystemBroadcast {
                import_level,
                region_id,
                min_time_seconds: min_time_seconds as u32,
                max_time_seconds: max_time_seconds as u32,
                odds: odds as u32,
                text_color: argb(&colors[..4]),
                back_color: argb(&colors[4..]),
                message: self
                    .string_table
                    .table()
                    .get_string_by_id(message_id)
                    .map_or_else(Vec::new, ToOwned::to_owned),
                interval_seconds: random(random_range).wrapping_add(min_time_seconds) as u32,
                last_notify_time_seconds: get_tick() / 1000,
            });
        }
        true
    }

}
