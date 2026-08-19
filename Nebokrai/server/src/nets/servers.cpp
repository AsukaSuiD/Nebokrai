#include "servers.h"

#include <asio/redirect_error.hpp>
#include <asio/use_awaitable.hpp>

#include <algorithm>
#include <array>
#include <bit>
#include <fstream>
#include <iterator>
#include <limits>
#include <utility>

namespace
{
template <class... T>
struct Overloaded : T...
{
    using T::operator()...;
};
template <class... T>
Overloaded(T...) -> Overloaded<T...>;

std::int32_t WrapAdd32(std::int32_t lhs, std::int32_t rhs) noexcept
{
    return std::bit_cast<std::int32_t>(
        std::bit_cast<std::uint32_t>(lhs) + std::bit_cast<std::uint32_t>(rhs));
}

void ShutdownSocket(const std::shared_ptr<asio::ip::tcp::socket>& socket) noexcept
{
    if (!socket || !socket->is_open()) {
        return;
    }
    asio::error_code ignored;
    socket->shutdown(asio::ip::tcp::socket::shutdown_both, ignored);
}

std::vector<std::string> SplitAsciiWhitespace(const std::vector<char>& source)
{
    std::vector<std::string> tokens;
    std::size_t begin = 0;
    while (begin < source.size()) {
        while (begin < source.size() &&
               (source[begin] == ' ' || source[begin] == '\t' || source[begin] == '\r' ||
                source[begin] == '\n' || source[begin] == '\f' || source[begin] == '\v')) {
            ++begin;
        }
        if (begin == source.size()) {
            break;
        }
        std::size_t end = begin;
        while (end < source.size() &&
               !(source[end] == ' ' || source[end] == '\t' || source[end] == '\r' ||
                 source[end] == '\n' || source[end] == '\f' || source[end] == '\v')) {
            ++end;
        }
        tokens.emplace_back(source.data() + begin, end - begin);
        begin = end;
    }
    return tokens;
}

std::optional<std::uint16_t> ParseU16(const std::string& token)
{
    if (token.empty()) {
        return std::nullopt;
    }

    std::uint32_t value = 0;
    for (const unsigned char ch : token) {
        if (ch < '0' || ch > '9') {
            return std::nullopt;
        }
        value = value * 10U + static_cast<std::uint32_t>(ch - '0');
        if (value > std::numeric_limits<std::uint16_t>::max()) {
            return std::nullopt;
        }
    }
    return static_cast<std::uint16_t>(value);
}
}

bool AllowedAddress::Matches(std::span<const std::uint8_t> peerIP,
                             std::uint16_t peerPort) const noexcept
{
    return std::equal(ip.begin(), ip.end(), peerIP.begin(), peerIP.end()) &&
           (port == 0U || port == peerPort);
}

ServerCommandHandle::ServerCommandHandle()
    : m_Commands(std::make_shared<CSocketCommands<ServerSocketCommand>>())
{
}

ServerCommandHandle::ServerCommandHandle(
    std::shared_ptr<CSocketCommands<ServerSocketCommand>> commands)
    : m_Commands(std::move(commands))
{
}

std::int32_t ServerCommandHandle::SendBySocketID(std::int32_t socketId,
                                                 std::span<const std::uint8_t> buffer) const
{
    if (buffer.empty()) {
        return 0;
    }
    Push(ServerCommandSendToSocket{socketId, {buffer.begin(), buffer.end()}});
    return 1;
}

std::int32_t ServerCommandHandle::SendByMapID(std::int32_t mapId,
                                              std::span<const std::uint8_t> buffer) const
{
    if (buffer.empty()) {
        return 0;
    }
    Push(ServerCommandSendToMapId{mapId, {buffer.begin(), buffer.end()}});
    return 1;
}

