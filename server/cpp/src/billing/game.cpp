#include "game.h"

#include "appbilling/billingmessage.h"
#include "appbilling/billingplayermanager.h"
#include "appbilling/playerfillmgr.h"
#include "appbilling/servermessage.h"
#include "../dbaccess/dbbilling/rsplayerfillmgr.h"
#include "../nets/mysocket.h"

#include <spdlog/spdlog.h>

#include <chrono>
#include <fstream>
#include <thread>
#include <utility>

namespace Billing
{
namespace
{
std::uint32_t LegacyTickMs()
{
    using namespace std::chrono;
    return static_cast<std::uint32_t>(
        duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count());
}

template <typename T>
bool ReadPair(std::istream& input, T& value)
{
    std::string label;
    return static_cast<bool>(input >> label >> value);
}
}

CGame::CGame() = default;

CGame::~CGame()
{
    static_cast<void>(Release());
}

bool CGame::LoadSetup(const std::filesystem::path& path)
{
    std::ifstream input(path);
    if (!input.is_open()) {
        const auto fallback = path.parent_path() / "setup.ini";
        input.open(fallback);
    }
    if (!input.is_open()) return false;

    return ReadPair(input, m_Setup.gsListenPort) &&
           ReadPair(input, m_Setup.sqlServerIp) &&
           ReadPair(input, m_Setup.sqlUserName) &&
           ReadPair(input, m_Setup.sqlPassword) &&
           ReadPair(input, m_Setup.databaseName) &&
           ReadPair(input, m_Setup.logServerEnabled) &&
           ReadPair(input, m_Setup.logServerIp) &&
           ReadPair(input, m_Setup.logServerUserName) &&
           ReadPair(input, m_Setup.logServerPassword) &&
           ReadPair(input, m_Setup.logDatabaseName) &&
           ReadPair(input, m_Setup.refreshInfoTimeMs) &&
           ReadPair(input, m_Setup.saveInfoTimeMs) &&
           ReadPair(input, m_Setup.saveLogServerTimeMs) &&
           ReadPair(input, m_Setup.databaseIoThreadCount) &&
           ReadPair(input, m_Setup.playerFillCheckEnabled);
}

BillingDatabaseSettings CGame::DatabaseSettings() const
{
    return BillingDatabaseSettings{
        .account = BillingDatabaseConnection{
            .host = m_Setup.sqlServerIp,
            .database = m_Setup.databaseName,
            .user = m_Setup.sqlUserName,
            .password = m_Setup.sqlPassword},
        .cashLog = BillingDatabaseConnection{
            .host = m_Setup.logServerIp,
            .database = m_Setup.logDatabaseName,
            .user = m_Setup.logServerUserName,
            .password = m_Setup.logServerPassword}};
}

BillingGameResult CGame::Initialize(const std::filesystem::path& runtimeDirectory)
{
    if (m_Initialized && !m_Released) return {};
    if (!LoadSetup(runtimeDirectory / "Setup.ini")) {
        return {BillingGameStatus::ConfigurationError,
                "не прочитаны 15 обязательных пар Setup.ini"};
    }
    if (m_Setup.gsListenPort == 0 || m_Setup.databaseIoThreadCount == 0) {
        return {BillingGameStatus::ConfigurationError,
                "в Setup.ini не задан listener или число DB-workers"};
    }

    m_GameServer = std::make_unique<BillingNet::CServerForGS>(
        m_Io.get_executor(), LegacyTickMs());
    std::error_code hostError;
    if (!m_GameServer->Host(m_Setup.gsListenPort, std::nullopt,
                            kDefaultSocketType, true, hostError)) {
        return {BillingGameStatus::NetworkError,
                "не открыт GameServer listener: " + hostError.message()};
    }

    // В исходном CGame allow-list читался после успешного Host и был нефатален.
    const auto allowed = runtimeDirectory / "GSInfoSetup.ini";
    if (!m_GameServer->LoadAllowedClients(allowed.string())) {
        const auto fallback = runtimeDirectory / "gsinfosetup.ini";
        if (!m_GameServer->LoadAllowedClients(fallback.string())) {
            spdlog::warn("BillingServer: GSInfoSetup.ini недоступен");
        }
    }

    const BillingDatabaseSettings settings = DatabaseSettings();
    const RsPlayerAccountFactory accountFactory =
        MakeMssqlOdbcRsPlayerAccountFactory(settings);
    m_PlayerManager = std::make_unique<CBillingPlayerManager>(
        m_GameServer->CommandHandle(), accountFactory);
    if (!m_PlayerManager->Start(m_Setup.databaseIoThreadCount,
                                m_Setup.logServerEnabled,
                                m_Setup.saveLogServerTimeMs)) {
        return {BillingGameStatus::RuntimeError,
                "не запущены Billing DB-workers"};
    }

    if (m_Setup.playerFillCheckEnabled) {
        m_PlayerFill = std::make_unique<CPlayerFillMgr>(
            m_GameServer->CommandHandle(),
            MakeMssqlOdbcRsPlayerFillFactory(settings.account, settings.transport),
            accountFactory);
        if (!m_PlayerFill->Start()) {
            return {BillingGameStatus::RuntimeError,
                    "не запущен PlayerFill-worker"};
        }
    }

    m_Network.nextAccept = std::chrono::steady_clock::now();
    m_LastRefreshInfoMs = LegacyTickMs();
    m_Initialized = true;
    m_Released = false;
    spdlog::info("BillingServer: инициализация завершена, порт {}",
                 m_Setup.gsListenPort);
    return {};
}

void CGame::PumpNetwork()
{
    if (!m_GameServer) return;
    m_Io.restart();
    static_cast<void>(m_Io.poll());

    const auto now = std::chrono::steady_clock::now();
    if (m_Network.accepted) {
        AcceptedTransport accepted = std::move(*m_Network.accepted);
        m_Network.accepted.reset();
        m_Network.acceptPending = false;
        m_Network.nextAccept = now + std::chrono::milliseconds(kAcceptThreadDelayMs);
        if (!accepted.error) {
            static_cast<void>(m_GameServer->QueueAccepted(
                std::move(accepted.socket), accepted.peer, LegacyTickMs()));
        } else {
            ++m_Network.failures;
        }
    }

    if (!m_Network.acceptPending && now >= m_Network.nextAccept) {
        switch (m_GameServer->BeginAccept()) {
        case AcceptStart::Pending:
            m_Network.acceptPending = true;
            asio::co_spawn(m_Io, m_GameServer->AcceptOne(),
                [this](std::exception_ptr error, AcceptedTransport result) {
                    if (error) result.error = std::make_error_code(std::errc::io_error);
                    m_Network.accepted = std::move(result);
                });
            break;
        case AcceptStart::AtCapacity:
            m_Network.nextAccept = now +
                std::chrono::milliseconds(kAcceptAtCapacityDelayMs);
            break;
        case AcceptStart::NotListening:
            ++m_Network.failures;
            break;
        }
    }

    ServerSnapshot snapshot = m_GameServer->ProcessCommandSnapshot(LegacyTickMs());
    m_Network.failures += snapshot.errors.size();
    const ServerCommandHandle commands = m_GameServer->CommandHandle();
    for (ServerIoAction& action : snapshot.ioActions) {
        ++m_Network.ioTasks;
        asio::co_spawn(m_Io, RunServerIoAction(std::move(action), commands),
            [this](std::exception_ptr error) {
                if (m_Network.ioTasks != 0) --m_Network.ioTasks;
                if (error) ++m_Network.failures;
            });
    }
    static_cast<void>(m_Io.poll());
}

void CGame::ProcessMessages()
{
    if (!m_GameServer) return;
    std::int32_t remaining = m_GameServer->PendingMessages();
    while (remaining-- > 0) {
        if (auto message = m_GameServer->PopReceivedMessage()) {
            static_cast<void>(message->Run(*this));
        }
    }
}

void CGame::OnBilling(BillingNet::CMessage& message)
{
    if (m_PlayerManager) OnBillingMessage(message, *m_PlayerManager);
}

void CGame::OnServer(BillingNet::CMessage& message)
{
    static_cast<void>(OnServerMessage(message, m_GameServer.get()));
}

BillingGameResult CGame::RunTurn()
{
    if (!m_Initialized || m_Released) {
        return {BillingGameStatus::RuntimeError,
                "BillingServer не инициализирован или уже остановлен"};
    }
    try {
        PumpNetwork();
        ProcessMessages();
        const std::uint32_t now = LegacyTickMs();
        if (m_Setup.refreshInfoTimeMs <
            static_cast<std::uint32_t>(now - m_LastRefreshInfoMs)) {
            m_LastRefreshInfoMs = now;
            spdlog::info("BillingServer: подключено GameServer {}",
                         m_GameServer ? m_GameServer->ClientCount() : 0);
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(1));
        return {};
    } catch (const std::exception& error) {
        return {BillingGameStatus::RuntimeError, error.what()};
    }
}

BillingGameResult CGame::Release()
{
    if (m_Released) return {};
    m_Released = true;

    if (m_PlayerManager) m_PlayerManager->RequestStop();
    if (m_PlayerFill) m_PlayerFill->RequestStop();
    if (m_GameServer) {
        m_GameServer->StopListening();
        static_cast<void>(m_GameServer->QuitAllClients());
        // Один snapshot публикует исходный QUITALL до уничтожения транспорта.
        static_cast<void>(m_GameServer->ProcessCommandSnapshot(LegacyTickMs()));
    }
    if (m_PlayerManager) m_PlayerManager->Join();
    if (m_PlayerFill) m_PlayerFill->Join();

    m_Io.stop();
    m_PlayerFill.reset();
    m_PlayerManager.reset();
    m_GameServer.reset();
    spdlog::info("BillingServer: завершён");
    return {};
}
}
