//! Организационные сообщения `OnOrgasysMessage` из `organsysmessage.cpp`,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Dispatcher разрешает player/faction/union/country и передаёт governance,
//! membership, city/war, application, transfer и billboard операции их
//! владельцам в исходном порядке. Дополнительные socket/ownership/tail gates
//! не добавляются; повторный lookup остаётся повторным.
//!
//! Membership/governance ветви (заявки, выходы, распуски, dub/purview,
//! leave-word/pronounce, налоги, городские операции, списки фракций) и
//! war-ветви (`0x60101`, двухфазная `0x6011F`, village/city application и
//! result, goods war `0x60139`/`0x6013A`, семейный `reload_attack_city`)
//! перенесены в `nebokrai_realm::app::organsysmessage` поверх статического
//! организационного view и узкого country-gate `WorldCountryWarGate`;
//! здешние одноимённые `pub(crate)` функции — только направляющая обвязка,
//! собирающая CGame-адаптеры контекстов, либо ветви, чьи адаптеры ещё
//! привязаны к старому владельцу игры (creation `0x60103`, upgrade `0x60126`,
//! billboard, transfer).
//!
//! Async session callback публикует terminal action в main-loop FIFO и лишь
//! там изменяет `CGame`; confirmation остаётся на исходной позиции callback.
//! Короткий payload останавливает ветку после уже выполненного префикса.

use std::ffi::CString;

use crate::dbaccess::worlddb::rsplayer::{RsPlayerOwner, TiberiusRsPlayer};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::regionrouter::{
    RegionRoutePoint, RegionRouter, RegionRouterChangeOutcome,
};
use crate::public::tools::put_string_to_file;
use crate::worldserver::appworld::country::country::{
    CountryAbsolveCounterReset, CountryExileMessageDelivery, CountryExileResultContext,
    CountryExileTarget, CountryExileTextArgument, CountryFactionSnapshot,
    CountryGovernanceContextBlock,
};
use nebokrai_realm::app::world_game_view::{
    WorldCountryKingGateBlock, WorldCountryWarGate,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::public::netsessionmanager::CNetSessionManager;
use crate::public::date::TagTime;
use crate::public::timer::{CTimer, TimerId};
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, GoodsOriginalNameIndex, query_goods_name,
};
use crate::worldserver::appworld::goodswarmember::{
    CGoodsWarMember, GoodsWarDeliveryContext,
};
use crate::worldserver::appworld::organizingsystem::faction::{
    CFaction, FactionApplyForJoinEffects, FactionContributorContext, FactionDemiseContext,
    FactionDoJoinEffects,
    FactionDubContext, FactionDubFormatArgument,
    FactionExitContext,
    FactionFireOutContext,
    goods_war_check_for_faction_id,
    FactionEnemyMutationContext,
    FactionEnemyWarLogArgument, FactionExperienceBlock,
    FactionLevelContext, FactionMemberInfoRequest, FactionOrganizingInfoContext,
    FactionPurviewChange, FactionPurviewChangeContext,
    FactionSetParameterContext,
    FactionUpgradeContext, FactionUpgradeFormatArgument,
    FactionUploadIconBlock, FactionUploadIconContext,
};
use crate::worldserver::appworld::organizingsystem::factionwarsys::CFactionWarSys;
use crate::worldserver::appworld::organizingsystem::attackcitysys::{
    AttackCityApplicationContext, AttackCityCallbacks, AttackCityEnemyRelationContext,
    AttackCityReloadBlock, AttackCityReloadReport, AttackCityWarEndContext,
    AttackCityWarResultContext, AttackCityWarResultFaction, AttackCityWarResultFormatArgument,
    AttackCityWarResultRegion, CAttackCitySys,
};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    AttackCityEndEffects, COrganizingCtrl,
    ConfederationCreationEffects,
    ConfederationCreationSessionBlock, ConfederationCreationSessionReport,
    ConfederationCreationSessionRequest,
    CityTransferEffects,
    CityTransferSessionBlock, CityTransferSessionReport, CityTransferSessionRequest,
    CityTransferStartBlock,
    FreeFactionLookup, FreePlayerLookup,

    OrganizingContributorBlock,
    FactionUnionMembershipLookupBlock,
    WorldFactionWarDeclarationEffects,
    OrganizingUnionApplyForJoinDispatchBlock,
    FactionCreationBlock, FactionCreationEffects, FactionCreationOutcome,
    FactionCreationPreparation,
    UnionOrganizingBridge,
    PlayerInviteFactionBlock, PlayerInviteFactionEffects,
    OrganizingFactionDoJoinBlock,
    begin_city_transfer_session, begin_confederation_creation_session,
    OrganizingUnionDemiseBlock,
    OrganizingUnionExitBlock,
    OrganizingUnionFireOutBlock,
};
use crate::worldserver::appworld::organizingsystem::organizing::{
    TagTimeValue,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::appworld::organizingsystem::union::{
    UnionAddFactionEffects, UnionApplicationSessionBlock,
    UnionApplicationSessionReport, UnionApplicationSessionRequest,
    UnionApplyForJoinEffects,
    UnionInvitationSessionRequest, UnionInviteEffects,
    UnionFactionStateMutationContext, UnionFireOutEffects,
    UnionFormatArgument, UnionOwnedCityMutationContext,
    begin_union_application_session, begin_union_invitation_session,
};
use crate::worldserver::appworld::organizingsystem::villagewarsys::{
    CVillageWarSys, VillageWarApplicationContext, VillageWarCallbacks,
    VillageWarResultContext, VillageWarResultFaction, VillageWarResultRegion,
};
use crate::worldserver::appworld::player::{
    PlayerCodecError, PlayerFactionInfoUpdateBlock, PlayerPropertyCoefficients,
};
use crate::worldserver::worldserver::game::{
    CGame, WorldRegionNameLookup, format_union_world_string, legacy_tick_ms,
};

pub use nebokrai_realm::app::organsysmessage::*;

/// Session-runtime владелец слит с realm [`WorldOrganizingSessionRuntimeOwner`]:
/// прежнее имя сохраняет импорты main-loop runtime и адаптеров; очереди едины —
/// это тот же самый `Arc`-state, type-алиас расхождения не создаёт.
pub(crate) use nebokrai_realm::app::organsysmessage::WorldOrganizingSessionRuntimeOwner as WorldUnionApplicationRuntimeOwner;

pub(crate) struct WorldUnionApplicationEffectCallbacks<'a> {
    pub(crate) random: &'a mut dyn FnMut(i32) -> i32,
    pub(crate) refresh_owned_city:
        &'a mut dyn FnMut(&CGame, i32, i32, i32, Option<u8>),
    pub(crate) faction_level_log_enabled: bool,
    pub(crate) write_faction_level_log:
        &'a mut dyn FnMut(i32, &[u8], i32, i32, &[u8]),
    pub(crate) faction_experience_log_enabled: bool,
    pub(crate) write_faction_experience_log:
        &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32, i32),
}

pub(crate) struct WorldUnionApplicationEffects<'a> {
    game: &'a CGame,
    manager: &'a CNetSessionManager,
    runtime: &'a WorldUnionApplicationRuntimeOwner,
    callbacks: WorldUnionApplicationEffectCallbacks<'a>,
}

impl<'a> WorldUnionApplicationEffects<'a> {
    pub(crate) fn new(
        game: &'a CGame,
        manager: &'a CNetSessionManager,
        runtime: &'a WorldUnionApplicationRuntimeOwner,
        callbacks: WorldUnionApplicationEffectCallbacks<'a>,
    ) -> Self {
        Self {
            game,
            manager,
            runtime,
            callbacks,
        }
    }
}

impl UnionApplyForJoinEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = UnionApplicationSessionReport;
    type SessionBlock = UnionApplicationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_union_application_session(
        &mut self,
        request: UnionApplicationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
 // `Beging` вызывает `DoAsyncCall` синхронно; route и клонируемый
 // transport handle снимаются непосредственно перед session creation.
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.recipient_player_id);
        let endpoint = self
            .runtime
            .endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_union_application_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }
}

impl UnionInviteEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = UnionApplicationSessionReport;
    type SessionBlock = UnionApplicationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_union_invitation_session(
        &mut self,
        request: UnionInvitationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.recipient_player_id);
        let endpoint = self
            .runtime
            .invitation_endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_union_invitation_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }
}

impl UnionAddFactionEffects for WorldUnionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl ConfederationCreationEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = ConfederationCreationSessionReport;
    type SessionBlock = ConfederationCreationSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_confederation_creation_session(
        &mut self,
        request: ConfederationCreationSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.second_player_id);
        let endpoint = self.runtime.confederation_creation_endpoint(
            self.game.current_game_server_sender(),
            game_server_id,
        );
        begin_confederation_creation_session(
            self.manager,
            request,
            endpoint,
            |upper_bound| (self.callbacks.random)(upper_bound),
        )
    }
}

struct WorldFactionCreationEffects<'game, 'callbacks, 'effects, 'log> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    persistent_name_exists: bool,
    faction_create_log_enabled: bool,
    write_faction_create_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8]),
}

impl FactionCreationEffects for WorldFactionCreationEffects<'_, '_, '_, '_> {
    fn check_invalid_organizing_string(&mut self, name: &mut Vec<u8>, strict: bool) -> bool {
        self.game.check_invalid_string(name, strict)
    }

    fn persistent_player_name_exists(&mut self, _name: &[u8]) -> bool {
        self.persistent_name_exists
    }

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn faction_create_log_enabled(&self) -> bool {
        self.faction_create_log_enabled
    }

    fn write_faction_create_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
    ) {
        (self.write_faction_create_log)(
            faction_id,
            faction_name,
            player_id,
            player_name,
        );
    }
}

struct WorldUnionFireOutEffects<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl UnionFireOutEffects for WorldUnionFireOutEffects<'_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }
}

struct WorldFactionDubEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    use_log_system: bool,
    faction_title_log_enabled: bool,
    write_faction_title_log:
        &'log mut dyn FnMut(i32, &[u8], &[u8], &[u8], i32, &[u8], i32, &[u8]),
}

impl FactionOrganizingInfoContext for WorldFactionDubEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDubContext for WorldFactionDubEffects<'_, '_, '_, '_, '_> {
    fn check_invalid_string(&mut self, value: &mut Vec<u8>, mode: bool) -> bool {
        self.game.check_invalid_string(value, mode)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionDubFormatArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionDubFormatArgument::Text(value) => UnionFormatArgument::Text(value),
                FactionDubFormatArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_title_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_title_log_enabled
    }

    fn write_faction_title_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        old_title: &[u8],
        new_title: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    ) {
        (self.write_faction_title_log)(
            member_id,
            member_name,
            old_title,
            new_title,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
        );
    }
}

struct WorldFactionPurviewEffects<'game, 'callbacks, 'effects, 'log> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    use_log_system: bool,
    add_log_enabled: bool,
    revoke_log_enabled: bool,
    write_log: &'log mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionPurviewEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionPurviewChangeContext for WorldFactionPurviewEffects<'_, '_, '_, '_> {
    fn format_world_string(&mut self, string_id: &'static [u8], member_name: &[u8]) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Text(member_name)],
        )
    }

    fn faction_purview_log_enabled(&self, change: FactionPurviewChange) -> bool {
        self.use_log_system
            && match change {
                FactionPurviewChange::Grant => self.add_log_enabled,
                FactionPurviewChange::Revoke => self.revoke_log_enabled,
            }
    }

    fn write_faction_purview_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        purview: i32,
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    ) {
        (self.write_log)(
            member_id,
            member_name,
            purview,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }
}

