#include "game.h"

#include "../public/tools.h"
#include "appworld/worldcityregion.h"
#include "appworld/worldcountrywarregion.h"
#include "appworld/worldvillageregion.h"
#include "appworld/worldwarregion.h"

#include <algorithm>
#include <cctype>
#include <fstream>
#include <limits>
#include <sstream>
#include <string_view>
#include <utility>

namespace
{
template<class Value>
void ReadLabeled(std::istream& input, std::string& label, Value& value)
{
    if (input >> label) input >> value;
}

bool ReadSetup(std::istream& input, CGame::Setup& s)
{
    std::string label;
    ReadLabeled(input, label, s.number); ReadLabeled(input, label, s.name);
    ReadLabeled(input, label, s.loginIp); ReadLabeled(input, label, s.loginPort);
    ReadLabeled(input, label, s.listenPort); ReadLabeled(input, label, s.sqlConnectionType);
    ReadLabeled(input, label, s.sqlServerIp); ReadLabeled(input, label, s.sqlUserName);
    ReadLabeled(input, label, s.sqlPassword); ReadLabeled(input, label, s.databaseName);
    ReadLabeled(input, label, s.checkNetwork); ReadLabeled(input, label, s.maximumBytes);
    ReadLabeled(input, label, s.maximumMessageLength); ReadLabeled(input, label, s.banIpTime);
    ReadLabeled(input, label, s.checkMessageContent); ReadLabeled(input, label, s.maximumConnections);
    ReadLabeled(input, label, s.maximumIoSends); ReadLabeled(input, label, s.maximumClientSendBuffer);
    ReadLabeled(input, label, s.refreshIntervalMs); ReadLabeled(input, label, s.saveIntervalMs);
    ReadLabeled(input, label, s.releaseLoginPlayerTimeMs); ReadLabeled(input, label, s.useLogSystem);
    ReadLabeled(input, label, s.logProvider); ReadLabeled(input, label, s.logServer);
    ReadLabeled(input, label, s.logDatabase); ReadLabeled(input, label, s.logUser);
    ReadLabeled(input, label, s.logPassword); ReadLabeled(input, label, s.costProvider);
    ReadLabeled(input, label, s.costIp); ReadLabeled(input, label, s.costDatabase);
    ReadLabeled(input, label, s.costUser); ReadLabeled(input, label, s.costPassword);
    ReadLabeled(input, label, s.loadLargessIntervalMs); ReadLabeled(input, label, s.loginCostProvider);
    ReadLabeled(input, label, s.loginCostIp); ReadLabeled(input, label, s.loginCostDatabase);
    ReadLabeled(input, label, s.loginCostUser); ReadLabeled(input, label, s.loginCostPassword);
    ReadLabeled(input, label, s.playerLoadThreads); ReadLabeled(input, label, s.languagePackage);
    ReadLabeled(input, label, s.useOldSaveLargessWay);
    return !input.bad() && s.listenPort <= 65535 && s.loginPort <= 65535;
}

std::optional<std::string> ReadTextFile(const std::filesystem::path& path)
{
    std::ifstream input(path, std::ios::binary);
    if (!input) return std::nullopt;
    return std::string(std::istreambuf_iterator<char>(input), {});
}

std::string Trim(std::string value)
{
    const auto first = value.find_first_not_of(" \t\r\n");
    if (first == std::string::npos) return {};
    const auto last = value.find_last_not_of(" \t\r\n");
    return value.substr(first, last - first + 1);
}
}

CGame::CGame() : CGame(Setup{}) {}

CGame::CGame(Setup setup) : m_Setup(std::move(setup))
{
    m_MessageHandlers.EnableWriteLog(m_Setup.useLogSystem);
}

