//! Db-context AuthServer (очереди требований/результатов и server info), перенесённый в Realm `access/`.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard};

use super::configreader::ConfigReader;
use crate::access::dbqueue::{
    AuthExResultData, AuthResultData, DbQuest, DbResult, LockResultData, ServerInfo,
    ServerInfoQueue,
};
use crate::access::kl_ipfilter::IpFilter;
use crate::access::kl_multi_list::MultiList;

#[derive(Clone)]
pub struct AuthDbContext {
    pub config: Arc<RwLock<ConfigReader>>,
    pub quests: Arc<MultiList<DbQuest>>,
    pub results: Arc<MultiList<DbResult>>,
    server_info: Arc<ServerInfoQueue>,
    pub client_ip_forbider: Arc<RwLock<IpFilter<false>>>,
}

impl AuthDbContext {
    pub fn new(config: ConfigReader) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            quests: Arc::new(MultiList::new()),
            results: Arc::new(MultiList::new()),
            server_info: Arc::new(ServerInfoQueue::new()),
            client_ip_forbider: Arc::new(RwLock::new(IpFilter::new())),
        }
    }

    pub fn config(&self) -> RwLockReadGuard<'_, ConfigReader> {
        self.config.read()
    }

    pub fn push_quest(&self, quest: DbQuest) -> bool {
        if self.quests.size() < self.config.read().max_auth_queue_size() as u32 {
            self.quests.push_back(quest);
            return true;
        }

        let rejection = match quest {
            DbQuest::Authenticate {
                return_socket_id,
                request,
            } => Some(DbResult::Authenticate {
                return_socket_id,
                result: AuthResultData::new(
                    6,
                    request.account,
                    request.client_ip,
                    request.client_socket_id,
                ),
            }),
            DbQuest::AuthenticateExtended {
                return_socket_id,
                request,
            } => Some(DbResult::AuthenticateExtended {
                return_socket_id,
                result: AuthExResultData::new(
                    6,
                    request.account,
                    request.client_ip,
                    request.client_socket_id,
                ),
            }),
            DbQuest::Lock {
                return_socket_id,
                request,
            } => Some(DbResult::Lock {
                return_socket_id,
                result: LockResultData {
                    account: request.account,
                    succeeded: false,
                },
            }),
            DbQuest::WriteServerInfo => None,
        };

        if let Some(rejection) = rejection {
            self.results.push_back(rejection);
        }
        false
    }

    pub fn quest_count(&self) -> u32 {
        self.quests.size()
    }

    pub fn pop_quest_until_stopped(&self, stopped: &AtomicBool) -> Option<DbQuest> {
        self.quests.pop_front_wait_until_stopped(stopped)
    }

    pub fn wake_quest_waiters(&self) {
        self.quests.wake_all();
    }

    pub fn push_result(&self, result: DbResult) {
        self.results.push_back(result);
    }

    pub fn push_server_info(&self, info: ServerInfo) {
        self.server_info.push_back(info);
    }

    pub fn pop_all_server_info(&self) -> std::collections::VecDeque<ServerInfo> {
        self.server_info.pop_all()
    }

    pub fn is_client_ip_allowed(&self, address: u32) -> bool {
        !self.config.read().client_ip_filter_enabled()
            || self
                .client_ip_forbider
                .read()
                .is_allowed(address.to_le_bytes())
    }
}
