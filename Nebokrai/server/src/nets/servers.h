#pragma once

#include "clients.h"
#include "mysocket.h"
#include "serverclient.h"
#include "socketcommands.h"

#include <asio.hpp>

#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <system_error>
#include <variant>
#include <vector>

/*
 * Исходный владелец: nets/servers.cpp / nets/servers.h
 *
 * Общий CServer подтверждён Auth/Billing/Login/Game/World EXE/PDB. Поздняя
 * Rust-реконструкция уже восстановила Host/admission, socket-command snapshot,
 * routing maps, временный forbid IPv4, таймер первого сообщения, traffic
 * counters и точный порядок close/send. Старый Miracle_server_linux полезен
 * только как C++-донор: его poll/eventfd/IOCP-замены не являются оригиналом.
 *
 * Новый владелец Linux использует самостоятельный Asio для приёма,
 * чтения, записи и завершения работы.
 * Это техническая замена WinSock/IOCP, а не новая игровая семантика. Один
 * CServer по-прежнему является единственным владельцем client/maps state;
 * producers меняют его только через CSocketCommands.
 *
 * Важные странности сохранены: duplicate ADD увеличивает signed client count и
 * не компенсирует его; отсутствующая строковая map identity возвращает socket
 * ID 1; timeout первого сообщения публикует QUIT только в следующий snapshot;
 * server-side partial write теряет хвост после одного completion; SENDALL при
 * переполнении удаляет client в другом порядке, чем обычный SendBy*.
 *
 * Конкретные CMessage/parser/OnClose/OnAccept реакции не получают default
 * пустую реализацию. Производный владелец службы обязан явно реализовать
 * обратные вызовы ниже.
 */

inline constexpr std::size_t kServerReceiveChunk = 0x2000;
inline constexpr std::uint32_t kAcceptThreadDelayMs = 100;
inline constexpr std::uint32_t kAcceptAtCapacityDelayMs = 1000;
inline constexpr std::uint32_t kNetThreadDelayMs = 1;

struct AllowedAddress
{
    std::vector<std::uint8_t> ip;
    std::uint16_t port{};

    [[nodiscard]] bool Matches(std::span<const std::uint8_t> peerIP,
                               std::uint16_t peerPort) const noexcept;
};

struct AcceptedServerClient
{
    std::shared_ptr<asio::ip::tcp::socket> socket;
    std::unique_ptr<CServerClient> state;
};

struct ServerCommandAdd
{
    std::int32_t socketId{};
    std::uint32_t acceptedAtMs{};
    AcceptedServerClient client;
};
struct ServerCommandSetMapName
{
    std::int32_t socketId{};
    std::vector<std::uint8_t> mapName;
};
struct ServerCommandSetMapId
{
    std::int32_t socketId{};
    std::int32_t mapId{};
};
struct ServerCommandDeleteBySocketId { std::int32_t socketId{}; };
struct ServerCommandQuitBySocketId { std::int32_t socketId{}; };
struct ServerCommandQuitByMapId { std::int32_t mapId{}; };
struct ServerCommandQuitByMapName { std::vector<std::uint8_t> mapName; };
struct ServerCommandQuitAll {};
struct ServerCommandReceive
{
    std::int32_t socketId{};
    std::vector<std::uint8_t> buffer;
};
struct ServerCommandSendToSocket
{
    std::int32_t socketId{};
    std::vector<std::uint8_t> buffer;
};
struct ServerCommandSendToMapId
{
    std::int32_t mapId{};
    std::vector<std::uint8_t> buffer;
};
struct ServerCommandSendToMapName
{
    std::vector<std::uint8_t> mapName;
    std::vector<std::uint8_t> buffer;
};
struct ServerCommandSendAll { std::vector<std::uint8_t> buffer; };
struct ServerCommandSendEnd { std::int32_t socketId{}; };

using ServerSocketCommand = std::variant<
    ServerCommandAdd,
    ServerCommandSetMapName,
    ServerCommandSetMapId,
    ServerCommandDeleteBySocketId,
    ServerCommandQuitBySocketId,
    ServerCommandQuitByMapId,
    ServerCommandQuitByMapName,
    ServerCommandQuitAll,
    ServerCommandReceive,
    ServerCommandSendToSocket,
    ServerCommandSendToMapId,
    ServerCommandSendToMapName,
    ServerCommandSendAll,
    ServerCommandSendEnd>;