std::optional<CGame::Setup> CGame::LoadSetup(const std::filesystem::path& path)
{
    std::ifstream source(path, std::ios::binary);
    std::string decoded;
    if (!source) {
        std::filesystem::path encoded = path;
        encoded.replace_extension(".dat");
        std::ifstream data(encoded, std::ios::binary);
        if (!data) return std::nullopt;
        const std::string bytes((std::istreambuf_iterator<char>(data)), {});
        decoded.resize(bytes.size() * 2U + 2U);
        std::string mutableBytes = bytes;
        IniDecoder(mutableBytes.data(), decoded.data(), static_cast<int>(mutableBytes.size()));
    } else {
        decoded.assign(std::istreambuf_iterator<char>(source), {});
    }
    std::istringstream input(decoded);
    Setup setup;
    if (!ReadSetup(input, setup)) return std::nullopt;
    return setup;
}

bool CGame::LoadStaticConfiguration(const std::filesystem::path& runtimeDirectory,
                                    std::string& error)
{
    m_Strings.Clear();
    if (!m_Strings.Load(runtimeDirectory / "data" / "Language.lag", error)) return false;
    if (!m_Setup.languagePackage.empty()) {
        std::string package = m_Setup.languagePackage;
        std::ranges::replace(package, '\\', '/');
        if (!m_Strings.Load(runtimeDirectory / package, error)) return false;
    }

    std::ifstream servers(runtimeDirectory / "serversetup.ini");
    if (!servers) {
        servers.open(runtimeDirectory / "serverSetup.ini");
    }
    if (!servers) {
        error = "не найден serversetup.ini";
        return false;
    }
    std::string token;
    std::int32_t declaredServers{};
    if (!(servers >> token >> declaredServers) || declaredServers < 0) {
        error = "повреждён заголовок serversetup.ini";
        return false;
    }
    std::int32_t loadedServers{};
    while (servers >> token) {
        if (token != "#") continue;
        GameServerInfo server;
        if (!(servers >> server.index >> server.ip >> server.port) || server.index == 0 ||
            server.index > std::numeric_limits<std::uint8_t>::max() ||
            server.port > 65535 || !RegisterGameServer(std::move(server))) {
            error = "повреждена или повторяется запись serversetup.ini";
            return false;
        }
        ++loadedServers;
    }
    if (loadedServers != declaredServers) {
        error = "число GameServer в serversetup.ini не совпадает с заголовком";
        return false;
    }

    const auto country = ReadTextFile(runtimeDirectory / "data" / "countryparam.ini");
    if (!country || !m_CountryParameters.Load(*country)) {
        error = "не удалось прочитать data/countryparam.ini";
        return false;
    }
    const auto organizing = ReadTextFile(runtimeDirectory / "data" / "factionparam.ini");
    if (!organizing || !m_OrganizingParameters.Load(*organizing)) {
        error = "не удалось прочитать data/factionparam.ini";
        return false;
    }

    std::ifstream factionWars(runtimeDirectory / "data" / "factionwarsys.ini");
    if (!factionWars) {
        error = "не найден data/factionwarsys.ini";
        return false;
    }
    std::string line;
    while (std::getline(factionWars, line)) {
        std::istringstream row(line);
        char marker{};
        FactionWarType type;
        std::int32_t minutes{};
        if (!(row >> marker) || marker != '#') continue;
        if (!(row >> type.type >> minutes >> type.money) || type.type <= 0 || minutes <= 0 ||
            minutes > std::numeric_limits<std::int32_t>::max() / 60000) {
            error = "повреждена запись data/factionwarsys.ini";
            return false;
        }
        type.fightTimeMilliseconds = minutes * 60000;
        m_FactionWars.AddWarType(type);
    }

    std::ifstream jjcRegions(runtimeDirectory / "data" / "JJCRegionlist.ini");
    if (!jjcRegions) {
        error = "не найден data/JJCRegionlist.ini";
        return false;
    }
    while (std::getline(jjcRegions, line)) {
        std::istringstream row(line);
        char marker{};
        std::int32_t region{};
        if ((row >> marker >> region) && marker == '#') m_Jjc.ConfigureRegion(region);
    }
    std::ifstream jjcLevels(runtimeDirectory / "data" / "JJcLevel.ini");
    if (!jjcLevels) {
        error = "не найден data/JJcLevel.ini";
        return false;
    }
    while (std::getline(jjcLevels, line)) {
        std::istringstream row(line);
        char marker{};
        std::int32_t step{}, minimum{}, maximum{};
        if ((row >> marker >> step >> minimum >> maximum) && marker == '#')
            m_Jjc.ConfigureLevelStep(minimum, maximum, step);
    }
    std::ifstream jjcConfig(runtimeDirectory / "setup" / "JJcConfig.ini");
    if (!jjcConfig) {
        error = "не найден setup/JJcConfig.ini";
        return false;
    }
    std::map<std::string, std::int32_t, std::less<>> values;
    while (std::getline(jjcConfig, line)) {
        const auto delimiter = line.find('=');
        if (delimiter == std::string::npos) continue;
        const std::string name = Trim(line.substr(0, delimiter));
        const std::string raw = Trim(line.substr(delimiter + 1));
        std::int32_t value{};
        std::istringstream(raw) >> value;
        values.insert_or_assign(name, value);
    }
    m_Jjc.ConfigureWeeklyReset(values["Day"], values["Hour"], values["Min"], values["Sec"]);
    return true;
}

