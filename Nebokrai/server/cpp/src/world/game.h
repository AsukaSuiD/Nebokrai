#pragma once

#include "savedb.h"
#include "honorranks.h"
#include "playerranks.h"
#include "appworld/country/countryhandler.h"
#include "appworld/country/countryparam.h"
#include "appworld/country/countrywarsys.h"
#include "appworld/goods/cgoodsfactory.h"
#include "appworld/goodswarmember.h"
#include "appworld/incrementlog/incrementlog.h"
#include "appworld/jjcsystem.h"
#include "appworld/leiting.h"
#include "appworld/message/servermessage.h"
#include "appworld/organizingsystem/organizingctrl.h"
#include "appworld/organizingsystem/organizingparam.h"
#include "appworld/organizingsystem/attackcitysys.h"
#include "appworld/organizingsystem/factionwarsys.h"
#include "appworld/organizingsystem/fournationwarsys.h"
#include "appworld/organizingsystem/villagewarsys.h"
#include "appworld/player.h"
#include "appworld/script/variablelist.h"
#include "appworld/session/cplug.h"
#include "appworld/session/csession.h"
#include "appworld/session/csessionfactory.h"
#include "appworld/worldregion.h"
#include "../public/stringtable.h"
#include "../setup/contributesetup.h"
#include "../setup/emotion.h"
#include "../setup/hitlevelsetup.h"
#include "../setup/honorelimilateconfig.h"

#include <cstdint>
#include <deque>
#include <filesystem>
#include <map>
#include <memory>
#include <optional>
#include <set>
#include <string>
#include <string_view>

/*
 * Исходный владелец: WorldServer/worldserver/game.cpp/.h из
 * Nworldserver.exe + WorldServer.pdb. Player/region/game-server registries,
 * FIFO сообщений, save snapshot, login timeout и порядок AI подтверждены
 * поздней Rust-реконструкцией. Старые globals, Win32 threads и голые указатели
 * заменены owned CGame и контейнерами; конкретные сетевые/DB effects остаются у
 * CWorldServer/CSaveDB. Неподтверждённые switch-body не имитируются: их
 * семейства подключаются через WorldMessageHandlers.
 */
class CGame {
public:
    struct Setup {
        std::uint32_t number{};
        std::string name{"WorldServer"};
        std::string loginIp{"127.0.0.1"};
        std::uint32_t loginPort{2345};
        std::uint32_t listenPort{8100};
        std::string sqlConnectionType;
        std::string sqlServerIp;
        std::string sqlUserName;
        std::string sqlPassword;
        std::string databaseName;
        bool checkNetwork{};
        std::uint32_t maximumBytes{};
        std::uint32_t maximumMessageLength{};
        std::uint32_t banIpTime{};
        bool checkMessageContent{};
        std::int32_t maximumConnections{};
        std::int32_t maximumIoSends{};
        std::int32_t maximumClientSendBuffer{};
        std::uint32_t refreshIntervalMs{1000};
        std::uint32_t saveIntervalMs{60000};
        std::uint32_t releaseLoginPlayerTimeMs{};
        bool useLogSystem{};
        std::string logProvider, logServer, logDatabase, logUser, logPassword;
        std::string costProvider, costIp, costDatabase, costUser, costPassword;
        std::uint32_t loadLargessIntervalMs{};
        std::string loginCostProvider, loginCostIp, loginCostDatabase,
                    loginCostUser, loginCostPassword;
        std::uint32_t playerLoadThreads{};
        std::string languagePackage;
        bool useOldSaveLargessWay{true};
    };

    struct GameServerInfo {
        bool connected{};
        std::uint32_t index{};
        std::string ip;
        std::uint32_t port{};
    };
    struct GameServerPing {
        std::uint32_t ipv4{};
        std::int32_t mapId{};
        std::int32_t players{};
    };
    struct GlobeVariable {
        std::int32_t worldCupTeam1{};
        std::int32_t worldCupTeam2{};
        std::int32_t worldCupTeam3{};
        std::int32_t worldCupTeam4{};
    };
    struct RegionRoute {
        const CWorldRegion* region{};
        std::uint32_t gameServerIndex{};
        RegionType type{};
    };

    enum class PlayerState { Creation, Login, Online, Offline };
    struct LoopResult { std::size_t messages{}; bool aiRan{}; };

    CGame();
    explicit CGame(Setup setup);
    [[nodiscard]] static std::optional<Setup> LoadSetup(const std::filesystem::path&);
    [[nodiscard]] bool LoadStaticConfiguration(const std::filesystem::path& runtimeDirectory,
                                               std::string& error);
    [[nodiscard]] bool LoadRegions(const std::filesystem::path& runtimeDirectory,
                                   std::string& error);
    [[nodiscard]] const Setup& GetSetup() const noexcept { return m_Setup; }

