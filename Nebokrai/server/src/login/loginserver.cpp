#include "loginserver.h"

#include "applogin/acclogthread.h"
#include "applogin/gasthread.h"
#include "applogin/message/asmessage.h"
#include "applogin/message/gmmessage.h"
#include "applogin/message/logmessage.h"
#include "applogin/message/servermessage.h"
#include "authhandler.h"
#include "authmanager.h"
#include "database_commands.h"
#include "game.h"
#include "loginqueue.h"
#include "servlogqueue.h"
#include "../dbaccess/logindb/rscdkey.h"
#include "../dbaccess/logindb/rscdkey_odbc.h"
#include "../dbaccess/myadobase.h"
#include "../nets/netlogin/message.h"
#include "../nets/netlogin/mynetclient_auth.h"
#include "../nets/netlogin/mynetserver_client.h"
#include "../nets/netlogin/mynetserver_world.h"

#include <spdlog/spdlog.h>

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstring>
#include <ctime>
#include <deque>
#include <future>
#include <limits>
#include <map>
#include <span>
#include <thread>
#include <utility>

namespace Login
{
namespace
{
constexpr std::int32_t kWorldPingRequestMessageType = 0x4FC01;

std::string Text(std::span<const std::uint8_t> bytes)
{
    return {reinterpret_cast<const char*>(bytes.data()), bytes.size()};
}

std::vector<std::uint8_t> Bytes(std::string_view text)
{
    return {reinterpret_cast<const std::uint8_t*>(text.data()),
            reinterpret_cast<const std::uint8_t*>(text.data()) + text.size()};
}

std::string Timestamp()
{
    std::array<char, 32> value{};
    if (CMyAdoBase::GetTimeString(value.data(), value.size()) == nullptr) {
        return {};
    }
    return value.data();
}

std::optional<LocalDateTime> DecodeOleDate(double value)
{
    if (!std::isfinite(value)) {
        return std::nullopt;
    }
    constexpr double kUnixEpochOleDays = 25569.0;
    const double secondsValue = (value - kUnixEpochOleDays) * 86400.0;
    if (secondsValue < static_cast<double>(std::numeric_limits<std::time_t>::min()) ||
        secondsValue > static_cast<double>(std::numeric_limits<std::time_t>::max())) {
        return std::nullopt;
    }
    const std::time_t seconds = static_cast<std::time_t>(std::floor(secondsValue));
    std::tm fields{};
    if (gmtime_r(&seconds, &fields) == nullptr) {
        return std::nullopt;
    }
    const double fraction = secondsValue - std::floor(secondsValue);
    return LocalDateTime{
        static_cast<std::uint16_t>(fields.tm_year + 1900),
        static_cast<std::uint16_t>(fields.tm_mon + 1),
        static_cast<std::uint16_t>(fields.tm_mday),
        static_cast<std::uint16_t>(fields.tm_hour),
        static_cast<std::uint16_t>(fields.tm_min),
        static_cast<std::uint16_t>(fields.tm_sec),
        static_cast<std::uint16_t>(std::clamp(fraction * 1000.0, 0.0, 999.0))};
}

AsRouteResult Route(std::variant<std::int32_t, LoginNet::SendMessageError> result,
                    AsRouteErrorKind kind)
{
    if (const auto* sent = std::get_if<std::int32_t>(&result)) {
        return *sent;
    }
    return AsRouteError{kind, std::get<LoginNet::SendMessageError>(result)};
}
}

class LoginServer::Impl
{
public:
    struct Context final : ILogMessageContext,
                           IGasThreadContext,
                           IGmMessageContext,
                           IServerMessageContext
    {
        explicit Context(Impl& owner) : owner(owner) {}
        Impl& owner;

        bool IsExitWorld(std::span<const std::uint8_t> worldName) const override
        {
            const std::string name = Text(worldName);
            return owner.game.IsExitWorld(name.c_str());
        }
        std::optional<std::vector<std::uint8_t>>
        LoginCdkeyWorldServer(std::span<const std::uint8_t> account) const override
        {
            const std::string key = Text(account);
            const char* world = owner.game.GetLoginCdkeyWorldServer(key.c_str());
            return world == nullptr ? std::nullopt
                                    : std::optional<std::vector<std::uint8_t>>(Bytes(world));
        }
        std::int32_t LoginWorldPlayerNumByName(
            std::span<const std::uint8_t> worldName) const override
        {
            const std::string name = Text(worldName);
            return owner.game.GetLoginWorldPlayerNumByWorldName(name.c_str());
        }
        bool ValidCodeEnabled() const noexcept override
        {
            return owner.game.SetupEx().bValidCode != 0;
        }
        std::int32_t ValidErrorUpperLimit() const noexcept override
        {
            return owner.game.SetupEx().iValidErrUpperLimit;
        }
        std::uint32_t QuestPlayerDataIntervalMs() const noexcept override
        {
            return static_cast<std::uint32_t>(owner.game.SetupEx().lQuestPlayerDataInterval);
        }
        std::uint32_t MatrixTimeoutMs() const noexcept override
        {
            return static_cast<std::uint32_t>(owner.game.SetupEx().matrix_timeout);
        }
        std::uint32_t ValidCodeOvertimeMs() const noexcept override
        {
            return static_cast<std::uint32_t>(owner.game.SetupEx().lValidCodeOvertime);
        }
        std::optional<std::int32_t> InsideUseMode() const noexcept override
        {
            return owner.game.Setup().m_lIsInsideUse;
        }
        bool HasRsCdKeyOwner() const noexcept override { return owner.rsCdKey != nullptr; }
        std::vector<std::uint8_t> FixPtAccount(
            std::span<const std::uint8_t> account) override
        {
            std::array<char, 256> value{};
            const std::string source = Text(account);
            std::memcpy(value.data(), source.data(),
                        std::min(source.size(), value.size() - 1U));
            owner.rsCdKey->FixPtAcc(value.data(), value.size());
            return Bytes(value.data());
        }
        double BanVariantTime(std::span<const std::uint8_t> account) override
        {
            double value = 0.0;
            const std::string key = Text(account);
            owner.rsCdKey->GetBanTime(key.c_str(), &value);
            return value;
        }
        std::optional<LocalDateTime> DecodeVariantTime(double variantTime) override
        {
            return DecodeOleDate(variantTime);
        }
        bool IpIsAllowed(std::uint32_t clientIp) override
        {
            return owner.rsCdKey->IPIsAllowed(clientIp);
        }
        bool IpIsForbidden(std::uint32_t clientIp) override
        {
            return owner.rsCdKey->IPIsForbidded(clientIp);
        }
        bool IsBetweenIp(std::span<const std::uint8_t> account,
                         std::uint32_t clientIp) override
        {
            const std::string key = Text(account);
            return owner.rsCdKey->IsBetweenIP(key.c_str(), clientIp);
        }
        bool MatrixUsed(std::span<const std::uint8_t> account) override
        {
            const std::string key = Text(account);
            return owner.rsCdKey->matrix_used(key.c_str());
        }
        std::optional<std::vector<std::uint8_t>> ValidateLocalPassword(
            std::span<const std::uint8_t> account,
            std::span<const std::uint8_t> passwordHex) override
        {
            const std::string key = Text(account);
            const std::string password = Text(passwordHex);
            std::array<char, 256> output{};
            if (!owner.rsCdKey->ValidateLocalPassord(
                    key.c_str(), password.c_str(), output.data(), output.size())) {
                return std::nullopt;
            }
            return Bytes(output.data());
        }

        bool IsConnectAS() const noexcept override { return owner.game.IsConnectAS(); }
        ClientSendQueue* AuthSendQueue() noexcept override
        {
            auto* client = owner.game.GetAuthClient();
            return client == nullptr ? nullptr : &client->SendQueue();
        }
        IAuthListener* AuthListener() noexcept override { return owner.authHandler.get(); }
        LoginNet::AuthClientEventPublisher AuthEventPublisher() const override
        {
            const auto* client = owner.game.GetAuthClient();
            return client == nullptr ? LoginNet::AuthClientEventPublisher{}
                                     : client->EventPublisher();
        }
        void KickOut(std::span<const std::uint8_t> account) override
        {
            const std::string key = Text(account);
            static_cast<void>(owner.game.KickOut(key.c_str()));
        }
        bool L2WPlayerBaseSend(std::span<const std::uint8_t> worldServer,
                               std::span<const std::uint8_t> account) override
        {
            const std::string world = Text(worldServer);
            const std::string key = Text(account);
            return owner.game.L2W_PlayerBase_Send(world.c_str(), key.c_str());
        }
        void SetLoginCdkeyWorldServer(std::span<const std::uint8_t> account,
                                      std::span<const std::uint8_t> worldServer) override
        {
            const std::string key = Text(account);
            const std::string world = Text(worldServer);
            owner.game.SetLoginCdkeyWorldServer(key.c_str(), world.c_str());
        }
        void L2WQuestDetailSend(
            std::optional<std::span<const std::uint8_t>> worldServer,
            std::span<const std::uint8_t> account,
            std::int32_t playerId,
            std::uint32_t clientIp) override
        {
            if (!worldServer) return;
            const std::string world = Text(*worldServer);
            const std::string key = Text(account);
            static_cast<void>(owner.game.L2W_QuestDetail_Send(
                world.c_str(), key.c_str(), playerId, clientIp));
        }
        void SendToClientCdkey(const LoginNet::CMessage& message,
                               std::span<const std::uint8_t> account) override
        {
            if (auto* server = owner.game.GetNetServer_Client()) {
                static_cast<void>(message.SendToClientCdkey(server->CommandHandle(), account));
            }
        }
        PrepareEnterOutcome PrepareEnter(const TagPwdChecked& checked) override
        {
            const std::string account = Text(checked.Account());
            const std::string world = Text(checked.WorldServer());
            return owner.game.PrepareEnter(account.c_str(), checked.ClientIP(),
                                           checked.SocketID(), world.c_str(),
                                           checked.HasMatrix()) == 0
                ? PrepareEnterOutcome::Continue : PrepareEnterOutcome::Finished;
        }
        void EnterGame(const TagPwdChecked& checked, bool) override
        {
            const std::string account = Text(checked.Account());
            const std::string world = Text(checked.WorldServer());
            owner.game.EnterGame(account.c_str(), checked.ClientIP(),
                                 checked.SocketID(), world.c_str());
        }
        MatrixCardValidation ValidateMatrixCard(
            std::span<const std::uint8_t> account,
            std::span<const std::uint8_t, 3> positions,
            std::span<const std::uint8_t, 3> answer) override
        {
            const std::string key = Text(account);
            const bool accepted = owner.rsCdKey->matrix_validate(
                key.c_str(), positions.data(), answer.data());
            return {MatrixCardValidationKind::Compared, accepted, 0, 0};
        }
        void SendToClient(const LoginNet::CMessage& message,
                          std::int32_t socketId) override
        {
            if (auto* server = owner.game.GetNetServer_Client()) {
                static_cast<void>(message.SendToClientSocket(server->CommandHandle(), socketId));
            }
        }

        std::optional<std::int32_t> ServerVersion() const override
        {
            return owner.game.Setup()._server_version;
        }
        std::uint32_t ValidErrorStayTimeMs() const noexcept override
        {
            return owner.game.SetupEx().dwValidErrStayTime;
        }
        std::int32_t WorldIDByName(std::span<const std::uint8_t> worldName) const override
        {
            const std::string name = Text(worldName);
            return owner.game.GetWorldIDByName(name.c_str());
        }
        void SendToWorld(const LoginNet::CMessage& message,
                         std::int32_t worldId) override
        {
            static_cast<void>(owner.game.SendMsg2World(message, worldId));
        }
        void ClearCdkey(std::span<const std::uint8_t> account) override
        {
            const std::string key = Text(account);
            owner.game.ClearCDKey(key.c_str());
        }
        bool ClearLoginCdkey(std::span<const std::uint8_t> account) override
        {
            const std::string key = Text(account);
            const bool existed = owner.game.GetLoginCdkeyWorldServer(key.c_str()) != nullptr;
            owner.game.ClearLoginCdkey(key.c_str());
            return existed;
        }
        void QuitClientByCdkey(std::span<const std::uint8_t> account) override
        {
            if (auto* server = owner.game.GetNetServer_Client()) {
                static_cast<void>(server->QuitClientByMapName(account));
            }
        }
        std::int32_t FindCdkey(std::span<const std::uint8_t> account) const override
        {
            const std::string key = Text(account);
            return owner.game.FindCdkey(key.c_str());
        }
        bool AddCdkey(std::span<const std::uint8_t> account,
                      std::int32_t worldId) override
        {
            const std::string key = Text(account);
            return owner.game.AddCdkey(key.c_str(), worldId);
        }
        void AccountLeaveLog(std::span<const std::uint8_t> account) override
        {
            owner.WriteLeaveLog(account, false);
        }
        void LeaveLog(std::span<const std::uint8_t> account) override
        {
            owner.WriteLeaveLog(account, true);
        }
        void RoleEnterLog(std::span<const std::uint8_t> account,
                          std::span<const std::uint8_t> roleName,
                          std::uint8_t level,
                          std::int32_t worldNumber) override
        {
            owner.WriteRoleEnterLog(account, roleName, level, worldNumber);
        }
        void L2WCreateRoleSend(
            std::optional<std::span<const std::uint8_t>> worldServer,
            std::span<const std::uint8_t> account,
            LoginNet::CMessage& source) override
        {
            if (!worldServer) return;
            const std::string world = Text(*worldServer);
            const std::string key = Text(account);
            static_cast<void>(owner.game.L2W_CreateRole_Send(
                world.c_str(), key.c_str(), source));
        }
        void L2WDeleteRoleSend(
            std::optional<std::span<const std::uint8_t>> worldServer,
            std::span<const std::uint8_t> account,
            std::int32_t playerId,
            std::uint32_t clientIp) override
        {
            if (!worldServer) return;
            const std::string world = Text(*worldServer);
            const std::string key = Text(account);
            static_cast<void>(owner.game.L2W_DeleteRole_Send(
                world.c_str(), key.c_str(), playerId, clientIp));
        }
        void L2WRestoreRoleSend(
            std::optional<std::span<const std::uint8_t>> worldServer,
            std::span<const std::uint8_t> account,
            std::uint32_t playerId) override
        {
            if (!worldServer) return;
            const std::string world = Text(*worldServer);
            const std::string key = Text(account);
            static_cast<void>(owner.game.L2W_RestoreRole_Send(
                world.c_str(), key.c_str(), playerId));
        }
        void AddWorldInfoToMsg(LoginNet::CMessage& message,
                               std::span<const std::uint8_t> account) override
        {
            const std::string key = Text(account);
            owner.game.AddWorldInfoToMsg(message, key.c_str());
        }

        std::int32_t AreaID() const noexcept override { return owner.game.SetupEx().iAreaId; }
        std::optional<bool> CdKeyBan(std::span<const std::uint8_t> account,
                                     std::int32_t minutes) override
        {
            const std::string key = Text(account);
            return owner.rsCdKey->CDKeyBan(key.c_str(), minutes);
        }

        void SetWorldSocketMapID(std::int32_t socketId, std::int32_t worldId) override
        {
            if (auto* server = owner.game.GetNetServer_World()) {
                static_cast<void>(server->SetClientMapID(socketId, worldId));
            }
        }
        std::int32_t AddWorld(std::int32_t worldId,
                              std::span<const std::uint8_t> worldName) override
        {
            const std::string name = Text(worldName);
            return owner.game.AddWorld(worldId, name.c_str());
        }
        void QueueWorldConnectedOperatorLog(std::span<const std::uint8_t> name) override
        {
            spdlog::info("LoginServer: WorldServer {} подключён", Text(name));
        }
        void SendToWorldSocket(const LoginNet::CMessage& message,
                               std::int32_t socketId) override
        {
            if (auto* server = owner.game.GetNetServer_World()) {
                static_cast<void>(message.SendToWorldSocket(server->CommandHandle(), socketId));
            }
        }
        std::optional<std::vector<std::uint8_t>> WorldNameByID(
            std::int32_t worldId) const override
        {
            const char* name = owner.game.GetWorldNameByID(worldId);
            return name == nullptr ? std::nullopt
                                   : std::optional<std::vector<std::uint8_t>>(Bytes(name));
        }
        void QueueWorldLostOperatorLog(std::span<const std::uint8_t> name) override
        {
            spdlog::warn("LoginServer: потеряно соединение с WorldServer {}", Text(name));
        }
        void ClearCdkeysByWorldID(std::int32_t worldId) override
        {
            owner.game.ClearCDKeyByWorldServerID(worldId);
        }
        std::int32_t DelWorld(std::int32_t worldId) override
        {
            return owner.game.DelWorld(worldId);
        }
        void QueueOnlineUserDatabaseUpdate(
            std::int32_t worldId,
            std::vector<std::vector<std::uint8_t>> accounts) override
        {
            owner.onlineUpdates.emplace_back(worldId, std::move(accounts));
        }
        void AppendPingWorldServerInfo(PingWorldServerInfo info) override
        {
            owner.pingWorlds.push_back(std::move(info));
        }
        std::optional<std::uint32_t> ServerInfoLogTime() const noexcept override
        {
            return owner.game.Setup().dwServerInfoLogTime;
        }
        void PushServerInfoLog(ServLog record) override
        {
            owner.serverLogs.Push(std::move(record));
        }

        std::string VerificationAddress() const override
        {
            return owner.game.Setup().m_strVerificationAddr;
        }
        std::int32_t VerificationSignUpper() const noexcept override
        {
            return owner.game.Setup().m_lVerifiSignUpper;
        }
        bool ExecuteProce(std::string nickname, std::string clientIp,
                          char* passwordHex, std::int32_t mode) override
        {
            return owner.game.ExecuteProce(std::move(nickname), std::move(clientIp),
                                           passwordHex, mode);
        }
    };

