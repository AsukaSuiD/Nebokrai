#include "worldserver.h"

#include "../dbaccess/odbc.h"
#include "../dbaccess/worlddb/dbcountry.h"
#include "../dbaccess/worlddb/dbincrementlog.h"
#include "../dbaccess/worlddb/dbmisc.h"
#include "../dbaccess/worlddb/rsenemyfactions.h"
#include "../dbaccess/worlddb/rsfaction.h"
#include "../dbaccess/worlddb/rsgenvar.h"
#include "../dbaccess/worlddb/rsgodsbattle.h"
#include "../dbaccess/worlddb/rsregion.h"
#include "../dbaccess/worlddb/rsunion.h"
#include "appworld/session/cteam.h"
#include "appworld/session/cteamate.h"

#include <spdlog/spdlog.h>

#include <algorithm>
#include <chrono>
#include <cctype>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <limits>
#include <optional>
#include <string_view>
#include <utility>

namespace
{
const WorldDbValue* FindValue(const WorldDbRow& row, const std::string_view name)
{
    const auto equal = [](const std::string_view left, const std::string_view right) {
        if (left.size() != right.size()) return false;
        for (std::size_t index = 0; index < left.size(); ++index) {
            if (std::tolower(static_cast<unsigned char>(left[index])) !=
                std::tolower(static_cast<unsigned char>(right[index]))) return false;
        }
        return true;
    };
    for (const auto& [column, value] : row)
        if (equal(column, name)) return &value;
    return nullptr;
}

std::optional<std::int64_t> DbInteger(const WorldDbRow& row, const std::string_view name)
{
    const WorldDbValue* value = FindValue(row, name);
    if (!value) return std::nullopt;
    if (const auto* number = std::get_if<std::int64_t>(value)) return *number;
    if (const auto* number = std::get_if<std::uint64_t>(value)) {
        if (*number <= static_cast<std::uint64_t>(std::numeric_limits<std::int64_t>::max()))
            return static_cast<std::int64_t>(*number);
    }
    if (const auto* number = std::get_if<double>(value)) return static_cast<std::int64_t>(*number);
    return std::nullopt;
}

std::optional<std::string> DbText(const WorldDbRow& row, const std::string_view name)
{
    const WorldDbValue* value = FindValue(row, name);
    if (!value) return std::nullopt;
    if (const auto* text = std::get_if<std::string>(value)) return *text;
    return std::nullopt;
}

bool DbBool(const WorldDbRow& row, const std::string_view name, bool& output)
{
    const auto value = DbInteger(row, name);
    if (!value) return false;
    output = *value != 0;
    return true;
}

bool DbI32(const WorldDbRow& row, const std::string_view name, std::int32_t& output)
{
    const auto value = DbInteger(row, name);
    if (!value || *value < std::numeric_limits<std::int32_t>::min() ||
        *value > std::numeric_limits<std::int32_t>::max()) return false;
    output = static_cast<std::int32_t>(*value);
    return true;
}

OrganizingTime DbTime(const WorldDbRow& row, const std::string_view name)
{
    OrganizingTime result;
    const auto text = DbText(row, name);
    if (!text) return result;
    unsigned year{}, month{}, day{}, hour{}, minute{}, second{};
    if (std::sscanf(text->c_str(), "%u-%u-%u %u:%u:%u", &year, &month, &day,
                    &hour, &minute, &second) == 6) {
        result.year = static_cast<std::uint16_t>(year);
        result.month = static_cast<std::uint16_t>(month);
        result.day = static_cast<std::uint16_t>(day);
        result.hour = static_cast<std::uint16_t>(hour);
        result.minute = static_cast<std::uint16_t>(minute);
        result.second = static_cast<std::uint16_t>(second);
    }
    return result;
}

bool LoadMemberRights(const WorldDbRow& row, OrganizingMemberInfo& member)
{
    static constexpr std::array<std::string_view, 11> names{
        "PV_Disband", "PV_Exit", "PV_DubJobLvl", "PV_ConMem", "PV_FireOut",
        "PV_Pronounce", "PV_LeaveWord", "PV_EditLeaveWord", "PV_ObtainTax",
        "PV_OperCityGate", "PV_EndueROR"};
    for (std::size_t index = 0; index < names.size(); ++index) {
        std::int32_t value{};
        if (!DbI32(row, names[index], value) || value < 0 || value > 2) return false;
        member.purview[index] = static_cast<EPurviewOwnState>(value);
    }
    return true;
}

bool LoadOrganizingMember(const WorldDbRow& row, const std::string_view idColumn,
                          const std::string_view nameColumn, OrganizingMemberInfo& member)
{
    std::int32_t contribute{};
    const auto name = DbText(row, nameColumn);
    const auto title = DbText(row, "Title");
    if (!DbI32(row, idColumn, member.id) || !DbI32(row, "MemberLvl", member.jobLevel) ||
        !DbI32(row, "bControbute", contribute) || !name || !title ||
        !member.SetName(*name) || !member.SetTitle(*title) || !LoadMemberRights(row, member))
        return false;
    member.contribute = contribute != 0;
    if (const auto level = DbInteger(row, "Levels")) member.level = static_cast<std::int32_t>(*level);
    if (const auto occupation = DbInteger(row, "Occupation"))
        member.occupation = static_cast<std::int32_t>(*occupation);
    member.lastOnlineTime = DbTime(row, "LastOnlineTime");
    return true;
}

bool ReadPacketString(std::span<const std::uint8_t> bytes, std::size_t& offset,
                      const std::size_t maximum, std::string_view& output)
{
    if (offset > bytes.size()) return false;
    const auto first = bytes.begin() + static_cast<std::ptrdiff_t>(offset);
    const auto limit = std::min(bytes.end(), first + static_cast<std::ptrdiff_t>(
        std::min(maximum + 1, bytes.size() - offset)));
    const auto nul = std::find(first, limit, std::uint8_t{0});
    if (nul == limit || nul == first) return false;
    output = {reinterpret_cast<const char*>(&*first), static_cast<std::size_t>(nul - first)};
    offset += output.size() + 1;
    return true;
}

template<class T>
bool ReadPacket(std::span<const std::uint8_t> bytes, std::size_t& offset, T& output)
{
    if (offset > bytes.size() || bytes.size() - offset < sizeof(T)) return false;
    std::memcpy(&output, bytes.data() + offset, sizeof(T));
    offset += sizeof(T);
    return true;
}
}

CWorldServer::CWorldServer() = default;
CWorldServer::~CWorldServer() { if (m_State != WorldServerState::Stopped) static_cast<void>(Shutdown()); }

std::uint32_t CWorldServer::Now() noexcept
{
    return static_cast<std::uint32_t>(std::chrono::duration_cast<std::chrono::milliseconds>(
        std::chrono::steady_clock::now().time_since_epoch()).count());
}

