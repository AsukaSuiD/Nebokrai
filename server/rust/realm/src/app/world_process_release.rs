//! Release-контекст процесса WorldServer и impl-ы драйвера
//! [`crate::app::world_runtime`] для [`WorldProcessRuntime`] (включая
//! `type Game = CGame`).
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает). Машинный порядок Release зафиксирован у владельца стадии
//! (см. `world_game_init::release` и драйвер); этот файл — его process-
//! проводка.
//!
//! `WorldProcessReleaseContext` держит ссылки на process owners во время
//! `CGame::Release`; делегирующий `impl WorldGameReleaseContext` для самого
//! runtime проводит те же вызовы через `release_context()`.

use std::convert::Infallible;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use nebokrai_shared::runtime::CTimer;

use crate::activities::attackcitysys::CAttackCitySys;
use crate::activities::factionwarsys::CFactionWarSys;
use crate::activities::villagewarsys::CVillageWarSys;
use crate::app::world_client::CMyNetClient;
use crate::app::world_game::CGame;
use crate::app::world_process::{
    WorldProcessDomainOwners, WorldProcessMainLoopBlock, WorldProcessMainLoopContexts,
    WorldProcessNetworkRuntime, WorldProcessRuntime,
};
use crate::app::world_process_init::WorldProcessInitContext;
use crate::app::world_process_resources::WorldProcessResources;
use crate::app::world_process_save::WorldProcessSaveRuntime;
use crate::app::world_runtime::{
    WorldGameInitResult, WorldGameReleaseContext, WorldGameReleaseDatabaseOwner,
    WorldGameReleaseOptionalOwner, WorldGameReleaseVoidOwner, WorldGameThreadRuntime,
    WorldRegionOwner,
};
use crate::app::world_server::CMyNetServer;
use crate::app::worldserver::WorldSaveThreadHandleState;
use crate::billing::incrementlog::CIncrementLog;
use crate::characters::playerranks::PlayerRanksReleaseReport;
use crate::content::cgoodsfactory::release_goods_registry;
use crate::content::skillfactory::CSkillFactory;
use crate::content::TimeToReturn;
use crate::organizations::goodswarmember::CGoodsWarMember;
use crate::organizations::organizingctrl::COrganizingCtrl;
use crate::organizations::organizingparam::OrganizingParamReleaseReport;

pub struct WorldProcessReleaseContext<'a> {
    runtime: tokio::runtime::Handle,
    runtime_directory: &'a Path,
    network: &'a mut WorldProcessNetworkRuntime,
    domains: &'a mut WorldProcessDomainOwners,
    init: &'a mut WorldProcessInitContext,
    resources: &'a mut WorldProcessResources,
    save: Option<&'a mut WorldProcessSaveRuntime>,
}

impl<'a> WorldProcessReleaseContext<'a> {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        runtime_directory: &'a Path,
        network: &'a mut WorldProcessNetworkRuntime,
        domains: &'a mut WorldProcessDomainOwners,
        init: &'a mut WorldProcessInitContext,
        resources: &'a mut WorldProcessResources,
        save: Option<&'a mut WorldProcessSaveRuntime>,
    ) -> Self {
        Self {
            runtime,
            runtime_directory,
            network,
            domains,
            init,
            resources,
            save,
        }
    }
}