std::int32_t ServerCommandHandle::SendByMapName(std::span<const std::uint8_t> mapName,
                                                std::span<const std::uint8_t> buffer) const
{
    if (buffer.empty()) {
        return 0;
    }
    Push(ServerCommandSendToMapName{{mapName.begin(), mapName.end()},
                                    {buffer.begin(), buffer.end()}});
    return 1;
}

std::int32_t ServerCommandHandle::SendAll(std::span<const std::uint8_t> buffer) const
{
    if (buffer.empty()) {
        return 0;
    }
    Push(ServerCommandSendAll{{buffer.begin(), buffer.end()}});
    return 1;
}

std::int32_t ServerCommandHandle::QuitBySocketID(std::int32_t socketId) const
{
    Push(ServerCommandQuitBySocketId{socketId});
    return 1;
}

std::int32_t ServerCommandHandle::QuitByMapID(std::int32_t mapId) const
{
    Push(ServerCommandQuitByMapId{mapId});
    return 1;
}

std::int32_t ServerCommandHandle::QuitByMapName(std::span<const std::uint8_t> mapName) const
{
    Push(ServerCommandQuitByMapName{{mapName.begin(), mapName.end()}});
    return 1;
}

std::int32_t ServerCommandHandle::SetClientMapID(std::int32_t socketId, std::int32_t mapId) const
{
    Push(ServerCommandSetMapId{socketId, mapId});
    return 1;
}

std::int32_t ServerCommandHandle::SetClientMapName(
    std::int32_t socketId,
    std::span<const std::uint8_t> mapName) const
{
    Push(ServerCommandSetMapName{socketId, {mapName.begin(), mapName.end()}});
    return 1;
}

std::int32_t ServerCommandHandle::QuitAll() const
{
    Push(ServerCommandQuitAll{});
    return 1;
}

std::int32_t ServerCommandHandle::Pending() const
{
    return m_Commands->GetSize();
}

void ServerCommandHandle::PublishReceive(std::int32_t socketId,
                                         std::vector<std::uint8_t> buffer) const
{
    Push(ServerCommandReceive{socketId, std::move(buffer)});
}

void ServerCommandHandle::PublishDelete(std::int32_t socketId) const
{
    Push(ServerCommandDeleteBySocketId{socketId});
}

void ServerCommandHandle::PublishSendEnd(std::int32_t socketId) const
{
    Push(ServerCommandSendEnd{socketId});
}

void ServerCommandHandle::Push(ServerSocketCommand command) const
{
    m_Commands->Push_Back(std::move(command));
}

CSocketCommands<ServerSocketCommand>::Queue ServerCommandHandle::TakeAll() const
{
    return m_Commands->CopyAllCommand();
}

asio::awaitable<void> RunServerIoAction(ServerIoAction action, ServerCommandHandle commands)
{
    if (auto* receive = std::get_if<ServerReceiveAction>(&action)) {
        std::array<std::uint8_t, kServerReceiveChunk> buffer{};
        for (;;) {
            asio::error_code error;
            const std::size_t received = co_await receive->socket->async_read_some(
                asio::buffer(buffer), asio::redirect_error(asio::use_awaitable, error));
            if (error || received == 0U) {
                commands.PublishDelete(receive->socketId);
                co_return;
            }
            commands.PublishReceive(receive->socketId,
                                    std::vector<std::uint8_t>(buffer.begin(),
                                                              buffer.begin() + received));
        }
    }

    auto send = std::get<ServerSendAction>(std::move(action));
    asio::error_code error;
    const auto bytes = send.batch.Bytes();
    const std::size_t transferred = co_await send.socket->async_write_some(
        asio::buffer(bytes.data(), bytes.size()),
        asio::redirect_error(asio::use_awaitable, error));

    if (send.shutdownAfter) {
        ShutdownSocket(send.socket);
    }

    if (error || transferred == 0U) {
        commands.PublishDelete(send.socketId);
        co_return;
    }

    // Partial success deliberately publishes SENDEND as the original IOCP
    // completion did; the unsent tail of this server-side batch is lost.
    commands.PublishSendEnd(send.socketId);
}