impl CityTransferEffects for WorldUnionApplicationEffects<'_> {
    type SessionReport = CityTransferSessionReport;
    type SessionBlock = CityTransferSessionBlock;

    fn world_string(&mut self, string_id: &'static [u8]) -> Vec<u8> {
        self.game.get_string_by_id(string_id).to_vec()
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }

    fn begin_city_transfer_session(
        &mut self,
        request: CityTransferSessionRequest,
    ) -> Result<Self::SessionReport, Self::SessionBlock> {
        let game_server_id = self
            .game
            .game_server_number_by_player_id(request.target_master_player_id);
        let endpoint = self
            .runtime
            .city_endpoint(self.game.current_game_server_sender(), game_server_id);
        begin_city_transfer_session(self.manager, request, endpoint, |upper_bound| {
            (self.callbacks.random)(upper_bound)
        })
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn broadcast_city_transfer(&mut self, text: &[u8]) -> Result<i32, SendMessageError> {
        COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFDA_EDFE,
            0x328F_93FC,
        )
    }
}

impl AttackCityEndEffects for WorldUnionApplicationEffects<'_> {
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[UnionFormatArgument<'_>],
    ) -> Vec<u8> {
        format_union_world_string(self.game.get_string_by_id(string_id), arguments)
    }

    fn refresh_owned_city(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
        country_id: Option<u8>,
    ) {
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
    }

    fn broadcast_city_war_result(&mut self, text: &[u8]) -> Result<i32, SendMessageError> {
        COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFDA_EDFE,
            0x328F_93FC,
        )
    }
}

impl FactionOrganizingInfoContext for WorldUnionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionApplicationListContext for WorldUnionApplicationEffects<'_> {
    fn online_player_country(&self, player_id: i32) -> Option<Option<u8>> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| player.country())
    }
}

/// Узкий war/string/log adapter faction-ветви `0x60108`. Держит игру прямо,
/// а не через общий session/transport owner: realm-обработчик ветви получает
/// `&mut`-ссылки и на этот адаптер, и на owner-а соседней union-ветви
/// одновременно, поэтому owner-borrow здесь не нужен и не допустим; игровой
/// объект тот же, что и в owner-е, во всех call-site-ах диспетчера.
struct WorldFactionApplicationEffects<'a> {
    game: &'a CGame,
    village_war: &'a CVillageWarSys,
    attack_city: &'a CAttackCitySys,
    use_log_system: bool,
    faction_apply_log_enabled: bool,
    write_faction_apply_log: &'a mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionApplicationEffects<'_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionApplyForJoinEffects for WorldFactionApplicationEffects<'_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn faction_apply_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_apply_log_enabled
    }

    fn write_faction_apply_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_apply_log)(
            faction_id,
            faction_name,
            player_id,
            player_name,
            log_type,
        );
    }
}

struct WorldFactionDoJoinEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    village_war: &'game CVillageWarSys,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game CGoodsWarMember,
    use_log_system: bool,
    faction_join_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_join_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionDoJoinEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDoJoinEffects for WorldFactionDoJoinEffects<'_, '_, '_, '_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_join(&self, faction_id: i32, _manager_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_join_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_join_log_enabled
    }

    fn write_faction_join_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_join_log)(
            member_id,
            member_name,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }
}

struct WorldFactionDemiseEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game CGoodsWarMember,
    country_handler: &'game CCountryHandler,
    faction_master_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_master_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
}

impl FactionOrganizingInfoContext for WorldFactionDemiseEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionDemiseContext for WorldFactionDemiseEffects<'_, '_, '_, '_, '_> {
    fn attack_city_system_declared(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_demise(&self, faction_id: i32, _old_master_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn country_blocks_demise(&self, country: u8, old_master_id: i32) -> bool {
        self.country_handler.get_country(country).is_some_and(|state| {
            state.king.id == old_master_id && !state.demise_faction
        })
    }

    fn format_demise_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(value)],
        )
    }

    fn format_demise_change(
        &mut self,
        string_id: &'static [u8],
        old_master_name: &[u8],
        new_master_name: &[u8],
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[
                UnionFormatArgument::Text(old_master_name),
                UnionFormatArgument::Text(new_master_name),
            ],
        )
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_master_log_enabled(&self) -> bool {
        self.faction_master_log_enabled
    }

    fn write_faction_master_log(
        &mut self,
        old_master_id: i32,
        old_master_name: &[u8],
        new_master_id: i32,
        new_master_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
    ) {
        (self.write_faction_master_log)(
            old_master_id,
            old_master_name,
            new_master_id,
            new_master_name,
            faction_id,
            faction_name,
        );
    }
}

struct WorldFactionExitEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    village_war: &'game CVillageWarSys,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game mut CGoodsWarMember,
    use_log_system: bool,
    faction_quit_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_quit_log: &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
}

impl FactionOrganizingInfoContext for WorldFactionExitEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionExitContext for WorldFactionExitEffects<'_, '_, '_, '_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_exit(&self, faction_id: i32, _player_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_quit_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_quit_log_enabled
    }

    fn write_faction_quit_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        player_id: i32,
        player_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_quit_log)(
            faction_id,
            faction_name,
            player_id,
            player_name,
            log_type,
        );
    }

    fn delete_goods_war_member(&mut self, player_id: i32) {
        let mut delivery = WorldFactionFireOutGoodsWarDelivery { game: self.game };
        let _ = self.goods_war.delete_one_member(player_id, &mut delivery);
    }
}

struct WorldFactionFireOutEffects<'game, 'callbacks, 'effects, 'update, 'log> {
    game: &'game CGame,
    village_war: &'game CVillageWarSys,
    attack_city: &'game CAttackCitySys,
    goods_war: &'game mut CGoodsWarMember,
    use_log_system: bool,
    faction_fire_out_log_enabled: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
    write_faction_fire_out_log:
        &'log mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
}

struct WorldFactionFireOutGoodsWarDelivery<'game> {
    game: &'game CGame,
}

impl GoodsWarDeliveryContext for WorldFactionFireOutGoodsWarDelivery<'_> {
    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

impl FactionOrganizingInfoContext for WorldFactionFireOutEffects<'_, '_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionFireOutContext for WorldFactionFireOutEffects<'_, '_, '_, '_, '_> {
    fn already_declared_for_village_war(&self, faction_id: i32) -> bool {
        self.village_war.is_already_declared_for_war(faction_id)
    }

    fn already_declared_for_city_war(&self, faction_id: i32) -> bool {
        self.attack_city.is_already_declared_for_war(faction_id)
    }

    fn goods_war_blocks_fire_out(&self, faction_id: i32, _manager_id: i32) -> bool {
        goods_war_check_for_faction_id(faction_id, |candidate| {
            self.goods_war.contains_faction_id(candidate)
        })
    }

    fn format_world_string(&mut self, string_id: &'static [u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(*argument))
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_fire_out_log_enabled(&self) -> bool {
        self.use_log_system && self.faction_fire_out_log_enabled
    }

    fn write_faction_fire_out_log(
        &mut self,
        member_id: i32,
        member_name: &[u8],
        manager_id: i32,
        manager_name: &[u8],
        faction_id: i32,
        faction_name: &[u8],
        log_type: i32,
    ) {
        (self.write_faction_fire_out_log)(
            member_id,
            member_name,
            manager_id,
            manager_name,
            faction_id,
            faction_name,
            log_type,
        );
    }

    fn delete_goods_war_member(&mut self, player_id: i32) {
        let mut delivery = WorldFactionFireOutGoodsWarDelivery { game: self.game };
        let _ = self.goods_war.delete_one_member(player_id, &mut delivery);
    }
}

struct WorldFactionSetParameterEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionSetParameterEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionLevelContext for WorldFactionSetParameterEffects<'_, '_, '_, '_> {
    fn format_world_string_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(value)],
        )
    }
}

impl FactionSetParameterContext for WorldFactionSetParameterEffects<'_, '_, '_, '_> {
    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }
}

struct WorldFactionUpgradeEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    registry: &'game GoodsBasePropertiesRegistry,
    original_name_index: &'game GoodsOriginalNameIndex,
    use_log_system: bool,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionLevelContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn format_world_string_signed(
        &mut self,
        string_id: &'static [u8],
        value: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(value)],
        )
    }
}

impl FactionUpgradeContext for WorldFactionUpgradeEffects<'_, '_, '_, '_> {
    fn player_money(&self, player_id: i32) -> Option<u32> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| player.money())
    }

    fn goods_in_packet(&self, player_id: i32, original_name: &[u8]) -> i32 {
        let Ok(original_name) = CString::new(original_name) else {
            return 0;
        };
        self.game
            .online_player_by_id(player_id as u32)
            .map_or(0, |player| {
                player.check_goods_in_packet(
                    Some(original_name.as_c_str()),
                    self.original_name_index,
                )
            })
    }

    fn goods_display_name(&self, original_name: &[u8]) -> Option<Vec<u8>> {
        let goods_id = self.original_name_index.get(original_name).copied()?;
        query_goods_name(self.registry, goods_id).map(ToOwned::to_owned)
    }

    fn format_upgrade_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionUpgradeFormatArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionUpgradeFormatArgument::Text(value) => UnionFormatArgument::Text(value),
                FactionUpgradeFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        format_union_world_string(self.game.get_string_by_id(string_id), &arguments)
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }

    fn faction_level_log_enabled(&self) -> bool {
        self.use_log_system && self.callbacks.faction_level_log_enabled
    }

    fn write_faction_level_log(
        &mut self,
        faction_id: i32,
        faction_name: &[u8],
        level: i32,
        master_id: i32,
        master_name: &[u8],
    ) {
        (self.callbacks.write_faction_level_log)(
            faction_id,
            faction_name,
            level,
            master_id,
            master_name,
        );
    }
}

struct WorldFactionUploadIconEffects<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl FactionOrganizingInfoContext for WorldFactionUploadIconEffects<'_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionUploadIconContext for WorldFactionUploadIconEffects<'_, '_, '_> {
    fn format_upload_icon_interval(
        &mut self,
        string_id: &'static [u8],
        interval_minutes: i32,
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Signed(interval_minutes)],
        )
    }
}

struct WorldFactionContributorEffects<'game, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl FactionOrganizingInfoContext for WorldFactionContributorEffects<'_, '_, '_, '_> {
    fn world_string(&mut self, string_id: &'static [u8]) -> Option<Vec<u8>> {
        Some(self.game.get_string_by_id(string_id).to_vec())
    }

    fn send_organizing_info(&mut self, request: FactionMemberInfoRequest<'_>) {
        let _ = COrganizingCtrl::send_organizing_info_to_client(self.game, request);
    }
}

impl FactionContributorContext for WorldFactionContributorEffects<'_, '_, '_, '_> {
    fn format_contributor_string(
        &mut self,
        string_id: &'static [u8],
        member_name: &[u8],
    ) -> Vec<u8> {
        format_union_world_string(
            self.game.get_string_by_id(string_id),
            &[UnionFormatArgument::Text(member_name)],
        )
    }

    fn update_player_faction_info(&mut self, faction: &CFaction, player_id: i32) {
        let _ = self.game.update_player_faction_info_from_faction(faction, player_id);
        (self.update_player)(player_id);
    }
}

