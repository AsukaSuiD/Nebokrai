//! Достигнутая часть поиска GameServer `CSessionFactory`.
//!
//! `QuerySession` RVA `0x000780C0` и `QueryPlug` RVA `0x00078190` подтверждены
//! дизассемблированием точной пары `GameServer/gameserver.exe +
//! GameServer/GameServer.pdb`; исходный владелец —
//! `server/gameserver/appserver/session/csessionfactory.cpp`. PDB подтверждает
//! две статические `hash_map<long, CSession*/CPlug*>`; оба поиска возвращают
//! null при отсутствии ключа.
//!
//! Один принадлежащий `CGame` экземпляр `CSessionFactory` заменяет две
//! статические карты процесса. `BTreeMap` служит детерминированной заменой:
//! достигнутые функции не наблюдают порядок обхода хеш-таблицы и выполняют
//! только поиск по точному ключу. `register_*` материализует хранилище реестра,
//! а `goodsmessage 0x8FC25` выполняет упорядоченный поиск разъёма сессии по
//! типу и идентификатору владельца.
//!
//! Закрытие сессии улучшения экипировки доведено до сборки завершённых сессий.
//! Сценарные входы трёх механизмов экипировки создают обычную сессию и
//! типизированный разъём, связывают владельца с сессией и сохраняют теневые
//! идентификаторы владельца, расширения и порядок вставки. Обработчик
//! контейнерных сообщений разрешает пару `(session, plug << 8)`, записывает и
//! снимает типизированные состояния upgrade, DaKong, compose и продавца
//! personal-shop в исходной ячейке игрока. Завершения `End/Exit` хранятся здесь,
//! а сборка завершённой сессии сохраняет порядок сессии и разъёмов, а также
//! идентификатор владельца для отключения слушателей на стадии сессий
//! `MainLoop`.
//!
//! Достигнутый жизненный цикл personal-shop создаёт обычную сессию `(1, 20,
//! 0)`, типизированные разъёмы продавца и покупателя, точные связи владельца с
//! сессией и теневые сведения магазина. Создание команды и вход в неё хранят
//! типизированные `CTeam/CTeamate`, общий распределитель идентификаторов,
//! порядок вставки и промежуточные снимки сериализации. Переходы распределения
//! и чата получают владельцев участников в порядке той же сессии. Снимок из
//! WorldServer и последующая репликация используют тот же реестр и сохраняют
//! порядок клиентских снимков. Session start timestamps берутся из текущего
//! MainLoop sample; ненулевой lifetime команды завершается через terminal
//! report, чтобы `CGame` сохранил derived World/client side effects.

use std::collections::BTreeMap;

use crate::gameserver::appserver::container::camountlimitgoodsshadowcontainer::AmountShadowAdded;
use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::ComposeShadowAddBlock;
use crate::gameserver::appserver::container::cequipmentdakongcontainer::{
    DaKongAddBlock, DaKongAddOutcome,
};
use crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::UpgradeShadowAddBlock;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::{
    PlacedShadowGoods, ShadowRecordBlock, ShadowRemovedReport,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

use super::cequipmentcompose::CEquipmentCompose;
use super::cequipmentdakong::CEquipmentDaKong;
use super::cequipmentupgrade::CEquipmentUpgrade;
use super::cpersonalshopbuyer::CPersonalShopBuyer;
use super::cpersonalshopseller::CPersonalShopSeller;
use super::cplug::CPlug;
use super::csession::CSession;
use super::cteam::CTeam;
use super::cteamate::{CTeamate, TeamMateAvailability};
use super::ctrader::CTrader;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionPlugKind {
    Upgrade,
    DaKong,
    Compose,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentSessionShadowAddBlock {
    MissingSessionOrPlug,
    OwnerMismatch,
    InvalidContainerIndex,
    Upgrade(UpgradeShadowAddBlock),
    DaKong(DaKongAddBlock),
    Compose(ComposeShadowAddBlock),
}

#[must_use = "shadow add содержит actual cell и AddShadow publication data"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSessionShadowAdded {
    pub(crate) kind: EquipmentSessionPlugKind,
    pub(crate) plug_id: i32,
    pub(crate) position: u32,
    pub(crate) shadow: AmountShadowAdded,
}

#[must_use = "shadow remove сохраняет original source и delete publication"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentSessionShadowRemoved {
    pub(crate) kind: EquipmentSessionPlugKind,
    pub(crate) plug_id: i32,
    pub(crate) original: PreviousContainer,
    pub(crate) removed: ShadowRemovedReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersonalShopShadowAddBlock {
    MissingSessionOrPlug,
    OwnerMismatch,
    InvalidContainerIndex,
    PositionUnavailable,
    Shadow(ShadowRecordBlock),
}

#[must_use = "personal-shop add сохраняет actual cell и AddShadow publication data"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopShadowAdded {
    pub(crate) plug_id: i32,
    pub(crate) position: u32,
    pub(crate) shadow: AmountShadowAdded,
}

#[must_use = "personal-shop remove сохраняет original source и delete publication"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopShadowRemoved {
    pub(crate) plug_id: i32,
    pub(crate) original: PreviousContainer,
    pub(crate) removed: ShadowRemovedReport,
}

#[must_use = "terminal GC report сохраняет session и ordered plug identities"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalEquipmentSessionCollected {
    pub(crate) session_id: i32,
    pub(crate) plugs: Vec<TerminalEquipmentPlugCollected>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TerminalEquipmentPlugCollected {
    pub(crate) plug_id: i32,
    pub(crate) owner_id: i32,
}

#[must_use = "team creation сохраняет intermediate client/World snapshots"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamSessionCreated {
    pub(crate) session_id: i32,
    pub(crate) team_id: u32,
    pub(crate) empty_snapshot: Vec<u8>,
    pub(crate) leader_snapshot: Vec<u8>,
    pub(crate) candidate_snapshot: Vec<u8>,
    pub(crate) teammate_ids: Vec<i32>,
}

#[must_use = "team insertion сохраняет new owner snapshot и resulting count"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamMemberInserted {
    pub(crate) session_id: i32,
    pub(crate) team_id: u32,
    pub(crate) snapshot: Vec<u8>,
    pub(crate) teammate_count: usize,
    pub(crate) teammate_ids: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamMemberSnapshot {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) owner_region_id: i32,
    pub(crate) owner_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamSessionSnapshot {
    pub(crate) minimum_plugs: u32,
    pub(crate) maximum_plugs: u32,
    pub(crate) lifetime: u32,
    pub(crate) team_id: u32,
    pub(crate) team_name: Vec<u8>,
    pub(crate) password: Vec<u8>,
    pub(crate) leader_id: i32,
    pub(crate) members: Vec<TeamMemberSnapshot>,
}

#[must_use = "restored team сохраняет insertion snapshots и ordered members"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamSessionRestored {
    pub(crate) session_id: i32,
    pub(crate) team_id: u32,
    pub(crate) insertion_snapshots: Vec<Vec<u8>>,
    pub(crate) player_ids: Vec<i32>,
}