CServer::CServer(asio::any_io_executor executor, std::uint32_t nowMs)
    : m_Executor(std::move(executor)),
      m_SendCounter(nowMs),
      m_ReceiveCounter(nowMs)
{
}

CServer::~CServer()
{
    if (m_Acceptor) {
        asio::error_code ignored;
        m_Acceptor->close(ignored);
    }
    for (auto& [_, client] : m_Clients) {
        ShutdownSocket(client.socket);
    }
}

bool CServer::Host(std::uint32_t port,
                   std::optional<asio::ip::address_v4> address,
                   std::int32_t socketType,
                   bool legacyFlag,
                   std::error_code& error)
{
    static_cast<void>(legacyFlag);
    error.clear();
    if (socketType != kDefaultSocketType) {
        error = std::make_error_code(std::errc::protocol_not_supported);
        return false;
    }

    auto acceptor = std::make_unique<asio::ip::tcp::acceptor>(m_Executor);
    asio::error_code asioError;
    acceptor->open(asio::ip::tcp::v4(), asioError);
    if (asioError) {
        error = asioError;
        return false;
    }

    const asio::ip::tcp::endpoint endpoint(
        address.value_or(asio::ip::address_v4::any()), LegacySocketPort(port));
    acceptor->bind(endpoint, asioError);
    if (asioError) {
        error = asioError;
        return false;
    }
    acceptor->listen(m_MaxBacklog, asioError);
    if (asioError) {
        error = asioError;
        return false;
    }

    m_Acceptor = std::move(acceptor);
    return true;
}

AcceptStart CServer::BeginAccept() const noexcept
{
    if (m_ClientCount >= m_MaxClients) {
        return AcceptStart::AtCapacity;
    }
    if (!m_Acceptor || !m_Acceptor->is_open()) {
        return AcceptStart::NotListening;
    }
    return AcceptStart::Pending;
}

asio::awaitable<AcceptedTransport> CServer::AcceptOne()
{
    if (!m_Acceptor || !m_Acceptor->is_open()) {
        co_return AcceptedTransport{
            .error = std::make_error_code(std::errc::bad_file_descriptor),
        };
    }

    auto socket = std::make_shared<asio::ip::tcp::socket>(m_Executor);
    asio::ip::tcp::endpoint peer;
    asio::error_code error;
    co_await m_Acceptor->async_accept(
        *socket, peer, asio::redirect_error(asio::use_awaitable, error));
    co_return AcceptedTransport{std::move(socket), peer, error};
}

AdmissionResult CServer::QueueAccepted(std::shared_ptr<asio::ip::tcp::socket> socket,
                                       const asio::ip::tcp::endpoint& peer,
                                       std::uint32_t nowMs)
{
    if (m_ClientCount >= m_MaxClients) {
        ShutdownSocket(socket);
        return {AdmissionOutcome::AtCapacity, 0};
    }
    if (!peer.address().is_v4()) {
        ShutdownSocket(socket);
        return {AdmissionOutcome::AddressRejected, 0};
    }

    const std::string peerText = peer.address().to_string();
    const std::span<const std::uint8_t> peerTextBytes(
        reinterpret_cast<const std::uint8_t*>(peerText.data()), peerText.size());
    if (m_CheckAllowedAddress && !IsAllowedAddress(peerTextBytes, peer.port())) {
        ShutdownSocket(socket);
        return {AdmissionOutcome::AddressRejected, 0};
    }

    const auto bytes = peer.address().to_v4().to_bytes();
    const IPv4Octets octets{bytes[0], bytes[1], bytes[2], bytes[3]};
    const std::uint32_t peerIPv4 = LegacyIPv4Word(octets);
    if (m_CheckReceiveRate && FindForbiddenIP(peerIPv4, nowMs)) {
        ShutdownSocket(socket);
        return {AdmissionOutcome::TemporarilyForbidden, 0};
    }

    const std::int32_t socketId = m_SocketIDs.Next();
    std::unique_ptr<CServerClient> state = CreateServerClient(socketId, peerIPv4, nowMs);
    if (!state) {
        ShutdownSocket(socket);
        return {AdmissionOutcome::CreateClientFailed, 0};
    }
    OnAccepted(*state);

    m_Commands.Push(ServerCommandAdd{
        .socketId = socketId,
        .acceptedAtMs = nowMs,
        .client = AcceptedServerClient{std::move(socket), std::move(state)},
    });
    return {AdmissionOutcome::Queued, socketId};
}