bool CGame::LoadRegions(const std::filesystem::path& runtimeDirectory, std::string& error)
{
    std::ifstream list(runtimeDirectory / "setup" / "regionlist.ini");
    if (!list) { error = "не найден setup/regionlist.ini"; return false; }

    std::size_t loaded{};
    std::string line;
    while (std::getline(list, line)) {
        std::istringstream input(line);
        char marker{};
        if (!(input >> marker) || marker != '#') continue;
        std::int32_t id{}, typeValue{}, noPk{}, noContribute{}, country{}, notify{};
        std::uint32_t resourceId{}, gameServerIndex{};
        float expScale{};
        std::string name;
        if (!(input >> id >> resourceId >> expScale >> typeValue >> noPk >> noContribute >>
              name >> gameServerIndex >> country >> notify)) {
            error = "повреждена строка setup/regionlist.ini";
            return false;
        }
        if (typeValue < static_cast<std::int32_t>(RegionType::Normal) ||
            typeValue > static_cast<std::int32_t>(RegionType::GodsBattle) ||
            country < 0 || country > 255) {
            error = "недопустимый тип или страна региона " + std::to_string(id);
            return false;
        }
        const auto type = static_cast<RegionType>(typeValue);
        std::unique_ptr<CWorldRegion> region;
        switch (type) {
        case RegionType::Normal: region = std::make_unique<CWorldRegion>(); break;
        case RegionType::Village: region = std::make_unique<CWorldVillageRegion>(); break;
        case RegionType::City: region = std::make_unique<CWorldCityRegion>(); break;
        case RegionType::Country: region = std::make_unique<WorldCountryWarRegion>(); break;
        case RegionType::Nation:
        case RegionType::GodsBattle: region = std::make_unique<CWorldWarRegion>(); break;
        }
        region->SetID(id);
        region->SetName(name);
        region->SetResourceID(static_cast<std::int32_t>(resourceId));
        region->SetExpScale(expScale);
        region->SetWarRegionType(type);
        region->SetNoPk(noPk != 0);
        region->SetNoContribute(noContribute != 0);
        region->SetCountry(static_cast<std::uint8_t>(country));
        region->SetNotify(notify);

        const auto regionPath = runtimeDirectory / "regions" / (std::to_string(id) + ".rgn");
        std::ifstream binary(regionPath, std::ios::binary);
        if (!binary) { error = "не найден ресурс региона " + regionPath.string(); return false; }
        const std::vector<std::uint8_t> bytes((std::istreambuf_iterator<char>(binary)), {});
        if (!region->LoadResource(regionPath.generic_string(), bytes)) {
            error = "повреждён ресурс региона " + regionPath.string(); return false;
        }
        const auto regionFile = [&](const std::initializer_list<std::string_view> extensions)
            -> std::optional<std::string> {
            for (const auto extension : extensions) {
                const auto path = runtimeDirectory / "regions" /
                    (std::to_string(id) + std::string(extension));
                if (auto contents = ReadTextFile(path)) return contents;
            }
            return std::nullopt;
        };
        if (const auto monsters = regionFile({".Monster", ".monster"}); monsters &&
            !region->LoadMonsterList(*monsters)) {
            error = "повреждён список монстров региона " + std::to_string(id); return false;
        }
        if (const auto npcs = regionFile({".npc", ".NPC"}); npcs &&
            !region->LoadNpcList(*npcs, [this](const std::string_view stringId) {
                return m_Strings.Resolve(stringId);
            })) {
            error = "повреждён список NPC региона " + std::to_string(id); return false;
        }
        const auto weather = regionFile({".weather", ".Weather"});
        if (!region->LoadWeatherSetup(weather ? std::optional<std::string_view>(*weather)
                                               : std::nullopt)) {
            error = "повреждены настройки погоды региона " + std::to_string(id); return false;
        }
        const auto setupPath = runtimeDirectory / "regions" / (std::to_string(id) + ".rs");
        if (std::ifstream setup(setupPath); setup) {
            const std::string text((std::istreambuf_iterator<char>(setup)), {});
            if (!region->LoadSetup(text)) {
                error = "повреждены настройки региона " + setupPath.string(); return false;
            }
        }
        const auto tax = regionFile({".Tax", ".tax"});
        if (!region->LoadTaxParam(tax ? std::optional<std::string_view>(*tax) : std::nullopt)) {
            error = "повреждены налоговые параметры региона " + std::to_string(id); return false;
        }
        if (auto* war = dynamic_cast<CWorldWarRegion*>(region.get())) {
            if (const auto warSetup = regionFile({".war", ".War"}); warSetup &&
                !war->LoadWarSetup(*warSetup)) {
                error = "повреждены параметры войны региона " + std::to_string(id); return false;
            }
        }
        if (auto* countryWar = dynamic_cast<WorldCountryWarRegion*>(region.get())) {
            const auto countrySetup = regionFile({".country", ".Country"});
            if (!countrySetup || !countryWar->LoadCountrySetup(*countrySetup)) {
                error = "повреждены параметры войны стран региона " + std::to_string(id);
                return false;
            }
        }
        if (auto* city = dynamic_cast<CWorldCityRegion*>(region.get())) {
            const auto citySetup = regionFile({".city", ".City"});
            if (!citySetup || !city->LoadCitySetup(*citySetup, [this](const std::string_view id) {
                    return m_Strings.Resolve(id);
                })) {
                error = "повреждены параметры города региона " + std::to_string(id);
                return false;
            }
        }
        if (!AddRegion(std::move(region), gameServerIndex, type)) {
            error = "повторяющийся регион " + std::to_string(id); return false;
        }
        ++loaded;
    }
    if (loaded == 0) { error = "setup/regionlist.ini не содержит активных регионов"; return false; }
    return true;
}