WorldInitializationResult CWorldServer::Initialize(const std::filesystem::path& runtimeDirectory)
{
    if (m_State != WorldServerState::Created) return {false, "WorldServer уже инициализировался"};
    const auto setup = CGame::LoadSetup(runtimeDirectory / "setup.ini");
    if (!setup) { m_State = WorldServerState::Error; return {false, "не удалось прочитать setup.ini/setup.dat"}; }

    const auto connection = Nebokrai::Database::BuildMssqlConnectionString(
        setup->sqlServerIp, setup->databaseName, setup->sqlUserName, setup->sqlPassword);
    if (const auto* error = std::get_if<Nebokrai::Database::OdbcError>(&connection)) {
        m_State = WorldServerState::Error;
        return {false, error->detail};
    }
    m_Database = std::make_unique<OdbcWorldDbExecutor>(std::get<std::string>(connection));
    if (!m_Database->Open()) {
        m_State = WorldServerState::Error;
        return {false, std::string(m_Database->LastError())};
    }

    m_Game = std::make_unique<CGame>(*setup);
    InstallMessageHandlers();
    std::string bootstrapError;
    if (!m_Game->LoadStaticConfiguration(runtimeDirectory, bootstrapError)) {
        m_State = WorldServerState::Error;
        return {false, std::move(bootstrapError)};
    }
    if (!m_Game->LoadRegions(runtimeDirectory, bootstrapError)) {
        m_State = WorldServerState::Error;
        return {false, std::move(bootstrapError)};
    }
    if (!LoadPersistentOwners(bootstrapError)) {
        m_State = WorldServerState::Error;
        return {false, std::move(bootstrapError)};
    }

    m_Server = std::make_unique<WorldNet::CMyNetServer>(m_Io.get_executor(), Now());
    std::error_code hostError;
    if (!m_Server->Host(setup->listenPort, std::nullopt, 1, true, hostError)) {
        m_State = WorldServerState::Error;
        return {false, "не удалось открыть порт WorldServer: " + hostError.message()};
    }
    m_Server->ConfigureTransportAfterHost(
        setup->checkNetwork, setup->maximumIoSends, setup->maximumBytes,
        setup->maximumConnections, setup->checkMessageContent, setup->banIpTime,
        setup->maximumMessageLength, setup->maximumClientSendBuffer);

    std::error_code addressError;
    const auto loginAddress = asio::ip::make_address_v4(setup->loginIp, addressError);
    if (addressError) {
        m_State = WorldServerState::Error;
        return {false, "неверный LoginServer IP: " + addressError.message()};
    }
    m_Login = std::make_unique<WorldNet::CMyNetClient>(m_Io.get_executor());
    m_State = WorldServerState::Initialized;
    asio::co_spawn(m_Io, AcceptLoop(), asio::detached);
    m_LoginEndpoint = asio::ip::tcp::endpoint{
        loginAddress, static_cast<std::uint16_t>(setup->loginPort)};
    m_LoginLoopActive = true;
    asio::co_spawn(m_Io, LoginReadLoop(*m_LoginEndpoint), asio::detached);
    m_State = WorldServerState::Running;
    m_LastSaveTick = Now();
    return {true, {}};
}

void CWorldServer::InstallMessageHandlers()
{
    m_Game->MessageHandlers().Set(WorldMessageFamily::Server,
        [this](WorldNet::CMessage& message) { HandleServerMessage(message); });
    m_Game->MessageHandlers().Set(WorldMessageFamily::Country,
        [this](WorldNet::CMessage& message) { HandleCountryMessage(message); });
    m_Game->MessageHandlers().Set(WorldMessageFamily::Player,
        [this](WorldNet::CMessage& message) { HandlePlayerMessage(message); });
    m_Game->MessageHandlers().Set(WorldMessageFamily::Team,
        [this](WorldNet::CMessage& message) { HandleTeamMessage(message); });
    m_Game->MessageHandlers().Set(WorldMessageFamily::Jjc,
        [this](WorldNet::CMessage& message) { HandleJjcMessage(message); });
    const std::pair<WorldMessageFamily, std::string_view> unhandled[] = {
        {WorldMessageFamily::Log, "log"}, {WorldMessageFamily::Gma, "gma"},
        {WorldMessageFamily::Other, "other"},
        {WorldMessageFamily::Gm, "gm"},
        {WorldMessageFamily::Organizing, "organizing"}, {WorldMessageFamily::WriteLog, "write-log"},
        {WorldMessageFamily::ServerAuction, "server-auction"},
        {WorldMessageFamily::MiscAuction, "misc-auction"},
    };
    for (const auto [family, name] : unhandled)
        m_Game->MessageHandlers().Set(family, [this, name](WorldNet::CMessage& message) {
            HandleUnhandledMessage(name, message);
        });
}