ServerSnapshot CServer::ProcessCommandSnapshot(std::uint32_t nowMs)
{
    auto commands = m_Commands.TakeAll();
    ServerSnapshot snapshot;
    snapshot.processedCommands =
        std::bit_cast<std::int32_t>(static_cast<std::uint32_t>(commands.size()));

    for (auto& command : commands) {
        std::visit(
            Overloaded{
                [&](ServerCommandAdd& value) {
                    m_ClientCount = WrapAdd32(m_ClientCount, 1);
                    if (auto old = m_Clients.find(value.socketId); old != m_Clients.end()) {
                        ShutdownSocket(old->second.socket);
                        m_Clients.erase(old);
                    }
                    auto stream = value.client.socket;
                    m_NewAcceptSockets[value.socketId] = value.acceptedAtMs;
                    m_Clients.insert_or_assign(value.socketId, std::move(value.client));
                    snapshot.ioActions.emplace_back(ServerReceiveAction{value.socketId,
                                                                         std::move(stream)});
                },
                [&](ServerCommandSetMapName& value) {
                    if (!AssignMapName(value.socketId, value.mapName)) {
                        OnMissingMapNameClient(value.mapName, value.socketId);
                    }
                },
                [&](const ServerCommandSetMapId& value) {
                    if (!AssignMapID(value.socketId, value.mapId)) {
                        OnMissingMapIDClient(value.mapId, value.socketId);
                    }
                },
                [&](const ServerCommandDeleteBySocketId& value) {
                    RemoveClientWithCallback(value.socketId);
                },
                [&](const ServerCommandQuitBySocketId& value) {
                    MarkClientClosing(value.socketId);
                },
                [&](const ServerCommandQuitByMapId& value) {
                    const std::int32_t socketId = GetSocketIDByMapID(value.mapId);
                    RemoveMapID(value.mapId);
                    MarkClientClosing(socketId);
                },
                [&](ServerCommandQuitByMapName& value) {
                    const std::int32_t socketId = GetSocketIDByMapName(value.mapName);
                    RemoveMapName(value.mapName);
                    MarkClientClosing(socketId);
                },
                [&](const ServerCommandQuitAll&) { MarkAllClientsClosing(); },
                [&](ServerCommandReceive& value) {
                    ProcessReceiveCommand(value.socketId, value.buffer, nowMs, snapshot.errors);
                },
                [&](ServerCommandSendToSocket& value) {
                    BufferSendCommand(value.socketId,
                                      value.buffer,
                                      SendOverflowRemoval::DelOneClient,
                                      snapshot.errors);
                },
                [&](ServerCommandSendToMapId& value) {
                    const std::int32_t socketId = GetSocketIDByMapID(value.mapId);
                    if (socketId != 0) {
                        BufferSendCommand(socketId,
                                          value.buffer,
                                          SendOverflowRemoval::DelOneClient,
                                          snapshot.errors);
                    }
                },
                [&](ServerCommandSendToMapName& value) {
                    const std::int32_t socketId = GetSocketIDByMapName(value.mapName);
                    if (socketId != 0) {
                        BufferSendCommand(socketId,
                                          value.buffer,
                                          SendOverflowRemoval::DelOneClient,
                                          snapshot.errors);
                    }
                },
                [&](ServerCommandSendAll& value) {
                    std::vector<std::int32_t> socketIds;
                    socketIds.reserve(m_Clients.size());
                    for (const auto& [socketId, _] : m_Clients) {
                        socketIds.push_back(socketId);
                    }
                    for (const std::int32_t socketId : socketIds) {
                        BufferSendCommand(socketId,
                                          value.buffer,
                                          SendOverflowRemoval::Broadcast,
                                          snapshot.errors);
                    }
                },
                [&](const ServerCommandSendEnd& value) {
                    if (CServerClient* client = FindClient(value.socketId)) {
                        client->FinishSendOperation();
                    }
                }},
            command);
    }

    std::vector<std::int32_t> shutdownAfterSend;
    for (auto& [socketId, client] : m_Clients) {
        if (client.state->IOOperations() >= m_MaxInFlightSends) {
            continue;
        }
        auto batch = client.state->BeginSend();
        if (!batch) {
            continue;
        }
        const bool shutdownAfter = client.state->IsClosing();
        if (shutdownAfter) {
            shutdownAfterSend.push_back(socketId);
        }
        AddSendSize(static_cast<std::int32_t>(batch->RequestedBytes()), nowMs);
        snapshot.ioActions.emplace_back(ServerSendAction{
            socketId, client.socket, std::move(*batch), shutdownAfter});
    }

    ExpireNewAccepts(nowMs);

    for (auto& [socketId, client] : m_Clients) {
        if (!client.state->BeginClose()) {
            continue;
        }
        if (std::find(shutdownAfterSend.begin(), shutdownAfterSend.end(), socketId) ==
            shutdownAfterSend.end()) {
            ShutdownSocket(client.socket);
        }
    }

    return snapshot;
}