    struct AsContext final : IAsMessageContext
    {
        explicit AsContext(Impl& owner) : owner(owner) {}
        Impl& owner;
        void DisconnectAuth() override { owner.game.DisconnectAS(); }
        bool StartReconnectThread() override { return owner.StartReconnect(); }
        std::int32_t WorldIDByName(
            std::span<const std::uint8_t> worldName) const override
        {
            const std::string name = Text(worldName);
            return owner.game.GetWorldIDByName(name.c_str());
        }
        std::int32_t AreaID() const noexcept override
        {
            return owner.game.SetupEx().iAreaId;
        }
        AsRouteResult SendToAuth(const LoginNet::CMessage& message) override
        {
            auto* client = owner.game.GetAuthClient();
            if (client == nullptr) return AsRouteError{AsRouteErrorKind::MissingAuthServer, {}};
            return Route(message.SendToAuth(client->SendQueue()),
                         AsRouteErrorKind::SendMessage);
        }
        AsRouteResult SendToWorld(const LoginNet::CMessage& message,
                                  std::int32_t worldId) override
        {
            auto* server = owner.game.GetNetServer_World();
            if (server == nullptr) return AsRouteError{AsRouteErrorKind::MissingWorldServer, {}};
            return Route(message.SendToWorldMap(server->CommandHandle(), worldId),
                         AsRouteErrorKind::SendMessage);
        }
        AsRouteResult SendAllWorld(const LoginNet::CMessage& message) override
        {
            auto* server = owner.game.GetNetServer_World();
            if (server == nullptr) return AsRouteError{AsRouteErrorKind::MissingWorldServer, {}};
            return Route(message.SendAllWorld(server->CommandHandle()),
                         AsRouteErrorKind::SendMessage);
        }
    };

