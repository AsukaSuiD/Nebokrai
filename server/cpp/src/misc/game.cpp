#include "game.h"

#include "miscservermessage.h"
#include "onbillserver.h"
#include "othermessage.h"

#include <asio/co_spawn.hpp>
#include <asio/use_future.hpp>

#include <chrono>
#include <bit>
#include <fstream>
#include <future>
#include <sstream>

#include <spdlog/spdlog.h>

namespace Misc
{
namespace
{
constexpr std::uint16_t kReconnectListenPort = 0x092F;
constexpr std::uint32_t kLogRefreshMilliseconds = 10'000;
constexpr std::uint64_t kMemorySyncResetKiB = 40U * 1'024U;

template <typename Result>
Result RunAwaitable(asio::io_context& io, asio::awaitable<Result> operation)
{
    std::future<Result> future = asio::co_spawn(io, std::move(operation), asio::use_future);
    io.restart();
    io.run();
    return future.get();
}

std::optional<std::uint64_t> ReadWorkingSetKiB()
{
    std::ifstream status("/proc/self/status");
    for (std::string line; std::getline(status, line);) {
        if (!line.starts_with("VmRSS:")) continue;
        std::istringstream fields(line.substr(6));
        std::uint64_t value{};
        if (fields >> value) return value;
        return std::nullopt;
    }
    return std::nullopt;
}
}

std::uint32_t LegacyTickMilliseconds() noexcept
{
    return static_cast<std::uint32_t>(
        std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::steady_clock::now().time_since_epoch()).count());
}

CGame::CGame()
    : m_SyncStartTime(LegacyTickMilliseconds())
{
}

MiscGameResult CGame::Initialize(const std::filesystem::path& runtimeDirectory)
{
    m_RuntimeDirectory = runtimeDirectory;
    std::ofstream debug(runtimeDirectory / "debug.txt", std::ios::binary | std::ios::trunc);
    if (!debug) {
        return {MiscGameStatus::ConfigurationError,
                "не удалось создать debug.txt в каталоге MiscServer"};
    }
    debug.close();
    m_ClientClose = true;
    const SetupLoadResult setup = m_Setup.LoadSetup(runtimeDirectory / "setup.ini");
    if (!setup.opened || setup.parsedPairs != 4U) {
        return {MiscGameStatus::ConfigurationError,
                "setup.ini MiscServer не содержит четыре позиционные пары"};
    }

    MiscGameResult connected = RunAwaitable(m_Io, ConnectWorld(std::nullopt, true));
    if (!connected) return connected;
    m_Initialized = true;
    m_Released = false;
    return {};
}