void CWorldServer::HandleJjcMessage(WorldNet::CMessage& message)
{
    if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
        m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
        spdlog::warn("WorldServer: JJC-сообщение пришло не от текущего GameServer");
        return;
    }
    const auto bytes = message.WireBytes();
    std::size_t offset = CBaseMessage::kHeaderSize;
    auto invalid = [&] { HandleUnhandledMessage("jjc-invalid", message); };
    auto ownedPlayer = [&](const std::int32_t playerId) -> CPlayer* {
        CPlayer* player = m_Game->GetPlayer(playerId);
        const auto* owner = m_Game->GetPlayerGameServer(playerId);
        return player && owner && owner->connected &&
                       owner->index == static_cast<std::uint32_t>(message.MapID())
                   ? player : nullptr;
    };
    const auto sender = m_Server->CommandHandle();

    switch (message.GetType()) {
    case 0x00060901U: {
        std::int32_t playerId{};
        std::uint8_t level{};
        std::uint32_t jjcLevel{}, jjcScore{};
        std::array<std::uint16_t, 8> counters{};
        if (!ReadPacket(bytes, offset, playerId) || !ReadPacket(bytes, offset, level) ||
            !ReadPacket(bytes, offset, jjcLevel) || !ReadPacket(bytes, offset, jjcScore)) {
            invalid(); return;
        }
        for (auto& counter : counters) {
            if (!ReadPacket(bytes, offset, counter)) { invalid(); return; }
        }
        CPlayer* player = ownedPlayer(playerId);
        if (!player || level == 0 || jjcLevel > std::numeric_limits<std::int32_t>::max() ||
            jjcScore > std::numeric_limits<std::int32_t>::max() || offset != bytes.size()) {
            invalid(); return;
        }
        player->UpdateJjcSummary(level, jjcLevel, jjcScore, counters);
        break;
    }
    case 0x00060902U: {
        std::int32_t playerId{}, oldRegion{}, posX{}, posY{};
        std::uint8_t level{};
        std::uint32_t jjcLevel{};
        if (!ReadPacket(bytes, offset, playerId) || !ReadPacket(bytes, offset, level) ||
            !ReadPacket(bytes, offset, jjcLevel) || !ReadPacket(bytes, offset, oldRegion) ||
            !ReadPacket(bytes, offset, posX) || !ReadPacket(bytes, offset, posY) ||
            offset != bytes.size() || level == 0 || oldRegion <= 0) {
            invalid(); return;
        }
        CPlayer* player = ownedPlayer(playerId);
        if (!player) { invalid(); return; }
        player->UpdateJjcApplication(level, jjcLevel);
        JjcInfo info{jjcLevel, oldRegion, posX, posY, 0, 0, 0};
        std::int32_t result{};
        if (!m_Game->Jjc().Apply(playerId, info)) {
            result = 1;
        } else {
            const auto region = m_Game->Jjc().AcquireRegion();
            if (!region) {
                static_cast<void>(m_Game->Jjc().Quit(playerId)); result = 4;
            } else if (const auto opponent = m_Game->Jjc().MatchOpponent(playerId)) {
                const auto now = static_cast<std::int32_t>(std::time(nullptr));
                if (!m_Game->Jjc().StartFight(*region, playerId, *opponent, now)) {
                    static_cast<void>(m_Game->Jjc().ReleaseRegion(*region));
                    static_cast<void>(m_Game->Jjc().Quit(playerId)); result = 4;
                } else {
                    const JjcInfo* first = m_Game->Jjc().QueryPlayer(playerId);
                    const JjcInfo* second = m_Game->Jjc().QueryPlayer(*opponent);
                    if (!first || !second) { invalid(); return; }
                    WorldNet::CMessage regionStart(0x00080503);
                    regionStart.Add(first, sizeof(*first));
                    regionStart.Add(second, sizeof(*second));
                    const auto regionMap = m_Game->GetRegionGameServer(*region).value_or(0);
                    static_cast<void>(regionStart.SendToMapID(
                        &sender, static_cast<std::int32_t>(regionMap)));
                    for (const auto id : {playerId, *opponent}) {
                        if (const auto* gameServer = m_Game->GetPlayerGameServer(id)) {
                            WorldNet::CMessage move(0x00080504);
                            move.Add(id); move.Add(*region);
                            static_cast<void>(move.SendToMapID(
                                &sender, static_cast<std::int32_t>(gameServer->index)));
                        }
                    }
                }
            } else {
                static_cast<void>(m_Game->Jjc().ReleaseRegion(*region));
                result = 3;
            }
        }
        if (result != 0) {
            WorldNet::CMessage response(0x00080502);
            response.Add(result); response.Add(playerId);
            static_cast<void>(response.SendToMapID(&sender, message.MapID()));
        }
        break;
    }
    case 0x00060903U: {
        std::int32_t playerId{}, regionId{};
        if (!ReadPacket(bytes, offset, playerId) || !ReadPacket(bytes, offset, regionId) ||
            offset != bytes.size() || !ownedPlayer(playerId)) { invalid(); return; }
        if (const JjcInfo* info = m_Game->Jjc().QueryPlayer(playerId);
            info && info->jjcRegionId != 0 && m_Game->Jjc().QueryFight(info->jjcRegionId)) {
            static_cast<void>(m_Game->Jjc().EndFight(info->jjcRegionId));
        } else {
            static_cast<void>(m_Game->Jjc().Quit(playerId));
        }
        break;
    }
    case 0x00060904U: {
        std::int32_t regionId{}, playerId{};
        if (!ReadPacket(bytes, offset, regionId) || !ReadPacket(bytes, offset, playerId) ||
            offset != bytes.size()) { invalid(); return; }
        const JjcFight* fight = m_Game->Jjc().QueryFight(regionId);
        const auto regionOwner = m_Game->GetRegionGameServer(regionId).value_or(0);
        if (!fight || regionOwner != static_cast<std::uint32_t>(message.MapID()) ||
            (fight->firstPlayerId != playerId && fight->secondPlayerId != playerId) ||
            !m_Game->Jjc().EndFight(regionId)) { invalid(); return; }
        break;
    }
    case 0x00060905U: {
        std::int32_t playerId{}, requested{};
        if (!ReadPacket(bytes, offset, playerId) || !ReadPacket(bytes, offset, requested) ||
            offset != bytes.size() || requested <= 0 || requested > 100 ||
            !ownedPlayer(playerId)) { invalid(); return; }
        const auto count = std::min<std::size_t>(
            static_cast<std::size_t>(requested), m_Game->Jjc().Ranks().size());
        WorldNet::CMessage response(0x00080508);
        response.Add(playerId); response.Add(static_cast<std::int32_t>(count));
        for (std::size_t index{}; index < count; ++index)
            response.Add(&m_Game->Jjc().Ranks()[index], sizeof(JjcRank));
        static_cast<void>(response.SendToMapID(&sender, message.MapID()));
        break;
    }
    case 0x00060906U:
    case 0x00060907U: {
        std::int32_t timestamp{};
        if (!ReadPacket(bytes, offset, timestamp) || offset != bytes.size() || timestamp <= 0 ||
            std::llabs(static_cast<long long>(std::time(nullptr)) - timestamp) > 300) {
            invalid(); return;
        }
        // Имена процедур сброса в PDB не сохранились: подтверждённое событие
        // рассылается, но выдуманная DB-команда намеренно не выполняется.
        WorldNet::CMessage response(
            message.GetType() == 0x00060906U ? 0x00080509 : 0x0008050A);
        response.Add(timestamp);
        static_cast<void>(response.SendAll(&sender));
        break;
    }
    default:
        HandleUnhandledMessage("jjc", message);
        break;
    }
}