    [[nodiscard]] CGoodsFactory& GoodsFactory() noexcept { return m_GoodsFactory; }
    [[nodiscard]] COrganizingCtrl& Organizations() noexcept { return m_Organizations; }
    [[nodiscard]] CCountryHandler& Countries() noexcept { return m_Countries; }
    [[nodiscard]] CCountryParam& CountryParameters() noexcept { return m_CountryParameters; }
    [[nodiscard]] COrganizingParam& OrganizingParameters() noexcept { return m_OrganizingParameters; }
    [[nodiscard]] CVariableList& Variables() noexcept { return m_Variables; }
    [[nodiscard]] CJJcSystem& Jjc() noexcept { return m_Jjc; }
    [[nodiscard]] CountryWarSys& CountryWars() noexcept { return m_CountryWars; }
    [[nodiscard]] CAttackCitySys& CityWars() noexcept { return m_CityWars; }
    [[nodiscard]] CVillageWarSys& VillageWars() noexcept { return m_VillageWars; }
    [[nodiscard]] CFourNationWarSys& FourNationWars() noexcept { return m_FourNationWars; }
    [[nodiscard]] CFactionWarSys& FactionWars() noexcept { return m_FactionWars; }
    [[nodiscard]] CIncrementLog& IncrementLog() noexcept { return m_IncrementLog; }
    [[nodiscard]] CGoodsWarMember& GoodsWarMembers() noexcept { return m_GoodsWarMembers; }
    [[nodiscard]] CHonorRanks& HonorRanks() noexcept { return m_HonorRanks; }
    [[nodiscard]] CPlayerRanks& PlayerRanks() noexcept { return m_PlayerRanks; }
    [[nodiscard]] std::vector<GodsBattleRegionDbRow>& GodsBattleRegions() noexcept { return m_GodsBattleRegions; }
    [[nodiscard]] std::vector<GodsBattleNpcDbRow>& GodsBattleNpcs() noexcept { return m_GodsBattleNpcs; }
    [[nodiscard]] std::set<std::int32_t>& AuctionOwners() noexcept { return m_AuctionOwners; }
    [[nodiscard]] WorldMessageHandlers& MessageHandlers() noexcept { return m_MessageHandlers; }
    [[nodiscard]] CSessionFactory& Sessions() noexcept { return m_Sessions; }
    [[nodiscard]] const StringTable& Strings() const noexcept { return m_Strings; }
    [[nodiscard]] const GlobeVariable& GlobeVariables() const noexcept { return m_GlobeVariables; }
    [[nodiscard]] const CHitLevelSetup& HitLevels() const noexcept { return m_HitLevels; }
    [[nodiscard]] const CEmotion& Emotions() const noexcept { return m_Emotions; }
    [[nodiscard]] const CContributeSetup& ContributeSetup() const noexcept { return m_ContributeSetup; }
    [[nodiscard]] const HonorElimilateConfig& HonorEliminateSetup() const noexcept { return m_HonorEliminateSetup; }

    bool AddRegion(std::unique_ptr<CWorldRegion>, std::uint32_t gameServerIndex,
                   RegionType type);
    void RestoreRegionState(RegionDbSnapshot snapshot);
    [[nodiscard]] CWorldRegion* GetRegion(std::int32_t regionId) noexcept;
    [[nodiscard]] const CWorldRegion* GetRegion(std::int32_t regionId) const noexcept;
    [[nodiscard]] std::optional<std::uint32_t> GetRegionGameServer(std::int32_t regionId) const;
    [[nodiscard]] std::vector<RegionRoute> RegionRoutes() const;

    bool AddPlayer(std::unique_ptr<CPlayer>, PlayerState);
    bool SetPlayerState(std::int32_t playerId, PlayerState, std::uint32_t nowMs = 0);
    [[nodiscard]] CPlayer* GetPlayer(std::int32_t playerId) noexcept;
    [[nodiscard]] CPlayer* GetOnlinePlayer(std::int32_t playerId) noexcept;
    [[nodiscard]] CPlayer* FindPlayer(std::string_view name, PlayerState) noexcept;
    [[nodiscard]] std::size_t PlayerCount(PlayerState) const noexcept;
    [[nodiscard]] std::optional<std::vector<std::string>> OnlineAccounts() const;
    [[nodiscard]] std::unique_ptr<CPlayer> RemovePlayer(std::int32_t playerId);
    std::size_t ClearOfflinePlayers();
    std::size_t ProcessLoginTimeouts(std::uint32_t nowMs);