bool CGame::AddRegion(std::unique_ptr<CWorldRegion> region,
                      const std::uint32_t gameServerIndex,
                      const RegionType type)
{
    if (!region || region->GetID() == 0) return false;
    const auto id = region->GetID();
    if (const auto restored = m_RestoredRegionState.find(id);
        restored != m_RestoredRegionState.end()) {
        const auto& state = restored->second;
        region->SetParamFromDB(state.factionId, state.unionId, state.taxRate,
                               state.todayTax, state.totalTax);
        m_RestoredRegionState.erase(restored);
    }
    return m_Regions.emplace(id, RegionOwner{std::move(region), gameServerIndex, type}).second;
}

void CGame::RestoreRegionState(RegionDbSnapshot snapshot)
{
    if (CWorldRegion* region = GetRegion(snapshot.regionId)) {
        region->SetParamFromDB(snapshot.factionId, snapshot.unionId, snapshot.taxRate,
                               snapshot.todayTax, snapshot.totalTax);
        return;
    }
    m_RestoredRegionState.insert_or_assign(snapshot.regionId, std::move(snapshot));
}

CWorldRegion* CGame::GetRegion(const std::int32_t id) noexcept
{
    const auto found = m_Regions.find(id);
    return found == m_Regions.end() ? nullptr : found->second.region.get();
}