void CWorldServer::HandleTeamMessage(WorldNet::CMessage& message)
{
    if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
        m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
        spdlog::warn("WorldServer: team-сообщение пришло не от владельца MapID");
        return;
    }
    const auto bytes = message.WireBytes();
    std::size_t offset = CBaseMessage::kHeaderSize;
    auto invalid = [&] { HandleUnhandledMessage("team-invalid", message); };
    std::uint32_t teamId{};
    std::int32_t ownerType{}, ownerId{};
    auto readTeam = [&] { return ReadPacket(bytes, offset, teamId) && teamId != 0; };
    auto readOwner = [&] {
        return ReadPacket(bytes, offset, ownerType) && ownerType > 0 &&
               ReadPacket(bytes, offset, ownerId) && ownerId > 0;
    };
    auto findTeam = [&]() -> CTeam* { return m_Game->Sessions().QueryTeam(teamId); };

    switch (message.GetType()) {
    case 0x00060001U: {
        const auto sessionId = m_Game->Sessions().UnserializeSession(bytes, offset);
        if (sessionId == 0 || offset != bytes.size()) { invalid(); return; }
        break;
    }
    case 0x00060002U:
        if (!readTeam() || offset != bytes.size()) { invalid(); return; }
        if (auto* value = findTeam()) static_cast<void>(value->End());
        break;
    case 0x00060003U: {
        std::int32_t regionId{};
        std::string_view name;
        if (!readTeam() || !readOwner() || !ReadPacket(bytes, offset, regionId) || regionId <= 0 ||
            !ReadPacketString(bytes, offset, 255, name) || offset != bytes.size()) { invalid(); return; }
        CTeam* value = findTeam();
        if (!value || value->QueryPlugByOwner(ownerType, ownerId)) break;
        const auto plugId = m_Game->Sessions().CreatePlug(
            CSessionFactory::PlugType::Teamate, ownerType, ownerId);
        auto* member = dynamic_cast<CTeamate*>(m_Game->Sessions().QueryPlug(plugId));
        if (!member) break;
        member->SetOwnerRegionID(regionId);
        member->SetOwnerName(std::string(name));
        if (!m_Game->Sessions().InsertPlug(value->GetID(), plugId))
            static_cast<void>(m_Game->Sessions().GarbageCollect(CSessionFactory::ObjectType::Plug, plugId));
        break;
    }
    case 0x00060004U:
        if (!readTeam() || !readOwner() || offset != bytes.size()) { invalid(); return; }
        if (CTeam* value = findTeam())
            if (CPlug* plug = value->QueryPlugByOwner(ownerType, ownerId)) static_cast<void>(plug->Exit());
        break;
    case 0x00060005U: {
        std::int32_t regionId{};
        if (!readTeam() || !readOwner() || !ReadPacket(bytes, offset, regionId) ||
            regionId <= 0 || offset != bytes.size()) { invalid(); return; }
        if (CTeam* value = findTeam())
            if (auto* member = dynamic_cast<CTeamate*>(value->QueryPlugByOwner(ownerType, ownerId)))
                member->SetOwnerRegionID(regionId);
        break;
    }
    case 0x00060006U:
    case 0x00060007U: {
        std::int32_t playerId{};
        if (!readTeam() || !ReadPacket(bytes, offset, playerId) || playerId <= 0 ||
            offset != bytes.size()) { invalid(); return; }
        if (CTeam* value = findTeam()) {
            if (message.GetType() == 0x00060006U) static_cast<void>(value->SetLeader(playerId));
            else static_cast<void>(value->KickPlayer(playerId));
        }
        break;
    }
    case 0x00060008U: {
        if (!readTeam() || offset != bytes.size()) { invalid(); return; }
        CTeam* value = findTeam();
        if (!value) break;
        std::vector<std::uint8_t> payload;
        if (!value->Serialize(payload)) break;
        WorldNet::CMessage response(0x0007FD08);
        if (!payload.empty()) response.Add(payload.data(), static_cast<std::int32_t>(payload.size()));
        const auto sender = m_Server->CommandHandle();
        static_cast<void>(response.SendToSocket(&sender, message.SocketID()));
        break;
    }
    case 0x00060009U: {
        std::int32_t plugId{}, exists{};
        if (!ReadPacket(bytes, offset, plugId) || plugId <= 0 || !readOwner() ||
            !ReadPacket(bytes, offset, exists) || (exists != 0 && exists != 1) ||
            offset != bytes.size()) { invalid(); return; }
        if (auto* member = dynamic_cast<CTeamate*>(m_Game->Sessions().QueryPlug(plugId)))
            member->AcceptPlayerExistenceResponse(message.MapID(), ownerType, ownerId, exists != 0);
        break;
    }
    case 0x0006000AU: {
        std::int32_t allocation{};
        if (!readTeam() || !ReadPacket(bytes, offset, allocation) || allocation < 0 || allocation > 1 ||
            offset != bytes.size()) { invalid(); return; }
        if (CTeam* value = findTeam()) value->SetAllocationScheme(static_cast<CTeam::Allocation>(allocation));
        break;
    }
    case 0x0006000BU: {
        std::string_view text;
        if (!readTeam() || !readOwner() || !ReadPacketString(bytes, offset, 511, text) ||
            offset != bytes.size()) { invalid(); return; }
        if (CTeam* value = findTeam())
            if (CPlug* plug = value->QueryPlugByOwner(ownerType, ownerId))
                static_cast<void>(value->OnPlugChangeState(plug->GetID(),
                    static_cast<std::int32_t>(CTeamate::State::Chat),
                    {reinterpret_cast<const std::uint8_t*>(text.data()), text.size()}, false));
        break;
    }
    case 0x0006000CU: {
        std::uint32_t state{};
        if (!readTeam() || !readOwner() || !ReadPacket(bytes, offset, state) ||
            offset != bytes.size()) { invalid(); return; }
        if (CTeam* value = findTeam())
            if (CPlug* plug = value->QueryPlugByOwner(ownerType, ownerId))
                static_cast<void>(value->OnPlugChangeState(plug->GetID(),
                    static_cast<std::int32_t>(CTeamate::State::ChangeState),
                    {reinterpret_cast<const std::uint8_t*>(&state), sizeof(state)}, false));
        break;
    }
    default: HandleUnhandledMessage("team", message); break;
    }
}

void CWorldServer::HandlePlayerMessage(WorldNet::CMessage& message)
{
    if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
        m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
        spdlog::warn("WorldServer: player-сообщение пришло не от владельца MapID");
        return;
    }
    const auto bytes = message.WireBytes();
    std::size_t offset = CBaseMessage::kHeaderSize;
    std::string_view player, setup;
    std::uint32_t response{};
    std::int32_t sourcePlayer{};
    if (!ReadPacketString(bytes, offset, 49, player)) {
        HandleUnhandledMessage("player-invalid", message); return;
    }
    switch (message.GetType()) {
    case 0x0005FC01U:
        if (!ReadPacketString(bytes, offset, 255, setup) || bytes.size() - offset != 6) {
            HandleUnhandledMessage("player-invalid", message); return;
        }
        sourcePlayer = static_cast<std::int32_t>(
            static_cast<std::uint32_t>(bytes[offset + 2]) |
            (static_cast<std::uint32_t>(bytes[offset + 3]) << 8U) |
            (static_cast<std::uint32_t>(bytes[offset + 4]) << 16U) |
            (static_cast<std::uint32_t>(bytes[offset + 5]) << 24U));
        if (!m_Game->GetPlayerGameServer(sourcePlayer) ||
            m_Game->GetPlayerGameServer(sourcePlayer)->index != static_cast<std::uint32_t>(message.MapID())) {
            spdlog::warn("WorldServer: отклонена смена навыка от чужого PlayerID={}", sourcePlayer);
            return;
        }
        response = 0x0007FA08U;
        break;
    case 0x0005FC02U:
        if (!ReadPacketString(bytes, offset, 255, setup) || offset != bytes.size()) {
            HandleUnhandledMessage("player-invalid", message); return;
        }
        response = 0x0007FA09U;
        break;
    case 0x0005FC03U:
        if (!ReadPacketString(bytes, offset, 255, setup) || bytes.size() - offset != 4) {
            HandleUnhandledMessage("player-invalid", message); return;
        }
        response = 0x0007FA0AU;
        break;
    case 0x0005FC04U:
        if (bytes.size() - offset != 1) { HandleUnhandledMessage("player-invalid", message); return; }
        response = 0x0007FA0BU;
        break;
    default:
        HandleUnhandledMessage("player", message); return;
    }
    message.SetType(response);
    if (response == 0x0007FA08U) message.Add(message.MapID());
    const auto sender = m_Server->CommandHandle();
    static_cast<void>(message.SendAll(&sender));
}