/// Декодирует общий session-result branch `OnOrgasysMessage` в исходном
/// порядке полей и передаёт callback менеджеру без дополнительных ответов.
pub(crate) fn dispatch_organizing_session_result(
    message: &mut CMessage,
    manager: &CNetSessionManager,
) -> OrganizingSessionResultDispatch {
    let message_type = message.message_type();
    if !SESSION_RESULT_MESSAGE_TYPES.contains(&message_type) {
        return OrganizingSessionResultDispatch::NotHandled;
    }

    let session_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie_second = message.base_mut().get_long().unwrap_or(0);
    let result = message
        .base_mut()
        .get_char()
        .map_or(0, |result| i32::from(result as u8));
    let cookie_first = message.base_mut().get_long().unwrap_or(0);
    let outcome =
        manager.on_sync_callback_result(session_id, cookie_first, cookie_second, result);
    OrganizingSessionResultDispatch::Delivered {
        message_type,
        session_id,
        cookie_first,
        cookie_second,
        result,
        outcome,
    }
}

pub(crate) fn dispatch_player_invite_faction<Effects>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingPlayerInviteFactionDispatch<
            <Effects as ConfederationCreationEffects>::SessionReport,
            <Effects as UnionApplyForJoinEffects>::SessionReport,
            <Effects as UnionInviteEffects>::SessionReport,
        >,
        PlayerInviteFactionBlock<
            <Effects as ConfederationCreationEffects>::SessionBlock,
            <Effects as UnionApplyForJoinEffects>::SessionBlock,
            <Effects as UnionInviteEffects>::SessionBlock,
        >,
    >,
>
where
    Effects: PlayerInviteFactionEffects,
{
    if message.message_type() != PLAYER_INVITE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let invited_faction_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .on_player_invite_faction(
                game,
                village_war,
                attack_city,
                player_id,
                invited_faction_id,
                effects,
            )
            .map(|outcome| OrganizingPlayerInviteFactionDispatch {
                player_id,
                invited_faction_id,
                outcome,
            }),
    )
}

pub(crate) fn dispatch_union_application<Effects>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingUnionApplicationDispatch<Effects::SessionReport>,
        OrganizingUnionApplyForJoinDispatchBlock<Effects::SessionBlock>,
    >,
>
where
    Effects: UnionApplyForJoinEffects,
{
    if message.message_type() != UNION_APPLICATION_MESSAGE_TYPE {
        return None;
    }

    let master_player_id = message.base_mut().get_long().unwrap_or(0);
    let applicant_faction_id = message.base_mut().get_long().unwrap_or(0);
    Some(
        organizing
            .apply_for_union_join(
                game,
                master_player_id,
                applicant_faction_id,
                0,
                master_player_id,
                effects,
            )
            .map(|outcome| OrganizingUnionApplicationDispatch {
                master_player_id,
                applicant_faction_id,
                outcome,
            }),
    )
}

// Ветви `0x6011A/0x6011B/0x6011C/0x6011D` (leave-word enable/edit, pronounce)
// перенесены в realm (`nebokrai_realm::app::organsysmessage`) волной
// организационного view; имена доступны здесь через glob re-export.

/// Ветвь `0x60101` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает faction-war effects адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_war_player_died(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    faction_war: &mut CFactionWarSys,
    update_player: &mut dyn FnMut(i32),
) -> Option<OrganizingFactionWarPlayerDiedDispatch> {
    let mut effects = WorldFactionWarDeclarationEffects::new(
        game,
        organizing,
        update_player,
    );
    nebokrai_realm::app::organsysmessage::dispatch_faction_war_player_died(
        message,
        faction_war,
        &mut effects,
    )
}

#[derive(Debug)]
pub(crate) enum OrganizingCreateFactionBlock {
    PlayerDecode(PlayerCodecError),
    Creation(FactionCreationBlock),
    PlayerRefresh(PlayerFactionInfoUpdateBlock),
    PlayerRefreshOwnerMissing,
}

#[allow(
    clippy::too_many_arguments,
    reason = "границы один к одному соответствуют exact World owner-ам"
)]
/// Local-time snapshot wire-полей остаётся у creation-ветви `0x60103`;
/// перенесённые в realm ветви получают ту же формулу через
/// `current_local_member_time` её владельца.
fn capture_local_tag_time() -> TagTimeValue {
    let local_time = TagTime::local_now();
    TagTimeValue {
        year: local_time.year,
        month: local_time.month,
        day_of_week: local_time.day_of_week,
        day: local_time.day,
        hour: local_time.hour,
        minute: local_time.minute,
        second: local_time.second,
        milliseconds: local_time.milliseconds,
    }
}

// Ветвь `0x60103` пока остаётся локальной: её середина связно держит
// `&CGame`-inherent (`prepare_faction_creation` → DB name-lookup →
// `finish_faction_creation` → client snapshots + organizing-info static), а
// wire-hвостовой decode того же `CGame` требует `&mut`-заёма — без свёртки
// середины в отдельный bridge-адаптер (форма B5 по карте волн) оба заёма не
// разводятся по границе вызова, как у `0x60126`. Единственный DB-контакт
// ветви (`is_name_exist`) уже закрыт boxed view `WorldCreateRoleDbView` по
// ADR-0013 (`TiberiusRsPlayer` делегирует тому же `RsPlayerOwner::is_name_exist`,
// отдельный `WorldFactionCreationDbView` не нужен), поэтому перенос — вопрос
// формы середины, а не DB-шва.
pub(crate) async fn dispatch_create_faction(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    countries: &CCountryHandler,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    rs_player: &mut TiberiusRsPlayer,
    player_database: Option<&mut WorldTdsClient>,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    faction_create_log_enabled: bool,
    write_faction_create_log: &mut dyn FnMut(i32, &[u8], i32, &[u8]),
) -> Option<Result<OrganizingCreateFactionDispatch, OrganizingCreateFactionBlock>> {
    if message.message_type() != CREATE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let request_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let country = message.base_mut().get_byte().unwrap_or(0);
    let mut faction_name = message.base_mut().get_str_bytes(20).unwrap_or_default();
    if faction_name.is_empty() {
        return Some(Ok(OrganizingCreateFactionDispatch {
            request_id,
            cookie,
            player_id,
            country,
            faction_name,
            outcome: OrganizingCreateFactionOutcome::EmptyName,
        }));
    }

    let established_time = capture_local_tag_time();

    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingCreateFactionBlock::PlayerDecode(source)));
            }
        }
    };
    if !player_online {
        return Some(Ok(OrganizingCreateFactionDispatch {
            request_id,
            cookie,
            player_id,
            country,
            faction_name,
            outcome: OrganizingCreateFactionOutcome::PlayerOffline,
        }));
    }

    let gate = if countries.get_country(country).is_none() {
        Some(OrganizingCreateFactionGate::CountryMissing)
    } else {
        let player = game
            .online_player_by_id(player_id as u32)
            .expect("успешный Decord сохранил тот же online owner");
        if i32::from(player.get_level()) < parameters.create_faction_player_level() {
            Some(OrganizingCreateFactionGate::PlayerLevel)
        } else {
            let required_goods = legacy_c_string_prefix(parameters.create_faction_goods());
            let has_required_goods = if required_goods == b"0" {
                true
            } else {
                CString::new(required_goods).is_ok_and(|name| {
                    player.check_goods_in_packet(
                        Some(name.as_c_str()),
                        original_name_index,
                    ) > 0
                })
            };
            if !has_required_goods {
                Some(OrganizingCreateFactionGate::RequiredGoods)
            } else if player.money() < parameters.create_faction_money() as u32 {
                Some(OrganizingCreateFactionGate::Money)
            } else {
                None
            }
        }
    };
    let map_id = message.map_id();
    if let Some(gate) = gate {
        let response = send_create_faction_response(
            game, map_id, request_id, cookie, player_id, 0,
        );
        return Some(Ok(OrganizingCreateFactionDispatch {
            request_id,
            cookie,
            player_id,
            country,
            faction_name,
            outcome: OrganizingCreateFactionOutcome::GateRejected { gate, response },
        }));
    }

    let mut effects = WorldFactionCreationEffects {
        game,
        callbacks,
        persistent_name_exists: false,
        faction_create_log_enabled,
        write_faction_create_log,
    };
    let preparation = match organizing.prepare_faction_creation(
        game,
        player_id,
        &mut faction_name,
        &mut effects,
    ) {
        Ok(preparation) => preparation,
        Err(source) => return Some(Err(OrganizingCreateFactionBlock::Creation(source))),
    };
    let creation = match preparation {
        FactionCreationPreparation::Rejected(reason) => {
            FactionCreationOutcome::Rejected(reason)
        }
        FactionCreationPreparation::ReadyForPersistentLookup => {
            let persistent_name_exists = rs_player
                .is_name_exist(legacy_c_string_prefix(&faction_name), player_database)
                .await;
            effects.persistent_name_exists = persistent_name_exists;
            match organizing.finish_faction_creation(
                game,
                parameters,
                player_id,
                0,
                established_time,
                &faction_name,
                country,
                persistent_name_exists,
                &mut effects,
            ) {
                Ok(outcome) => outcome,
                Err(source) => {
                    return Some(Err(OrganizingCreateFactionBlock::Creation(source)));
                }
            }
        }
    };
    let (result, notice_id, created) = match &creation {
        FactionCreationOutcome::Rejected(
            crate::worldserver::appworld::organizingsystem::organizingctrl::FactionCreationRejection::NameExists,
        ) => (0, b"WS0116".as_slice(), false),
        FactionCreationOutcome::Rejected(_) => (0, b"WS0115".as_slice(), false),
        FactionCreationOutcome::Created(_) => (1, b"WS0117".as_slice(), true),
    };
    drop(effects);
    let player_refresh = if created {
        match game.update_player_faction_info(organizing, player_id) {
            Ok(Some(report)) => Some(report),
            Ok(None) => {
                return Some(Err(
                    OrganizingCreateFactionBlock::PlayerRefreshOwnerMissing,
                ));
            }
            Err(source) => {
                return Some(Err(OrganizingCreateFactionBlock::PlayerRefresh(source)));
            }
        }
    } else {
        None
    };
    let faction_snapshot = created.then(|| {
        organizing.add_faction_to_client_by_player_id(game, player_id)
    });
    let all_factions_snapshot = created.then(|| {
        organizing.add_all_faction_info_to_client_by_player_id(game, player_id)
    });
    let response = send_create_faction_response(
        game, map_id, request_id, cookie, player_id, result,
    );
    let first_text = game.get_string_by_id(notice_id).to_vec();
    let second_text = game.get_string_by_id(b"WS0118").to_vec();
    let notice = COrganizingCtrl::send_organizing_info_to_client(
        game,
        FactionMemberInfoRequest {
            recipient_player_id: player_id,
            first_text: legacy_c_string_prefix(&first_text),
            second_text: legacy_c_string_prefix(&second_text),
            information_type: map_id,
            color: 0xFFDA_EDFE,
            trailing_value: 0,
        },
    );
    Some(Ok(OrganizingCreateFactionDispatch {
        request_id,
        cookie,
        player_id,
        country,
        faction_name,
        outcome: OrganizingCreateFactionOutcome::Creation {
            outcome: creation,
            response,
            notice,
            player_refresh,
            faction_snapshot,
            all_factions_snapshot,
        },
    }))
}

fn send_create_faction_response(
    game: &CGame,
    map_id: i32,
    request_id: i64,
    cookie: i32,
    player_id: i32,
    result: i32,
) -> OrganizingCreateFactionResponse {
    let mut response = CMessage::new(CREATE_FACTION_RESPONSE_TYPE);
    response.base_mut().add_long64(request_id);
    response.base_mut().add_long(cookie);
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(result);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = game.send_msg_to_game_server(map_id, &response);
    OrganizingCreateFactionResponse {
        request_id,
        cookie,
        player_id,
        result,
        map_id,
        wire,
        delivery,
    }
}