const CWorldRegion* CGame::GetRegion(const std::int32_t id) const noexcept
{
    const auto found = m_Regions.find(id);
    return found == m_Regions.end() ? nullptr : found->second.region.get();
}

std::optional<std::uint32_t> CGame::GetRegionGameServer(const std::int32_t id) const
{
    const auto found = m_Regions.find(id);
    if (found == m_Regions.end()) return std::nullopt;
    return found->second.gameServerIndex;
}

std::vector<CGame::RegionRoute> CGame::RegionRoutes() const
{
    std::vector<RegionRoute> result;
    result.reserve(m_Regions.size());
    for (const auto& [id, owner] : m_Regions) {
        (void)id;
        if (owner.region) result.push_back({owner.region.get(), owner.gameServerIndex, owner.type});
    }
    return result;
}

std::set<std::int32_t>& CGame::StateSet(const PlayerState state) noexcept
{
    switch (state) {
    case PlayerState::Creation: return m_CreationPlayers;
    case PlayerState::Login: return m_LoginPlayers;
    case PlayerState::Online: return m_OnlinePlayers;
    case PlayerState::Offline: return m_OfflinePlayers;
    }
    return m_OfflinePlayers;
}

const std::set<std::int32_t>& CGame::StateSet(const PlayerState state) const noexcept
{
    return const_cast<CGame*>(this)->StateSet(state);
}

bool CGame::AddPlayer(std::unique_ptr<CPlayer> player, const PlayerState state)
{
    if (!player || player->GetID() == 0 || m_Players.contains(player->GetID())) return false;
    const auto id = player->GetID();
    m_Players.emplace(id, std::move(player));
    StateSet(state).insert(id);
    return true;
}

bool CGame::SetPlayerState(const std::int32_t id, const PlayerState state, const std::uint32_t nowMs)
{
    if (!m_Players.contains(id)) return false;
    m_CreationPlayers.erase(id); m_LoginPlayers.erase(id);
    m_OnlinePlayers.erase(id); m_OfflinePlayers.erase(id);
    StateSet(state).insert(id);
    if (state == PlayerState::Login) m_LoginStartedAt[id] = nowMs;
    else m_LoginStartedAt.erase(id);
    return true;
}

CPlayer* CGame::GetPlayer(const std::int32_t id) noexcept
{
    const auto found = m_Players.find(id);
    return found == m_Players.end() ? nullptr : found->second.get();
}

CPlayer* CGame::GetOnlinePlayer(const std::int32_t id) noexcept
{
    return m_OnlinePlayers.contains(id) ? GetPlayer(id) : nullptr;
}

std::string CGame::FoldName(const std::string_view name)
{
    std::string folded(name);
    std::transform(folded.begin(), folded.end(), folded.begin(), [](const unsigned char ch) {
        return static_cast<char>(std::tolower(ch));
    });
    return folded;
}

CPlayer* CGame::FindPlayer(const std::string_view name, const PlayerState state) noexcept
{
    const std::string expected = FoldName(name);
    for (const auto id : StateSet(state)) {
        CPlayer* player = GetPlayer(id);
        if (player && FoldName(player->GetName()) == expected) return player;
    }
    return nullptr;
}

std::size_t CGame::PlayerCount(const PlayerState state) const noexcept { return StateSet(state).size(); }

std::optional<std::vector<std::string>> CGame::OnlineAccounts() const
{
    std::vector<std::string> accounts;
    accounts.reserve(m_OnlinePlayers.size());
    for (const auto id : m_OnlinePlayers) {
        const auto player = m_Players.find(id);
        if (player == m_Players.end() || player->second->GetAccount().empty()) return std::nullopt;
        accounts.emplace_back(player->second->GetAccount());
    }
    return accounts;
}

std::unique_ptr<CPlayer> CGame::RemovePlayer(const std::int32_t id)
{
    auto found = m_Players.find(id);
    if (found == m_Players.end()) return {};
    m_CreationPlayers.erase(id); m_LoginPlayers.erase(id); m_OnlinePlayers.erase(id);
    m_OfflinePlayers.erase(id); m_LoginStartedAt.erase(id); m_PlayerGameServers.erase(id);
    auto player = std::move(found->second);
    m_Players.erase(found);
    return player;
}