bool CWorldServer::SendInitialGameServerState(
    const std::int32_t socketId,
    const CGame::GameServerInfo& gameServer)
{
    if (!m_Server || socketId <= 0) return false;
    const auto sender = m_Server->CommandHandle();
    const auto sent = [&](const WorldNet::CMessage& message) {
        const auto result = message.SendToSocket(&sender, socketId);
        return std::holds_alternative<std::int32_t>(result) &&
               std::get<std::int32_t>(result) != 0;
    };
    const auto sendSetup = [&](const std::int32_t subtype,
                               const std::vector<std::uint8_t>& payload) {
        WorldNet::CMessage message(0x0007F801);
        message.Add(subtype);
        if (!payload.empty()) message.Add(payload.data(), static_cast<std::int32_t>(payload.size()));
        return sent(message);
    };

    WorldNet::CMessage globe(0x0007F80D);
    const auto& variables = m_Game->GlobeVariables();
    globe.Add(&variables, sizeof(variables));
    if (!sent(globe)) return false;

    std::vector<std::uint8_t> payload;
    if (!m_Game->Strings().Serialize(payload) || !sendSetup(0x2F, payload)) return false;

    payload.clear();
    if (!m_Game->CountryParameters().AddToByteArray(payload) ||
        !sendSetup(0x18, payload)) return false;

    for (const CGame::RegionRoute& route : m_Game->RegionRoutes()) {
        payload.clear();
        if (route.gameServerIndex == gameServer.index) {
            if (!route.region->AddToByteArray(payload, true)) {
                spdlog::error("WorldServer: регион {} не готов к полной сериализации",
                              route.region->GetID());
                return false;
            }
            WorldNet::CMessage region(0x0007F801);
            region.Add(0x0E);
            region.Add(static_cast<std::int32_t>(route.type));
            if (!payload.empty()) region.Add(payload.data(), static_cast<std::int32_t>(payload.size()));
            if (!sent(region)) return false;
        } else {
            if (!route.region->AddToByteArrayForProxy(payload, true) ||
                !sendSetup(0x0F, payload)) {
                spdlog::error("WorldServer: регион {} не готов к proxy-сериализации",
                              route.region->GetID());
                return false;
            }
        }
    }

    payload.clear();
    if (!m_Game->Variables().Serialize(payload) || !sendSetup(0x0C, payload)) return false;

    WorldNet::CMessage index(0x0007F801);
    index.Add(0x12);
    index.Add(static_cast<std::uint8_t>(gameServer.index));
    if (!sent(index)) return false;

    WorldNet::CMessage identity(0x0007F801);
    identity.Add(0x3B);
    identity.Add(m_Game->LoginServerId());
    identity.Add(m_Game->GetSetup().number);
    return sent(identity);
}

void CWorldServer::HandleServerMessage(WorldNet::CMessage& message)
{
    const auto sender = m_Server ? std::optional(m_Server->CommandHandle()) : std::nullopt;
    switch (message.GetType()) {
    case 0x0003FC01U:
        // Разрыв LoginServer уже переведён transport-owner-ом в reconnect.
        break;
    case 0x0003FC02U: {
        if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
            message.WireBytes().size() != CBaseMessage::kHeaderSize + sizeof(std::int32_t)) {
            spdlog::warn("WorldServer: отклонена повреждённая команда отключения GameServer");
            break;
        }
        const std::int32_t index = message.GetLong();
        if (index != message.MapID()) {
            spdlog::warn("WorldServer: отключение GameServer не совпадает с MapID");
            break;
        }
        static_cast<void>(m_Game->SetGameServerConnected(static_cast<std::uint32_t>(index), false));
        spdlog::info("WorldServer: GameServer {} отключён", index);
        break;
    }
    case 0x0004FC01U: {
        if (message.MapID() != 0 || message.SocketID() != 0 ||
            message.WireBytes().size() != CBaseMessage::kHeaderSize) {
            spdlog::warn("WorldServer: отклонён чужой запрос ping GameServer");
            break;
        }
        m_Game->BeginGameServerPing(Now());
        WorldNet::CMessage ping(0x0007F809);
        static_cast<void>(ping.SendAll(sender ? &*sender : nullptr));
        break;
    }
    case 0x0004FC02U: {
        if (message.MapID() != 0 || message.SocketID() != 0 ||
            message.WireBytes().size() != CBaseMessage::kHeaderSize) {
            spdlog::warn("WorldServer: отклонён чужой broadcast LoginServer");
            break;
        }
        WorldNet::CMessage broadcast(0x0007F80B);
        static_cast<void>(broadcast.SendAll(sender ? &*sender : nullptr));
        break;
    }
    case 0x0004FC03U:
        if (message.MapID() != 0 || message.SocketID() != 0 ||
            message.WireBytes().size() != CBaseMessage::kHeaderSize + sizeof(std::int32_t)) {
            spdlog::warn("WorldServer: отклонена чужая идентификация LoginServer");
            break;
        }
        m_Game->SetLoginServerId(message.GetLong());
        break;
    case 0x0005FA01U: {
        if (!m_Server || message.MapID() != 0 || message.SocketID() <= 0) {
            spdlog::warn("WorldServer: отклонена повторная регистрация GameServer");
            break;
        }
        const auto bytes = message.WireBytes();
        std::size_t offset = CBaseMessage::kHeaderSize;
        if (bytes.size() - offset < 5) {
            spdlog::warn("WorldServer: короткая регистрация GameServer");
            break;
        }
        const bool reconnect = bytes[offset++] != 0;
        const std::uint32_t port = static_cast<std::uint32_t>(bytes[offset]) |
            (static_cast<std::uint32_t>(bytes[offset + 1]) << 8U) |
            (static_cast<std::uint32_t>(bytes[offset + 2]) << 16U) |
            (static_cast<std::uint32_t>(bytes[offset + 3]) << 24U);
        offset += 4;
        std::string_view host;
        if (port == 0 || port > 65535 || !ReadPacketString(bytes, offset, 255, host) ||
            (!reconnect && offset != bytes.size())) {
            spdlog::warn("WorldServer: повреждён адрес регистрации GameServer");
            break;
        }
        if (reconnect && m_Game->PlayerCount(CGame::PlayerState::Online) != 0) {
            spdlog::warn("WorldServer: reconnect GameServer требует ещё не восстановленный resync игроков");
            break;
        }
        CGame::GameServerInfo* server = m_Game->FindGameServer(host, port);
        if (!server) {
            spdlog::warn("WorldServer: незарегистрированный GameServer {}:{}", host, port);
            break;
        }
        if (m_Server->SetClientMapID(message.SocketID(), static_cast<std::int32_t>(server->index)) == 0) {
            spdlog::warn("WorldServer: очередь отклонила MapID GameServer {}", server->index);
            break;
        }
        server->connected = true;
        if (!reconnect && !SendInitialGameServerState(message.SocketID(), *server)) {
            server->connected = false;
            const auto control = m_Server->CommandHandle();
            static_cast<void>(control.QuitByMapID(static_cast<std::int32_t>(server->index)));
            spdlog::error("WorldServer: начальная синхронизация GameServer {} не завершена",
                          server->index);
            break;
        }
        spdlog::info("WorldServer: GameServer {} зарегистрирован как MapID {}{}",
                     host, server->index, reconnect ? " после переподключения" : "");
        break;
    }
    case 0x0005FA0AU:
        if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
            m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
            spdlog::warn("WorldServer: отклонён ping-ответ чужого GameServer");
            break;
        }
        m_Game->RecordGameServerPing({message.PeerIPv4(), message.MapID(), message.GetLong()});
        break;
    case 0x0005FA0BU: {
        if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
            m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
            spdlog::warn("WorldServer: отклонена региональная пересылка чужого GameServer");
            break;
        }
        static_cast<void>(message.GetChar());
        const std::int32_t regionId = message.GetLong();
        const auto mapId = m_Game->GetRegionGameServer(regionId).value_or(0);
        message.SetType(0x0007F80AU);
        static_cast<void>(message.SendToMapID(sender ? &*sender : nullptr,
                                              static_cast<std::int32_t>(mapId)));
        break;
    }
    case 0x0005FA0CU: {
        if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
            m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
            spdlog::warn("WorldServer: отклонена пересылка счётчика чужого GameServer");
            break;
        }
        const std::int32_t value = message.GetLong();
        if (m_Game->GetSetup().number == 0 || message.MapID() == 0 || !m_Login) break;
        WorldNet::CMessage relay(0x0001FE07);
        relay.Add(m_Game->GetSetup().number);
        relay.Add(message.MapID());
        relay.Add(value);
        static_cast<void>(relay.Send(&m_Login->SendQueue()));
        break;
    }
    case 0x0005FA0DU:
        if (!m_Server || message.MapID() <= 0 || message.SocketID() <= 0 ||
            m_Server->GetSocketIDByMapID(message.MapID()) != message.SocketID()) {
            spdlog::warn("WorldServer: отклонено текстовое сообщение чужого GameServer");
            break;
        }
        static_cast<void>(message.GetLong());
        static_cast<void>(message.GetStrBytes(0x80));
        break;
    default:
        HandleUnhandledMessage("server", message);
        break;
    }
}

