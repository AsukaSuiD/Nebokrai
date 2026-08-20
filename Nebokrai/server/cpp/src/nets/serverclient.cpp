#include "serverclient.h"

#include <bit>
#include <limits>
#include <utility>

namespace
{
constexpr std::int32_t kReceiveRateSampleThreshold = 0xC7FF;

std::int32_t WrapAdd(std::int32_t lhs, std::int32_t rhs) noexcept
{
    const auto left = std::bit_cast<std::uint32_t>(lhs);
    const auto right = std::bit_cast<std::uint32_t>(rhs);
    return std::bit_cast<std::int32_t>(left + right);
}
}

ServerSendBatch::ServerSendBatch(std::vector<std::uint8_t> buffer) noexcept
    : m_Buffer(std::move(buffer))
{
}

std::size_t ServerSendBatch::RequestedBytes() const noexcept
{
    return m_Buffer.size();
}

std::span<const std::uint8_t> ServerSendBatch::Bytes() const noexcept
{
    return m_Buffer;
}

CServerClient::CServerClient(std::int32_t socketId,
                             std::uint32_t peerIPv4,
                             std::uint32_t nowMs,
                             std::size_t receiveCapacity)
    : m_SocketID(socketId),
      m_PeerIPv4(peerIPv4),
      m_ReceiveRate(nowMs)
{
    m_ReceiveBuffer.reserve(receiveCapacity);
}

ServerClientMessageContext CServerClient::MessageContext() const noexcept
{
    return {
        .socketId = m_SocketID,
        .mapId = m_MapID,
        .mapName = m_MapName,
        .peerIPv4 = m_PeerIPv4,
    };
}

void CServerClient::SetMapIdentity(std::int32_t mapId, std::span<const std::uint8_t> mapName)
{
    m_MapID = mapId;
    SetMapName(mapName);
}

void CServerClient::SetMapID(std::int32_t mapId) noexcept
{
    m_MapID = mapId;
}

void CServerClient::SetMapName(std::span<const std::uint8_t> mapName)
{
    m_MapName.assign(mapName.begin(), mapName.end());
}

void CServerClient::MarkClosing() noexcept
{
    m_Closing = true;
}

void CServerClient::MarkOpen() noexcept
{
    m_Closing = false;
    m_CloseStarted = false;
}

bool CServerClient::IsClosing() const noexcept
{
    return m_Closing;
}

bool CServerClient::BeginClose() noexcept
{
    if (!m_Closing || m_CloseStarted) {
        return false;
    }

    m_CloseStarted = true;
    return true;
}

std::optional<ServerClientSizeError>
CServerClient::AddReceiveData(std::span<const std::uint8_t> received)
{
    if (received.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) ||
        m_ReceiveBuffer.size() >
            static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) - received.size()) {
        return ServerClientSizeError::ReceiveSizeOverflowReactionUnknown;
    }

    m_ReceiveBuffer.insert(m_ReceiveBuffer.end(), received.begin(), received.end());
    return std::nullopt;
}

std::size_t CServerClient::PendingReceiveBytes() const noexcept
{
    return m_ReceiveBuffer.size();
}

std::span<const std::uint8_t> CServerClient::ReceiveBytes() const noexcept
{
    return m_ReceiveBuffer;
}

bool CServerClient::ConsumeReceivePrefix(std::size_t count)
{
    if (count > m_ReceiveBuffer.size()) {
        return false;
    }

    m_ReceiveBuffer.erase(m_ReceiveBuffer.begin(),
                          m_ReceiveBuffer.begin() + static_cast<std::ptrdiff_t>(count));
    return true;
}

void CServerClient::DiscardReceiveData() noexcept
{
    m_ReceiveBuffer.clear();
}

AddSendDataResult CServerClient::AddSendData(std::span<const std::uint8_t> data,
                                             std::int32_t permittedMax)
{
    if (m_Closing) {
        return AddSendDataOutcome::IgnoredWhileClosing;
    }

    if (data.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) ||
        m_SendBuffer.size() >
            static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) - data.size()) {
        return ServerClientSizeError::SendSizeOverflowReactionUnknown;
    }

    const auto combined = static_cast<std::int32_t>(m_SendBuffer.size() + data.size());
    if (combined > permittedMax) {
        return AddSendDataOutcome::LimitExceeded;
    }

    m_SendBuffer.insert(m_SendBuffer.end(), data.begin(), data.end());
    return AddSendDataOutcome::Buffered;
}

std::size_t CServerClient::PendingSendBytes() const noexcept
{
    return m_SendBuffer.size();
}

std::optional<ServerSendBatch> CServerClient::BeginSend()
{
    if (m_SendBuffer.empty()) {
        return std::nullopt;
    }

    m_IOOperations = WrapAdd(m_IOOperations, 1);
    std::vector<std::uint8_t> batch;
    batch.swap(m_SendBuffer);
    return ServerSendBatch(std::move(batch));
}

void CServerClient::FinishSendOperation() noexcept
{
    m_IOOperations = WrapAdd(m_IOOperations, -1);
}

std::int32_t CServerClient::IOOperations() const noexcept
{
    return m_IOOperations;
}

std::int32_t CServerClient::AddPackageSize(std::int32_t amount, std::uint32_t nowMs) noexcept
{
    return m_ReceiveRate.Add(amount, nowMs);
}

std::int32_t CServerClient::PackageRateCounter::Add(std::int32_t amount,
                                                    std::uint32_t nowMs) noexcept
{
    m_WindowBytes = WrapAdd(m_WindowBytes, amount);
    if (m_WindowBytes > kReceiveRateSampleThreshold) {
        const std::uint32_t elapsedMs = nowMs - m_WindowStartedMs;
        if (elapsedMs != 0U) {
            const std::uint32_t scaled =
                std::bit_cast<std::uint32_t>(m_WindowBytes) * std::uint32_t{1000};
            m_BytesPerSecond = std::bit_cast<std::int32_t>(scaled / elapsedMs);
            m_WindowStartedMs = nowMs;
            m_WindowBytes = 0;
        }
    }

    return m_BytesPerSecond;
}