    struct AuthContext final : IAuthHandlerContext
    {
        explicit AuthContext(Impl& owner) : owner(owner) {}
        Impl& owner;
        void PushBackPwdChecked(TagPwdChecked checked) override
        {
            owner.queue->PushBackPwdChecked(
                std::move(checked),
                [this](std::span<const std::uint8_t> account) {
                    owner.context->KickOut(account);
                });
        }
        std::variant<std::int32_t, LoginNet::SendMessageError>
        SendToClient(const LoginNet::CMessage& message, std::int32_t socketId) override
        {
            auto* server = owner.game.GetNetServer_Client();
            return server == nullptr
                ? std::variant<std::int32_t, LoginNet::SendMessageError>(std::int32_t{0})
                : message.SendToClientSocket(server->CommandHandle(), socketId);
        }
        PasswordFailureOutcome RegisterPasswordFailure(
            std::span<const std::uint8_t> account) override
        {
            const auto& setup = owner.game.Setup();
            if (setup.lforbitTime == 0) {
                return {PasswordFailureOutcomeKind::Disabled, 0, false};
            }
            const std::string key = Text(account);
            auto [it, inserted] = owner.passwordFailures.emplace(key, 1);
            if (inserted) {
                return {PasswordFailureOutcomeKind::Counted, 1, false};
            }
            if (it->second < setup.lforbitNum) {
                ++it->second;
                return {PasswordFailureOutcomeKind::Counted, it->second, false};
            }
            const bool banned = owner.rsCdKey->CDKeyBan(key.c_str(), setup.lforbitTime);
            owner.passwordFailures.erase(it);
            return {PasswordFailureOutcomeKind::BanAttempted,
                    setup.lforbitNum,
                    banned};
        }
        void PushAuthHandlerNotice(AuthHandlerNotice notice) override
        {
            owner.authNotices.push_back(std::move(notice));
        }
    };