void CWorldServer::HandleCountryMessage(WorldNet::CMessage& message)
{
    if (message.GetType() != 0x00060318U) {
        HandleUnhandledMessage("country", message);
        return;
    }
    const std::uint8_t country = message.GetByte();
    CountryWarSys::Context context;
    context.regionName = [this](const std::int32_t id) -> std::optional<std::string> {
        const CWorldRegion* region = m_Game->GetRegion(id);
        return region ? std::optional(std::string(region->GetName())) : std::nullopt;
    };
    context.countryExists = [this](const std::uint8_t id) {
        return m_Game->Countries().GetCountry(id) != nullptr;
    };
    context.setWarResult = [this](const std::uint8_t id, const std::int32_t result) {
        static_cast<void>(m_Game->Countries().SetCountryWarResult(id, result));
    };
    context.sendDestroyedFlag = [sender = m_Server ? std::optional(m_Server->CommandHandle()) : std::nullopt]
        (const std::uint8_t id) {
        WorldNet::CMessage outgoing(0x0007FF18);
        outgoing.Add(id);
        static_cast<void>(outgoing.SendAll(sender ? &*sender : nullptr));
    };
    context.sendCountryInfo = [sender = m_Server ? std::optional(m_Server->CommandHandle()) : std::nullopt]
        (const std::string_view text, const std::uint32_t title, const std::uint32_t color) {
        WorldNet::CMessage outgoing(0x0007FA03);
        outgoing.Add(std::string(text).c_str());
        outgoing.Add(title);
        outgoing.Add(color);
        static_cast<void>(outgoing.SendAll(sender ? &*sender : nullptr));
    };
    m_Game->CountryWars().OnFlagDestroyed(country, context);
}

void CWorldServer::HandleUnhandledMessage(const std::string_view family,
                                          WorldNet::CMessage& message)
{
    spdlog::warn("WorldServer: неподтверждённое сообщение {} type=0x{:08X} map={} socket={}",
                 family, message.GetType(), message.MapID(), message.SocketID());
}