ServerCommandHandle CServer::CommandHandle() const
{
    return m_Commands;
}

std::int32_t CServer::SendBySocketID(std::int32_t socketId,
                                     std::span<const std::uint8_t> buffer) const
{
    return m_Commands.SendBySocketID(socketId, buffer);
}

std::int32_t CServer::SendByMapID(std::int32_t mapId,
                                  std::span<const std::uint8_t> buffer) const
{
    return m_Commands.SendByMapID(mapId, buffer);
}

std::int32_t CServer::SendByMapName(std::span<const std::uint8_t> mapName,
                                    std::span<const std::uint8_t> buffer) const
{
    return m_Commands.SendByMapName(mapName, buffer);
}

std::int32_t CServer::SendAll(std::span<const std::uint8_t> buffer) const
{
    return m_Commands.SendAll(buffer);
}

std::int32_t CServer::QuitClientBySocketID(std::int32_t socketId) const
{
    return m_Commands.QuitBySocketID(socketId);
}

std::int32_t CServer::QuitClientByMapID(std::int32_t mapId) const
{
    return m_Commands.QuitByMapID(mapId);
}

std::int32_t CServer::QuitClientByMapName(std::span<const std::uint8_t> mapName) const
{
    return m_Commands.QuitByMapName(mapName);
}

std::int32_t CServer::QuitAllClients() const
{
    return m_Commands.QuitAll();
}

std::int32_t CServer::SetClientMapID(std::int32_t socketId, std::int32_t mapId) const
{
    return m_Commands.SetClientMapID(socketId, mapId);
}

std::int32_t CServer::SetClientMapName(std::int32_t socketId,
                                       std::span<const std::uint8_t> mapName) const
{
    return m_Commands.SetClientMapName(socketId, mapName);
}

std::int32_t CServer::ClientCount() const noexcept
{
    return m_ClientCount;
}

bool CServer::HasClients() const noexcept
{
    return !m_Clients.empty();
}

std::int32_t CServer::GetSocketIDByMapID(std::int32_t mapId) const noexcept
{
    const auto found = m_MapIDSocketID.find(mapId);
    return found == m_MapIDSocketID.end() ? 0 : found->second;
}