std::size_t CGame::ClearOfflinePlayers()
{
    const auto ids = m_OfflinePlayers;
    for (const auto id : ids) static_cast<void>(RemovePlayer(id));
    return ids.size();
}

std::size_t CGame::ProcessLoginTimeouts(const std::uint32_t nowMs)
{
    if (m_Setup.releaseLoginPlayerTimeMs == 0) return 0;
    std::vector<std::int32_t> expired;
    for (const auto [id, started] : m_LoginStartedAt) {
        if (nowMs - started >= m_Setup.releaseLoginPlayerTimeMs) expired.push_back(id);
    }
    for (const auto id : expired) static_cast<void>(RemovePlayer(id));
    return expired.size();
}

bool CGame::RegisterGameServer(GameServerInfo server)
{
    if (server.index == 0) return false;
    m_GameServers.insert_or_assign(server.index, std::move(server));
    return true;
}

bool CGame::SetGameServerConnected(const std::uint32_t index, const bool connected)
{
    const auto found = m_GameServers.find(index);
    if (found == m_GameServers.end()) return false;
    found->second.connected = connected;
    if (!connected) std::erase_if(m_PlayerGameServers, [index](const auto& item) { return item.second == index; });
    return true;
}

CGame::GameServerInfo* CGame::FindGameServer(const std::string_view host,
                                             const std::uint32_t port) noexcept
{
    for (auto& [index, server] : m_GameServers) {
        (void)index;
        if (server.port == port && server.ip == host) return &server;
    }
    return nullptr;
}

CGame::GameServerInfo* CGame::GetGameServer(const std::uint32_t index) noexcept
{
    const auto found = m_GameServers.find(index);
    return found == m_GameServers.end() ? nullptr : &found->second;
}

bool CGame::BindPlayerGameServer(const std::int32_t playerId, const std::uint32_t index)
{
    const auto server = m_GameServers.find(index);
    if (!m_Players.contains(playerId) || server == m_GameServers.end() || !server->second.connected) return false;
    m_PlayerGameServers[playerId] = index;
    return true;
}

const CGame::GameServerInfo* CGame::GetPlayerGameServer(const std::int32_t playerId) const noexcept
{
    const auto binding = m_PlayerGameServers.find(playerId);
    if (binding == m_PlayerGameServers.end()) return nullptr;
    const auto server = m_GameServers.find(binding->second);
    return server == m_GameServers.end() ? nullptr : &server->second;
}

std::size_t CGame::ConnectedGameServerCount() const noexcept
{
    return std::ranges::count_if(m_GameServers, [](const auto& item) { return item.second.connected; });
}

void CGame::BeginGameServerPing(const std::uint32_t nowMs)
{
    m_GameServerPings.clear();
    m_GameServerPingStartedAt = nowMs;
}

void CGame::RecordGameServerPing(GameServerPing ping)
{
    m_GameServerPings.push_back(std::move(ping));
}

void CGame::QueueMessage(std::unique_ptr<WorldNet::CMessage> message)
{
    if (message) m_Messages.push_back(std::move(message));
}

bool CGame::ProcessMessage()
{
    if (m_Messages.empty()) return false;
    auto message = std::move(m_Messages.front());
    m_Messages.pop_front();
    static_cast<void>(message->Run(m_MessageHandlers));
    return true;
}

