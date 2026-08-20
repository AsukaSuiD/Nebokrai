#pragma once

#include "../nets/netbilling/message.h"
#include "../nets/netbilling/serverforgs.h"
#include "../dbaccess/dbbilling/rsplayeraccount.h"

#include <asio.hpp>

#include <chrono>
#include <cstdint>
#include <filesystem>
#include <memory>
#include <optional>
#include <string>

namespace Billing
{
class CBillingPlayerManager;
class CPlayerFillMgr;

enum class BillingGameStatus
{
    Ok,
    ConfigurationError,
    NetworkError,
    RuntimeError,
};

struct BillingGameResult
{
    BillingGameStatus status{BillingGameStatus::Ok};
    std::string detail;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return status == BillingGameStatus::Ok;
    }
};

/*
 * Исходный владелец: server/billingserver/billingserver/game.cpp / .h.
 *
 * CGame соединяет уже восстановленные owners в отдельный BillingServer:
 * позиционный Setup.ini, listener GameServer, три FIFO/DB-workers, optional
 * PlayerFill и исходный message snapshot. Windows GameThread/IOCP заменены
 * пошаговым Asio runtime; протокол и порядок доменных эффектов не меняются.
 */
class CGame final : private BillingNet::IBillingMessageHandlers
{
public:
    CGame();
    ~CGame();

    CGame(const CGame&) = delete;
    CGame& operator=(const CGame&) = delete;

    [[nodiscard]] BillingGameResult Initialize(
        const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] BillingGameResult RunTurn();
    [[nodiscard]] BillingGameResult Release();

private:
    struct Setup
    {
        std::uint32_t gsListenPort{};
        std::string sqlServerIp;
        std::string sqlUserName;
        std::string sqlPassword;
        std::string databaseName;
        bool logServerEnabled{};
        std::string logServerIp;
        std::string logServerUserName;
        std::string logServerPassword;
        std::string logDatabaseName;
        std::uint32_t refreshInfoTimeMs{};
        std::uint32_t saveInfoTimeMs{};
        std::uint32_t saveLogServerTimeMs{};
        std::uint32_t databaseIoThreadCount{};
        bool playerFillCheckEnabled{};
    };

    struct NetworkRuntime
    {
        bool acceptPending{};
        std::optional<AcceptedTransport> accepted;
        std::chrono::steady_clock::time_point nextAccept{};
        std::size_t ioTasks{};
        std::size_t failures{};
    };

    [[nodiscard]] bool LoadSetup(const std::filesystem::path& path);
    [[nodiscard]] BillingDatabaseSettings DatabaseSettings() const;
    void PumpNetwork();
    void ProcessMessages();
    void OnBilling(BillingNet::CMessage& message) override;
    void OnServer(BillingNet::CMessage& message) override;

    asio::io_context m_Io;
    Setup m_Setup;
    std::unique_ptr<BillingNet::CServerForGS> m_GameServer;
    std::unique_ptr<CBillingPlayerManager> m_PlayerManager;
    std::unique_ptr<CPlayerFillMgr> m_PlayerFill;
    NetworkRuntime m_Network;
    std::uint32_t m_LastRefreshInfoMs{};
    bool m_Initialized{};
    bool m_Released{};
};
}