#[must_use = "team removal сохраняет removed owner и remaining members"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamMemberRemoved {
    pub(crate) session_id: i32,
    pub(crate) team_id: u32,
    pub(crate) player_id: i32,
    pub(crate) leader_id: i32,
    pub(crate) remaining_player_ids: Vec<i32>,
}

#[must_use = "recovered teammate требует восстановления membership и snapshot"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamMemberRecovered {
    pub(crate) session_id: i32,
    pub(crate) team_id: u32,
    pub(crate) player_id: i32,
    pub(crate) snapshot: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TeamPlugAiReport {
    pub(crate) recovered: Vec<TeamMemberRecovered>,
    pub(crate) expired: Vec<TeamMemberRemoved>,
}

#[must_use = "team disband сохраняет ordered owners до registry GC"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamSessionDisbanded {
    pub(crate) session_id: i32,
    pub(crate) team_id: u32,
    pub(crate) leader_id: i32,
    pub(crate) player_ids: Vec<i32>,
}

#[derive(Debug)]
pub(crate) struct CSessionFactory {
    sessions: BTreeMap<i32, CSession>,
    plugs: BTreeMap<i32, CPlug>,
    equipment_compose_plugs: BTreeMap<i32, CEquipmentCompose>,
    equipment_da_kong_plugs: BTreeMap<i32, CEquipmentDaKong>,
    equipment_upgrade_plugs: BTreeMap<i32, CEquipmentUpgrade>,
    personal_shop_seller_plugs: BTreeMap<i32, CPersonalShopSeller>,
    personal_shop_buyer_plugs: BTreeMap<i32, CPersonalShopBuyer>,
    trader_plugs: BTreeMap<i32, CTrader>,
    teams: BTreeMap<i32, CTeam>,
    teammates: BTreeMap<i32, CTeamate>,
    next_session_id: i32,
    next_plug_id: i32,
}

impl Default for CSessionFactory {
    fn default() -> Self {
        Self {
            sessions: BTreeMap::new(),
            plugs: BTreeMap::new(),
            equipment_compose_plugs: BTreeMap::new(),
            equipment_da_kong_plugs: BTreeMap::new(),
            equipment_upgrade_plugs: BTreeMap::new(),
            personal_shop_seller_plugs: BTreeMap::new(),
            personal_shop_buyer_plugs: BTreeMap::new(),
            trader_plugs: BTreeMap::new(),
            teams: BTreeMap::new(),
            teammates: BTreeMap::new(),
            next_session_id: 1,
            next_plug_id: 1,
        }
    }
}

impl CSessionFactory {
    pub(crate) fn create_team_session(
        &mut self,
        now_ms: u32,
        team_id: u32,
        leader: (i32, i32, &[u8]),
        candidate: (i32, i32, &[u8]),
    ) -> Option<TeamSessionCreated> {
        let session_id = self.next_session_id;
        let leader_plug_id = self.next_plug_id;
        let candidate_plug_id = self.next_plug_id.wrapping_add(1);
        let mut session = CSession::normal(2, 8, 0);
        let mut team = CTeam::new(team_id);
        if !session.start(now_ms) {
            return None;
        }
        let empty_snapshot = team.serialize(&session, now_ms, std::iter::empty());

        let mut leader_base = CPlug::new();
        leader_base.set_id(leader_plug_id);
        leader_base.set_owner(400, leader.0);
        leader_base.set_session(session_id);
        leader_base.set_plug_type(5);
        let leader_teammate = CTeamate::new(leader_plug_id, leader.0, leader.1, leader.2);
        if !session.insert_plug(leader_plug_id) {
            return None;
        }
        let leader_snapshot = team.serialize(&session, now_ms, [&leader_teammate]);

        let mut candidate_base = CPlug::new();
        candidate_base.set_id(candidate_plug_id);
        candidate_base.set_owner(400, candidate.0);
        candidate_base.set_session(session_id);
        candidate_base.set_plug_type(5);
        let candidate_teammate =
            CTeamate::new(candidate_plug_id, candidate.0, candidate.1, candidate.2);
        if !session.insert_plug(candidate_plug_id) {
            return None;
        }
        let candidate_snapshot = team.serialize(
            &session,
            now_ms,
            [&leader_teammate, &candidate_teammate],
        );
        team.set_leader(leader.0);

        self.next_session_id = self.next_session_id.wrapping_add(1);
        self.next_plug_id = self.next_plug_id.wrapping_add(2);
        self.sessions.insert(session_id, session);
        self.plugs.insert(leader_plug_id, leader_base);
        self.plugs.insert(candidate_plug_id, candidate_base);
        self.teammates.insert(leader_plug_id, leader_teammate);
        self.teammates.insert(candidate_plug_id, candidate_teammate);
        self.teams.insert(session_id, team);
        Some(TeamSessionCreated {
            session_id,
            team_id,
            empty_snapshot,
            leader_snapshot,
            candidate_snapshot,
            teammate_ids: vec![leader.0, candidate.0],
        })
    }

    pub(crate) fn insert_team_member(
        &mut self,
        now_ms: u32,
        session_id: i32,
        owner_id: i32,
        owner_region_id: i32,
        owner_name: &[u8],
    ) -> Option<TeamMemberInserted> {
        self.insert_team_member_owned(
            now_ms,
            session_id,
            400,
            owner_id,
            owner_region_id,
            owner_name,
        )
    }

    pub(crate) fn insert_team_member_owned(
        &mut self,
        now_ms: u32,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
        owner_region_id: i32,
        owner_name: &[u8],
    ) -> Option<TeamMemberInserted> {
        if self
            .query_session_plug_by_owner(session_id, owner_type, owner_id)
            .is_some()
            || !self.teams.contains_key(&session_id)
        {
            return None;
        }
        let plug_id = self.next_plug_id;
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(owner_type, owner_id);
        base.set_session(session_id);
        base.set_plug_type(5);
        let teammate =
            CTeamate::new_owned(plug_id, owner_type, owner_id, owner_region_id, owner_name);
        if !self.sessions.get_mut(&session_id)?.insert_plug(plug_id) {
            return None;
        }
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        self.plugs.insert(plug_id, base);
        self.teammates.insert(plug_id, teammate);
        let session = self.sessions.get(&session_id)?;
        let team = self.teams.get(&session_id)?;
        let snapshot = team.serialize(
            session,
            now_ms,
            session
                .plug_ids_storage()
                .iter()
                .filter_map(|id| self.teammates.get(id)),
        );
        let teammate_ids = session
            .plug_ids_storage()
            .iter()
            .filter_map(|id| self.teammates.get(id).map(CTeamate::owner_id))
            .collect();
        Some(TeamMemberInserted {
            session_id,
            team_id: team.team_id(),
            snapshot,
            teammate_count: session.plug_ids_storage().len(),
            teammate_ids,
        })
    }