    bool RegisterGameServer(GameServerInfo);
    bool SetGameServerConnected(std::uint32_t index, bool connected);
    [[nodiscard]] GameServerInfo* FindGameServer(std::string_view host,
                                                 std::uint32_t port) noexcept;
    [[nodiscard]] GameServerInfo* GetGameServer(std::uint32_t index) noexcept;
    bool BindPlayerGameServer(std::int32_t playerId, std::uint32_t index);
    [[nodiscard]] const GameServerInfo* GetPlayerGameServer(std::int32_t playerId) const noexcept;
    [[nodiscard]] std::size_t ConnectedGameServerCount() const noexcept;
    void BeginGameServerPing(std::uint32_t nowMs);
    void RecordGameServerPing(GameServerPing ping);
    [[nodiscard]] const std::vector<GameServerPing>& GameServerPings() const noexcept { return m_GameServerPings; }
    void SetLoginServerId(std::int32_t value) noexcept { m_LoginServerId = value; }
    [[nodiscard]] std::int32_t LoginServerId() const noexcept { return m_LoginServerId; }

    void QueueMessage(std::unique_ptr<WorldNet::CMessage>);
    [[nodiscard]] bool ProcessMessage();
    [[nodiscard]] LoopResult MainLoopTurn(std::uint32_t nowMs,
                                          std::size_t maximumMessages = 256);

    void StagePlayerSave(WorldPlayerSave, bool creation = false);
    void StagePlayerRestore(std::int32_t playerId);
    void StagePlayerDeletion(WorldDeletedPlayer);
    void SetNextIds(std::int32_t playerId, std::int32_t leaveWordId) noexcept;
    void RestoreNextIds(std::int32_t playerId, std::int32_t leaveWordId) noexcept;
    [[nodiscard]] std::int32_t NextPlayerId() const noexcept { return m_NextPlayerId; }
    [[nodiscard]] std::int32_t NextLeaveWordId() const noexcept { return m_NextLeaveWordId; }
    [[nodiscard]] WorldSaveBatch GenerateDBData(bool forceOrganizations = false);

private:
    struct RegionOwner { std::unique_ptr<CWorldRegion> region; std::uint32_t gameServerIndex{}; RegionType type{}; };
    [[nodiscard]] std::set<std::int32_t>& StateSet(PlayerState) noexcept;
    [[nodiscard]] const std::set<std::int32_t>& StateSet(PlayerState) const noexcept;
    static std::string FoldName(std::string_view);

    Setup m_Setup;
    CGoodsFactory m_GoodsFactory;
    std::map<std::int32_t, RegionOwner> m_Regions;
    std::map<std::int32_t, RegionDbSnapshot> m_RestoredRegionState;
    std::map<std::int32_t, std::unique_ptr<CPlayer>> m_Players;
    std::set<std::int32_t> m_CreationPlayers, m_LoginPlayers, m_OnlinePlayers, m_OfflinePlayers;
    std::map<std::int32_t, std::uint32_t> m_LoginStartedAt;
    std::map<std::uint32_t, GameServerInfo> m_GameServers;
    std::map<std::int32_t, std::uint32_t> m_PlayerGameServers;
    std::vector<GameServerPing> m_GameServerPings;
    std::uint32_t m_GameServerPingStartedAt{};
    std::int32_t m_LoginServerId{};
    GlobeVariable m_GlobeVariables;
    std::deque<std::unique_ptr<WorldNet::CMessage>> m_Messages;
    CSessionFactory m_Sessions;
    COrganizingCtrl m_Organizations;
    COrganizingParam m_OrganizingParameters;
    CCountryHandler m_Countries;
    CCountryParam m_CountryParameters;
    StringTable m_Strings;
    CHitLevelSetup m_HitLevels;
    CEmotion m_Emotions;
    CContributeSetup m_ContributeSetup;
    HonorElimilateConfig m_HonorEliminateSetup;
    CVariableList m_Variables;
    CJJcSystem m_Jjc;
    CountryWarSys m_CountryWars;
    CAttackCitySys m_CityWars;
    CVillageWarSys m_VillageWars;
    CFourNationWarSys m_FourNationWars;
    CFactionWarSys m_FactionWars;
    CIncrementLog m_IncrementLog;
    CGoodsWarMember m_GoodsWarMembers;
    CLeiTing m_LeiTing;
    CHonorRanks m_HonorRanks;
    CPlayerRanks m_PlayerRanks;
    std::vector<GodsBattleRegionDbRow> m_GodsBattleRegions;
    std::vector<GodsBattleNpcDbRow> m_GodsBattleNpcs;
    std::set<std::int32_t> m_AuctionOwners;
    WorldMessageHandlers m_MessageHandlers;
    WorldSaveBatch m_StagedSave;
    std::int32_t m_NextPlayerId{};
    std::int32_t m_NextLeaveWordId{};
    std::uint32_t m_LastAiTick{};
    std::uint32_t m_LastMinuteTick{};
    std::uint32_t m_LastLoopTick{};
};

static_assert(sizeof(CGame::GlobeVariable) == 0x10);