    struct Router final : LoginNet::ILoginMessageHandlers
    {
        explicit Router(Impl& owner) : owner(owner) {}
        Impl& owner;
        void OnAuth(LoginNet::CMessage& message) override
        {
            const auto result = owner.asHandler->OnASMessage(message);
            if (std::holds_alternative<AsMessageError>(result)) {
                spdlog::error("LoginServer: ошибка Auth-message {}", message.MessageType());
            }
        }
        void OnGM(LoginNet::CMessage& message) override
        {
            static_cast<void>(owner.gmHandler->OnGMMessage(message));
        }
        void OnGMA(LoginNet::CMessage& message) override
        {
            static_cast<void>(owner.asHandler->OnGMAMessage(message));
        }
        void OnLog(LoginNet::CMessage& message) override
        {
            if (auto error = owner.logHandler->OnLogMessage(message)) {
                spdlog::error("LoginServer: ошибка Log-message {}: {}",
                              message.MessageType(), error->detail);
            }
        }
        void OnServer(LoginNet::CMessage& message) override
        {
            owner.serverHandler->OnServerMessage(message);
        }
    };

    LoginServerResult Initialize(const std::filesystem::path& runtimeDirectory);
    LoginServerResult RunTurn();
    LoginServerResult Release();

    bool StartReconnect();
    void WriteLeaveLog(std::span<const std::uint8_t> account, bool role);
    void WriteRoleEnterLog(std::span<const std::uint8_t> account,
                           std::span<const std::uint8_t> role,
                           std::uint8_t level,
                           std::int32_t world);
    void ProcessWorldPing();
    void ProcessDatabaseQueues();
    void CollectDatabaseTasks(bool waitForAll);
    void ProcessMessages();
    void PumpNetwork();
    bool UpsertServerInfo(std::string_view ip,
                          std::int32_t serverType,
                          std::int32_t serverNumber,
                          std::uint32_t onlineUsers,
                          std::int32_t worldId);