asio::awaitable<MiscGameResult>
CGame::ConnectWorld(std::optional<std::uint16_t> registrationPort,
                    bool sendInitialSync)
{
    // Исходный connect/reconnect сначала удалял прежнего client-owner.
    m_NetClient.reset();
    const IpPortSetup& setup = m_Setup.IpPort();
    if (!setup.worldIp || !setup.worldPort || !setup.localIp || !setup.listenPort) {
        co_return MiscGameResult{MiscGameStatus::ConfigurationError,
                                 "не хватает поля соединения WorldServer"};
    }

    asio::ip::tcp::resolver resolver(m_Io);
    asio::error_code resolveError;
    auto endpoints = resolver.resolve(*setup.worldIp,
                                      std::to_string(*setup.worldPort),
                                      resolveError);
    if (resolveError) {
        co_return MiscGameResult{MiscGameStatus::NetworkError,
                                 "не удалось разрешить IPv4 WorldServer: " +
                                     resolveError.message()};
    }
    std::optional<asio::ip::tcp::endpoint> endpoint;
    for (const auto& candidate : endpoints) {
        if (candidate.endpoint().address().is_v4()) {
            endpoint = candidate.endpoint();
            break;
        }
    }
    if (!endpoint) {
        co_return MiscGameResult{MiscGameStatus::NetworkError,
                                 "WorldServer не имеет IPv4 endpoint"};
    }

    auto client = std::make_unique<MiscNet::CMyNetClient>(m_Io.get_executor());
    const ClientConnectResult connected = co_await client->Connect(*endpoint);
    if (connected.status != ClientConnectStatus::Connected) {
        co_return MiscGameResult{MiscGameStatus::NetworkError,
                                 "не удалось подключиться к WorldServer"};
    }
    client->EnableControlSend();

    MiscNet::CMessage registration(0x0005'FA01);
    registration.Base().Add(std::uint8_t{0});
    registration.Base().Add(registrationPort.value_or(*setup.listenPort));
    registration.Base().Add(setup.localIp->c_str());
    static_cast<void>(registration.Send(&client->SendQueue(), false));
    if (sendInitialSync) {
        MiscNet::CMessage initialSync(0x0015'EB05);
        static_cast<void>(initialSync.Send(&client->SendQueue(), false));
    }

    m_NetClient = std::move(client);
    m_ClientClose = false;
    m_ReconnectRequested = false;
    co_return MiscGameResult{};
}

MiscGameResult CGame::RunTurn()
{
    if (!m_Initialized || m_Released) {
        return {MiscGameStatus::RuntimeError,
                "MiscServer не инициализирован либо уже завершён"};
    }
    m_DoneSyncCount = 0;

    const std::uint32_t now = LegacyTickMilliseconds();
    if (m_LastLogRefreshTime + kLogRefreshMilliseconds < now) {
        m_LastLogRefreshTime = now;
        spdlog::info(
            "MiscServer: аукцион={}, синхронизация={}, добавлено={}, отброшено={}",
            m_AuctionRoom.AuctionGoodsCount(),
            static_cast<int>(m_DoneSyncMessage),
            m_AddNewCount,
            m_DeletedNewCount);
        std::size_t emitted{};
        for (const auto& [type, count] : m_MessageRecord) {
            if (emitted++ == 10U) break;
            spdlog::info("MiscServer: сообщение 0x{:08X}, количество={}",
                         std::bit_cast<std::uint32_t>(type),
                         count);
        }
        m_MessageRecord.clear();
        if (const auto workingSet = ReadWorkingSetKiB();
            workingSet && *workingSet > kMemorySyncResetKiB) {
            m_DoneSyncMessage = false;
            m_SyncStartTime = LegacyTickMilliseconds();
        }
    }

    if (m_NetClient && m_NetClient->IsConnected()) {
        if (m_NetClient->SendQueue().Pending() != 0) {
            const ClientFlushResult flushed =
                RunAwaitable(m_Io, m_NetClient->FlushOutgoing());
            if (flushed.status != ClientFlushStatus::Drained) {
                spdlog::warn("MiscServer: ошибка отправки очереди WorldServer");
            }
        }
        const auto read = m_NetClient->PollReadOnce();
        if (read && !*read) {
            spdlog::warn("MiscServer: ошибка входного Misc frame либо соединения WorldServer");
        }
    }

    static_cast<void>(m_AuctionRoom.AI(SendQueue()));
    MiscGameResult messages = RunAwaitable(m_Io, ProcessMessages());
    if (!messages) return messages;

    if (m_ClientClose || m_ReconnectRequested) {
        MiscGameResult reconnect =
            RunAwaitable(m_Io, ConnectWorld(kReconnectListenPort, false));
        if (!reconnect) {
            spdlog::warn("MiscServer: повторное соединение не установлено: {}",
                         reconnect.detail);
        }
        MiscNet::CMessage confirmation(0x0010'EF00);
        static_cast<void>(confirmation.Send(SendQueue(), false));
    }
    return {};
}

asio::awaitable<MiscGameResult> CGame::ProcessMessages()
{
    if (!m_NetClient) co_return MiscGameResult{};
    auto messages = m_NetClient->TakeAllMessages();
    for (auto& message : messages) {
        if (!message) continue;
        m_CurrentMessage = std::bit_cast<std::uint32_t>(message->MessageType());
        auto& count = m_MessageRecord[message->MessageType()];
        count = std::bit_cast<std::int32_t>(
            std::bit_cast<std::uint32_t>(count) + 1U);
        m_ReconnectRequested = false;
        static_cast<void>(message->Run(*this));
        m_CurrentMessage = 0;
        if (m_ReconnectRequested) {
            MiscGameResult reconnect =
                co_await ConnectWorld(kReconnectListenPort, false);
            if (!reconnect) {
                spdlog::warn("MiscServer: reconnect из сообщения не удался: {}",
                             reconnect.detail);
            }
        }
    }
    co_return MiscGameResult{};
}

MiscGameResult CGame::Release()
{
    if (m_Released) return {};
    m_Released = true;
    if (m_NetClient) {
        static_cast<void>(m_AuctionRoom.AI(&m_NetClient->SendQueue()));
        static_cast<void>(m_NetClient->Close());
        m_NetClient.reset();
    }
    m_AuctionRoom.Clear();
    return {};
}

Auction::CAuctionRoom& CGame::AuctionRoom() noexcept { return m_AuctionRoom; }
const Auction::CAuctionRoom& CGame::AuctionRoom() const noexcept { return m_AuctionRoom; }
MiscNet::CMyNetClient* CGame::NetClient() noexcept { return m_NetClient.get(); }
ClientSendQueue* CGame::SendQueue() noexcept
{
    return m_NetClient ? &m_NetClient->SendQueue() : nullptr;
}
void CGame::CountNewAuctionItem() noexcept { ++m_AddNewCount; }

std::optional<Auction::AuctionRoomError>
CGame::AddAuctionItem(std::unique_ptr<Auction::CGoodsNode> item, bool& added)
{
    return m_AuctionRoom.AddItemToAuctionRoom(std::move(item),
                                              m_DeletedNewCount,
                                              added);
}

std::uint32_t CGame::AddNewCount() const noexcept { return m_AddNewCount; }
std::uint32_t CGame::DeletedNewCount() const noexcept { return m_DeletedNewCount; }
bool CGame::AuctionSyncEnabled() const noexcept { return m_DoneSyncMessage; }
std::uint32_t CGame::AuctionSyncStartTime() const noexcept { return m_SyncStartTime; }
void CGame::EnableAuctionSync() noexcept { m_DoneSyncMessage = true; }
std::optional<std::uint32_t> CGame::AuctionSyncCount() const noexcept
{
    return m_DoneSyncCount;
}
void CGame::FinishAuctionSyncTurn() noexcept { m_DoneSyncCount = 1; }
void CGame::RequestReconnect() noexcept
{
    m_ClientClose = true;
    m_ReconnectRequested = true;
}

void CGame::OnWorldAuction(MiscNet::CMessage& message)
{
    OnMSG_W2M_AUCTION(message, *this);
}
void CGame::OnMiscFunction(MiscNet::CMessage& message)
{
    OnMSG_M2M_Fuction(message, *this);
}
void CGame::OnOther(MiscNet::CMessage& message)
{
    OnOtherMsg(message, *this);
}
}