std::int32_t CServer::GetSocketIDByMapName(std::span<const std::uint8_t> mapName) const
{
    const std::vector<std::uint8_t> key(mapName.begin(), mapName.end());
    const auto found = m_MapNameSocketID.find(key);
    return found == m_MapNameSocketID.end() ? 1 : found->second;
}

void CServer::SetLocalIdentity(std::span<const std::uint8_t> ip, std::uint32_t ipv4Word)
{
    m_LocalIP.assign(ip.begin(), ip.end());
    m_LocalIPv4Word = ipv4Word;
}

std::span<const std::uint8_t> CServer::LocalIP() const noexcept
{
    return m_LocalIP;
}

std::uint32_t CServer::LocalIPv4Word() const noexcept
{
    return m_LocalIPv4Word;
}

bool CServer::LoadAllowedClients(const std::string& path)
{
    m_AllowedAddresses.clear();

    std::ifstream input(path, std::ios::binary);
    if (!input) {
        return false;
    }
    const std::vector<char> source((std::istreambuf_iterator<char>(input)),
                                   std::istreambuf_iterator<char>());
    const std::vector<std::string> tokens = SplitAsciiWhitespace(source);
    if (tokens.size() < 2U) {
        return true;
    }

    if (tokens[1] == "0") {
        m_CheckAllowedAddress = false;
        return true;
    }
    if (tokens[1] != "1") {
        return true;
    }
    m_CheckAllowedAddress = true;

    std::size_t cursor = 2;
    while (cursor < tokens.size()) {
        while (cursor < tokens.size() && tokens[cursor] != "#") {
            ++cursor;
        }
        if (cursor == tokens.size() || cursor + 2U >= tokens.size()) {
            break;
        }
        const std::string& ip = tokens[cursor + 1U];
        const auto port = ParseU16(tokens[cursor + 2U]);
        if (!port) {
            break;
        }
        const auto* first = reinterpret_cast<const std::uint8_t*>(ip.data());
        m_AllowedAddresses.push_back(AllowedAddress{
            std::vector<std::uint8_t>(first, first + ip.size()),
            *port});
        cursor += 3U;
    }
    return true;
}

bool CServer::IsAllowedAddress(std::span<const std::uint8_t> peerIP,
                               std::uint16_t peerPort) const noexcept
{
    return std::any_of(m_AllowedAddresses.begin(),
                       m_AllowedAddresses.end(),
                       [&](const AllowedAddress& address) {
                           return address.Matches(peerIP, peerPort);
                       });
}

void CServer::ConfigureReceiveGuard(bool enabled,
                                    std::int32_t bytesPerSecond,
                                    std::int32_t forbidTimeMs) noexcept
{
    m_CheckReceiveRate = enabled;
    m_ReceiveRateLimit = bytesPerSecond;
    m_ForbidTimeMs = forbidTimeMs;
}

void CServer::ConfigureSendLimits(std::int32_t maxInFlightSends,
                                  std::int32_t permittedSendBytes) noexcept
{
    m_MaxInFlightSends = maxInFlightSends;
    m_PermittedSendBytes = permittedSendBytes;
}

void CServer::ConfigureMaxClients(std::int32_t maxClients) noexcept
{
    m_MaxClients = maxClients;
}

void CServer::ConfigureTransportAfterHost(bool checkReceiveRate,
                                          std::int32_t maxInFlightSends,
                                          std::uint32_t receiveRateLimit,
                                          std::int32_t maxClients,
                                          bool checkMessageContent,
                                          std::uint32_t forbidTimeMs,
                                          std::uint32_t maximumMessageLength,
                                          std::int32_t permittedSendBytes) noexcept
{
    m_CheckReceiveRate = checkReceiveRate;
    m_MaxInFlightSends = maxInFlightSends;
    m_ReceiveRateLimit = std::bit_cast<std::int32_t>(receiveRateLimit);
    m_MaxClients = maxClients;
    m_CheckMessageContent = checkMessageContent;
    m_ForbidTimeMs = std::bit_cast<std::int32_t>(forbidTimeMs);
    m_MaximumMessageLength = maximumMessageLength;
    m_PermittedSendBytes = permittedSendBytes;
}