pub(crate) fn dispatch_initial_organizing_data(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
) -> Option<OrganizingInitialDataDispatch> {
    if message.message_type() != INITIAL_ORGANIZING_DATA_MESSAGE_TYPE {
        return None;
    }
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let Some(player) = game.online_player_by_id(player_id as u32) else {
        return Some(OrganizingInitialDataDispatch {
            player_id,
            outcome: Ok(OrganizingInitialDataOutcome::PlayerOffline),
        });
    };
    if player.faction_data_received() {
        return Some(OrganizingInitialDataDispatch {
            player_id,
            outcome: Ok(OrganizingInitialDataOutcome::AlreadyReceived),
        });
    }

    let faction_snapshot = match organizing.add_faction_to_client_by_player_id(game, player_id) {
        Ok(outcome) => outcome,
        Err(source) => {
            return Some(OrganizingInitialDataDispatch {
                player_id,
                outcome: Err(OrganizingInitialDataBlock::Faction(source)),
            });
        }
    };
    let union_snapshot = match organizing.add_union_to_client_by_player_id(game, player_id) {
        Ok(outcome) => outcome,
        Err(source) => {
            return Some(OrganizingInitialDataDispatch {
                player_id,
                outcome: Err(OrganizingInitialDataBlock::Union {
                    faction_snapshot,
                    source,
                }),
            });
        }
    };
    let all_factions_snapshot =
        match organizing.add_all_faction_info_to_client_by_player_id(game, player_id) {
            Ok(outcome) => outcome,
            Err(source) => {
                return Some(OrganizingInitialDataDispatch {
                    player_id,
                    outcome: Err(OrganizingInitialDataBlock::AllFactions {
                        faction_snapshot,
                        union_snapshot,
                        source,
                    }),
                });
            }
        };
    Some(OrganizingInitialDataDispatch {
        player_id,
        outcome: Ok(OrganizingInitialDataOutcome::Sent {
            faction_snapshot,
            union_snapshot,
            all_factions_snapshot,
        }),
    })
}

// Контракт country-lookup-а и сама ветвь `0x60107` faction list перенесены в
// realm (`nebokrai_realm::app::organsysmessage`); impl контракта для
// `WorldUnionApplicationEffects` ниже резолвится через glob re-export.

/// Выполняет `0x60108`: `(player ID, discarded Long, name[20])`,
/// online/country gate и virtual `ApplyForJoin(player ID, 0, 0)`.
/// Ветвь `0x60108` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает war/string/log адаптер faction-пути и передаёт оба
/// effects-владельца обработчику в исходных типах.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_application(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    use_log_system: bool,
    faction_apply_log_enabled: bool,
    write_faction_apply_log: &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    effects: &mut WorldUnionApplicationEffects<'_>,
) -> Option<
    Result<
        OrganizingFactionApplicationDispatch<UnionApplicationSessionReport>,
        OrganizingFactionApplicationDispatchBlock<UnionApplicationSessionBlock>,
    >,