    pub(crate) fn restore_team_session(
        &mut self,
        now_ms: u32,
        snapshot: TeamSessionSnapshot,
    ) -> Option<TeamSessionRestored> {
        let session_id = self.next_session_id;
        let team_id = snapshot.team_id;
        let mut session = CSession::normal(
            snapshot.minimum_plugs,
            snapshot.maximum_plugs,
            snapshot.lifetime,
        );
        if !session.start(now_ms) {
            return None;
        }
        let team = CTeam::restored(
            snapshot.team_id,
            snapshot.team_name,
            snapshot.password,
            snapshot.leader_id,
        );
        self.next_session_id = self.next_session_id.wrapping_add(1);
        self.sessions.insert(session_id, session);
        self.teams.insert(session_id, team);

        let mut insertion_snapshots = Vec::with_capacity(snapshot.members.len());
        let mut player_ids = Vec::with_capacity(snapshot.members.len());
        for member in snapshot.members {
            let inserted = self.insert_team_member_owned(
                now_ms,
                session_id,
                member.owner_type,
                member.owner_id,
                member.owner_region_id,
                &member.owner_name,
            )?;
            player_ids.push(member.owner_id);
            insertion_snapshots.push(inserted.snapshot);
        }
        Some(TeamSessionRestored {
            session_id,
            team_id,
            insertion_snapshots,
            player_ids,
        })
    }

    pub(crate) fn update_team_member_region(
        &mut self,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
        owner_region_id: i32,
    ) -> Option<Vec<i32>> {
        let plug_id = self
            .query_session_plug_by_owner(session_id, owner_type, owner_id)?
            .id();
        self.teammates
            .get_mut(&plug_id)?
            .set_owner_region_id(owner_region_id);
        self.team_player_ids(session_id)
    }