void CServer::ConfigureAcceptLimitsAfterHost(std::int32_t maxBacklog,
                                             std::int32_t newAcceptTimeoutMs) noexcept
{
    // Сохранён исходный порядок: после Host изменение backlog уже не влияет на
    // существующий listener; меняется только поле для следующего Host.
    m_MaxBacklog = maxBacklog;
    m_NewAcceptTimeoutMs = newAcceptTimeoutMs;
}

void CServer::AddForbiddenIP(std::uint32_t peerIPv4, std::uint32_t nowMs)
{
    m_ForbiddenIPs[peerIPv4] = nowMs;
}

bool CServer::FindForbiddenIP(std::uint32_t peerIPv4, std::uint32_t nowMs)
{
    const auto found = m_ForbiddenIPs.find(peerIPv4);
    if (found == m_ForbiddenIPs.end()) {
        return false;
    }
    if (nowMs - found->second <= std::bit_cast<std::uint32_t>(m_ForbidTimeMs)) {
        return true;
    }
    m_ForbiddenIPs.erase(found);
    return false;
}

std::vector<std::int32_t> CServer::ExpireNewAccepts(std::uint32_t nowMs)
{
    const std::uint32_t timeout = std::bit_cast<std::uint32_t>(m_NewAcceptTimeoutMs);
    std::vector<std::int32_t> expired;
    for (const auto& [socketId, acceptedAt] : m_NewAcceptSockets) {
        if (nowMs - acceptedAt >= timeout) {
            expired.push_back(socketId);
        }
    }
    for (const std::int32_t socketId : expired) {
        m_Commands.QuitBySocketID(socketId);
        m_NewAcceptSockets.erase(socketId);
    }
    return expired;
}

std::int32_t CServer::AddSendSize(std::int32_t amount, std::uint32_t nowMs) noexcept
{
    return m_SendCounter.Add(amount, nowMs);
}

std::int32_t CServer::AddReceiveSize(std::int32_t amount, std::uint32_t nowMs) noexcept
{
    return m_ReceiveCounter.Add(amount, nowMs);
}

CServerClient* CServer::FindClient(std::int32_t socketId) noexcept
{
    const auto found = m_Clients.find(socketId);
    return found == m_Clients.end() ? nullptr : found->second.state.get();
}

const CServerClient* CServer::FindClient(std::int32_t socketId) const noexcept
{
    const auto found = m_Clients.find(socketId);
    return found == m_Clients.end() ? nullptr : found->second.state.get();
}

void CServer::ProcessReceiveCommand(std::int32_t socketId,
                                    std::span<const std::uint8_t> buffer,
                                    std::uint32_t nowMs,
                                    std::vector<ServerSnapshotError>& errors)
{
    AcknowledgeFirstReceive(socketId);
    if (buffer.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) {
        errors.push_back({ServerSnapshotErrorKind::ClientSize,
                          socketId,
                          ServerClientSizeError::ReceiveSizeOverflowReactionUnknown});
        return;
    }

    CServerClient* client = FindClient(socketId);
    if (!client || client->IsClosing()) {
        return;
    }

    if (m_CheckReceiveRate) {
        const auto received = static_cast<std::int32_t>(buffer.size());
        AddReceiveSize(received, nowMs);
        const std::int32_t actual = client->AddPackageSize(received, nowMs);
        if (actual > m_ReceiveRateLimit) {
            OnReceiveRateExceeded(*client, actual, m_ReceiveRateLimit);
            AddForbiddenIP(client->MessageContext().peerIPv4, nowMs);
            m_Commands.QuitBySocketID(socketId);
            return;
        }
    }

    if (const auto error = client->AddReceiveData(buffer)) {
        errors.push_back({ServerSnapshotErrorKind::ClientSize, socketId, *error});
        return;
    }

    const ReceiveCallbackResult result = OnReceive(*client, nowMs);
    if (!result.success) {
        errors.push_back({ServerSnapshotErrorKind::Component, socketId, std::nullopt});
        if (result.action == ComponentReceiveErrorAction::ForbidAndQuit) {
            AddForbiddenIP(client->MessageContext().peerIPv4, nowMs);
            m_Commands.QuitBySocketID(socketId);
        }
    }
}