CGame::LoopResult CGame::MainLoopTurn(const std::uint32_t nowMs, const std::size_t maximumMessages)
{
    LoopResult result;
    const std::uint32_t elapsed = m_LastLoopTick == 0 ? 0U : nowMs - m_LastLoopTick;
    m_LastLoopTick = nowMs;
    while (result.messages < maximumMessages && ProcessMessage()) ++result.messages;
    static_cast<void>(ProcessLoginTimeouts(nowMs));
    for (const EnemyFactionRelation& expired : m_FactionWars.Run(elapsed)) {
        if (CFaction* faction = m_Organizations.GetFaction(expired.firstFaction))
            faction->RemoveEnemy(expired.secondFaction);
        if (CFaction* faction = m_Organizations.GetFaction(expired.secondFaction))
            faction->RemoveEnemy(expired.firstFaction);
    }
    if (nowMs - m_LastAiTick >= m_Setup.refreshIntervalMs) {
        const std::uint32_t elapsedMinutes = m_LastMinuteTick == 0
            ? 0U : (nowMs - m_LastMinuteTick) / 60000U;
        if (elapsedMinutes != 0) m_LastMinuteTick += elapsedMinutes * 60000U;
        else if (m_LastMinuteTick == 0) m_LastMinuteTick = nowMs;
        m_LastAiTick = nowMs;
        m_Sessions.AI();
        m_Organizations.Run(static_cast<std::int32_t>(elapsedMinutes), [this](const std::int32_t, const std::int32_t factionId) {
            static_cast<void>(m_Organizations.DeleteFaction(factionId));
        });
        m_Countries.Run(static_cast<std::int32_t>(elapsedMinutes), {});
        for (auto& [id, owner] : m_Regions) { (void)id; owner.region->AI(); }
        result.aiRan = true;
    }
    return result;
}

void CGame::StagePlayerSave(WorldPlayerSave player, const bool creation)
{
    (creation ? m_StagedSave.newPlayers : m_StagedSave.players).push_back(std::move(player));
}

void CGame::StagePlayerRestore(const std::int32_t playerId)
{
    m_StagedSave.restoredPlayers.push_back(playerId);
}

void CGame::StagePlayerDeletion(WorldDeletedPlayer player)
{
    m_StagedSave.deletedPlayers.push_back(std::move(player));
}

void CGame::SetNextIds(const std::int32_t playerId, const std::int32_t leaveWordId) noexcept
{
    m_NextPlayerId = playerId;
    m_NextLeaveWordId = leaveWordId;
    m_StagedSave.nextPlayerId = playerId;
    m_StagedSave.nextLeaveWordId = leaveWordId;
}

void CGame::RestoreNextIds(const std::int32_t playerId, const std::int32_t leaveWordId) noexcept
{
    m_NextPlayerId = playerId;
    m_NextLeaveWordId = leaveWordId;
}

WorldSaveBatch CGame::GenerateDBData(const bool forceOrganizations)
{
    WorldSaveBatch batch = std::move(m_StagedSave);
    m_StagedSave = {};
    for (const VariableSaveRow& row : m_Variables.SaveRows())
        batch.variables.push_back({row.name, row.initialValue, row.currentValue});
    OrganizingSaveBatch organizations = m_Organizations.GenerateSaveData(forceOrganizations);
    batch.factions.insert(batch.factions.end(),
                          std::make_move_iterator(organizations.factions.begin()),
                          std::make_move_iterator(organizations.factions.end()));
    batch.unions.insert(batch.unions.end(),
                        std::make_move_iterator(organizations.unions.begin()),
                        std::make_move_iterator(organizations.unions.end()));
    batch.deletedFactions.insert(batch.deletedFactions.end(), organizations.deletedFactions.begin(),
                                 organizations.deletedFactions.end());
    batch.deletedUnions.insert(batch.deletedUnions.end(), organizations.deletedUnions.begin(),
                               organizations.deletedUnions.end());
    const auto control = m_CountryParameters.MaxKingControlPoint();
    const auto material = m_CountryParameters.MaxKingMaterialPoint();
    const auto war = m_CountryParameters.MaxKingWarPoint();
    batch.countries = m_Countries.GenerateSaveData({control.value_or(std::numeric_limits<std::int32_t>::max()),
                                                    material.value_or(std::numeric_limits<std::int32_t>::max()),
                                                    war.value_or(std::numeric_limits<std::int32_t>::max())});
    for (const auto& [id, owner] : m_Regions) {
        const RegionParam param = owner.region->GenerateSaveData();
        batch.regions.push_back({id, param.ownedFactionId, param.ownedUnionId,
                                 param.currentTaxRate, static_cast<std::int32_t>(param.todayTotalTax),
                                 static_cast<std::int32_t>(param.totalTax)});
    }
    return batch;
}
