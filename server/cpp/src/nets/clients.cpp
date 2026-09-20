#include "clients.h"

#include <asio/cancel_after.hpp>
#include <asio/redirect_error.hpp>
#include <asio/use_awaitable.hpp>

#include <bit>
#include <utility>

namespace
{
constexpr std::int32_t kRateSampleThreshold = 0x9F'FFFF;

std::int32_t WrapAdd32(std::int32_t lhs, std::int32_t rhs) noexcept
{
    return std::bit_cast<std::int32_t>(
        std::bit_cast<std::uint32_t>(lhs) + std::bit_cast<std::uint32_t>(rhs));
}

std::int64_t WrapAdd64(std::int64_t lhs, std::int64_t rhs) noexcept
{
    return std::bit_cast<std::int64_t>(
        std::bit_cast<std::uint64_t>(lhs) + std::bit_cast<std::uint64_t>(rhs));
}
}

asio::awaitable<ClientConnectResult>
ConnectTcpIPv4(asio::ip::tcp::socket& socket, const asio::ip::tcp::endpoint& remote)
{
    asio::error_code error;
    co_await socket.async_connect(
        remote,
        asio::cancel_after(kClientConnectTimeout,
                           asio::redirect_error(asio::use_awaitable, error)));

    if (!error) {
        co_return ClientConnectResult{ClientConnectStatus::Connected, {}};
    }
    if (error == asio::error::operation_aborted) {
        co_return ClientConnectResult{ClientConnectStatus::Timeout, error};
    }
    co_return ClientConnectResult{ClientConnectStatus::IoError, error};
}

ClientSendCommand::ClientSendCommand(std::span<const std::uint8_t> buffer,
                                     std::int32_t flags)
    : m_Buffer(buffer.begin(), buffer.end()),
      m_Flags(flags)
{
}

std::span<const std::uint8_t> ClientSendCommand::Remaining() const noexcept
{
    return std::span<const std::uint8_t>(m_Buffer).subspan(m_Offset);
}

std::int32_t ClientSendCommand::Flags() const noexcept
{
    return m_Flags;
}

void ClientSendCommand::Advance(std::size_t written) noexcept
{
    m_Offset += written;
}

std::int32_t ClientSendQueue::SendToServer(std::span<const std::uint8_t> buffer,
                                           bool prioritized,
                                           std::int32_t flags)
{
    ClientSendCommand command(buffer, flags);
    if (prioritized) {
        m_Commands.Push_Front(std::move(command));
    } else {
        m_Commands.Push_Back(std::move(command));
    }
    return 1;
}

std::int32_t ClientSendQueue::Pending() const
{
    return m_Commands.GetSize();
}

asio::awaitable<ClientFlushResult> ClientSendQueue::Flush(asio::ip::tcp::socket& socket)
{
    Queue batch = m_Commands.CopyAllCommand();
    std::uint64_t bytesSent = 0;

    while (!batch.empty()) {
        ClientSendCommand command = std::move(batch.front());
        batch.pop_front();

        if (command.Flags() != 0) {
            const std::int32_t flags = command.Flags();
            RestoreUnsent(std::move(command), std::move(batch));
            co_return ClientFlushResult{
                .status = ClientFlushStatus::UnsupportedFlags,
                .bytesSent = bytesSent,
                .unsupportedFlags = flags,
            };
        }

        while (!command.Remaining().empty()) {
            asio::error_code error;
            const std::size_t written = co_await socket.async_write_some(
                asio::buffer(command.Remaining().data(), command.Remaining().size()),
                asio::redirect_error(asio::use_awaitable, error));

            if (error) {
                RestoreUnsent(std::move(command), std::move(batch));
                co_return ClientFlushResult{
                    .status = ClientFlushStatus::IoError,
                    .bytesSent = bytesSent,
                    .error = error,
                };
            }
            if (written == 0U) {
                RestoreUnsent(std::move(command), std::move(batch));
                co_return ClientFlushResult{
                    .status = ClientFlushStatus::WriteZero,
                    .bytesSent = bytesSent,
                };
            }

            command.Advance(written);
            bytesSent += static_cast<std::uint64_t>(written);
        }
    }

    co_return ClientFlushResult{
        .status = ClientFlushStatus::Drained,
        .bytesSent = bytesSent,
    };
}

void ClientSendQueue::RestoreUnsent(ClientSendCommand command, Queue remainder)
{
    remainder.push_front(std::move(command));
    m_Commands.AddCommandsQueueToFront(std::move(remainder));
}

std::int32_t TransferCounter::Add(std::int32_t amount, std::uint32_t nowMs) noexcept
{
    m_WindowBytes = WrapAdd32(m_WindowBytes, amount);
    m_TotalBytes = WrapAdd64(m_TotalBytes, static_cast<std::int64_t>(amount));

    if (m_WindowBytes > kRateSampleThreshold) {
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

std::int64_t TransferCounter::TotalBytes() const noexcept
{
    return m_TotalBytes;
}