void CServer::BufferSendCommand(std::int32_t socketId,
                                std::span<const std::uint8_t> buffer,
                                SendOverflowRemoval removal,
                                std::vector<ServerSnapshotError>& errors)
{
    CServerClient* client = FindClient(socketId);
    if (!client) {
        return;
    }

    AddSendDataResult result = client->AddSendData(buffer, m_PermittedSendBytes);
    if (const auto* error = std::get_if<ServerClientSizeError>(&result)) {
        errors.push_back({ServerSnapshotErrorKind::ClientSize, socketId, *error});
        return;
    }
    if (std::get<AddSendDataOutcome>(result) != AddSendDataOutcome::LimitExceeded) {
        return;
    }

    if (removal == SendOverflowRemoval::Broadcast) {
        RemoveBroadcastOverflowClient(socketId);
    } else {
        RemoveClientWithCallback(socketId);
    }
}

void CServer::RemoveClientWithCallback(std::int32_t socketId)
{
    auto found = m_Clients.find(socketId);
    if (found == m_Clients.end()) {
        return;
    }

    m_ClientCount = WrapAdd32(m_ClientCount, -1);
    OnClose(*found->second.state);
    ShutdownSocket(found->second.socket);
    m_Clients.erase(found);
}

void CServer::RemoveBroadcastOverflowClient(std::int32_t socketId)
{
    auto found = m_Clients.find(socketId);
    if (found == m_Clients.end()) {
        return;
    }

    AcceptedServerClient removed = std::move(found->second);
    m_Clients.erase(found);
    OnClose(*removed.state);
    ShutdownSocket(removed.socket);
    removed.state.reset();
    m_ClientCount = WrapAdd32(m_ClientCount, -1);
}

void CServer::MarkClientClosing(std::int32_t socketId)
{
    if (CServerClient* client = FindClient(socketId)) {
        client->MarkClosing();
    }
}

void CServer::MarkAllClientsClosing()
{
    for (auto& [_, client] : m_Clients) {
        client.state->MarkClosing();
    }
}

bool CServer::AssignMapID(std::int32_t socketId, std::int32_t mapId)
{
    CServerClient* client = FindClient(socketId);
    if (!client) {
        return false;
    }
    client->SetMapID(mapId);
    m_MapIDSocketID[mapId] = socketId;
    return true;
}

bool CServer::AssignMapName(std::int32_t socketId, std::span<const std::uint8_t> mapName)
{
    CServerClient* client = FindClient(socketId);
    if (!client) {
        return false;
    }
    client->SetMapName(mapName);
    m_MapNameSocketID[std::vector<std::uint8_t>(mapName.begin(), mapName.end())] = socketId;
    return true;
}

void CServer::RemoveMapID(std::int32_t mapId)
{
    m_MapIDSocketID.erase(mapId);
}

void CServer::RemoveMapName(std::span<const std::uint8_t> mapName)
{
    m_MapNameSocketID.erase(std::vector<std::uint8_t>(mapName.begin(), mapName.end()));
}

void CServer::AcknowledgeFirstReceive(std::int32_t socketId)
{
    m_NewAcceptSockets.erase(socketId);
}