class ServerCommandHandle
{
public:
    ServerCommandHandle();

    [[nodiscard]] std::int32_t SendBySocketID(std::int32_t socketId,
                                              std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t SendByMapID(std::int32_t mapId,
                                           std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t SendByMapName(std::span<const std::uint8_t> mapName,
                                             std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t SendAll(std::span<const std::uint8_t> buffer) const;

    [[nodiscard]] std::int32_t QuitBySocketID(std::int32_t socketId) const;
    [[nodiscard]] std::int32_t QuitByMapID(std::int32_t mapId) const;
    [[nodiscard]] std::int32_t QuitByMapName(std::span<const std::uint8_t> mapName) const;
    [[nodiscard]] std::int32_t SetClientMapID(std::int32_t socketId, std::int32_t mapId) const;
    [[nodiscard]] std::int32_t SetClientMapName(std::int32_t socketId,
                                                std::span<const std::uint8_t> mapName) const;
    [[nodiscard]] std::int32_t QuitAll() const;
    [[nodiscard]] std::int32_t Pending() const;

    void PublishReceive(std::int32_t socketId, std::vector<std::uint8_t> buffer) const;
    void PublishDelete(std::int32_t socketId) const;
    void PublishSendEnd(std::int32_t socketId) const;

private:
    friend class CServer;

    explicit ServerCommandHandle(std::shared_ptr<CSocketCommands<ServerSocketCommand>> commands);
    void Push(ServerSocketCommand command) const;
    [[nodiscard]] CSocketCommands<ServerSocketCommand>::Queue TakeAll() const;

    std::shared_ptr<CSocketCommands<ServerSocketCommand>> m_Commands;
};

enum class ComponentReceiveErrorAction
{
    None,
    ForbidAndQuit,
};

struct ReceiveCallbackResult
{
    bool success{true};
    ComponentReceiveErrorAction action{ComponentReceiveErrorAction::None};
};

enum class ServerSnapshotErrorKind
{
    ClientSize,
    Component,
};

struct ServerSnapshotError
{
    ServerSnapshotErrorKind kind{};
    std::int32_t socketId{};
    std::optional<ServerClientSizeError> sizeError;
};

struct ServerReceiveAction
{
    std::int32_t socketId{};
    std::shared_ptr<asio::ip::tcp::socket> socket;
};

struct ServerSendAction
{
    std::int32_t socketId{};
    std::shared_ptr<asio::ip::tcp::socket> socket;
    ServerSendBatch batch;
    bool shutdownAfter{};
};

using ServerIoAction = std::variant<ServerReceiveAction, ServerSendAction>;

struct ServerSnapshot
{
    std::vector<ServerIoAction> ioActions;
    std::int32_t processedCommands{};
    std::vector<ServerSnapshotError> errors;
};

[[nodiscard]] asio::awaitable<void>
RunServerIoAction(ServerIoAction action, ServerCommandHandle commands);

enum class AcceptStart
{
    NotListening,
    AtCapacity,
    Pending,
};

struct AcceptedTransport
{
    std::shared_ptr<asio::ip::tcp::socket> socket;
    asio::ip::tcp::endpoint peer;
    std::error_code error;
};

enum class AdmissionOutcome
{
    AtCapacity,
    AddressRejected,
    TemporarilyForbidden,
    CreateClientFailed,
    Queued,
};

struct AdmissionResult
{
    AdmissionOutcome outcome{AdmissionOutcome::AtCapacity};
    std::int32_t socketId{};
};

class CServer
{
public:
    explicit CServer(asio::any_io_executor executor, std::uint32_t nowMs = 0);
    virtual ~CServer();

    CServer(const CServer&) = delete;
    CServer& operator=(const CServer&) = delete;

    [[nodiscard]] bool Host(std::uint32_t port,
                            std::optional<asio::ip::address_v4> address,
                            std::int32_t socketType,
                            bool legacyFlag,
                            std::error_code& error);

    void StopListening() noexcept;

    [[nodiscard]] AcceptStart BeginAccept() const noexcept;
    [[nodiscard]] asio::awaitable<AcceptedTransport> AcceptOne();
    [[nodiscard]] AdmissionResult QueueAccepted(std::shared_ptr<asio::ip::tcp::socket> socket,
                                                const asio::ip::tcp::endpoint& peer,
                                                std::uint32_t nowMs);

    [[nodiscard]] ServerSnapshot ProcessCommandSnapshot(std::uint32_t nowMs);
    [[nodiscard]] ServerCommandHandle CommandHandle() const;

    [[nodiscard]] std::int32_t SendBySocketID(std::int32_t socketId,
                                              std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t SendByMapID(std::int32_t mapId,
                                           std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t SendByMapName(std::span<const std::uint8_t> mapName,
                                             std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t SendAll(std::span<const std::uint8_t> buffer) const;
    [[nodiscard]] std::int32_t QuitClientBySocketID(std::int32_t socketId) const;
    [[nodiscard]] std::int32_t QuitClientByMapID(std::int32_t mapId) const;
    [[nodiscard]] std::int32_t QuitClientByMapName(std::span<const std::uint8_t> mapName) const;
    [[nodiscard]] std::int32_t QuitAllClients() const;
    [[nodiscard]] std::int32_t SetClientMapID(std::int32_t socketId, std::int32_t mapId) const;
    [[nodiscard]] std::int32_t SetClientMapName(std::int32_t socketId,
                                                std::span<const std::uint8_t> mapName) const;

    [[nodiscard]] std::int32_t ClientCount() const noexcept;
    [[nodiscard]] bool HasClients() const noexcept;
    [[nodiscard]] std::int32_t GetSocketIDByMapID(std::int32_t mapId) const noexcept;
    [[nodiscard]] std::int32_t GetSocketIDByMapName(std::span<const std::uint8_t> mapName) const;

    void SetLocalIdentity(std::span<const std::uint8_t> ip, std::uint32_t ipv4Word);
    [[nodiscard]] std::span<const std::uint8_t> LocalIP() const noexcept;
    [[nodiscard]] std::uint32_t LocalIPv4Word() const noexcept;

    [[nodiscard]] bool LoadAllowedClients(const std::string& path);
    [[nodiscard]] bool IsAllowedAddress(std::span<const std::uint8_t> peerIP,
                                        std::uint16_t peerPort) const noexcept;

    void ConfigureReceiveGuard(bool enabled,
                               std::int32_t bytesPerSecond,
                               std::int32_t forbidTimeMs) noexcept;
    void ConfigureSendLimits(std::int32_t maxInFlightSends,
                             std::int32_t permittedSendBytes) noexcept;
    void ConfigureMaxClients(std::int32_t maxClients) noexcept;
    void ConfigureTransportAfterHost(bool checkReceiveRate,
                                     std::int32_t maxInFlightSends,
                                     std::uint32_t receiveRateLimit,
                                     std::int32_t maxClients,
                                     bool checkMessageContent,
                                     std::uint32_t forbidTimeMs,
                                     std::uint32_t maximumMessageLength,
                                     std::int32_t permittedSendBytes) noexcept;
    void ConfigureClientTransportForReload(bool checkReceiveRate,
                                           bool checkMessageContent,
                                           std::uint32_t receiveRateLimit,
                                           std::uint32_t forbidTimeMs,
                                           std::int32_t maxClients,
                                           std::int32_t maxInFlightSends,
                                           std::uint32_t maximumMessageLength,
                                           std::int32_t permittedSendBytes) noexcept;
    void ConfigureWorldTransportForReload(bool checkReceiveRate,
                                          bool checkMessageContent,
                                          std::uint32_t receiveRateLimit,
                                          std::uint32_t forbidTimeMs,
                                          std::int32_t maxClients,
                                          std::int32_t maxInFlightSends,
                                          std::uint32_t maximumMessageLength,
                                          std::int32_t permittedSendBytes) noexcept;
    void ConfigureAcceptLimitsAfterHost(std::int32_t maxBacklog,
                                        std::int32_t newAcceptTimeoutMs) noexcept;

    void AddForbiddenIP(std::uint32_t peerIPv4, std::uint32_t nowMs);
    [[nodiscard]] bool FindForbiddenIP(std::uint32_t peerIPv4, std::uint32_t nowMs);
    [[nodiscard]] std::vector<std::int32_t> ExpireNewAccepts(std::uint32_t nowMs);

    [[nodiscard]] std::int32_t AddSendSize(std::int32_t amount, std::uint32_t nowMs) noexcept;
    [[nodiscard]] std::int32_t AddReceiveSize(std::int32_t amount, std::uint32_t nowMs) noexcept;

protected:
    [[nodiscard]] virtual std::unique_ptr<CServerClient>
    CreateServerClient(std::int32_t socketId, std::uint32_t peerIPv4, std::uint32_t nowMs) = 0;
    virtual void OnAccepted(CServerClient& client) = 0;
    [[nodiscard]] virtual ReceiveCallbackResult OnReceive(CServerClient& client,
                                                          std::uint32_t nowMs) = 0;
    virtual void OnClose(CServerClient& client) = 0;
    virtual void OnReceiveRateExceeded(CServerClient& client,
                                       std::int32_t actual,
                                       std::int32_t permitted) = 0;
    virtual void OnMissingMapIDClient(std::int32_t mapId, std::int32_t socketId) = 0;
    virtual void OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                        std::int32_t socketId) = 0;

    [[nodiscard]] CServerClient* FindClient(std::int32_t socketId) noexcept;
    [[nodiscard]] const CServerClient* FindClient(std::int32_t socketId) const noexcept;

private:
    enum class SendOverflowRemoval
    {
        DelOneClient,
        Broadcast,
    };

    void ProcessReceiveCommand(std::int32_t socketId,
                               std::span<const std::uint8_t> buffer,
                               std::uint32_t nowMs,
                               std::vector<ServerSnapshotError>& errors);
    void BufferSendCommand(std::int32_t socketId,
                           std::span<const std::uint8_t> buffer,
                           SendOverflowRemoval removal,
                           std::vector<ServerSnapshotError>& errors);
    void RemoveClientWithCallback(std::int32_t socketId);
    void RemoveBroadcastOverflowClient(std::int32_t socketId);
    void MarkClientClosing(std::int32_t socketId);
    void MarkAllClientsClosing();
    [[nodiscard]] bool AssignMapID(std::int32_t socketId, std::int32_t mapId);
    [[nodiscard]] bool AssignMapName(std::int32_t socketId,
                                     std::span<const std::uint8_t> mapName);
    void RemoveMapID(std::int32_t mapId);
    void RemoveMapName(std::span<const std::uint8_t> mapName);
    void AcknowledgeFirstReceive(std::int32_t socketId);

    asio::any_io_executor m_Executor;
    std::unique_ptr<asio::ip::tcp::acceptor> m_Acceptor;
    ServerCommandHandle m_Commands;
    std::map<std::int32_t, AcceptedServerClient> m_Clients;
    std::map<std::int32_t, std::int32_t> m_MapIDSocketID;
    std::map<std::vector<std::uint8_t>, std::int32_t> m_MapNameSocketID;
    std::map<std::uint32_t, std::uint32_t> m_ForbiddenIPs;
    std::map<std::int32_t, std::uint32_t> m_NewAcceptSockets;
    std::vector<AllowedAddress> m_AllowedAddresses;
    std::vector<std::uint8_t> m_LocalIP{kDefaultSocketIPv4.begin(), kDefaultSocketIPv4.end()};
    std::uint32_t m_LocalIPv4Word{LegacyIPv4Word(kDefaultSocketIPv4)};
    SocketIdAllocator m_SocketIDs;
    std::int32_t m_ClientCount{};
    std::int32_t m_MaxClients{100};
    std::int32_t m_MaxBacklog{0x7FFF'FFFF};
    std::int32_t m_NewAcceptTimeoutMs{8000};
    std::int32_t m_ReceiveRateLimit{0x186A'0000};
    std::int32_t m_ForbidTimeMs{};
    std::int32_t m_MaxInFlightSends{1};
    std::int32_t m_PermittedSendBytes{kDefaultPermittedServerSendBytes};
    bool m_CheckReceiveRate{};
    bool m_CheckMessageContent{};
    std::optional<std::uint32_t> m_MaximumMessageLength;
    bool m_CheckAllowedAddress{};
    TransferCounter m_SendCounter;
    TransferCounter m_ReceiveCounter;
};