impl WorldGameReleaseContext for WorldProcessReleaseContext<'_> {
    fn put_debug_string(&mut self, payload: &'static [u8]) {
        eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
    }

    fn save_city_region(&mut self, region_id: i32, region: &mut WorldRegionOwner) {
        match region.save_to_resource_directory(self.runtime_directory) {
            Ok(0) => eprintln!(
                "WorldServer: файл региона {region_id} не открыт во время shutdown"
            ),
            Ok(_) => {}
            Err(error) => eprintln!(
                "WorldServer: регион {region_id} не сохранён во время shutdown: {error:?}"
            ),
        }
    }

    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer) {
        let runtime = self.runtime.clone();
        let result = tokio::task::block_in_place(|| {
            runtime.block_on(self.network.release_server(server))
        });
        if let Err(error) = result {
            eprintln!("WorldServer: GameServer network-worker не завершён: {error}");
        }
    }

    fn exit_network_client_worker(&mut self, client: &mut CMyNetClient) {
        self.network.release_client(client);
    }

    fn release_void_owner(&mut self, owner: WorldGameReleaseVoidOwner) {
        match owner {
            WorldGameReleaseVoidOwner::UninitializeTimeToReturn => {
                self.domains.time_to_return = TimeToReturn::new();
            }
            WorldGameReleaseVoidOwner::ReleaseCountryHandler => {
                let previous = std::mem::take(&mut self.domains.country_handler);
                let _ = previous.release();
            }
            WorldGameReleaseVoidOwner::ReleaseOrganizingController => {
                let previous = std::mem::replace(
                    &mut self.domains.organizing,
                    COrganizingCtrl::with_reached_callback_state(),
                );
                let _ = previous.release(&mut self.domains.timer);
            }
            WorldGameReleaseVoidOwner::ReleaseAttackCity => {
                self.domains.attack_city = CAttackCitySys::new();
            }
            WorldGameReleaseVoidOwner::ReleaseVillageWar => {
                self.domains.village_war = CVillageWarSys::new();
            }
            WorldGameReleaseVoidOwner::ReleaseFactionWar => {
                self.domains.faction_war = CFactionWarSys::new(0);
            }
            WorldGameReleaseVoidOwner::ReleaseTimer => {
                self.domains.timer = CTimer::new();
            }
            WorldGameReleaseVoidOwner::ReleaseGoodsFactory => {
                release_goods_registry(
                    Arc::make_mut(&mut self.resources.goods),
                    Arc::make_mut(&mut self.resources.goods_by_original_name),
                    &mut self.resources.goods_by_name,
                );
            }
            WorldGameReleaseVoidOwner::ReleaseNetSessionManager => {
                let _ = self.domains.net_sessions.release();
            }
            WorldGameReleaseVoidOwner::UninitializeLargess => {
                drop(self.init.largess.take());
            }
            WorldGameReleaseVoidOwner::UninitializeDatabaseLayer => {
                drop(self.init.player_connection.take());
                drop(self.init.country_connection.take());
                drop(self.init.goods_war_connection.take());
                drop(self.init.gods_battle_connection.take());
                drop(self.init.log_connection.take());
                drop(self.init.database_settings.take());
                drop(self.init.log_database_settings.take());
            }
            WorldGameReleaseVoidOwner::ReleaseGoodsLinks
            | WorldGameReleaseVoidOwner::UninitializeIncrementLog
            | WorldGameReleaseVoidOwner::ReleaseWordsFilter
            | WorldGameReleaseVoidOwner::ReleaseQuestSystem
            | WorldGameReleaseVoidOwner::ClearSkillCache
            | WorldGameReleaseVoidOwner::ClearSkillUsageCache
            | WorldGameReleaseVoidOwner::CleanupSocket
            | WorldGameReleaseVoidOwner::ReleaseBaseMessage
            | WorldGameReleaseVoidOwner::RequestWriteLogWorkerExit => {}
        }
    }

    fn release_optional_owner(&mut self, owner: WorldGameReleaseOptionalOwner) -> bool {
        match owner {
            WorldGameReleaseOptionalOwner::GeneralVariableList => {
                self.domains.general_variables.take().is_some()
            }
            WorldGameReleaseOptionalOwner::DefaultClientResource => {
                self.resources.default_client_resource.clear()
            }
            WorldGameReleaseOptionalOwner::FunctionListFileData
            | WorldGameReleaseOptionalOwner::VariableListFileData
            | WorldGameReleaseOptionalOwner::ScriptFileData
            | WorldGameReleaseOptionalOwner::DupliRegionSetup => false,
        }
    }

    fn release_database_owner(&mut self, owner: WorldGameReleaseDatabaseOwner) -> bool {
        match owner {
            WorldGameReleaseDatabaseOwner::RsPlayer => self.init.player.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsSetup => self.init.setup.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsGenVar => self.init.gen_var.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsFaction => self.init.faction.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsUnion => self.init.union.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsEnemyFactions => {
                self.init.enemy_factions.take().is_some()
            }
            WorldGameReleaseDatabaseOwner::RsVillageWar => {
                std::mem::take(&mut self.init.rs_village_war_created)
            }
            WorldGameReleaseDatabaseOwner::RsCityWar => {
                std::mem::take(&mut self.init.rs_city_war_created)
            }
            WorldGameReleaseDatabaseOwner::GoodsWarMember => false,
            WorldGameReleaseDatabaseOwner::RsRegion => self.init.region.take().is_some(),
            WorldGameReleaseDatabaseOwner::DbCountry => self.init.country.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsGodsBattle => {
                self.init.gods_battle.take().is_some()
            }
        }
    }

    fn release_player_ranks(&mut self) -> PlayerRanksReleaseReport {
        self.domains.player_ranks.release(&mut self.domains.timer)
    }

    fn release_organizing_parameters(&mut self) -> OrganizingParamReleaseReport {
        std::mem::take(&mut self.domains.organizing_parameters)
            .release(&mut self.domains.timer)
    }

    fn join_save_worker(&mut self) -> WorldSaveThreadHandleState {
        self.save
            .as_deref_mut()
            .map_or(WorldSaveThreadHandleState::Empty, WorldProcessSaveRuntime::join)
    }
}