bool CWorldServer::LoadPersistentOwners(std::string& error)
{
    auto load = [&](const std::string_view owner, WorldDbResult result)
        -> std::optional<WorldDbResult> {
        if (result.success) return result;
        error = "не загрузился World DB owner " + std::string(owner) + ": " + result.error;
        return std::nullopt;
    };

    auto setup = load("setup", CRsSetup::Load(*m_Database));
    if (!setup || setup->rows.size() != 1) { if (error.empty()) error = "таблица csl_setup не содержит единственную строку"; return false; }
    std::int32_t playerId{}, leaveWordId{};
    if (!DbI32(setup->rows.front(), "playerID", playerId) ||
        !DbI32(setup->rows.front(), "LeaveWordID", leaveWordId)) {
        error = "повреждены идентификаторы csl_setup"; return false;
    }
    m_Game->RestoreNextIds(playerId, leaveWordId);

    auto variables = load("variables", CRsGenVar::Load(*m_Database));
    if (!variables) return false;
    for (const WorldDbRow& row : variables->rows) {
        const auto name = DbText(row, "VarName"), saved = DbText(row, "SValue"), current = DbText(row, "CValue");
        if (!name || !saved || !current || !m_Game->Variables().Restore(*name, *saved, *current)) {
            error = "повреждена строка CSL_GENVAR"; return false;
        }
    }

    auto countries = load("countries", CDBCountry::Load(*m_Database));
    if (!countries) return false;
    for (const WorldDbRow& row : countries->rows) {
        CountrySaveSnapshot snapshot;
        std::int32_t countryId{};
        if (!DbI32(row, "id", countryId) || countryId <= 0 || countryId > 255 ||
            !DbI32(row, "treasury", snapshot.treasury) || !DbI32(row, "power", snapshot.power) ||
            !DbI32(row, "tech_exp", snapshot.techExperience) || !DbI32(row, "tech_lel", snapshot.techLevel) ||
            !DbI32(row, "king_id", snapshot.king.id) || !DbI32(row, "control_point", snapshot.king.controlPoint) ||
            !DbI32(row, "material_point", snapshot.king.materialPoint) || !DbI32(row, "war_point", snapshot.king.warPoint) ||
            !DbI32(row, "war_res", snapshot.warResult)) { error = "повреждена строка CSL_Countrys"; return false; }
        snapshot.countryId = static_cast<std::uint8_t>(countryId);
        snapshot.king.name = DbText(row, "king_name").value_or(std::string{});
        snapshot.king.job = 1;
        if (!DbBool(row, "king_appoint", snapshot.king.appointed) ||
            !DbBool(row, "king_salary", snapshot.king.salaryReceived)) { error = "повреждены флаги правителя CSL_Countrys"; return false; }
        for (std::size_t index = 0; index < snapshot.ministers.size(); ++index) {
            const std::string prefix = "minister_" + std::to_string(index + 2);
            CountryMinisterSnapshot minister;
            if (!DbI32(row, prefix + "_id", minister.id)) { error = "повреждён министр CSL_Countrys"; return false; }
            if (minister.id == 0) continue;
            minister.name = DbText(row, prefix + "_name").value_or(std::string{});
            minister.job = static_cast<std::uint8_t>(index + 2);
            if (!DbBool(row, prefix + "_appoint", minister.appointed) ||
                !DbBool(row, prefix + "_salary", minister.salaryReceived)) { error = "повреждены флаги министра CSL_Countrys"; return false; }
            snapshot.ministers[index] = std::move(minister);
        }
        auto country = std::make_unique<CCountry>();
        country->Restore(snapshot);
        if (!m_Game->Countries().AddCountry(std::move(country))) { error = "повторяющийся идентификатор страны"; return false; }
    }

    auto regions = load("regions", CRsRegion::Load(*m_Database));
    if (!regions) return false;
    for (const WorldDbRow& row : regions->rows) {
        RegionDbSnapshot snapshot;
        if (!DbI32(row, "RegionID", snapshot.regionId) || !DbI32(row, "OwnedFactionID", snapshot.factionId) ||
            !DbI32(row, "OwnedUnionID", snapshot.unionId) || !DbI32(row, "CurTaxRate", snapshot.taxRate) ||
            !DbI32(row, "TodayTotalTax", snapshot.todayTax) || !DbI32(row, "TotalTax", snapshot.totalTax)) {
            error = "повреждена строка CSL_Region"; return false;
        }
        m_Game->RestoreRegionState(snapshot);
    }

    auto factions = load("factions", CRsFaction::Load(*m_Database));
    if (!factions) return false;
    for (const WorldDbRow& row : factions->rows) {
        std::int32_t id{}, master{};
        const auto name = DbText(row, "Name");
        if (!DbI32(row, "ID", id) || !DbI32(row, "MasterID", master) || !name || id <= 0) {
            error = "повреждена строка CSL_FACTION_BaseProperty"; return false;
        }
        auto faction = std::make_unique<CFaction>(id, master, DbTime(row, "EstablishedTime"), *name);
        auto& property = faction->Property();
        std::int32_t permit{}, country{};
        if (!DbI32(row, "Levels", property.level) || !DbI32(row, "Experience", property.experience) ||
            !DbI32(row, "OffenseVictorCounts", property.offenseWins) ||
            !DbI32(row, "DefenceVictorCounts", property.defenseWins) ||
            !DbI32(row, "VillageWarVictorCounts", property.villageWins) ||
            !DbI32(row, "MemberNums", property.memberCount) || !DbI32(row, "UnionID", property.unionId) ||
            !DbI32(row, "bPermit", permit) || !DbI32(row, "lPro1", property.property1) ||
            !DbI32(row, "lPro2", property.property2) || !DbI32(row, "country", country)) {
            error = "повреждены свойства фракции"; return false;
        }
        property.permit = permit != 0;
        property.country = static_cast<std::uint8_t>(country);
        std::int32_t deleteMinutes{};
        if (DbI32(row, "DelRemainTime", deleteMinutes)) faction->SetDeleteRemainTime(deleteMinutes);
        if (!m_Game->Organizations().AddFaction(std::move(faction))) { error = "повторяющийся идентификатор фракции"; return false; }
    }
    auto factionMembers = load("faction-members", CRsFaction::LoadMembers(*m_Database));
    if (!factionMembers) return false;
    for (const WorldDbRow& row : factionMembers->rows) {
        std::int32_t factionId{}; OrganizingMemberInfo member;
        if (!DbI32(row, "FactionID", factionId) || !LoadOrganizingMember(row, "PlayerID", "Name", member) ||
            !m_Game->Organizations().GetFaction(factionId) ||
            !m_Game->Organizations().GetFaction(factionId)->AddMember(std::move(member))) {
            error = "повреждён участник CSL_FACTION_Members"; return false;
        }
    }
    for (const WorldDbRow& row : factions->rows) {
        std::int32_t factionId{};
        if (DbI32(row, "ID", factionId))
            if (CFaction* faction = m_Game->Organizations().GetFaction(factionId))
                faction->SetChangeData(0);
    }

    auto unions = load("unions", CRsUnion::Load(*m_Database));
    if (!unions) return false;
    for (const WorldDbRow& row : unions->rows) {
        std::int32_t id{}, master{}; const auto name = DbText(row, "Name");
        if (!DbI32(row, "ID", id) || !DbI32(row, "MasterID", master) || !name ||
            !m_Game->Organizations().AddUnion(std::make_unique<CUnion>(id, master, *name))) {
            error = "повреждена строка CSL_UNION_BaseProperty"; return false;
        }
    }
    auto unionMembers = load("union-members", CRsUnion::LoadMembers(*m_Database));
    if (!unionMembers) return false;
    for (const WorldDbRow& row : unionMembers->rows) {
        std::int32_t unionId{}; OrganizingMemberInfo member;
        if (!DbI32(row, "UnionID", unionId) ||
            !LoadOrganizingMember(row, "FactionID", "FactionName", member) ||
            !m_Game->Organizations().GetUnion(unionId) ||
            !m_Game->Organizations().GetUnion(unionId)->AddFaction(std::move(member))) {
            error = "повреждён участник CSL_UNION_Members"; return false;
        }
    }
    for (const WorldDbRow& row : unions->rows) {
        std::int32_t unionId{};
        if (DbI32(row, "ID", unionId))
            if (CUnion* unionValue = m_Game->Organizations().GetUnion(unionId))
                unionValue->SetChangeData(0);
    }

    auto enemies = load("enemy-factions", CRsEnemyFactions::Load(*m_Database));
    if (!enemies) return false;
    for (const WorldDbRow& row : enemies->rows) {
        std::int32_t first{}, second{}, duration{};
        if (!DbI32(row, "FactionID1", first) || !DbI32(row, "FactionID2", second) ||
            !DbI32(row, "LeaveTime", duration) || first <= 0 || second <= 0 || duration <= 0) {
            error = "повреждена строка CSL_FactionWar"; return false;
        }
        m_Game->FactionWars().AddEnemyRelation(first, second, static_cast<std::uint32_t>(duration));
        if (CFaction* faction = m_Game->Organizations().GetFaction(first)) faction->AddEnemy(second);
        if (CFaction* faction = m_Game->Organizations().GetFaction(second)) faction->AddEnemy(first);
    }

    auto godsRegions = load("gods-battle-regions", CRsGodsBattle::LoadRegions(*m_Database));
    if (!godsRegions) return false;
    for (const WorldDbRow& row : godsRegions->rows) {
        GodsBattleRegionDbRow value;
        if (!DbI32(row, "RegionID", value.regionId) || !DbI32(row, "AFactionXYD", value.factionA) ||
            !DbI32(row, "BFactionXYD", value.factionB)) { error = "повреждена строка CSL_GODSBATTLE"; return false; }
        m_Game->GodsBattleRegions().push_back(value);
    }
    auto godsNpcs = load("gods-battle-npcs", CRsGodsBattle::LoadNpcs(*m_Database));
    if (!godsNpcs) return false;
    for (const WorldDbRow& row : godsNpcs->rows) {
        GodsBattleNpcDbRow value;
        const auto name = DbText(row, "NPC_NAME");
        if (!name || !DbI32(row, "Faciton", value.faction)) { error = "повреждена строка CSL_GODSBATTLE_NPC"; return false; }
        value.npcName = *name;
        m_Game->GodsBattleNpcs().push_back(std::move(value));
    }

    auto increment = load("increment-log", CDBIncrementLog::LoadRecent(*m_Database, 30));
    if (!increment) return false;
    for (const WorldDbRow& row : increment->rows) {
        std::int32_t playerId{}, type{}, money{};
        const auto description = DbText(row, "description");
        if (!DbI32(row, "player_id", playerId) || !DbI32(row, "type", type) ||
            !DbI32(row, "money", money) || !description || type < 0 || type > 255) {
            error = "повреждена строка increment_log"; return false;
        }
        IncrementLogEntry entry{DbTime(row, "log_time"), static_cast<std::uint8_t>(type), money, *description};
        if (!m_Game->IncrementLog().Add(playerId, std::move(entry))) {
            error = "не удалось восстановить increment_log"; return false;
        }
    }

    auto auction = load("auction", CDBMisc::LoadAuctionOwners(*m_Database));
    if (!auction) return false;
    for (const WorldDbRow& row : auction->rows) {
        std::int32_t owner{};
        if (!DbI32(row, "dwowerid", owner)) { error = "повреждён владелец Auction"; return false; }
        m_Game->AuctionOwners().insert(owner);
    }
    return true;
}