    pub(crate) fn team_member(
        &self,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> Option<&CTeamate> {
        let plug_id = self
            .query_session_plug_by_owner(session_id, owner_type, owner_id)?
            .id();
        self.teammates.get(&plug_id)
    }

    pub(crate) fn query_team(&self, session_id: i32) -> Option<&CTeam> {
        self.teams.get(&session_id)
    }

    pub(crate) fn team_member_count(&self, session_id: i32) -> Option<usize> {
        self.teams
            .contains_key(&session_id)
            .then(|| self.sessions.get(&session_id))
            .flatten()
            .map(|session| session.plug_ids_storage().len())
    }

    pub(crate) fn set_team_leader(&mut self, session_id: i32, player_id: i32) -> Option<i32> {
        self.query_session_plug_by_owner(session_id, 400, player_id)?;
        let team = self.teams.get_mut(&session_id)?;
        let previous = team.leader_id();
        team.set_leader(player_id);
        Some(previous)
    }

    pub(crate) fn set_team_allocation_scheme(
        &mut self,
        session_id: i32,
        actor_id: i32,
        allocation_scheme: i32,
    ) -> Option<(u32, Vec<i32>)> {
        self.query_session_plug_by_owner(session_id, 400, actor_id)?;
        let team = self.teams.get(&session_id)?;
        if team.leader_id() != actor_id {
            return None;
        }
        let player_ids = self.team_player_ids(session_id)?;
        let team = self.teams.get_mut(&session_id)?;
        team.set_allocation_scheme(allocation_scheme);
        Some((team.team_id(), player_ids))
    }

    pub(crate) fn team_player_ids(&self, session_id: i32) -> Option<Vec<i32>> {
        let session = self.sessions.get(&session_id)?;
        self.teams.get(&session_id)?;
        session
            .plug_ids_storage()
            .iter()
            .map(|plug_id| self.teammates.get(plug_id).map(CTeamate::owner_id))
            .collect()
    }

    /// Base `CSession::AI` lifetime gate для concrete `CTeam`: сначала
    /// выставляется terminal End state, затем registry owner собирает сессию.
    /// Возвращённый отчёт позволяет `CGame` исполнить derived callbacks и
    /// межсерверную публикацию до следующего session pass.
    pub(crate) fn expire_team_sessions(&mut self, now_ms: u32) -> Vec<TeamSessionDisbanded> {
        let session_ids = self
            .teams
            .keys()
            .copied()
            .filter(|session_id| {
                self.sessions
                    .get(session_id)
                    .is_some_and(|session| session.lifetime_expired(now_ms))
            })
            .collect::<Vec<_>>();
        session_ids
            .into_iter()
            .filter_map(|session_id| {
                let _plug_callbacks = self.sessions.get_mut(&session_id)?.end();
                self.disband_team(session_id)
            })
            .collect()
    }

    /// Base `CSession::AI` plug traversal для concrete teammate plugs. Один
    /// missing/expired plug завершает проход конкретной session, как ранний
    /// `return` исходного списка; recovered plugs продолжают ordered обход.
    pub(crate) fn run_team_plug_ai(
        &mut self,
        now_ms: u32,
        mut owner_is_local: impl FnMut(i32) -> bool,
        mut region_is_local: impl FnMut(i32) -> bool,
    ) -> TeamPlugAiReport {
        let session_ids = self.teams.keys().copied().collect::<Vec<_>>();
        let mut report = TeamPlugAiReport::default();
        for session_id in session_ids {
            if self
                .sessions
                .get(&session_id)
                .is_none_or(|session| !session.is_available_prefix())
            {
                continue;
            }
            let plug_ids = self
                .sessions
                .get(&session_id)
                .expect("active team session проверена")
                .plug_ids_storage()
                .to_vec();
            for plug_id in plug_ids {
                if !self.plugs.contains_key(&plug_id) {
                    let _removed = self
                        .sessions
                        .get_mut(&session_id)
                        .expect("team session остаётся active")
                        .remove_plug(plug_id);
                    break;
                }
                let availability = {
                    let Some(teammate) = self.teammates.get_mut(&plug_id) else {
                        continue;
                    };
                    teammate.availability(
                        now_ms,
                        teammate.owner_type() == 400 && owner_is_local(teammate.owner_id()),
                        region_is_local(teammate.owner_region_id()),
                    )
                };
                match availability {
                    TeamMateAvailability::Available => {}
                    TeamMateAvailability::Recovered => {
                        let session = self
                            .sessions
                            .get(&session_id)
                            .expect("recovered team session остаётся canonical");
                        let team = self
                            .teams
                            .get(&session_id)
                            .expect("recovered team owner остаётся canonical");
                        let snapshot = team.serialize(
                            session,
                            now_ms,
                            session
                                .plug_ids_storage()
                                .iter()
                                .filter_map(|id| self.teammates.get(id)),
                        );
                        report.recovered.push(TeamMemberRecovered {
                            session_id,
                            team_id: team.team_id(),
                            player_id: self
                                .teammates
                                .get(&plug_id)
                                .expect("recovered teammate остаётся canonical")
                                .owner_id(),
                            snapshot,
                        });
                    }
                    TeamMateAvailability::Expired => {
                        let player_id = self
                            .teammates
                            .get(&plug_id)
                            .expect("expired teammate остаётся до callback")
                            .owner_id();
                        if let Some(expired) = self.remove_team_member(session_id, player_id) {
                            report.expired.push(expired);
                        }
                        break;
                    }
                }
            }
        }
        report
    }

    /// Достигнутая derived-часть `CTeam::AI`: возвращает только локальные
    /// team-проекции, у которых после минутной границы не осталось ни одного
    /// живого player-owner-а на этом GameServer.
    pub(crate) fn idle_team_sessions(
        &mut self,
        now_ms: u32,
        mut owner_is_local: impl FnMut(i32) -> bool,
    ) -> Vec<(i32, u32)> {
        let session_ids: Vec<i32> = self.teams.keys().copied().collect();
        session_ids
            .into_iter()
            .filter_map(|session_id| {
                if !self.teams.get_mut(&session_id)?.idle_check_due(now_ms) {
                    return None;
                }
                let has_local_player = self
                    .sessions
                    .get(&session_id)?
                    .plug_ids_storage()
                    .iter()
                    .filter_map(|plug_id| self.teammates.get(plug_id))
                    .map(CTeamate::owner_id)
                    .any(&mut owner_is_local);
                if has_local_player {
                    return None;
                }
                Some((session_id, self.teams.get(&session_id)?.team_id()))
            })
            .collect()
    }

    pub(crate) fn team_member_descriptors(&self, session_id: i32) -> Option<Vec<(i32, i32, i32)>> {
        let session = self.sessions.get(&session_id)?;
        self.teams.get(&session_id)?;
        session
            .plug_ids_storage()
            .iter()
            .map(|plug_id| {
                self.teammates.get(plug_id).map(|teammate| {
                    (
                        teammate.owner_type(),
                        teammate.owner_id(),
                        teammate.owner_region_id(),
                    )
                })
            })
            .collect()
    }

    pub(crate) fn remove_team_member(
        &mut self,
        session_id: i32,
        player_id: i32,
    ) -> Option<TeamMemberRemoved> {
        let plug_id = self
            .query_session_plug_by_owner(session_id, 400, player_id)?
            .id();
        let team = self.teams.get(&session_id)?;
        let team_id = team.team_id();
        let leader_id = team.leader_id();
        if !self.sessions.get_mut(&session_id)?.remove_plug(plug_id) {
            return None;
        }
        self.plugs.remove(&plug_id);
        self.teammates.remove(&plug_id);
        let remaining_player_ids = self
            .sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .filter_map(|id| self.teammates.get(id).map(CTeamate::owner_id))
            .collect();
        Some(TeamMemberRemoved {
            session_id,
            team_id,
            player_id,
            leader_id,
            remaining_player_ids,
        })
    }

    pub(crate) fn disband_team(&mut self, session_id: i32) -> Option<TeamSessionDisbanded> {
        let team = self.teams.get(&session_id)?;
        let team_id = team.team_id();
        let leader_id = team.leader_id();
        let player_ids = self
            .sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .filter_map(|id| self.teammates.get(id).map(CTeamate::owner_id))
            .collect();
        let _removed = self.garbage_collect_session(session_id);
        Some(TeamSessionDisbanded {
            session_id,
            team_id,
            leader_id,
            player_ids,
        })
    }
    /// Exact normal `(2, 2, 0)` player trade: первый plug принадлежит
    /// пригласившему, второй — отвечающему, как два последовательных
    /// `CreatePlug/InsertPlug` в `0x8FA07`.
    pub(crate) fn create_player_trade_session(
        &mut self,
        now_ms: u32,
        inviter_id: i32,
        answerer_id: i32,
    ) -> Option<(i32, i32, i32)> {
        let session_id = self.next_session_id;
        let mut session = CSession::normal(2, 2, 0);
        if !session.start(now_ms) {
            return None;
        }
        let inviter_plug_id = self.next_plug_id;
        let answerer_plug_id = self.next_plug_id.wrapping_add(1);
        let mut inviter = CPlug::new();
        inviter.set_id(inviter_plug_id);
        inviter.set_owner(400, inviter_id);
        inviter.set_session(session_id);
        inviter.set_plug_type(1);
        let mut answerer = CPlug::new();
        answerer.set_id(answerer_plug_id);
        answerer.set_owner(400, answerer_id);
        answerer.set_session(session_id);
        answerer.set_plug_type(1);
        if !session.insert_plug(inviter_plug_id) || !session.insert_plug(answerer_plug_id) {
            return None;
        }
        self.next_session_id = self.next_session_id.wrapping_add(1);
        self.next_plug_id = self.next_plug_id.wrapping_add(2);
        self.sessions.insert(session_id, session);
        self.plugs.insert(inviter_plug_id, inviter);
        self.plugs.insert(answerer_plug_id, answerer);
        self.trader_plugs.insert(
            inviter_plug_id,
            CTrader::inserted(inviter_plug_id, session_id, inviter_id),
        );
        self.trader_plugs.insert(
            answerer_plug_id,
            CTrader::inserted(answerer_plug_id, session_id, answerer_id),
        );
        Some((session_id, inviter_plug_id, answerer_plug_id))
    }

    pub(crate) fn query_trader(&self, plug_id: i32) -> Option<&CTrader> {
        self.trader_plugs.get(&plug_id)
    }

    pub(crate) fn query_trader_mut(&mut self, plug_id: i32) -> Option<&mut CTrader> {
        self.trader_plugs.get_mut(&plug_id)
    }

    pub(crate) fn trader_plug_by_owner(&self, session_id: i32, owner_id: i32) -> Option<i32> {
        let plug = self.query_session_plug_by_owner(session_id, 400, owner_id)?;
        self.trader_plugs
            .contains_key(&plug.id())
            .then_some(plug.id())
    }

    pub(crate) fn contrary_trader_id(&self, session_id: i32, plug_id: i32) -> Option<i32> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .copied()
            .find(|candidate| *candidate != plug_id && self.trader_plugs.contains_key(candidate))
    }

    pub(crate) fn trade_session_plug_ids(&self, session_id: i32) -> Option<Vec<i32>> {
        let session = self.sessions.get(&session_id)?;
        let ids: Vec<_> = session
            .plug_ids_storage()
            .iter()
            .copied()
            .filter(|plug_id| self.trader_plugs.contains_key(plug_id))
            .collect();
        (ids.len() == session.plug_ids_storage().len()).then_some(ids)
    }

    pub(crate) fn trade_session_available(
        &self,
        session_id: i32,
        mut owner_available: impl FnMut(i32) -> bool,
    ) -> bool {
        let Some(session) = self.sessions.get(&session_id) else {
            return false;
        };
        if !session.is_available_prefix() {
            return false;
        }
        let available = session
            .plug_ids_storage()
            .iter()
            .filter_map(|plug_id| self.trader_plugs.get(plug_id))
            .filter(|trader| owner_available(trader.owner_id()))
            .count();
        session.minimum_plugs() as usize <= available
    }