impl WorldGameReleaseContext for WorldProcessRuntime {
    fn put_debug_string(&mut self, payload: &'static [u8]) {
        self.release_context().put_debug_string(payload)
    }

    fn save_city_region(&mut self, region_id: i32, region: &mut WorldRegionOwner) {
        self.release_context().save_city_region(region_id, region)
    }

    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer) {
        self.release_context().exit_network_server_worker(server)
    }

    fn exit_network_client_worker(&mut self, client: &mut CMyNetClient) {
        self.release_context().exit_network_client_worker(client)
    }

    fn release_void_owner(&mut self, owner: WorldGameReleaseVoidOwner) {
        self.release_context().release_void_owner(owner)
    }

    fn release_optional_owner(&mut self, owner: WorldGameReleaseOptionalOwner) -> bool {
        self.release_context().release_optional_owner(owner)
    }

    fn release_database_owner(&mut self, owner: WorldGameReleaseDatabaseOwner) -> bool {
        self.release_context().release_database_owner(owner)
    }

    fn release_player_ranks(&mut self) -> PlayerRanksReleaseReport {
        self.release_context().release_player_ranks()
    }

    fn release_organizing_parameters(&mut self) -> OrganizingParamReleaseReport {
        self.release_context().release_organizing_parameters()
    }

    fn join_save_worker(&mut self) -> WorldSaveThreadHandleState {
        self.release_context().join_save_worker()
    }
}

impl WorldGameThreadRuntime for WorldProcessRuntime {
    type Game = CGame;
    type InitBlock = Infallible;
    type MainLoopBlock = WorldProcessMainLoopBlock;

    fn initialize_game<'game>(
        &'game mut self,
        game: &'game mut CGame,
    ) -> Pin<Box<dyn Future<Output = WorldGameInitResult<Self::InitBlock>> + 'game>> {
        Box::pin(async move {
            let result = self
                .domains
                .initialize_game(
                    game,
                    &self.runtime_directory,
                    &mut self.init,
                    &mut self.resources,
                )
                .await;
            if result.is_ok() {
                match WorldProcessSaveRuntime::after_game_init(
                    self.runtime.clone(),
                    &self.init,
                    &self.domains,
                ) {
                    Ok(save) => self.save = Some(save),
                    Err(error) => self.post_init_error = Some(error),
                }
                if self.post_init_error.is_none() {
                    match WorldProcessMainLoopContexts::after_game_init(
                        self.runtime.clone(),
                        game,
                        &mut self.init,
                        &mut self.resources,
                        &self.domains,
                    ) {
                        Ok(contexts) => self.contexts = Some(contexts),
                        Err(error) => self.post_init_error = Some(error),
                    }
                }
            }
            result
        })
    }

    fn game_thread_exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Acquire)
    }

    fn run_main_loop<'game>(
        &'game mut self,
        game: &'game mut CGame,
    ) -> Pin<Box<dyn Future<Output = Result<i32, Self::MainLoopBlock>> + 'game>> {
        Box::pin(async move {
            if let Some(error) = self.post_init_error.take() {
                return Err(WorldProcessMainLoopBlock::PostInit(error));
            }
            self.run_connected_main_loop(game).await
        })
    }

    fn wait_for_save_barrier(&mut self) {
        if let Some(save) = self.save.as_ref() {
            save.wait_for_barrier();
        }
    }

    fn take_goods_war_member(&mut self) -> CGoodsWarMember {
        std::mem::replace(
            &mut self.domains.goods_war,
            CGoodsWarMember::with_reached_empty_state(),
        )
    }

    fn restore_goods_war_member(&mut self, owner: CGoodsWarMember) {
        self.domains.goods_war = owner;
    }

    fn take_increment_log(&mut self) -> CIncrementLog {
        std::mem::replace(&mut self.domains.increment_log, CIncrementLog::new())
    }

    fn restore_increment_log(&mut self, owner: CIncrementLog) {
        self.domains.increment_log = owner;
    }

    fn take_skill_factory(&mut self) -> CSkillFactory {
        std::mem::take(&mut self.domains.skills)
    }

    fn restore_skill_factory(&mut self, owner: CSkillFactory) {
        self.domains.skills = owner;
    }

    fn signal_game_thread_exit(&mut self) {
        self.game_thread_exited = true;
    }

    fn request_window_close(&mut self) {
        self.window_close_requested = true;
    }
}