> {
    let mut faction_effects = WorldFactionApplicationEffects {
        game,
        village_war,
        attack_city,
        use_log_system,
        faction_apply_log_enabled,
        write_faction_apply_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_application(
        message,
        game,
        organizing,
        parameters,
        &mut faction_effects,
        effects,
    )
}

// Ветвь `0x60109` cancel application перенесена в realm
// (`nebokrai_realm::app::organsysmessage`); имя доступно через glob re-export.

/// Ветвь `0x6010A` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает join-effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_application_decision(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    goods_war: &CGoodsWarMember,
    use_log_system: bool,
    faction_join_log_enabled: bool,
    write_faction_join_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<
    Result<OrganizingFactionApplicationDecisionDispatch, OrganizingFactionDoJoinBlock>,
> {
    let mut effects = WorldFactionDoJoinEffects {
        game,
        village_war,
        attack_city,
        goods_war,
        use_log_system,
        faction_join_log_enabled,
        callbacks,
        update_player,
        write_faction_join_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_application_decision(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x6010B` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает fire-out effects адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_fire_out(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    goods_war: &mut CGoodsWarMember,
    use_log_system: bool,
    faction_fire_out_log_enabled: bool,
    write_faction_fire_out_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8], i32),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionFireOutDispatch, OrganizingFactionFireOutBlock>> {
    let mut effects = WorldFactionFireOutEffects {
        game,
        village_war,
        attack_city,
        goods_war,
        use_log_system,
        faction_fire_out_log_enabled,
        callbacks,
        update_player,
        write_faction_fire_out_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_fire_out(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x6010C` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_fire_out(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionFireOutDispatch, OrganizingUnionFireOutBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    nebokrai_realm::app::organsysmessage::dispatch_union_fire_out(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
        update_player,
    )
}

/// Ветвь `0x6010D` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает exit effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_exit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    village_war: &CVillageWarSys,
    attack_city: &CAttackCitySys,
    goods_war: &mut CGoodsWarMember,
    use_log_system: bool,
    faction_quit_log_enabled: bool,
    write_faction_quit_log: &mut dyn FnMut(i32, &[u8], i32, &[u8], i32),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionExitDispatch, OrganizingFactionExitBlock>> {
    let mut effects = WorldFactionExitEffects {
        game,
        village_war,
        attack_city,
        goods_war,
        use_log_system,
        faction_quit_log_enabled,
        callbacks,
        update_player,
        write_faction_quit_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_exit(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x6010E` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_exit(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionExitDispatch, OrganizingUnionExitBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    nebokrai_realm::app::organsysmessage::dispatch_union_exit(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
        update_player,
    )
}

/// Ветвь `0x6010F` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает demise effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_demise(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    country_handler: &CCountryHandler,
    attack_city: &CAttackCitySys,
    goods_war: &CGoodsWarMember,
    faction_master_log_enabled: bool,
    write_faction_master_log:
        &mut dyn FnMut(i32, &[u8], i32, &[u8], i32, &[u8]),
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionDemiseDispatch, OrganizingFactionDemiseBlock>> {
    let mut effects = WorldFactionDemiseEffects {
        game,
        attack_city,
        goods_war,
        country_handler,
        faction_master_log_enabled,
        callbacks,
        update_player,
        write_faction_master_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_demise(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60110` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и исходный
/// tick-callback, передавая ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_demise(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDemiseDispatch, OrganizingUnionDemiseBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    let mut get_tick = legacy_tick_ms;
    nebokrai_realm::app::organsysmessage::dispatch_union_demise(
        message,
        game,
        organizing,
        &mut effects,
        &mut get_tick,
        update_player,
    )
}

// Ветвь `0x60111` disband и её finalize-continuation перенесены в realm
// (`nebokrai_realm::app::organsysmessage`); имена доступны через glob re-export.

/// Ветвь `0x60112` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает union effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
pub(crate) fn dispatch_union_disband(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingUnionDisbandDispatch, OrganizingUnionDisbandBlock>> {
    let mut effects = WorldUnionFireOutEffects { game, callbacks };
    nebokrai_realm::app::organsysmessage::dispatch_union_disband(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
        update_player,
    )
}

/// Ветвь `0x60113` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает dub effects адаптер старого пакета и передаёт ход
/// realm-обработчику в исходных типах.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_dub(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    use_log_system: bool,
    faction_title_log_enabled: bool,
    write_faction_title_log: &mut dyn FnMut(
        i32,
        &[u8],
        &[u8],
        &[u8],
        i32,
        &[u8],
        i32,
        &[u8],
    ),
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionDubDispatch, OrganizingFactionDubBlock>> {
    let mut effects = WorldFactionDubEffects {
        game,
        callbacks,
        update_player,
        use_log_system,
        faction_title_log_enabled,
        write_faction_title_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_dub(
        message,
        game,
        organizing,
        &mut effects,
    )
}

/// Ветви `0x60114/0x60115` перенесены в realm
/// (`nebokrai_realm::app::organsysmessage`); обвязка лишь собирает purview
/// effects адаптер старого пакета и передаёт ход realm-обработчику.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_faction_purview(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    use_log_system: bool,
    add_log_enabled: bool,
    revoke_log_enabled: bool,
    write_log: &mut dyn FnMut(i32, &[u8], i32, i32, &[u8], i32, &[u8], i32),
) -> Option<Result<OrganizingFactionPurviewDispatch, OrganizingFactionPurviewBlock>> {
    let mut effects = WorldFactionPurviewEffects {
        game,
        callbacks,
        use_log_system,
        add_log_enabled,
        revoke_log_enabled,
        write_log,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_purview(
        message,
        game,
        organizing,
        &mut effects,
    )
}

pub(crate) fn dispatch_consumed_long(
    message: &mut CMessage,
) -> Option<OrganizingConsumedLongDispatch> {
    let message_type = message.message_type();
    if !CONSUMED_LONG_MESSAGE_TYPES.contains(&message_type) {
        return None;
    }
    let value = message.base_mut().get_long().unwrap_or(0);
    Some(OrganizingConsumedLongDispatch {
        message_type,
        value,
    })
}

// Ветвь `0x6011E` declare war faction list перенесена в realm
// (`nebokrai_realm::app::organsysmessage`); имя доступно через glob re-export.

/// Ветвь `0x6011F` перенесена в realm (`nebokrai_realm::app::organsysmessage`)
/// двухфазно: realm разбирает шапку, обвязка выполняет decode игрока из
/// wire-хвоста прежним владельцем игры (общий заём effects-адаптера и
/// `&mut`-decode одной игры не проходят одну границу вызова), затем realm
/// завершает declaration и ответ в исходном порядке.
pub(crate) fn dispatch_declare_faction_war(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    faction_wars: &mut CFactionWarSys,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    update_player: &mut dyn FnMut(i32),
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingDeclareFactionWarDispatch, OrganizingDeclareFactionWarBlock>> {
    let request = nebokrai_realm::app::organsysmessage::parse_declare_faction_war(message)?;
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            request.player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingDeclareFactionWarBlock::PlayerDecode(source)));
            }
        }
    };
    let mut effects = WorldFactionWarDeclarationEffects::new(
        game,
        organizing,
        update_player,
    );
    Some(nebokrai_realm::app::organsysmessage::finish_declare_faction_war(
        request,
        player_online,
        faction_wars,
        &mut effects,
        sender,
    ))
}

pub(crate) fn dispatch_faction_billboard(
    message: &mut CMessage,
    organizing: &COrganizingCtrl,
    world_string: &mut dyn FnMut(&[u8]) -> Vec<u8>,
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingFactionBillboardOutcome, OrganizingFactionBillboardBlock>> {
    if message.message_type() != FACTION_BILLBOARD_MESSAGE_TYPE {
        return None;
    }

 // Три `GetStringByID` исходник выполнял при первом входе в case, ещё до
 // чтения request и проверки типа, после чего process-static строки уже не
 // реагировали на reload string table.
    let titles = FACTION_BILLBOARD_TITLES.get_or_init(|| {
        [
            legacy_c_string_prefix(&world_string(b"WS0134")).to_vec(),
            legacy_c_string_prefix(&world_string(b"WS0133")).to_vec(),
            legacy_c_string_prefix(&world_string(b"WS0132")).to_vec(),
        ]
    });
    let socket_id = message.socket_id();
    let request_id = message.base_mut().get_long().unwrap_or(0);
    let billboard_type = message.base_mut().get_long().unwrap_or(0);
    if billboard_type > 2 {
        return Some(Ok(OrganizingFactionBillboardOutcome::TypeAboveRange {
            request_id,
            billboard_type,
        }));
    }
    let Ok(title_index) = usize::try_from(billboard_type) else {
 // Оригинал проверяет только `2 < type`, поэтому отрицательный selector
 // индексирует статический `std::string[3]` до начала массива. Rust не
 // воспроизводит результат такого чтения за границами.
        return Some(Err(OrganizingFactionBillboardBlock {
            request_id,
            billboard_type,
        }));
    };

    let title = titles[title_index].clone();
    let mut payload = Vec::new();
    organizing.add_faction_billboard_to_byte_array(&mut payload, billboard_type);

    let mut response = CMessage::new(FACTION_BILLBOARD_RESPONSE_TYPE);
    response.base_mut().add_long(request_id);
    response.base_mut().add(&title);
    response.base_mut().add_byte(0);
    response.base_mut().add(&payload);
    response.base_mut().update();
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_to_socket(sender, socket_id);

    Some(Ok(OrganizingFactionBillboardOutcome::Sent {
        request_id,
        billboard_type,
        response: OrganizingFactionBillboardResponse {
            socket_id,
            title,
            payload,
            wire,
            delivery,
        },
    }))
}

#[allow(
    clippy::too_many_arguments,
    reason = "opcode использует прежние game/faction/goods/string/log singleton-ы"
)]
/// Ветвь `0x60126` пока остаётся локальной: realm-обработчику нужна `&mut`
/// игра для чтения игрока из wire-хвоста одновременно с заранее построенным
/// upgrade effects-адаптером, который держит ту же игру общим заёмом, а сам
/// вызов `upgrade_faction` — `&CGame`-inherent контроллера вне организационного
/// view; двухфазная форма `0x6011F` снимает только первую часть, поэтому ветвь
/// ждёт свёртки вызова (ближайший родственник — `0x60103`).
pub(crate) fn dispatch_faction_upgrade(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    registry: &GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    coefficients: &PlayerPropertyCoefficients,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionUpgradeDispatch, OrganizingFactionUpgradeBlock>> {
    if message.message_type() != UPGRADE_FACTION_MESSAGE_TYPE {
        return None;
    }

    let faction_id = message.base_mut().get_long().unwrap_or(0);
    let player_id = message.base_mut().get_long().unwrap_or(0);
    let player_online = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decord_online_player_by_id(
            player_id as u32,
            source,
            cursor,
            registry,
            coefficients,
        ) {
            Ok(player_online) => player_online,
            Err(source) => {
                return Some(Err(OrganizingFactionUpgradeBlock::PlayerDecode(source)));
            }
        }
    };
    if !player_online {
        return Some(Ok(OrganizingFactionUpgradeDispatch {
            faction_id,
            player_id,
            outcome: OrganizingFactionUpgradeOutcome::PlayerOffline,
        }));
    }

    let game_ref: &CGame = game;
    let mut effects = WorldFactionUpgradeEffects {
        game: game_ref,
        registry,
        original_name_index,
        use_log_system,
        callbacks,
        update_player,
    };
    let outcome = match organizing.upgrade_faction(
        game_ref,
        parameters,
        faction_id,
        player_id,
        &mut effects,
    ) {
        Ok(Some(outcome)) => OrganizingFactionUpgradeOutcome::Upgrade(outcome),
        Ok(None) => OrganizingFactionUpgradeOutcome::FactionMissing,
        Err(source) => return Some(Err(OrganizingFactionUpgradeBlock::Upgrade(source))),
    };

    Some(Ok(OrganizingFactionUpgradeDispatch {
        faction_id,
        player_id,
        outcome,
    }))
}

/// Ветвь `0x60127` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает upload-icon effects адаптер старого пакета и
/// передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_upload_icon(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionUploadIconDispatch, FactionUploadIconBlock>> {
    let mut effects = WorldFactionUploadIconEffects { game, callbacks };
    nebokrai_realm::app::organsysmessage::dispatch_faction_upload_icon(
        message,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60128` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает contributor effects адаптер старого пакета и
/// передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_contributor(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionContributorDispatch, OrganizingContributorBlock>> {
    let mut effects = WorldFactionContributorEffects {
        game,
        callbacks,
        update_player,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_contributor(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

/// Ветвь `0x60129` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь раскрывает скаляры log-callback старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_experience(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    use_log_system: bool,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
) -> Option<Result<OrganizingFactionExperienceDispatch, FactionExperienceBlock>> {
    nebokrai_realm::app::organsysmessage::dispatch_faction_experience(
        message,
        game,
        organizing,
        use_log_system,
        callbacks.faction_experience_log_enabled,
        &mut *callbacks.write_faction_experience_log,
    )
}

// Ветвь `0x6012A` изменение состояния члена перенесена в realm
// (`nebokrai_realm::app::organsysmessage`); имя доступно через glob re-export.

// Ветви `0x6012B/0x6012C` operate/adjust tax перенесены в realm
// (`nebokrai_realm::app::organsysmessage`); имя доступно через glob re-export.

// `OrganizingRegionParamDispatch` и `dispatch_region_param_update` перенесены в
// realm (`nebokrai_realm::app::organsysmessage`) первой без-организационной
// ветвью диспетчера; имена доступны здесь через glob re-export выше.
pub(crate) fn dispatch_region_route(
    message: &mut CMessage,
    game: &CGame,
) -> Option<OrganizingRegionRouteDispatch> {
    if message.message_type() != ROUTE_REGION_MESSAGE_TYPE {
        return None;
    }

    let region_id = message.base_mut().get_long().unwrap_or(0);
    let game_server_number = game.game_server_number_by_region_id(region_id);
    message.set_message_type(ROUTE_REGION_RESPONSE_TYPE);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = game.send_msg_to_game_server(game_server_number, message);
    Some(OrganizingRegionRouteDispatch {
        region_id,
        game_server_number,
        wire,
        delivery,
    })
}

// Ветвь `0x6012F` operate city gate перенесена в realm
// (`nebokrai_realm::app::organsysmessage`); имя доступно через glob re-export.

pub(crate) fn dispatch_city_transfer<Effects>(
    message: &mut CMessage,
    game: &CGame,
    countries: &CCountryHandler,
    organizing: &mut COrganizingCtrl,
    attack_city: &CAttackCitySys,
    village_war: &CVillageWarSys,
    effects: &mut Effects,
) -> Option<
    Result<
        OrganizingCityTransferDispatch<Effects::SessionReport>,
        CityTransferStartBlock<Effects::SessionBlock>,
    >,
>
where
    Effects: CityTransferEffects,
{
    if message.message_type() != TRANSFER_CITY_OWNER_MESSAGE_TYPE {
        return None;
    }

    let requester_player_id = message.base_mut().get_long().unwrap_or(0);
    let target_faction_id = message.base_mut().get_long().unwrap_or(0);
    let region_id = message.base_mut().get_long().unwrap_or(0);
    let outcome = organizing.transfer_city_owner(
        game,
        countries,
        attack_city,
        village_war,
        requester_player_id,
        target_faction_id,
        region_id,
        effects,
    );
    Some(outcome.map(|outcome| OrganizingCityTransferDispatch {
        requester_player_id,
        target_faction_id,
        region_id,
        outcome,
    }))
}

// Ветви `0x60132` admission permit и `0x60133` attack city end перенесены в
// realm (`nebokrai_realm::app::organsysmessage`); имена доступны через glob
// re-export.

struct WorldVillageWarApplicationContext<'game, 'callbacks, 'effects> {
    game: &'game CGame,
    organizing: &'game COrganizingCtrl,
    organizing_parameters: &'game COrganizingParam,
    attack_city: &'game CAttackCitySys,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
}

impl VillageWarApplicationContext for WorldVillageWarApplicationContext<'_, '_, '_> {
    type Block = OrganizingVillageWarApplicationBlock;

    fn faction_master_for_player(&mut self, player_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingVillageWarApplicationBlock::FactionMaster)
    }

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).is_some())
    }

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block> {
        Ok(self.game.has_materialized_region(region_id))
    }

    fn attack_village_min_level(&mut self) -> Result<i32, Self::Block> {
        Ok(self.organizing_parameters.attack_village_minimum_level())
    }

    fn faction_level(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFaction { faction_id })?
            .level()
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFactionLevel { faction_id })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingVillageWarApplicationBlock::MissingRegionOwner { region_id },
        )
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingVillageWarApplicationBlock::NullUnion { map_key })
            }
        }
    }

    fn faction_owned_city_count(&mut self, faction_id: i32) -> Result<usize, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.owned_cities().len())
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFaction { faction_id })
    }

    fn already_declared_for_city_war(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.attack_city.is_already_declared_for_war(faction_id))
    }

    fn faction_name(&mut self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingVillageWarApplicationBlock::MissingFaction { faction_id })
    }

    fn region_name(&mut self, region_id: i32) -> Result<Vec<u8>, Self::Block> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Ok(legacy_c_string_prefix(name).to_vec()),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => Err(
                OrganizingVillageWarApplicationBlock::MissingRegionName { region_id },
            ),
        }
    }

    fn send_level_rejection(
        &mut self,
        player_id: i32,
        title_string_id: &'static [u8],
        text_string_id: &'static [u8],
    ) -> Result<(), Self::Block> {
        let title = self.game.get_string_by_id(title_string_id).to_vec();
        let text = self.game.get_string_by_id(text_string_id).to_vec();
        let _ = COrganizingCtrl::send_organizing_info_to_client(
            self.game,
            FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: legacy_c_string_prefix(&text),
                second_text: legacy_c_string_prefix(&title),
                information_type: -1,
                color: 0xFFFF_0000,
                trailing_value: u32::MAX,
            },
        );
        Ok(())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(legacy_c_string_prefix(argument)))
            .collect::<Vec<_>>();
        let formatted = self.game.format_world_string(string_id, &arguments);
        let formatted = legacy_c_string_prefix(&formatted);
        Ok(formatted.to_vec())
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

/// Ветвь `0x60135` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает application-контекст адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_village_war_application(
    message: &mut CMessage,
    game: &CGame,
    organizing: &COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    attack_city: &CAttackCitySys,
    village_war: &mut CVillageWarSys,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    sender: Option<&ServerCommandHandle>,
) -> Option<
    Result<OrganizingVillageWarApplicationDispatch, OrganizingVillageWarApplicationBlock>,
> {
    let mut context = WorldVillageWarApplicationContext {
        game,
        organizing,
        organizing_parameters,
        attack_city,
        callbacks,
    };
    nebokrai_realm::app::organsysmessage::dispatch_village_war_application(
        message,
        village_war,
        &mut context,
        sender,
    )
}

struct CityWarEnemyMutationEffects<'game> {
    game: &'game CGame,
    enemy_id: i32,
    enemy_name: Vec<u8>,
}

impl FactionEnemyMutationContext for CityWarEnemyMutationEffects<'_> {
    fn organizing_name(&self, organizing_id: i32) -> Option<Vec<u8>> {
        (organizing_id == self.enemy_id).then(|| self.enemy_name.clone())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[FactionEnemyWarLogArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                FactionEnemyWarLogArgument::Text(text) => UnionFormatArgument::Text(text),
                FactionEnemyWarLogArgument::Unsigned(value) => {
                    UnionFormatArgument::Signed(*value as i32)
                }
            })
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn put_war_log(&mut self, text: &[u8]) {
        put_string_to_file("war", text);
    }
}

struct WorldAttackCityApplicationContext<'game, 'organizing, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    globe_setup: &'game GlobeSetupSnapshot,
    organizing: &'organizing mut COrganizingCtrl,
    organizing_parameters: &'game COrganizingParam,
    village_war: &'game CVillageWarSys,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl AttackCityEnemyRelationContext
    for WorldAttackCityApplicationContext<'_, '_, '_, '_, '_>
{
    type Block = OrganizingCityWarApplicationBlock;

    fn clear_all_city_faction_relations(&mut self) -> Result<(), Self::Block> {
        self.organizing.clear_all_city_faction_relations();
        Ok(())
    }

    fn city_owner_faction_id(
        &mut self,
        city_region_id: i32,
    ) -> Result<Option<i32>, Self::Block> {
        Ok(self.game.region_owned_faction_id(city_region_id))
    }

    fn expand_faction_organizings(
        &mut self,
        faction_id: i32,
    ) -> Result<Vec<i32>, Self::Block> {
        self.organizing
            .expand_city_war_faction_organizings(faction_id)
            .map_err(|FactionUnionMembershipLookupBlock { map_key }| {
                OrganizingCityWarApplicationBlock::NullUnion { map_key }
            })
    }

    fn add_city_war_enemy_organizing(
        &mut self,
        organizing_id: i32,
        enemy_organizing_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_organizing_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingCityWarApplicationBlock::MissingEnemyOrganizing {
                organizing_id: enemy_organizing_id,
            })?;
        let mut effects = CityWarEnemyMutationEffects {
            game: self.game,
            enemy_id: enemy_organizing_id,
            enemy_name,
        };
        let found = self
            .organizing
            .add_city_war_enemy_organizing(
                organizing_id,
                enemy_organizing_id,
                &mut effects,
            )
            .map_err(|source| OrganizingCityWarApplicationBlock::EnemyMutation {
                organizing_id,
                enemy_organizing_id,
                source,
            })?;
        if !found {
            return Err(
                OrganizingCityWarApplicationBlock::MissingEnemyOrganizing { organizing_id },
            );
        }
        Ok(())
    }

    fn set_all_city_faction_enemy_changed(
        &mut self,
        changed: bool,
    ) -> Result<(), Self::Block> {
        self.organizing
            .set_all_city_faction_enemy_changed(changed);
        Ok(())
    }

    fn update_all_city_enemy_faction_relations(&mut self) -> Result<(), Self::Block> {
        let _ = self
            .organizing
            .update_all_city_enemy_faction_relations(self.game, self.update_player);
        Ok(())
    }
}

impl AttackCityApplicationContext
    for WorldAttackCityApplicationContext<'_, '_, '_, '_, '_>
{
    fn faction_master_for_player(&mut self, player_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_id_by_master_player(player_id)
            .map_err(OrganizingCityWarApplicationBlock::FactionMaster)
    }

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).is_some())
    }

    fn region_exists(&mut self, region_id: i32) -> Result<bool, Self::Block> {
        Ok(self.game.has_materialized_region(region_id))
    }

    fn attack_city_min_level(&mut self) -> Result<i32, Self::Block> {
        Ok(self.organizing_parameters.attack_city_minimum_level())
    }

    fn faction_level(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })?
            .level()
            .ok_or(OrganizingCityWarApplicationBlock::MissingFactionLevel { faction_id })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingCityWarApplicationBlock::MissingRegionOwner { region_id },
        )
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingCityWarApplicationBlock::NullUnion { map_key })
            }
        }
    }

    fn faction_owned_city_count(&mut self, faction_id: i32) -> Result<usize, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.owned_cities().len())
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })
    }

    fn already_declared_for_village_war(
        &mut self,
        faction_id: i32,
    ) -> Result<bool, Self::Block> {
        Ok(self.village_war.is_already_declared_for_war(faction_id))
    }

    fn faction_organizing_id(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| faction.faction_id())
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })
    }

    fn faction_name(&mut self, faction_id: i32) -> Result<Vec<u8>, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingCityWarApplicationBlock::MissingFaction { faction_id })
    }

    fn region_name(&mut self, region_id: i32) -> Result<Vec<u8>, Self::Block> {
        match self.game.region_name(region_id) {
            WorldRegionNameLookup::Name(name) => Ok(legacy_c_string_prefix(name).to_vec()),
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => {
                Err(OrganizingCityWarApplicationBlock::MissingRegionName { region_id })
            }
        }
    }

    fn region_country(&mut self, region_id: i32) -> Result<u8, Self::Block> {
        self.game.region_country_id(region_id).ok_or(
            OrganizingCityWarApplicationBlock::MissingRegionCountry { region_id },
        )
    }

    fn send_level_rejection(
        &mut self,
        player_id: i32,
        title_string_id: &'static [u8],
        text_string_id: &'static [u8],
    ) -> Result<(), Self::Block> {
        let title = self.game.get_string_by_id(title_string_id).to_vec();
        let text = self.game.get_string_by_id(text_string_id).to_vec();
        let _ = COrganizingCtrl::send_organizing_info_to_client(
            self.game,
            FactionMemberInfoRequest {
                recipient_player_id: player_id,
                first_text: legacy_c_string_prefix(&text),
                second_text: legacy_c_string_prefix(&title),
                information_type: -1,
                color: 0xFFFF_0000,
                trailing_value: u32::MAX,
            },
        );
        Ok(())
    }

    fn format_declaration_notice(
        &mut self,
        country_id: u8,
        faction_name: &[u8],
    ) -> Result<Vec<u8>, Self::Block> {
        let fallback = self.game.get_string_by_id(b"WS0103").to_vec();
        let (string_id, notice) = match self.globe_setup.country_name(country_id) {
            Some(country_name) => (
                b"WS0145" as &'static [u8],
                self.game.format_world_string(
                    b"WS0145",
                    &[
                        UnionFormatArgument::Text(country_name),
                        UnionFormatArgument::Text(legacy_c_string_prefix(faction_name)),
                        UnionFormatArgument::Text(country_name),
                    ],
                ),
            ),
            None => (b"WS0103" as &'static [u8], fallback),
        };
        bounded_city_war_notice(string_id, notice)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[AttackCityWarResultFormatArgument<'_>],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                AttackCityWarResultFormatArgument::Text(text) => {
                    UnionFormatArgument::Text(legacy_c_string_prefix(text))
                }
                AttackCityWarResultFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        bounded_city_war_notice(
            string_id,
            self.game.format_world_string(string_id, &arguments),
        )
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

fn bounded_city_war_notice(
    _string_id: &'static [u8],
    notice: Vec<u8>,
) -> Result<Vec<u8>, OrganizingCityWarApplicationBlock> {
    let notice = legacy_c_string_prefix(&notice);
    Ok(notice.to_vec())
}

/// Ветвь `0x60137` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает application-контекст адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
#[allow(
    clippy::too_many_arguments,
    reason = "аргументы явно связывают исходные singleton-owner-ы без глобального состояния"
)]
pub(crate) fn dispatch_city_war_application(
    message: &mut CMessage,
    game: &CGame,
    globe_setup: &GlobeSetupSnapshot,
    organizing: &mut COrganizingCtrl,
    organizing_parameters: &COrganizingParam,
    attack_city: &mut CAttackCitySys,
    village_war: &CVillageWarSys,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    sender: Option<&ServerCommandHandle>,
) -> Option<Result<OrganizingCityWarApplicationDispatch, OrganizingCityWarApplicationBlock>> {
    let mut context = WorldAttackCityApplicationContext {
        game,
        globe_setup,
        organizing,
        organizing_parameters,
        village_war,
        callbacks,
        update_player,
    };
    nebokrai_realm::app::organsysmessage::dispatch_city_war_application(
        message,
        attack_city,
        &mut context,
        sender,
    )
}

/// Результат-контекст ветви `0x60138` и семейного reload: game/organizing
/// доступы идут напрямую в прежних owner-ах, а все country-контакты
/// (присутствие, флаг войны, governance-переход короля) — через узкий
/// [`WorldCountryWarGate`], чья реализация собирается у dispatcher-а в
/// `worldserver/game.rs` над живым `CCountryHandler`.
/// Результат-контекст ветви `0x60138` и семейного reload: game/organizing
/// доступы идут напрямую в прежних owner-ах, а все country-контакты
/// (присутствие, флаг войны, governance-переход короля) — через узкий
/// [`WorldCountryWarGate`], чья реализация собирается у dispatcher-а в
/// `worldserver/game.rs` над живым `CCountryHandler`. Gate хранится в
/// `Option` и изымается (`take`) ровно на один SetKing-переход, потому что
/// governance context-сеанс `CCountry::SetKing` обслуживает сам
/// результат-контекст (`self`), а `&mut dyn` gate-а и `&mut self` одного
/// адаптера одновременно заиметь нельзя; до и после перехода slot всегда
/// полон (см. лёгкие методы ниже).
struct WorldAttackCityResultContext<
    'game,
    'organizing,
    'country,
    'callbacks,
    'effects,
    'update,
    'configuration,
> {
    game: &'game mut CGame,
    organizing: &'organizing mut COrganizingCtrl,
    country_gate: Option<&'country mut dyn WorldCountryWarGate>,
    globe_setup: &'configuration GlobeSetupSnapshot,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl AttackCityEnemyRelationContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    type Block = OrganizingCityWarResultContextBlock;

    fn clear_all_city_faction_relations(&mut self) -> Result<(), Self::Block> {
        self.organizing.clear_all_city_faction_relations();
        Ok(())
    }

    fn city_owner_faction_id(
        &mut self,
        city_region_id: i32,
    ) -> Result<Option<i32>, Self::Block> {
        Ok(self.game.region_owned_faction_id(city_region_id))
    }

    fn expand_faction_organizings(
        &mut self,
        faction_id: i32,
    ) -> Result<Vec<i32>, Self::Block> {
        self.organizing
            .expand_city_war_faction_organizings(faction_id)
            .map_err(|FactionUnionMembershipLookupBlock { map_key }| {
                OrganizingCityWarResultContextBlock::NullUnion { map_key }
            })
    }

    fn add_city_war_enemy_organizing(
        &mut self,
        organizing_id: i32,
        enemy_organizing_id: i32,
    ) -> Result<(), Self::Block> {
        let enemy_name = self
            .organizing
            .faction_by_id(enemy_organizing_id)
            .map(|faction| legacy_c_string_prefix(faction.name()).to_vec())
            .ok_or(OrganizingCityWarResultContextBlock::MissingEnemyOrganizing {
                organizing_id: enemy_organizing_id,
            })?;
        let mut effects = CityWarEnemyMutationEffects {
            game: self.game,
            enemy_id: enemy_organizing_id,
            enemy_name,
        };
        let found = self
            .organizing
            .add_city_war_enemy_organizing(
                organizing_id,
                enemy_organizing_id,
                &mut effects,
            )
            .map_err(|source| OrganizingCityWarResultContextBlock::EnemyMutation {
                organizing_id,
                enemy_organizing_id,
                source,
            })?;
        if !found {
            return Err(
                OrganizingCityWarResultContextBlock::MissingEnemyOrganizing { organizing_id },
            );
        }
        Ok(())
    }

    fn set_all_city_faction_enemy_changed(
        &mut self,
        changed: bool,
    ) -> Result<(), Self::Block> {
        self.organizing
            .set_all_city_faction_enemy_changed(changed);
        Ok(())
    }

    fn update_all_city_enemy_faction_relations(&mut self) -> Result<(), Self::Block> {
        let _ = self
            .organizing
            .update_all_city_enemy_faction_relations(self.game, self.update_player);
        Ok(())
    }
}

impl AttackCityWarEndContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    fn clear_region_country_warring_if_present(
        &mut self,
        city_region_id: i32,
    ) -> Result<(), Self::Block> {
        let Some(country_id) = self.game.region_country_id(city_region_id) else {
            return Ok(());
        };
        let gate = self
            .country_gate
            .as_deref_mut()
            .expect("country gate изымается только на SetKing-переход");
        let _ = gate.set_country_warring(country_id, false);
        Ok(())
    }
}

impl CountryExileResultContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>> {
        self.game
            .map_player(player_id as u32)
            .map(|player| legacy_c_string_prefix(player.get_name()).to_vec())
    }

    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget> {
        self.game
            .online_player_by_id(player_id as u32)
            .map(|player| CountryExileTarget {
                name: legacy_c_string_prefix(player.get_name()).to_vec(),
                country: player.country(),
                level: player.get_level(),
                credit: player.credit(),
                pk_count: player.pk_count(),
                is_god: player.is_god(),
            })
    }

    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset> {
        self.game
            .reset_online_player_murder_counters(player_id as u32)
            .map(|reset| CountryAbsolveCounterReset {
                previous_kill_count: reset.previous_kill_count,
                previous_pk_count: reset.previous_pk_count,
            })
    }

    fn faction_id_by_player(
        &mut self,
        player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        match self.organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => Ok(0),
            FreePlayerLookup::Faction(faction_id) => Ok(faction_id),
            FreePlayerLookup::BlockedNullFaction { .. } => {
                Err(CountryGovernanceContextBlock::PlayerFactionLookup)
            }
        }
    }

    fn faction_snapshot(&mut self, faction_id: i32) -> Option<CountryFactionSnapshot> {
        self.organizing
            .faction_by_id(faction_id)
            .map(|faction| CountryFactionSnapshot {
                faction_id: faction.faction_id(),
                name: faction.name().to_vec(),
                owned_cities: faction.owned_cities().iter().copied().collect(),
            })
    }

    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        self.globe_setup
            .country_name(country_id)
            .unwrap_or_default()
            .to_vec()
    }

    fn country_identity_name(&mut self, identity: u8) -> Vec<u8> {
        self.globe_setup
            .country_identity_name(identity)
            .unwrap_or_default()
            .to_vec()
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                CountryExileTextArgument::Text(text) => UnionFormatArgument::Text(text),
                CountryExileTextArgument::Signed(value) => UnionFormatArgument::Signed(*value),
            })
            .collect::<Vec<_>>();
        self.game.format_world_string(string_id, &arguments)
    }

    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32 {
        self.game.game_server_number_by_player_id(player_id)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }

    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery> {
        let sender = self.game.current_game_server_sender();
        self.game
            .connected_game_server_indices()
            .map(|map_id| CountryExileMessageDelivery {
                map_id,
                delivery: message.send_to_map_id(sender.as_ref(), map_id),
            })
            .collect()
    }

    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError> {
        message.send_all(self.game.current_game_server_sender().as_ref())
    }

    fn put_king_log(&mut self, text: &[u8]) {
        put_string_to_file("king", text);
    }
}

impl AttackCityWarResultContext
    for WorldAttackCityResultContext<'_, '_, '_, '_, '_, '_, '_>
{
    fn region(
        &mut self,
        region_id: i32,
    ) -> Result<Option<AttackCityWarResultRegion>, Self::Block> {
        Ok(match self.game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
            WorldRegionNameLookup::Name(name) => Some(AttackCityWarResultRegion {
                name: legacy_c_string_prefix(name).to_vec(),
            }),
        })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingCityWarResultContextBlock::MissingRegionOwner { region_id },
        )
    }

    fn faction(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<AttackCityWarResultFaction>, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).map(|faction| {
            AttackCityWarResultFaction {
                name: legacy_c_string_prefix(faction.name()).to_vec(),
            }
        }))
    }

    fn faction_exists(&mut self, faction_id: i32) -> Result<bool, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).is_some())
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingCityWarResultContextBlock::NullUnion { map_key })
            }
        }
    }

    fn refresh_owned_city_org(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<(), Self::Block> {
        let country_id = self
            .organizing
            .faction_by_id(faction_id)
            .and_then(CFaction::country);
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
        Ok(())
    }

    fn add_owned_city(&mut self, faction_id: i32, region_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_add_owned_city(
            &mut bridge,
            faction_id,
            region_id,
            self.update_player,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::OwnedCity {
            faction_id,
            operation: "AddOwnedCity",
            source,
        })?;
        if !found {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn clear_owned_city(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_clear_owned_cities(
            &mut bridge,
            faction_id,
            self.update_player,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::OwnedCity {
            faction_id,
            operation: "ClearOwnedCity",
            source,
        })?;
        if !found {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "ClearOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn faction_master_id(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        self.organizing
            .faction_by_id(faction_id)
            .ok_or(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "GetMasterID",
                },
            )?
            .master_id()
            .ok_or(OrganizingCityWarResultContextBlock::MissingFactionMaster { faction_id })
    }

    fn add_defence_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionFactionStateMutationContext::faction_add_defence_victor_count(
            &mut bridge,
            faction_id,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::VictorCount {
                faction_id,
                operation: "AddDefenceVictorCounts",
                source,
            })?;
        if found.is_none() {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddDefenceVictorCounts",
                },
            );
        }
        Ok(())
    }

    fn add_offense_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionFactionStateMutationContext::faction_add_offense_victor_count(
            &mut bridge,
            faction_id,
        )
        .map_err(|source| OrganizingCityWarResultContextBlock::VictorCount {
                faction_id,
                operation: "AddOffenseVictorCounts",
                source,
            })?;
        if found.is_none() {
            return Err(
                OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddOffenseVictorCounts",
                },
            );
        }
        Ok(())
    }

    fn stat_billboard(&mut self) -> Result<(), Self::Block> {
        self.organizing
            .stat_billboard()
            .map_err(OrganizingCityWarResultContextBlock::Billboard)
    }

    fn faction_country(&mut self, faction_id: i32) -> Result<u8, Self::Block> {
        let country = self
            .organizing
            .country_by_faction(faction_id)
            .map_err(|_| OrganizingCityWarResultContextBlock::MissingFactionCountry {
                faction_id,
            })?;
        country.ok_or(
            OrganizingCityWarResultContextBlock::MissingFactionForMutation {
                faction_id,
                operation: "GetCountry",
            },
        )
    }

    fn country_exists(&mut self, country_id: u8) -> Result<bool, Self::Block> {
        Ok(self
            .country_gate
            .as_deref()
            .expect("country gate изымается только на SetKing-переход")
            .country_exists(country_id))
    }

 /// Тяжёлый governance-шов выполняет bridge-адаптер [`WorldCountryWarGate`] у
 /// dispatcher-а: связность `take/SetKing/city_id/restore` сохраняется одним
 /// вызовом, а governance context-сеанс `SetKing` по-прежнему обслуживает сам
 /// результат-контекст (`self`). Gate изымается из адаптера на один переход,
 /// чтобы `&mut dyn` gate-а и `&mut self` прошли раздельно, и сразу
 /// возвращается — прежний полный контекст SetKing буквально тот же `self`.
    fn set_country_king_and_city(
        &mut self,
        country_id: u8,
        master_id: i32,
        city_region_id: i32,
    ) -> Result<(), Self::Block> {
        let gate = self
            .country_gate
            .take()
            .expect("country gate изымается только на SetKing-переход");
        let outcome =
            gate.set_country_king_and_city(country_id, master_id, city_region_id, &mut *self);
        self.country_gate = Some(gate);
        outcome.map_err(|block| match block {
            WorldCountryKingGateBlock::MissingCountryOwner => {
                OrganizingCityWarResultContextBlock::MissingCountryOwner { country_id }
            }
            WorldCountryKingGateBlock::Governance(source) => {
                OrganizingCityWarResultContextBlock::CountryGovernance {
                    country_id,
                    source,
                }
            }
        })
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[AttackCityWarResultFormatArgument<'_>],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                AttackCityWarResultFormatArgument::Text(text) => {
                    UnionFormatArgument::Text(legacy_c_string_prefix(text))
                }
                AttackCityWarResultFormatArgument::Signed(value) => {
                    UnionFormatArgument::Signed(*value)
                }
            })
            .collect::<Vec<_>>();
        let formatted = self.game.format_world_string(string_id, &arguments);
        let formatted = legacy_c_string_prefix(&formatted);
        Ok(formatted.to_vec())
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        parameter: i32,
        text: &[u8],
    ) -> Result<(), Self::Block> {
        let _ = self.organizing.send_top_info_to_client(
            self.game,
            top_info_id,
            timer_flag,
            parameter,
            text,
        );
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

/// Ветвь `0x60138` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает результат-контекст адаптер старого пакета с country-
/// gate от dispatcher-а и передаёт ход realm-обработчику в исходных типах.
#[allow(
    clippy::too_many_arguments,
    reason = "явные параметры сохраняют границы исходных singleton-owner-ов"
)]
pub(crate) fn dispatch_city_war_result<Callback: Copy>(
    message: &mut CMessage,
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    country_gate: &mut dyn WorldCountryWarGate,
    attack_city: &mut CAttackCitySys,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
    globe_setup: &GlobeSetupSnapshot,
) -> Option<OrganizingCityWarResultDispatch> {
    let mut context = WorldAttackCityResultContext {
        game,
        organizing,
        country_gate: Some(country_gate),
        globe_setup,
        callbacks: effects,
        update_player,
    };
    nebokrai_realm::app::organsysmessage::dispatch_city_war_result(
        message,
        attack_city,
        timer,
        attack_callbacks,
        &mut context,
    )
}

/// Соединяет concrete `CAttackCitySys::Reload` с World runtime owners; сам
/// reload-контракт перенесён в realm (`nebokrai_realm::app::organsysmessage`).
///
/// `Reload` сначала снимает прежние timer events и завершает активные войны;
/// поэтому вызывающий не имеет права превращать ошибку последующего `Initialize`
/// в транзакционный rollback. Контекст буквально совпадает с result-ingress
/// `0x60138`, а `send_all` сохраняет `0x7FE22` между end и новым Initialize.
#[allow(
    clippy::too_many_arguments,
    reason = "все singleton-владельцы и callback-грань подтверждены CAttackCitySys::Reload"
)]
pub(crate) fn reload_attack_city<Callback: Copy>(
    game: &mut CGame,
    attack_city: &mut CAttackCitySys,
    source: Option<&[u8]>,
    now: TagTime,
    timer: &mut CTimer<Callback>,
    attack_callbacks: AttackCityCallbacks<Callback>,
    today_tax_event_id: Option<TimerId>,
    organizing: &mut COrganizingCtrl,
    country_gate: &mut dyn WorldCountryWarGate,
    globe_setup: &GlobeSetupSnapshot,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Result<AttackCityReloadReport, AttackCityReloadBlock<OrganizingCityWarResultContextBlock>> {
    let sender = game.current_game_server_sender();
    let mut context = WorldAttackCityResultContext {
        game,
        organizing,
        country_gate: Some(country_gate),
        globe_setup,
        callbacks: effects,
        update_player,
    };
    nebokrai_realm::app::organsysmessage::reload_attack_city(
        attack_city,
        source,
        now,
        timer,
        attack_callbacks,
        today_tax_event_id,
        &mut context,
        |message| message.send_all(sender.as_ref()).unwrap_or(0),
    )
}

// Ветви `0x60139` goods war command и `0x6013A` goods war faction win
// перенесены в realm (`nebokrai_realm::app::organsysmessage`) вместе с их
// видовым member-контекстом (закрытие заёмного блокера волны организационного
// view); имена доступны здесь через glob re-export.

pub(crate) fn dispatch_player_quest_command(
    message: &mut CMessage,
    game: &CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingPlayerQuestCommandDispatch> {
    let (kind, response_type) = match message.message_type() {
        PLAYER_ADD_QUEST_MESSAGE_TYPE => {
            (OrganizingPlayerQuestCommandKind::Add, GAME_ADD_QUEST_MESSAGE_TYPE)
        }
        PLAYER_REMOVE_QUEST_MESSAGE_TYPE => (
            OrganizingPlayerQuestCommandKind::Remove,
            GAME_REMOVE_QUEST_MESSAGE_TYPE,
        ),
        _ => return None,
    };

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let quest_id = message.base_mut().get_short().unwrap_or(0);
    let game_server_id = game.game_server_number_by_player_id(player_id);
    let delivery = (game_server_id != 0).then(|| {
        let mut response = CMessage::new(response_type);
        response.base_mut().add_long(player_id);
        response.base_mut().add_short(quest_id);
        response.send_to_map_id(sender, game_server_id)
    });
    Some(OrganizingPlayerQuestCommandDispatch {
        kind,
        player_id,
        quest_id,
        game_server_id,
        delivery,
    })
}

pub(crate) fn dispatch_player_run_script(
    message: &mut CMessage,
    game: &CGame,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingPlayerRunScriptDispatch> {
    if message.message_type() != PLAYER_RUN_SCRIPT_MESSAGE_TYPE {
        return None;
    }

    let player_id = message.base_mut().get_long().unwrap_or(0);
    let script = message
        .base_mut()
        .get_str_bytes(PLAYER_SCRIPT_CAPACITY)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let game_server_id = game.game_server_number_by_player_id(player_id);
    let delivery = (game_server_id != 0).then(|| {
        let script = CString::new(script.as_slice())
            .expect("bounded GetStr возвращает bytes до первого NUL");
        let mut response = CMessage::new(GAME_RUN_SCRIPT_MESSAGE_TYPE);
        response.base_mut().add_long(player_id);
        response.base_mut().add_str(Some(&script));
        response.send_to_map_id(sender, game_server_id)
    });
    Some(OrganizingPlayerRunScriptDispatch {
        player_id,
        script,
        game_server_id,
        delivery,
    })
}

/// Ветвь `0x6013E` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает set-parameter effects адаптер старого пакета и
/// передаёт ход realm-обработчику в исходных типах.
pub(crate) fn dispatch_faction_parameter(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    parameters: &COrganizingParam,
    callbacks: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<Result<OrganizingFactionParameterDispatch, OrganizingFactionParameterBlock>> {
    let mut effects = WorldFactionSetParameterEffects {
        game,
        callbacks,
        update_player,
    };
    nebokrai_realm::app::organsysmessage::dispatch_faction_parameter(
        message,
        game,
        organizing,
        parameters,
        &mut effects,
    )
}

pub(crate) fn dispatch_change_region_router(
    message: &mut CMessage,
    router: &RegionRouter,
    sender: Option<&ServerCommandHandle>,
) -> Option<OrganizingChangeRegionRouterDispatch> {
    if message.message_type() != CHANGE_REGION_ROUTER_MESSAGE_TYPE {
        return None;
    }

    let request_id = message.base_mut().get_long().unwrap_or(0);
    let from_region = message.base_mut().get_long().unwrap_or(0);
    let to_region = message.base_mut().get_long().unwrap_or(0);
    let target = RegionRoutePoint {
        x: message.base_mut().get_long().unwrap_or(0),
        y: message.base_mut().get_long().unwrap_or(0),
    };
    let outcome = router.change_region_router(from_region, to_region, target);

    let route = match &outcome {
        RegionRouterChangeOutcome::RegionNotFound { .. } => &[][..],
        RegionRouterChangeOutcome::Complete(route) => route.as_slice(),
    };
    let mut response = CMessage::new(CHANGE_REGION_ROUTER_RESPONSE_TYPE);
    response.base_mut().add_long(request_id);
 // `vector::size()` попадал в 32-битный `Add` низшими битами без range gate.
    response.base_mut().add_long(route.len() as i32);
    for step in route {
        response.base_mut().add_long(step.region_id);
        response.base_mut().add_long(step.point.x);
        response.base_mut().add_long(step.point.y);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send_all(sender);
    Some(OrganizingChangeRegionRouterDispatch {
        request_id,
        from_region,
        to_region,
        target,
        outcome,
        wire,
        delivery,
    })
}

struct WorldVillageWarResultContext<'game, 'organizing, 'callbacks, 'effects, 'update> {
    game: &'game CGame,
    organizing: &'organizing mut COrganizingCtrl,
    callbacks: &'callbacks mut WorldUnionApplicationEffectCallbacks<'effects>,
    update_player: &'update mut dyn FnMut(i32),
}

impl VillageWarResultContext for WorldVillageWarResultContext<'_, '_, '_, '_, '_> {
    type Block = OrganizingVillageWarResultContextBlock;

    fn region(
        &mut self,
        region_id: i32,
    ) -> Result<Option<VillageWarResultRegion>, Self::Block> {
        Ok(match self.game.region_name(region_id) {
            WorldRegionNameLookup::RegionNotFound
            | WorldRegionNameLookup::NullRegionPointer => None,
            WorldRegionNameLookup::Name(name) => Some(VillageWarResultRegion {
                name: legacy_c_string_prefix(name).to_vec(),
            }),
        })
    }

    fn region_owner_faction_id(&mut self, region_id: i32) -> Result<i32, Self::Block> {
        self.game.region_owned_faction_id(region_id).ok_or(
            OrganizingVillageWarResultContextBlock::MissingRegionOwner { region_id },
        )
    }

    fn faction(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<VillageWarResultFaction>, Self::Block> {
        Ok(self.organizing.faction_by_id(faction_id).map(|faction| {
            VillageWarResultFaction {
                organizing_id: faction.faction_id(),
                name: legacy_c_string_prefix(faction.name()).to_vec(),
            }
        }))
    }

    fn union_for_faction(&mut self, faction_id: i32) -> Result<i32, Self::Block> {
        match self.organizing.is_free_faction(faction_id) {
            FreeFactionLookup::NoUnion => Ok(0),
            FreeFactionLookup::Union(union_id) => Ok(union_id),
            FreeFactionLookup::BlockedNullConfederation { map_key } => {
                Err(OrganizingVillageWarResultContextBlock::NullUnion { map_key })
            }
        }
    }

    fn refresh_owned_city_org(
        &mut self,
        region_id: i32,
        faction_id: i32,
        union_id: i32,
    ) -> Result<(), Self::Block> {
        let country_id = self
            .organizing
            .faction_by_id(faction_id)
            .and_then(CFaction::country);
        (self.callbacks.refresh_owned_city)(
            self.game,
            region_id,
            faction_id,
            union_id,
            country_id,
        );
        Ok(())
    }

    fn add_owned_city(&mut self, faction_id: i32, region_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_add_owned_city(
            &mut bridge,
            faction_id,
            region_id,
            self.update_player,
        )
        .map_err(|source| OrganizingVillageWarResultContextBlock::AddOwnedCity {
            faction_id,
            source,
        })?;
        if !found {
            return Err(
                OrganizingVillageWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn clear_owned_city(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionOwnedCityMutationContext::faction_clear_owned_cities(
            &mut bridge,
            faction_id,
            self.update_player,
        )
        .map_err(|source| OrganizingVillageWarResultContextBlock::ClearOwnedCity {
            faction_id,
            source,
        })?;
        if !found {
            return Err(
                OrganizingVillageWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "ClearOwnedCity",
                },
            );
        }
        Ok(())
    }

    fn add_village_war_victor_count(&mut self, faction_id: i32) -> Result<(), Self::Block> {
        let mut bridge = UnionOrganizingBridge {
            organizing: self.organizing,
            game: self.game,
        };
        let found = UnionFactionStateMutationContext::faction_add_village_war_victor_count(
            &mut bridge,
            faction_id,
        )
        .map_err(|source| OrganizingVillageWarResultContextBlock::AddVictorCount {
            faction_id,
            source,
        })?;
        if found.is_none() {
            return Err(
                OrganizingVillageWarResultContextBlock::MissingFactionForMutation {
                    faction_id,
                    operation: "AddVillageWarVictorCounts",
                },
            );
        }
        Ok(())
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[&[u8]],
    ) -> Result<Vec<u8>, Self::Block> {
        let arguments = arguments
            .iter()
            .map(|argument| UnionFormatArgument::Text(legacy_c_string_prefix(argument)))
            .collect::<Vec<_>>();
        let formatted = self.game.format_world_string(string_id, &arguments);
        let formatted = legacy_c_string_prefix(&formatted);
        Ok(formatted.to_vec())
    }

    fn send_organizing_info(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        let _ = COrganizingCtrl::send_organizing_info_to_all(
            self.game,
            text,
            0xFFFF_FE92,
            0xFFFF_0000,
        );
        Ok(())
    }

    fn write_war_log(&mut self, text: &[u8]) -> Result<(), Self::Block> {
        put_string_to_file("war", text);
        Ok(())
    }

    fn send_top_info(
        &mut self,
        top_info_id: i32,
        timer_flag: i32,
        parameter: i32,
        text: &[u8],
    ) -> Result<(), Self::Block> {
        let _ = self.organizing.send_top_info_to_client(
            self.game,
            top_info_id,
            timer_flag,
            parameter,
            text,
        );
        Ok(())
    }

    fn send_all(&mut self, message: &CMessage) -> i32 {
        message
            .send_all(self.game.current_game_server_sender().as_ref())
            .unwrap_or(0)
    }
}

/// Ветвь `0x60136` перенесена в realm (`nebokrai_realm::app::organsysmessage`);
/// обвязка лишь собирает результат-контекст адаптер старого пакета и передаёт
/// ход realm-обработчику в исходных типах.
#[allow(
    clippy::too_many_arguments,
    reason = "timer/callbacks/effects/update грань подтверждена CVillageWarSys::OnFactionWinVillage"
)]
pub(crate) fn dispatch_village_war_result<Callback: Copy>(
    message: &mut CMessage,
    game: &CGame,
    organizing: &mut COrganizingCtrl,
    village_war: &mut CVillageWarSys,
    timer: &mut CTimer<Callback>,
    callbacks: VillageWarCallbacks<Callback>,
    effects: &mut WorldUnionApplicationEffectCallbacks<'_>,
    update_player: &mut dyn FnMut(i32),
) -> Option<OrganizingVillageWarResultDispatch> {
    let mut context = WorldVillageWarResultContext {
        game,
        organizing,
        callbacks: effects,
        update_player,
    };
    nebokrai_realm::app::organsysmessage::dispatch_village_war_result(
        message,
        village_war,
        timer,
        callbacks,
        &mut context,
    )
}

// notice/response хелперы ветвей faction list, cancel application и declare
// war faction list перенесены в realm вместе с их диспетчерами.

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}