    pub(crate) fn abort_session(&mut self, session_id: i32) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            tracing::trace!(session_id, "прерываемая сессия не найдена");
            return;
        };
        let callback_plug_ids = session.abort();
        let callback_plug_ids = callback_plug_ids
            .into_iter()
            .filter(|plug_id| self.plugs.contains_key(plug_id))
            .collect::<Vec<_>>();
        let session = self.sessions.get(&session_id).expect("сессия остаётся до GC");
        tracing::trace!(session_id, ended = session.is_ended(), remove_requested = session.remove_requested(), ?callback_plug_ids, "сессия прервана");
    }

    /// Exact normal `(1, 20, 0)` session + personal-shop seller plug `(400,
    /// player)`. Свежая session всегда проходит `Start(0)` и первый insertion;
    /// поэтому safe atomic publication не меняет достижимый legacy outcome.
    pub(crate) fn create_personal_shop_seller_session(
        &mut self,
        now_ms: u32,
        player_id: i32,
    ) -> Option<(i32, i32)> {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        let mut session = CSession::normal(1, 20, 0);
        if !session.start(now_ms) {
            return None;
        }

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(400, player_id);
        base.set_session(session_id);
        base.set_plug_type(2);
        if !session.insert_plug(plug_id) {
            return None;
        }

        self.sessions.insert(session_id, session);
        self.plugs.insert(plug_id, base);
        self.personal_shop_seller_plugs
            .insert(plug_id, CPersonalShopSeller::inserted(plug_id));
        Some((session_id, plug_id))
    }

    pub(crate) fn personal_shop_seller(&self, plug_id: i32) -> Option<&CPersonalShopSeller> {
        self.personal_shop_seller_plugs.get(&plug_id)
    }

    pub(crate) fn personal_shop_seller_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CPersonalShopSeller> {
        self.personal_shop_seller_plugs.get_mut(&plug_id)
    }

    pub(crate) fn personal_shop_buyer(&self, plug_id: i32) -> Option<&CPersonalShopBuyer> {
        self.personal_shop_buyer_plugs.get(&plug_id)
    }

    pub(crate) fn personal_shop_seller_plug_id(&self, session_id: i32) -> Option<i32> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .copied()
            .find(|plug_id| self.personal_shop_seller_plugs.contains_key(plug_id))
    }

    pub(crate) fn personal_shop_session_available(&self, session_id: i32) -> bool {
        self.sessions
            .get(&session_id)
            .is_some_and(CSession::is_available_prefix)
    }

    pub(crate) fn insert_personal_shop_buyer(
        &mut self,
        session_id: i32,
        owner_id: i32,
    ) -> Option<i32> {
        let session = self.sessions.get_mut(&session_id)?;
        if !session.is_available_prefix() {
            return None;
        }
        let plug_id = self.next_plug_id;
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(400, owner_id);
        base.set_session(session_id);
        base.set_plug_type(3);
        if !session.insert_plug(plug_id) {
            return None;
        }
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        self.plugs.insert(plug_id, base);
        self.personal_shop_buyer_plugs.insert(
            plug_id,
            CPersonalShopBuyer::inserted(plug_id, session_id, owner_id),
        );
        Some(plug_id)
    }

    pub(crate) fn remove_personal_shop_buyer(
        &mut self,
        session_id: i32,
        plug_id: i32,
    ) -> Option<CPersonalShopBuyer> {
        let buyer = self.personal_shop_buyer_plugs.remove(&plug_id)?;
        if buyer.session_id() != session_id {
            self.personal_shop_buyer_plugs.insert(plug_id, buyer);
            return None;
        }
        let _ = self.sessions.get_mut(&session_id)?.remove_plug(plug_id);
        self.plugs.remove(&plug_id);
        Some(buyer)
    }

    pub(crate) fn personal_shop_participants(
        &self,
        session_id: i32,
    ) -> Option<(i32, i32, Vec<CPersonalShopBuyer>)> {
        let seller_plug_id = self.personal_shop_seller_plug_id(session_id)?;
        let seller_owner_id = self.plugs.get(&seller_plug_id)?.owner_id();
        let buyers = self
            .sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .filter_map(|plug_id| self.personal_shop_buyer_plugs.get(plug_id).copied())
            .collect();
        Some((seller_plug_id, seller_owner_id, buyers))
    }

    fn resolve_personal_shop_seller_plug(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
    ) -> Result<i32, PersonalShopShadowAddBlock> {
        if extend_id & 0xff != 0 {
            return Err(PersonalShopShadowAddBlock::InvalidContainerIndex);
        }
        let plug_id = extend_id >> 8;
        let plug = self
            .query_session_plug_by_owner(session_id, 400, player_id)
            .ok_or(PersonalShopShadowAddBlock::MissingSessionOrPlug)?;
        if plug.id() != plug_id {
            return Err(PersonalShopShadowAddBlock::OwnerMismatch);
        }
        self.personal_shop_seller_plugs
            .contains_key(&plug_id)
            .then_some(plug_id)
            .ok_or(PersonalShopShadowAddBlock::MissingSessionOrPlug)
    }

    pub(crate) fn is_personal_shop_seller_container(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
    ) -> bool {
        self.resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .is_ok()
    }

    pub(crate) fn record_personal_shop_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        requested_position: u32,
        goods: &CGoods,
        previous: PreviousContainer,
    ) -> Result<PersonalShopShadowAdded, PersonalShopShadowAddBlock> {
        let plug_id = self.resolve_personal_shop_seller_plug(session_id, extend_id, player_id)?;
        let seller = self
            .personal_shop_seller_plugs
            .get_mut(&plug_id)
            .expect("personal-shop plug проверен по concrete registry");
        let position = if requested_position == u32::MAX {
            seller
                .goods()
                .query_goods_position(goods.identity().ex_id)
                .or_else(|| {
                    (0..seller.goods().size())
                        .find(|position| seller.goods().is_space_enough(*position))
                })
                .ok_or(PersonalShopShadowAddBlock::PositionUnavailable)?
        } else {
            requested_position
        };
        if seller.goods().query_goods_position(goods.identity().ex_id) != Some(position)
            && !seller.goods().is_space_enough(position)
        {
            return Err(PersonalShopShadowAddBlock::PositionUnavailable);
        }
        let placed = PlacedShadowGoods {
            identity: goods.identity().ex_id,
            position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        let shadow = seller
            .goods_mut()
            .base_mut()
            .record_placed_goods(previous, placed)
            .map_err(PersonalShopShadowAddBlock::Shadow)?;
        if !seller
            .goods_mut()
            .occupy_cell(position, goods.identity().ex_id)
            && seller.goods().query_goods_position(goods.identity().ex_id) != Some(position)
        {
            let _ = seller.goods_mut().remove_shadow(goods.identity().ex_id);
            return Err(PersonalShopShadowAddBlock::PositionUnavailable);
        }
        Ok(PersonalShopShadowAdded {
            plug_id,
            position,
            shadow,
        })
    }

    pub(crate) fn personal_shop_shadow_original(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        let plug_id = self
            .resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .ok()?;
        self.personal_shop_seller_plugs
            .get(&plug_id)?
            .goods()
            .base()
            .base()
            .original_container_information(goods_id)
    }

    pub(crate) fn personal_shop_shadow_position(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<u32> {
        let plug_id = self
            .resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .ok()?;
        self.personal_shop_seller_plugs
            .get(&plug_id)?
            .goods()
            .query_goods_position(goods_id)
    }

    pub(crate) fn remove_personal_shop_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<PersonalShopShadowRemoved> {
        let plug_id = self
            .resolve_personal_shop_seller_plug(session_id, extend_id, player_id)
            .ok()?;
        let original =
            self.personal_shop_shadow_original(session_id, extend_id, player_id, goods_id)?;
        let seller = self.personal_shop_seller_plugs.get_mut(&plug_id)?;
        seller.remove_goods_price(goods_id);
        let removed = seller.goods_mut().remove_shadow(goods_id)?;
        Some(PersonalShopShadowRemoved {
            plug_id,
            original,
            removed,
        })
    }

    fn resolve_equipment_plug(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
    ) -> Result<(EquipmentSessionPlugKind, i32), EquipmentSessionShadowAddBlock> {
        if extend_id & 0xff != 0 {
            return Err(EquipmentSessionShadowAddBlock::InvalidContainerIndex);
        }
        let plug_id = extend_id >> 8;
        let plug = self
            .query_session_plug_by_owner(session_id, 400, player_id)
            .ok_or(EquipmentSessionShadowAddBlock::MissingSessionOrPlug)?;
        if plug.id() != plug_id {
            return Err(EquipmentSessionShadowAddBlock::OwnerMismatch);
        }
        let kind = if self.equipment_upgrade_plugs.contains_key(&plug_id) {
            EquipmentSessionPlugKind::Upgrade
        } else if self.equipment_da_kong_plugs.contains_key(&plug_id) {
            EquipmentSessionPlugKind::DaKong
        } else if self.equipment_compose_plugs.contains_key(&plug_id) {
            EquipmentSessionPlugKind::Compose
        } else {
            return Err(EquipmentSessionShadowAddBlock::MissingSessionOrPlug);
        };
        Ok((kind, plug_id))
    }

    pub(crate) fn record_equipment_session_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        requested_position: u32,
        goods: &CGoods,
        previous: PreviousContainer,
        factory: &CGoodsFactory,
    ) -> Result<EquipmentSessionShadowAdded, EquipmentSessionShadowAddBlock> {
        let (kind, plug_id) = self.resolve_equipment_plug(session_id, extend_id, player_id)?;
        let goods_id = goods.identity().ex_id;
        let placed_position = previous.goods_position;
        let (position, shadow) = match kind {
            EquipmentSessionPlugKind::Upgrade => {
                let plug = self
                    .equipment_upgrade_plugs
                    .get_mut(&plug_id)
                    .expect("kind проверен по concrete registry");
                let cell = if requested_position == u32::MAX {
                    plug.upgrade_container()
                        .select_cell(goods, factory)
                        .map_err(EquipmentSessionShadowAddBlock::Upgrade)?
                } else {
                    crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::UpgradeEquipmentCell::from_position(requested_position)
                        .ok_or(EquipmentSessionShadowAddBlock::Upgrade(
                            UpgradeShadowAddBlock::InvalidPosition { position: requested_position },
                        ))?
                };
                let added = plug
                    .upgrade_container_mut()
                    .record_placed_goods(cell, goods, previous, goods_id, placed_position, factory)
                    .map_err(EquipmentSessionShadowAddBlock::Upgrade)?;
                (added.cell.position(), added.shadow)
            }
            EquipmentSessionPlugKind::DaKong => {
                let plug = self
                    .equipment_da_kong_plugs
                    .get_mut(&plug_id)
                    .expect("kind проверен по concrete registry");
                let cell = if requested_position == u32::MAX {
                    plug.upgrade_container()
                        .select_cell(goods, factory)
                        .map_err(EquipmentSessionShadowAddBlock::DaKong)?
                } else {
                    crate::gameserver::appserver::container::cequipmentdakongcontainer::DaKongCell::from_position(requested_position)
                        .ok_or(EquipmentSessionShadowAddBlock::DaKong(
                            DaKongAddBlock::InvalidPosition { position: requested_position },
                        ))?
                };
                match plug.upgrade_container_mut().record_placed_goods(
                    cell,
                    goods,
                    previous,
                    goods_id,
                    placed_position,
                ) {
                    DaKongAddOutcome::Added { effects, shadow } => {
                        (effects.cell.position(), shadow)
                    }
                    DaKongAddOutcome::Rejected { block, .. } => {
                        return Err(EquipmentSessionShadowAddBlock::DaKong(block));
                    }
                }
            }
            EquipmentSessionPlugKind::Compose => {
                let plug = self
                    .equipment_compose_plugs
                    .get_mut(&plug_id)
                    .expect("kind проверен по concrete registry");
                let cell = if requested_position == u32::MAX {
                    plug.compose_container()
                        .select_cell()
                        .map_err(EquipmentSessionShadowAddBlock::Compose)?
                } else {
                    crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::ComposeEquipmentCell::from_position(requested_position)
                        .ok_or(EquipmentSessionShadowAddBlock::Compose(
                            ComposeShadowAddBlock::InvalidPosition { position: requested_position },
                        ))?
                };
                let added = plug
                    .compose_container_mut()
                    .record_placed_goods(cell, goods, previous, goods_id, placed_position)
                    .map_err(EquipmentSessionShadowAddBlock::Compose)?;
                (added.cell.position(), added.shadow)
            }
        };
        Ok(EquipmentSessionShadowAdded {
            kind,
            plug_id,
            position,
            shadow,
        })
    }

    pub(crate) fn equipment_session_shadow_original(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        let (kind, plug_id) = self
            .resolve_equipment_plug(session_id, extend_id, player_id)
            .ok()?;
        match kind {
            EquipmentSessionPlugKind::Upgrade => self
                .equipment_upgrade_plugs
                .get(&plug_id)?
                .upgrade_container()
                .base()
                .base()
                .original_container_information(goods_id),
            EquipmentSessionPlugKind::DaKong => self
                .equipment_da_kong_plugs
                .get(&plug_id)?
                .upgrade_container()
                .original_container_information(goods_id),
            EquipmentSessionPlugKind::Compose => self
                .equipment_compose_plugs
                .get(&plug_id)?
                .compose_container()
                .original_container_information(goods_id),
        }
    }

    pub(crate) fn equipment_session_shadow_position(
        &self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<u32> {
        let (kind, plug_id) = self
            .resolve_equipment_plug(session_id, extend_id, player_id)
            .ok()?;
        match kind {
            EquipmentSessionPlugKind::Upgrade => self
                .equipment_upgrade_plugs
                .get(&plug_id)?
                .upgrade_container()
                .positions()
                .iter()
                .find_map(|(cell, id)| (*id == goods_id).then_some(cell.position())),
            EquipmentSessionPlugKind::DaKong => self
                .equipment_da_kong_plugs
                .get(&plug_id)?
                .upgrade_container()
                .positions()
                .iter()
                .find_map(|(cell, id)| (*id == goods_id).then_some(cell.position())),
            EquipmentSessionPlugKind::Compose => self
                .equipment_compose_plugs
                .get(&plug_id)?
                .compose_container()
                .query_goods_position(goods_id),
        }
    }

    pub(crate) fn remove_equipment_session_shadow(
        &mut self,
        session_id: i32,
        extend_id: i32,
        player_id: i32,
        goods_id: CGuid,
    ) -> Option<EquipmentSessionShadowRemoved> {
        let (kind, plug_id) = self
            .resolve_equipment_plug(session_id, extend_id, player_id)
            .ok()?;
        let original =
            self.equipment_session_shadow_original(session_id, extend_id, player_id, goods_id)?;
        let removed = match kind {
            EquipmentSessionPlugKind::Upgrade => self
                .equipment_upgrade_plugs
                .get_mut(&plug_id)?
                .upgrade_container_mut()
                .remove_shadow(goods_id)?,
            EquipmentSessionPlugKind::DaKong => self
                .equipment_da_kong_plugs
                .get_mut(&plug_id)?
                .upgrade_container_mut()
                .remove_shadow(goods_id)?,
            EquipmentSessionPlugKind::Compose => self
                .equipment_compose_plugs
                .get_mut(&plug_id)?
                .compose_container_mut()
                .remove_shadow(goods_id)?,
        };
        Some(EquipmentSessionShadowRemoved {
            kind,
            plug_id,
            original,
            removed,
        })
    }
    pub(crate) fn create_equipment_session(
        &mut self,
        now_ms: u32,
        kind: EquipmentSessionPlugKind,
        player_id: i32,
    ) -> Option<(i32, i32)> {
        let session_id = self.next_session_id;
        self.next_session_id = self.next_session_id.wrapping_add(1);
        let mut session = CSession::normal(1, 1, 0);
        if !session.start(now_ms) {
            return None;
        }

        let plug_id = self.next_plug_id;
        self.next_plug_id = self.next_plug_id.wrapping_add(1);
        let mut base = CPlug::new();
        base.set_id(plug_id);
        base.set_owner(400, player_id);
        base.set_session(session_id);
        base.set_plug_type(match kind {
            EquipmentSessionPlugKind::Upgrade => 4,
            EquipmentSessionPlugKind::DaKong => 6,
            EquipmentSessionPlugKind::Compose => 7,
        });
        if !session.insert_plug(plug_id) {
            return None;
        }

        self.sessions.insert(session_id, session);
        self.plugs.insert(plug_id, base);
        match kind {
            EquipmentSessionPlugKind::Upgrade => {
                let mut plug = CEquipmentUpgrade::new();
                let shadow = plug.upgrade_container_mut().base_mut().base_mut();
                shadow.base_mut().set_owner(10, session_id);
                shadow.set_container_extend_id(plug_id.wrapping_shl(8));
                self.equipment_upgrade_plugs.insert(plug_id, plug);
            }
            EquipmentSessionPlugKind::DaKong => {
                let mut plug = CEquipmentDaKong::new();
                let shadow = plug.upgrade_container_mut().base_mut().base_mut();
                shadow.base_mut().set_owner(10, session_id);
                shadow.set_container_extend_id(plug_id.wrapping_shl(8));
                self.equipment_da_kong_plugs.insert(plug_id, plug);
            }
            EquipmentSessionPlugKind::Compose => {
                let mut plug = CEquipmentCompose::new();
                let shadow = plug.compose_container_mut().base_mut().base_mut();
                shadow.base_mut().set_owner(10, session_id);
                shadow.set_container_extend_id(plug_id.wrapping_shl(8));
                self.equipment_compose_plugs.insert(plug_id, plug);
            }
        }
        Some((session_id, plug_id))
    }
    pub(crate) fn register_session(
        &mut self,
        session_id: i32,
        session: CSession,
    ) -> Option<CSession> {
        self.sessions.insert(session_id, session)
    }

    pub(crate) fn register_plug(&mut self, plug_id: i32, mut plug: CPlug) -> Option<CPlug> {
        plug.set_id(plug_id);
        self.plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_session(&self, session_id: i32) -> Option<&CSession> {
        self.sessions.get(&session_id)
    }

    pub(crate) fn query_plug(&self, plug_id: i32) -> Option<&CPlug> {
        self.plugs.get(&plug_id)
    }

    pub(crate) fn query_session_plug_by_owner(
        &self,
        session_id: i32,
        owner_type: i32,
        owner_id: i32,
    ) -> Option<&CPlug> {
        self.sessions
            .get(&session_id)?
            .plug_ids_storage()
            .iter()
            .find_map(|plug_id| {
                self.plugs
                    .get(plug_id)
                    .filter(|plug| plug.has_owner(owner_type, owner_id))
            })
    }

    /// Получатели legacy `SendToSession` в исходном порядке plug-ов. Не-player
    /// владельцы не имеют клиентского соединения и в рассылку не входят.
    pub(crate) fn session_player_ids(&self, session_id: i32) -> Option<Vec<i32>> {
        Some(
            self.sessions
                .get(&session_id)?
                .plug_ids_storage()
                .iter()
                .filter_map(|plug_id| self.plugs.get(plug_id))
                .filter(|plug| plug.has_owner(400, plug.owner_id()))
                .map(CPlug::owner_id)
                .collect(),
        )
    }

    /// Exact registry half of `query_session_by_owner`: legacy traversal is
    /// pointer-based, while IDs here remain deterministic owner relations.
    pub(crate) fn query_session_id_by_owner(&self, owner_type: i32, owner_id: i32) -> Option<i32> {
        self.plugs
            .values()
            .find(|plug| plug.has_owner(owner_type, owner_id))
            .map(|plug| plug.session_id())
            .filter(|session_id| self.sessions.contains_key(session_id))
    }

    pub(crate) fn end_session(&mut self, session_id: i32) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            tracing::trace!(session_id, "завершаемая сессия не найдена");
            return;
        };
        let callback_plug_ids = session.end();
        let callback_plug_ids = callback_plug_ids
            .into_iter()
            .filter(|plug_id| self.plugs.contains_key(plug_id))
            .collect::<Vec<_>>();
        let session = self
            .sessions
            .get(&session_id)
            .expect("ended session остаётся в registry до factory GC");
        tracing::trace!(session_id, ended = session.is_ended(), remove_requested = session.remove_requested(), ?callback_plug_ids, "сессия завершена");
    }

    pub(crate) fn exit_plug(&mut self, session_id: i32, plug_id: i32) {
        let session_found = self.sessions.contains_key(&session_id);
        let exited = if session_found {
            self.plugs
                .get_mut(&plug_id)
                .filter(|plug| plug.session_id() == session_id)
                .map(|plug| {
                    plug.mark_ended();
                    true
                })
                .unwrap_or(false)
        } else {
            false
        };
        tracing::trace!(session_id, plug_id, session_found, exited, "plug покинул сессию");
    }

    pub(crate) fn register_equipment_compose_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentCompose,
    ) -> Option<CEquipmentCompose> {
        self.equipment_compose_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_compose_plug(&self, plug_id: i32) -> Option<&CEquipmentCompose> {
        self.equipment_compose_plugs.get(&plug_id)
    }

    pub(crate) fn query_equipment_compose_plug_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CEquipmentCompose> {
        self.equipment_compose_plugs.get_mut(&plug_id)
    }

    pub(crate) fn take_equipment_compose_plug(
        &mut self,
        plug_id: i32,
    ) -> Option<CEquipmentCompose> {
        self.equipment_compose_plugs.remove(&plug_id)
    }

    pub(crate) fn register_equipment_da_kong_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentDaKong,
    ) -> Option<CEquipmentDaKong> {
        self.equipment_da_kong_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_da_kong_plug(&self, plug_id: i32) -> Option<&CEquipmentDaKong> {
        self.equipment_da_kong_plugs.get(&plug_id)
    }

    pub(crate) fn query_equipment_da_kong_plug_mut(
        &mut self,
        plug_id: i32,
    ) -> Option<&mut CEquipmentDaKong> {
        self.equipment_da_kong_plugs.get_mut(&plug_id)
    }

    pub(crate) fn take_equipment_da_kong_plug(&mut self, plug_id: i32) -> Option<CEquipmentDaKong> {
        self.equipment_da_kong_plugs.remove(&plug_id)
    }

    pub(crate) fn register_equipment_upgrade_plug(
        &mut self,
        plug_id: i32,
        plug: CEquipmentUpgrade,
    ) -> Option<CEquipmentUpgrade> {
        self.equipment_upgrade_plugs.insert(plug_id, plug)
    }

    pub(crate) fn query_equipment_upgrade_plug(&self, plug_id: i32) -> Option<&CEquipmentUpgrade> {
        self.equipment_upgrade_plugs.get(&plug_id)
    }

    pub(crate) fn take_equipment_upgrade_plug(
        &mut self,
        plug_id: i32,
    ) -> Option<CEquipmentUpgrade> {
        self.equipment_upgrade_plugs.remove(&plug_id)
    }

    /// Session-GC удаляет саму session и все её plug identities из base и
    /// concrete registries. Возвращаемый порядок совпадает с `m_lPlugs`.
    pub(crate) fn garbage_collect_session(&mut self, session_id: i32) -> Vec<i32> {
        let Some(session) = self.sessions.remove(&session_id) else {
            return Vec::new();
        };
        let plug_ids = session.plug_ids_storage().to_vec();
        for plug_id in &plug_ids {
            self.plugs.remove(plug_id);
            self.equipment_compose_plugs.remove(plug_id);
            self.equipment_da_kong_plugs.remove(plug_id);
            self.equipment_upgrade_plugs.remove(plug_id);
            self.personal_shop_seller_plugs.remove(plug_id);
            self.personal_shop_buyer_plugs.remove(plug_id);
            self.trader_plugs.remove(plug_id);
            self.teammates.remove(plug_id);
        }
        self.teams.remove(&session_id);
        plug_ids
    }

    pub(crate) fn garbage_collect_terminal_equipment_sessions(
        &mut self,
    ) -> Vec<TerminalEquipmentSessionCollected> {
        let session_ids: Vec<i32> = self
            .sessions
            .iter()
            .filter_map(|(session_id, session)| {
                let has_equipment_plug = session.plug_ids_storage().iter().any(|plug_id| {
                    self.equipment_compose_plugs.contains_key(plug_id)
                        || self.equipment_da_kong_plugs.contains_key(plug_id)
                        || self.equipment_upgrade_plugs.contains_key(plug_id)
                });
                (session.remove_requested() && has_equipment_plug).then_some(*session_id)
            })
            .collect();
        session_ids
            .into_iter()
            .map(|session_id| {
                let plugs = self
                    .sessions
                    .get(&session_id)
                    .expect("terminal session выбрана из registry")
                    .plug_ids_storage()
                    .iter()
                    .filter_map(|plug_id| {
                        self.plugs
                            .get(plug_id)
                            .map(|plug| TerminalEquipmentPlugCollected {
                                plug_id: *plug_id,
                                owner_id: plug.owner_id(),
                            })
                    })
                    .collect();
                let _plug_ids = self.garbage_collect_session(session_id);
                TerminalEquipmentSessionCollected { session_id, plugs }
            })
            .collect()
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp

// ============================================================================
// FUNCTION: CSessionFactory::query_session_by_owner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:81
// RVA: 0x00078060
// ADDRESS: 00478060
// PROTOTYPE: CSession * __cdecl query_session_by_owner(OBJECT_TYPE param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `QuerySession` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: owned map removal и ordered concrete plug cleanup выполняет
// `garbage_collect_session`; MSVC hash/destructor RAW удалён.
// ============================================================================
// FUNCTION: CSessionFactory::InsertPlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:104
// RVA: 0x000781C0
// ADDRESS: 004781c0
// PROTOTYPE: int __cdecl InsertPlug(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// MATERIALIZED: live equipment-session registry sweep и team snapshot
// retry-queue (`0x60008`, 60 секунд) выполняются на обеих MainLoop
// session-stage; сама очередь принадлежит owned `CGame`.
// ============================================================================
// FUNCTION: CSessionFactory::CreateSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:37
// RVA: 0x00078DA0
// ADDRESS: 00478da0
// PROTOTYPE: long __cdecl CreateSession(ulong param_1, ulong param_2, ulong param_3, SESSION_TYPE param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::CreatePlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:244
// RVA: 0x00078EC0
// ADDRESS: 00478ec0
// PROTOTYPE: long __cdecl CreatePlug(PLUG_TYPE param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::UnserializePlug
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:297
// RVA: 0x000790D0
// ADDRESS: 004790d0
// PROTOTYPE: long __cdecl UnserializePlug(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSessionFactory::UnserializeSession
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\csessionfactory.cpp:324
// RVA: 0x000791B0
// ADDRESS: 004791b0
// PROTOTYPE: long __cdecl UnserializeSession(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