asio::awaitable<void> CWorldServer::AcceptLoop()
{
    while (m_State == WorldServerState::Running || m_State == WorldServerState::Initialized) {
        const AcceptStart start = m_Server->BeginAccept();
        if (start == AcceptStart::NotListening) co_return;
        if (start == AcceptStart::AtCapacity) {
            asio::steady_timer pause(m_Io, std::chrono::milliseconds(100));
            co_await pause.async_wait(asio::use_awaitable);
            continue;
        }
        AcceptedTransport accepted = co_await m_Server->AcceptOne();
        if (!accepted.error && accepted.socket)
            static_cast<void>(m_Server->QueueAccepted(std::move(accepted.socket), accepted.peer, Now()));
    }
}

asio::awaitable<void> CWorldServer::LoginReadLoop(const asio::ip::tcp::endpoint endpoint)
{
    const ClientConnectResult connected = co_await m_Login->Connect(endpoint);
    if (connected.status != ClientConnectStatus::Connected) {
        spdlog::warn("WorldServer: LoginServer недоступен: {}", connected.error.message());
        m_LoginLoopActive = false;
        m_LoginReconnectAt = Now() + 5000U;
        co_return;
    }
    const auto accounts = m_Game->OnlineAccounts();
    if (!accounts) {
        m_AsyncError = "невозможно сформировать снимок аккаунтов для LoginServer";
        static_cast<void>(m_Login->Close());
        m_LoginLoopActive = false;
        co_return;
    }
    WorldNet::CMessage snapshot(0x0001FE02);
    snapshot.Add(m_Game->GetSetup().number);
    snapshot.Add(static_cast<std::uint32_t>(accounts->size()));
    for (const std::string& account : *accounts) snapshot.Add(account.c_str());
    WorldNet::CMessage registration(0x0001FE01);
    registration.Add(m_Game->GetSetup().number);
    registration.Add(m_Game->GetSetup().name.c_str());
    const auto snapshotQueued = snapshot.Send(&m_Login->SendQueue(), true);
    const auto registrationQueued = registration.Send(&m_Login->SendQueue());
    if (!std::holds_alternative<std::int32_t>(snapshotQueued) ||
        !std::holds_alternative<std::int32_t>(registrationQueued)) {
        m_AsyncError = "не удалось поставить регистрацию WorldServer в очередь LoginServer";
        static_cast<void>(m_Login->Close());
        m_LoginLoopActive = false;
        co_return;
    }
    spdlog::info("WorldServer зарегистрирован в LoginServer как {} ({})",
                 m_Game->GetSetup().name, m_Game->GetSetup().number);
    while (m_State == WorldServerState::Running || m_State == WorldServerState::Initialized) {
        const WorldNet::ClientReadResult read = co_await m_Login->ReadOnce();
        if (!read) {
            spdlog::warn("WorldServer: связь с LoginServer потеряна: {}",
                         read.ioError ? read.ioError.message() : "повреждённое сообщение");
            static_cast<void>(m_Login->Close());
            m_LoginLoopActive = false;
            m_LoginReconnectAt = Now() + 5000U;
            co_return;
        }
        for (auto& message : m_Login->TakeAllMessages()) m_Game->QueueMessage(std::move(message));
    }
}

asio::awaitable<void> CWorldServer::FlushLogin()
{
    const ClientFlushResult result = co_await m_Login->FlushOutgoing();
    if (result.status != ClientFlushStatus::Drained) {
        spdlog::warn("WorldServer: не удалось отправить сообщение LoginServer: {}",
                     result.error ? result.error.message() : "неполная отправка");
        static_cast<void>(m_Login->Close());
    }
    m_LoginFlushActive = false;
}

void CWorldServer::QueueServerIo(ServerSnapshot snapshot, std::size_t& count)
{
    count += snapshot.ioActions.size();
    for (ServerIoAction& action : snapshot.ioActions)
        asio::co_spawn(m_Io, RunServerIoAction(std::move(action), m_Server->CommandHandle()), asio::detached);
}

WorldLoopResult CWorldServer::RunOne()
{
    if (m_State != WorldServerState::Running) return {false, 0, 0, false, "WorldServer не запущен"};
    WorldLoopResult result;
    m_Io.restart();
    static_cast<void>(m_Io.poll());
    if (!m_AsyncError.empty()) {
        result.success = false; result.error = std::exchange(m_AsyncError, {});
        return result;
    }
    const std::uint32_t now = Now();
    if (!m_LoginLoopActive && m_LoginEndpoint && now >= m_LoginReconnectAt &&
        m_State == WorldServerState::Running) {
        m_LoginLoopActive = true;
        asio::co_spawn(m_Io, LoginReadLoop(*m_LoginEndpoint), asio::detached);
    }
    if (m_Login->IsConnected() && !m_LoginFlushActive && m_Login->SendQueue().Pending() > 0) {
        m_LoginFlushActive = true;
        asio::co_spawn(m_Io, FlushLogin(), asio::detached);
    }
    QueueServerIo(m_Server->ProcessCommandSnapshot(now), result.ioActions);
    while (auto message = m_Server->PopReceivedMessage()) m_Game->QueueMessage(std::move(message));
    result.messages = m_Game->MainLoopTurn(now).messages;

    if (now - m_LastSaveTick >= m_Game->GetSetup().saveIntervalMs) {
        if (!m_PendingSave) m_PendingSave = m_Game->GenerateDBData();
        const WorldSaveReport saved = CSaveDB::Save(*m_Database, *m_PendingSave);
        if (!saved.success) {
            result.success = false;
            result.error = "ошибка фазы сохранения " + saved.failedPhase;
            return result;
        }
        m_PendingSave.reset();
        m_LastSaveTick = now;
        result.saved = true;
    }
    return result;
}

void CWorldServer::RequestStop() noexcept
{
    if (m_State == WorldServerState::Running || m_State == WorldServerState::Initialized)
        m_State = WorldServerState::Stopping;
}

bool CWorldServer::Shutdown()
{
    if (m_State == WorldServerState::Stopped) return true;
    const bool persist = m_State == WorldServerState::Running ||
                         m_State == WorldServerState::Initialized ||
                         m_State == WorldServerState::Stopping;
    m_State = WorldServerState::Stopping;
    bool clean = true;
    if (m_Server) { m_Server->StopListening(); static_cast<void>(m_Server->QuitAllClients()); }
    if (m_Login) static_cast<void>(m_Login->Close());
    if (persist && m_Game && m_Database) {
        if (!m_PendingSave) m_PendingSave = m_Game->GenerateDBData(true);
        const WorldSaveReport saved = CSaveDB::Save(*m_Database, *m_PendingSave);
        clean = saved.success;
        if (clean) m_PendingSave.reset();
    }
    m_Io.stop();
    m_State = clean ? WorldServerState::Stopped : WorldServerState::Error;
    return clean;
}
