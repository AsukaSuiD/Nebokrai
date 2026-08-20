#pragma once

#include "socketcommands.h"

#include <asio.hpp>

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <span>
#include <system_error>
#include <vector>

/*
 * Исходный владелец: nets/clients.cpp / nets/clients.h
 *
 * Общий исходящий CClient подтверждён Login/Misc/Game/World EXE/PDB. Поздняя
 * Rust-реконструкция уже отделила семантику Nebokrai от обвязки WinSock:
 * send-команда немедленно владеет копией bytes, priority определяет front/back,
 * partial send продолжается с хвоста, а при terminal error текущий хвост и весь
 * оставшийся snapshot возвращаются перед командами, пришедшими параллельно.
 * Connect ждёт ровно 10 секунд уже после legacy hostname resolution.
 *
 * Новый владелец Linux использует самостоятельный Asio вместо WSAEventSelect,
 * FD_WRITE/FD_CONNECT, thread handles и busy Sleep. Asio меняет только способ
 * ожидания readiness; порядок очереди, ownership и 10-секундная граница
 * сохраняются. DNS и malformed receive framing остаются service-specific
 * владельцам, потому что их недоказанные края нельзя унифицировать здесь.
 */

inline constexpr auto kClientConnectTimeout = std::chrono::seconds(10);
inline constexpr std::size_t kClientInitialReceiveCapacity = 0x10'0000;
inline constexpr std::size_t kClientMaxReceiveChunk = 0x2800;

enum class ClientConnectStatus
{
    Connected,
    Timeout,
    IoError,
};

struct ClientConnectResult
{
    ClientConnectStatus status{ClientConnectStatus::IoError};
    std::error_code error;
};

[[nodiscard]] asio::awaitable<ClientConnectResult>
ConnectTcpIPv4(asio::ip::tcp::socket& socket, const asio::ip::tcp::endpoint& remote);

class ClientSendCommand
{
public:
    ClientSendCommand(std::span<const std::uint8_t> buffer, std::int32_t flags);

    [[nodiscard]] std::span<const std::uint8_t> Remaining() const noexcept;
    [[nodiscard]] std::int32_t Flags() const noexcept;
    void Advance(std::size_t written) noexcept;

private:
    std::vector<std::uint8_t> m_Buffer;
    std::size_t m_Offset{};
    std::int32_t m_Flags{};
};

enum class ClientFlushStatus
{
    Drained,
    UnsupportedFlags,
    WriteZero,
    IoError,
};

struct ClientFlushResult
{
    ClientFlushStatus status{ClientFlushStatus::Drained};
    std::uint64_t bytesSent{};
    std::int32_t unsupportedFlags{};
    std::error_code error;
};

class ClientSendQueue
{
public:
    [[nodiscard]] std::int32_t SendToServer(std::span<const std::uint8_t> buffer,
                                            bool prioritized,
                                            std::int32_t flags);
    [[nodiscard]] std::int32_t Pending() const;

    [[nodiscard]] asio::awaitable<ClientFlushResult> Flush(asio::ip::tcp::socket& socket);

private:
    using Queue = typename CSocketCommands<ClientSendCommand>::Queue;

    void RestoreUnsent(ClientSendCommand command, Queue remainder);

    CSocketCommands<ClientSendCommand> m_Commands;
};

class TransferCounter
{
public:
    explicit constexpr TransferCounter(std::uint32_t nowMs = 0) noexcept
        : m_WindowStartedMs(nowMs)
    {
    }

    [[nodiscard]] std::int32_t Add(std::int32_t amount, std::uint32_t nowMs) noexcept;
    [[nodiscard]] std::int64_t TotalBytes() const noexcept;

private:
    std::uint32_t m_WindowStartedMs{};
    std::int32_t m_WindowBytes{};
    std::int32_t m_BytesPerSecond{};
    std::int64_t m_TotalBytes{};
};