    struct ServerRuntime
    {
        CServer* server{};
        bool acceptPending{};
        std::optional<AcceptedTransport> accepted;
        std::chrono::steady_clock::time_point nextAccept{};
        std::size_t ioTasks{};
        std::size_t failures{};
    };

    CGame game;
    std::filesystem::path runtimeDirectory;
    std::unique_ptr<CLoginQueue> queue;
    AuthManager authManager;
    std::unique_ptr<MssqlOdbcRsCdKeyDatabase> rsDatabase;
    std::unique_ptr<MssqlOdbcRsCdKeyDatabase> serverInfoDatabase;
    std::unique_ptr<CRsCDKey> rsCdKey;
    std::unique_ptr<Context> context;
    std::unique_ptr<AsContext> asContext;
    std::unique_ptr<AuthContext> authContext;
    std::unique_ptr<AuthHandler> authHandler;
    std::unique_ptr<AsMessageHandler> asHandler;
    std::unique_ptr<GmMessageHandler> gmHandler;
    std::unique_ptr<LogMessageHandler> logHandler;
    std::unique_ptr<ServerMessageHandler> serverHandler;
    std::unique_ptr<Router> router;
    std::unique_ptr<CGasThread> gasThread;
    std::unique_ptr<AccLogThread> accountLogThread;
    std::thread accountLogWorker;
    ServLogQueue serverLogs;
    std::vector<PingWorldServerInfo> pingWorlds;
    std::deque<std::pair<std::int32_t, std::vector<std::vector<std::uint8_t>>>> onlineUpdates;
    std::map<std::string, std::int32_t> passwordFailures;
    std::deque<AuthHandlerNotice> authNotices;
    std::vector<std::future<void>> databaseTasks;
    ServerRuntime clientRuntime;
    ServerRuntime worldRuntime;
    bool reconnectPending{};
    bool authReadPending{};
    bool authWritePending{};
    bool pingInFlight{};
    std::uint32_t lastPingWorldMs{};
    std::uint32_t lastServerInfoLogMs{};
    bool initialized{};
    bool released{};
};

LoginServerResult LoginServer::Impl::Initialize(
    const std::filesystem::path& directory)
{
    if (initialized) {
        return {};
    }
    runtimeDirectory = directory;
    queue = std::make_unique<CLoginQueue>(runtimeDirectory);
    game.AttachLoginQueue(queue.get());

    if (!game.LoadSetup() || !game.LoadSetupEx() ||
        !game.load_listen_port(runtimeDirectory) || !game.LoadWorldSetup() ||
        !game.LoadASList(runtimeDirectory / "aslist.ini")) {
        return {LoginServerStatus::ConfigurationError,
                "не загружен обязательный runtime-конфиг LoginServer"};
    }
    game.SetListWorldInfoBySetup();
    queue->SetWorldCount(static_cast<std::uint32_t>(
        std::max<std::size_t>(game.ConfiguredWorldCount(), 1U)));
    static_cast<void>(authManager.Init(game.Setup().authTimeOut));

    const auto& setup = game.Setup();
    static_cast<void>(CMyAdoBase::Initialize(setup.strSqlConType,
                           setup.strSqlServerIP,
                           setup.strDBName,
                           setup.strSqlUserName,
                           setup.strSqlPassWord,
                           "0",
                           "SSPI"));
    rsDatabase = std::make_unique<MssqlOdbcRsCdKeyDatabase>(
        setup.strSqlServerIP,
        setup.strDBName,
        setup.strSqlUserName,
        setup.strSqlPassWord);
    if (setup.dwServerInfoLogTime != 0) {
        serverInfoDatabase = std::make_unique<MssqlOdbcRsCdKeyDatabase>(
            setup.strServerInfoLogIP,
            setup.strServerInfoLogDB,
            setup.strServerInfoLogUID,
            setup.strServerInfoLogPWD);
    }
    rsCdKey = std::make_unique<CRsCDKey>();
    CRsCDKey::Configure({setup.bCheckForbidIP,
                         setup.bCheckAllowIP,
                         setup.bCheckBetweenIP});
    CRsCDKey::SetDatabase(rsDatabase.get());

    try {
        if (!rsDatabase->Execute("DELETE FROM online_user")) {
            return {LoginServerStatus::DatabaseError,
                    "не очищена таблица online_user"};
        }
    } catch (const std::exception& error) {
        return {LoginServerStatus::DatabaseError, error.what()};
    }

    context = std::make_unique<Context>(*this);
    asContext = std::make_unique<AsContext>(*this);
    authContext = std::make_unique<AuthContext>(*this);
    authHandler = std::make_unique<AuthHandler>(*authContext);
    asHandler = std::make_unique<AsMessageHandler>(*asContext, authManager, *authHandler);
    gmHandler = std::make_unique<GmMessageHandler>(*context);
    logHandler = std::make_unique<LogMessageHandler>(*context, *queue);
    serverHandler = std::make_unique<ServerMessageHandler>(*context);
    router = std::make_unique<Router>(*this);
    gasThread = std::make_unique<CGasThread>(*queue, *context);

    if (!game.InitNetServer_World() || !game.InitNetServer_Client()) {
        return {LoginServerStatus::NetworkError,
                "не открыты Client/World listener-ы LoginServer"};
    }

    try {
        auto connected = asio::co_spawn(game.IoContext(),
                                        game.InitAuthClient(),
                                        asio::use_future);
        game.IoContext().run();
        game.IoContext().restart();
        static_cast<void>(connected.get());
    } catch (const std::exception& error) {
        return {LoginServerStatus::NetworkError,
                std::string("ошибка подключения AuthServer: ") + error.what()};
    }

    clientRuntime.server = game.GetNetServer_Client();
    worldRuntime.server = game.GetNetServer_World();
    clientRuntime.nextAccept = std::chrono::steady_clock::now();
    worldRuntime.nextAccept = clientRuntime.nextAccept;

    accountLogThread = std::make_unique<AccLogThread>(game.AccountLogs());
    try {
        accountLogWorker = std::thread([this] { accountLogThread->Run(); });
    } catch (const std::exception& error) {
        return {LoginServerStatus::RuntimeError,
                std::string("не запущен журнал аккаунтов: ") + error.what()};
    }

    initialized = true;
    released = false;
    spdlog::info("LoginServer: инициализация завершена");
    return {};
}

bool LoginServer::Impl::StartReconnect()
{
    if (reconnectPending || released) {
        return false;
    }
    reconnectPending = true;
    asio::co_spawn(
        game.IoContext(),
        game.ReconnectAS(),
        [this](std::exception_ptr error, bool connected) {
            reconnectPending = false;
            if (error) {
                try {
                    std::rethrow_exception(error);
                } catch (const std::exception& exception) {
                    spdlog::error("LoginServer: переподключение Auth: {}",
                                  exception.what());
                }
            } else if (!connected) {
                spdlog::warn("LoginServer: AuthServer пока недоступен");
            }
        });
    return true;
}

void LoginServer::Impl::WriteLeaveLog(std::span<const std::uint8_t> account,
                                      bool role)
{
    const std::string timestamp = Timestamp();
    if (timestamp.empty()) return;
    game.AccountLogs().Push(AccLogRecord{
        .kind = role ? AccLogKind::SessionLeave : AccLogKind::AccountLeave,
        .account = Text(account),
        .recordedAt = timestamp,
    });
}

void LoginServer::Impl::WriteRoleEnterLog(
    std::span<const std::uint8_t> account,
    std::span<const std::uint8_t> role,
    std::uint8_t level,
    std::int32_t world)
{
    const std::string timestamp = Timestamp();
    if (timestamp.empty()) return;
    game.AccountLogs().Push(AccLogRecord{
        .kind = AccLogKind::RoleEnter,
        .account = Text(account),
        .recordedAt = timestamp,
        .roleName = Text(role),
        .roleLevel = level,
        .worldNumber = world,
    });
}

void LoginServer::Impl::ProcessWorldPing()
{
    const std::uint32_t now = AuthManager::LegacyTickMs();
    if (lastPingWorldMs == 0) {
        lastPingWorldMs = now;
        return;
    }

    if (!pingInFlight) {
        if (game.Setup().dwPingWorldServerTime >=
            static_cast<std::uint32_t>(now - lastPingWorldMs)) {
            return;
        }
        lastPingWorldMs = now;
        pingInFlight = true;
        pingWorlds.clear();

        if (auto* world = game.GetNetServer_World()) {
            const LoginNet::CMessage request(kWorldPingRequestMessageType);
            const auto sent = request.SendAllWorld(world->CommandHandle());
            if (std::holds_alternative<LoginNet::SendMessageError>(sent)) {
                spdlog::error("LoginServer: не сформирован запрос ping WorldServer");
            }
        }
        return;
    }

    const std::size_t expectedWorlds = game.ActiveWorldCount();
    if (expectedWorlds <= pingWorlds.size()) {
        pingInFlight = false;
    }
    if (game.Setup().dwPingWorldServerErrorTime <
        static_cast<std::uint32_t>(now - lastPingWorldMs)) {
        if (pingInFlight) {
            spdlog::warn("LoginServer: ping WorldServer завершён по таймауту: ответов {} из {}",
                         pingWorlds.size(), expectedWorlds);
        }
        pingInFlight = false;
    }
}

bool LoginServer::Impl::UpsertServerInfo(std::string_view ip,
                                         std::int32_t serverType,
                                         std::int32_t serverNumber,
                                         std::uint32_t onlineUsers,
                                         std::int32_t worldId)
{
    if (!login_database::IsBoundedValue(ip, login_database::kServerIpMaxBytes) ||
        serverType < 0 || serverType > 2) {
        return false;
    }

    const std::string quotedIp = "'" + login_database::EscapeSqlLiteral(ip) + "'";
    std::string predicate = "server_ip=" + quotedIp +
                            " AND server_type=" + std::to_string(serverType);
    if (serverType != 0) {
        predicate += " AND server_num=" + std::to_string(serverNumber);
    }
    if (serverType == 2) {
        predicate += " AND world_id=" + std::to_string(worldId);
    }

    if (!serverInfoDatabase) {
        return false;
    }
    const auto rows = serverInfoDatabase->Query(
        "SELECT * FROM server_info WHERE " + predicate);
    if (!rows.empty()) {
        return serverInfoDatabase->Execute(
            "UPDATE server_info SET online_user=" + std::to_string(onlineUsers) +
            ", last_time=GETDATE(), server_state=0 WHERE " + predicate);
    }
    return serverInfoDatabase->Execute(
        "INSERT INTO server_info(server_ip,server_type,server_num,online_user,"
        "last_time,server_state,world_id) VALUES(" + quotedIp + "," +
        std::to_string(serverType) + "," + std::to_string(serverNumber) + "," +
        std::to_string(onlineUsers) + ",GETDATE(),0," +
        std::to_string(worldId) + ")");
}

void LoginServer::Impl::ProcessDatabaseQueues()
{
    CollectDatabaseTasks(false);

    while (!onlineUpdates.empty()) {
        auto [worldId, accounts] = std::move(onlineUpdates.front());
        onlineUpdates.pop_front();
        std::vector<std::string> values;
        values.reserve(accounts.size());
        for (const auto& account : accounts) values.push_back(Text(account));

        databaseTasks.push_back(std::async(
            std::launch::async,
            [this, worldId, values = std::move(values)] {
                try {
                    static_cast<void>(rsDatabase->Execute(
                        "DELETE FROM online_user WHERE worldNumber=" +
                        std::to_string(worldId)));
                    for (const std::string& account : values) {
                        static_cast<void>(rsDatabase->Execute(
                            "INSERT INTO online_user(userAccount,worldNumber) VALUES('" +
                            login_database::EscapeSqlLiteral(account) + "'," +
                            std::to_string(worldId) + ")"));
                    }
                } catch (const std::exception& error) {
                    spdlog::error("LoginServer: обновление online_user world={}: {}",
                                  worldId, error.what());
                }
            }));
    }

    const auto interval = game.Setup().dwServerInfoLogTime;
    const std::uint32_t now = AuthManager::LegacyTickMs();
    if (lastServerInfoLogMs == 0) {
        lastServerInfoLogMs = now;
    }
    if (interval == 0 ||
        static_cast<std::uint32_t>(now - lastServerInfoLogMs) < interval) {
        return;
    }
    lastServerInfoLogMs = now;

    if (pingWorlds.empty() && serverLogs.Size() == 0) {
        return;
    }

    const auto* worldServer = game.GetNetServer_World();
    const std::string loginIp = worldServer == nullptr
        ? std::string{} : Text(worldServer->LocalIP());
    const std::uint32_t onlineUsers = game.GetCdkeyCount();
    const std::vector<PingWorldServerInfo> telemetry = pingWorlds;
    databaseTasks.push_back(std::async(
        std::launch::async,
        [this, loginIp, onlineUsers, telemetry] {
            try {
                if (!loginIp.empty()) {
                    static_cast<void>(UpsertServerInfo(
                        loginIp, 0, 0, onlineUsers, 0));
                }
                for (const PingWorldServerInfo& world : telemetry) {
                    const std::string worldIp = Text(world.ip);
                    static_cast<void>(UpsertServerInfo(
                        worldIp, 1, static_cast<std::int32_t>(world.port),
                        world.playerCount, static_cast<std::int32_t>(world.port)));
                    for (const PingGameServerInfo& gameServer : world.gameServers) {
                        static_cast<void>(UpsertServerInfo(
                            Text(gameServer.ip), 2,
                            static_cast<std::int32_t>(gameServer.port),
                            gameServer.playerCount,
                            static_cast<std::int32_t>(world.port)));
                    }
                }

                const std::int32_t count = serverLogs.Size();
                for (std::int32_t i = 0; i < count; ++i) {
                    ServLog record;
                    if (!serverLogs.Pop(record)) continue;
                    static_cast<void>(serverInfoDatabase->Execute(
                        login_database::BuildServerLogInsertSql(
                            record.sourceIp, record.serverType,
                            record.serverNumber, Text(record.description))));
                }
            } catch (const std::exception& error) {
                spdlog::error("LoginServer: запись server-info: {}", error.what());
            }
        }));
}

void LoginServer::Impl::CollectDatabaseTasks(bool waitForAll)
{
    for (auto task = databaseTasks.begin(); task != databaseTasks.end();) {
        if (!waitForAll &&
            task->wait_for(std::chrono::milliseconds(0)) != std::future_status::ready) {
            ++task;
            continue;
        }
        try {
            task->get();
        } catch (const std::exception& error) {
            spdlog::error("LoginServer: аварийное завершение DB-worker: {}", error.what());
        }
        task = databaseTasks.erase(task);
    }
}

void LoginServer::Impl::ProcessMessages()
{
    if (auto* world = game.GetNetServer_World()) {
        std::int32_t remaining = world->PendingMessages();
        while (remaining-- > 0) {
            if (auto message = world->PopReceivedMessage()) {
                static_cast<void>(message->Run(*router));
            }
        }
    }
    if (auto* client = game.GetNetServer_Client()) {
        std::int32_t remaining = client->PendingMessages();
        while (remaining-- > 0) {
            if (auto message = client->PopReceivedMessage()) {
                static_cast<void>(message->Run(*router));
            }
        }
    }
    if (auto* auth = game.GetAuthClient()) {
        std::int32_t remaining = auth->PendingEvents();
        while (remaining-- > 0) {
            auto event = auth->PopEvent();
            if (!event) continue;
            if (auto* message = std::get_if<std::unique_ptr<LoginNet::CMessage>>(
                    &event->payload)) {
                if (*message) static_cast<void>((*message)->Run(*router));
            } else if (auto* reconnected = std::get_if<LoginNet::AuthClientReconnected>(
                           &event->payload)) {
                static_cast<void>(game.ReassignAS(std::move(reconnected->client)));
            }
        }
    }

    static_cast<void>(gasThread->RunOnce());
    static_cast<void>(queue->Run(
        static_cast<ILogMessageContext&>(*context), authManager));

    while (!authNotices.empty()) {
        AuthHandlerNotice notice = std::move(authNotices.front());
        authNotices.pop_front();
        switch (notice.kind) {
        case AuthHandlerNoticeKind::ClientResponseFailed:
            spdlog::error("LoginServer: ответ Auth не отправлен клиенту socket={}",
                          notice.socketId);
            break;
        case AuthHandlerNoticeKind::CdKeyBanFailed:
            spdlog::error("LoginServer: не записана блокировка аккаунта {}",
                          Text(notice.account));
            break;
        case AuthHandlerNoticeKind::CdKeyBanOwnerMissing:
            spdlog::error("LoginServer: отсутствует владелец блокировки аккаунта {}",
                          Text(notice.account));
            break;
        }
    }
}

void LoginServer::Impl::PumpNetwork()
{
    auto& io = game.IoContext();
    io.restart();
    static_cast<void>(io.poll());

    const auto pumpServer = [this, &io](ServerRuntime& runtime) {
        if (runtime.server == nullptr) return;
        const auto now = std::chrono::steady_clock::now();
        if (runtime.accepted) {
            AcceptedTransport accepted = std::move(*runtime.accepted);
            runtime.accepted.reset();
            runtime.acceptPending = false;
            runtime.nextAccept = now + std::chrono::milliseconds(kAcceptThreadDelayMs);
            if (!accepted.error) {
                static_cast<void>(runtime.server->QueueAccepted(
                    std::move(accepted.socket), accepted.peer,
                    AuthManager::LegacyTickMs()));
            }
        }
        if (!runtime.acceptPending && now >= runtime.nextAccept) {
            switch (runtime.server->BeginAccept()) {
            case AcceptStart::Pending:
                runtime.acceptPending = true;
                asio::co_spawn(io, runtime.server->AcceptOne(),
                    [&runtime](std::exception_ptr error, AcceptedTransport result) {
                        if (error) {
                            result.error = std::make_error_code(std::errc::io_error);
                        }
                        runtime.accepted = std::move(result);
                    });
                break;
            case AcceptStart::AtCapacity:
                runtime.nextAccept = now +
                    std::chrono::milliseconds(kAcceptAtCapacityDelayMs);
                break;
            case AcceptStart::NotListening:
                ++runtime.failures;
                break;
            }
        }

        ServerSnapshot snapshot = runtime.server->ProcessCommandSnapshot(
            AuthManager::LegacyTickMs());
        runtime.failures += snapshot.errors.size();
        const ServerCommandHandle commands = runtime.server->CommandHandle();
        for (ServerIoAction& action : snapshot.ioActions) {
            ++runtime.ioTasks;
            asio::co_spawn(io, RunServerIoAction(std::move(action), commands),
                [&runtime](std::exception_ptr error) {
                    if (runtime.ioTasks != 0) --runtime.ioTasks;
                    if (error) ++runtime.failures;
                });
        }
    };

    pumpServer(worldRuntime);
    pumpServer(clientRuntime);

    if (auto* auth = game.GetAuthClient(); auth != nullptr && auth->IsConnected()) {
        if (!authReadPending) {
            authReadPending = true;
            asio::co_spawn(io, auth->ReadOnce(),
                [this](std::exception_ptr error, LoginNet::AuthClientReadResult result) {
                    authReadPending = false;
                    if (error || !result) game.GetAuthClient()->HandleTransportClose();
                });
        }
        if (!authWritePending && auth->SendQueue().Pending() != 0) {
            authWritePending = true;
            asio::co_spawn(io, auth->FlushOutgoing(),
                [this](std::exception_ptr error, ClientFlushResult result) {
                    authWritePending = false;
                    if (error || result.status != ClientFlushStatus::Drained) {
                        game.GetAuthClient()->HandleTransportClose();
                    }
                });
        }
    }
    static_cast<void>(io.poll());
}

LoginServerResult LoginServer::Impl::RunTurn()
{
    if (!initialized || released) {
        return {LoginServerStatus::RuntimeError,
                "LoginServer не инициализирован или уже остановлен"};
    }
    try {
        PumpNetwork();
        ProcessMessages();
        ProcessWorldPing();
        ProcessDatabaseQueues();
        std::this_thread::sleep_for(std::chrono::milliseconds(1));
        return {};
    } catch (const std::exception& error) {
        return {LoginServerStatus::RuntimeError, error.what()};
    }
}

LoginServerResult LoginServer::Impl::Release()
{
    if (released) return {};
    released = true;
    game.ReleaseNetworkOwners();
    game.IoContext().stop();

    game.AccountLogs().Stop();
    if (accountLogWorker.joinable()) accountLogWorker.join();
    accountLogThread.reset();
    CollectDatabaseTasks(true);

    try {
        if (rsDatabase) static_cast<void>(rsDatabase->Execute("DELETE FROM online_user"));
    } catch (const std::exception& error) {
        spdlog::error("LoginServer: финальная очистка online_user: {}", error.what());
    }
    CRsCDKey::SetDatabase(nullptr);
    rsCdKey.reset();
    rsDatabase.reset();
    serverInfoDatabase.reset();
    static_cast<void>(CMyAdoBase::Uninitalize());
    game.AttachLoginQueue(nullptr);
    initialized = false;
    spdlog::info("LoginServer: освобождение завершено");
    return {};
}

LoginServer::LoginServer() : m_Impl(std::make_unique<Impl>()) {}

LoginServer::~LoginServer()
{
    static_cast<void>(m_Impl->Release());
}

LoginServerResult LoginServer::Initialize(const std::filesystem::path& runtimeDirectory)
{
    return m_Impl->Initialize(runtimeDirectory);
}

LoginServerResult LoginServer::RunTurn()
{
    return m_Impl->RunTurn();
}

LoginServerResult LoginServer::Release()
{
    return m_Impl->Release();
}
}
