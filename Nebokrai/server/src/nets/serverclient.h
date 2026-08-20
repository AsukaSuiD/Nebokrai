#pragma once

#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Исходный владелец: nets/serverclient.cpp / nets/serverclient.h
 *
 * Поздняя реконструкция подтверждает общий CServerClient во всех пяти
 * входящих направлениях Auth/Billing/Login/Game/World. Источник истины —
 * соответствующие EXE/PDB; сохранённый Rust содержит уже выполненный reverse,
 * а архивная Linux-попытка используется только как практический C++-донор.
 *
 * Живой общий контракт: metadata принятого соединения, накопитель TCP receive,
 * накопитель server-side send, строгий send-limit, число незавершённых send
 * operations, двухступенчатый close flag и per-client receive-rate. Конкретный
 * 12-байтовая обёртка и CMessage принадлежат сетевым владельцам конкретных служб.
 *
 * Старые IOCP/PER_IO_OPERATION_DATA, raw buffers и ручные realloc/delete не
 * переносятся. Владение bytes выражено std::vector, а одна исходная WSASend
 * остаётся одним ServerSendBatch. Важная странность оригинала сохранена:
 * partial completion серверной отправки не повторял хвост — completion только
 * освобождал весь batch и публиковал SENDEND.
 */

inline constexpr std::size_t kServerClientInitialReceiveCapacity = 0x10'0000;
inline constexpr std::int32_t kDefaultPermittedServerSendBytes = 0xC800;

struct ServerClientMessageContext
{
    std::int32_t socketId{};
    std::int32_t mapId{};
    std::span<const std::uint8_t> mapName;
    std::uint32_t peerIPv4{};
};

enum class ServerClientSizeError
{
    ReceiveSizeOverflowReactionUnknown,
    SendSizeOverflowReactionUnknown,
};

enum class AddSendDataOutcome
{
    Buffered,
    IgnoredWhileClosing,
    LimitExceeded,
};

using AddSendDataResult = std::variant<AddSendDataOutcome, ServerClientSizeError>;

class ServerSendBatch
{
public:
    explicit ServerSendBatch(std::vector<std::uint8_t> buffer) noexcept;

    [[nodiscard]] std::size_t RequestedBytes() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Bytes() const noexcept;

private:
    std::vector<std::uint8_t> m_Buffer;
};

struct ServerSendCompletion
{
    std::size_t requested{};
    std::size_t transferred{};

    [[nodiscard]] constexpr bool IsComplete() const noexcept
    {
        return requested == transferred;
    }

    [[nodiscard]] constexpr std::size_t DroppedTailBytes() const noexcept
    {
        return requested - transferred;
    }
};

class CServerClient
{
public:
    CServerClient(std::int32_t socketId,
                  std::uint32_t peerIPv4,
                  std::uint32_t nowMs,
                  std::size_t receiveCapacity = kServerClientInitialReceiveCapacity);
    virtual ~CServerClient() = default;

    CServerClient(const CServerClient&) = delete;
    CServerClient& operator=(const CServerClient&) = delete;
    CServerClient(CServerClient&&) noexcept = default;
    CServerClient& operator=(CServerClient&&) noexcept = default;

    [[nodiscard]] ServerClientMessageContext MessageContext() const noexcept;

    void SetMapIdentity(std::int32_t mapId, std::span<const std::uint8_t> mapName);
    void SetMapID(std::int32_t mapId) noexcept;
    void SetMapName(std::span<const std::uint8_t> mapName);

    void MarkClosing() noexcept;
    void MarkOpen() noexcept;
    [[nodiscard]] bool IsClosing() const noexcept;
    [[nodiscard]] bool BeginClose() noexcept;

    [[nodiscard]] std::optional<ServerClientSizeError>
    AddReceiveData(std::span<const std::uint8_t> received);
    [[nodiscard]] std::size_t PendingReceiveBytes() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> ReceiveBytes() const noexcept;
    [[nodiscard]] bool ConsumeReceivePrefix(std::size_t count);
    void DiscardReceiveData() noexcept;

    [[nodiscard]] AddSendDataResult
    AddSendData(std::span<const std::uint8_t> data, std::int32_t permittedMax);
    [[nodiscard]] std::size_t PendingSendBytes() const noexcept;
    [[nodiscard]] std::optional<ServerSendBatch> BeginSend();
    void FinishSendOperation() noexcept;
    [[nodiscard]] std::int32_t IOOperations() const noexcept;

    [[nodiscard]] std::int32_t AddPackageSize(std::int32_t amount, std::uint32_t nowMs) noexcept;

private:
    class PackageRateCounter
    {
    public:
        explicit constexpr PackageRateCounter(std::uint32_t nowMs) noexcept
            : m_WindowStartedMs(nowMs)
        {
        }

        [[nodiscard]] std::int32_t Add(std::int32_t amount, std::uint32_t nowMs) noexcept;

    private:
        std::uint32_t m_WindowStartedMs{};
        std::int32_t m_WindowBytes{};
        std::int32_t m_BytesPerSecond{};
    };

    std::int32_t m_SocketID{};
    std::uint32_t m_PeerIPv4{};
    std::int32_t m_MapID{};
    std::vector<std::uint8_t> m_MapName;
    bool m_Closing{};
    bool m_CloseStarted{};
    std::vector<std::uint8_t> m_ReceiveBuffer;
    std::vector<std::uint8_t> m_SendBuffer;
    std::int32_t m_IOOperations{};
    PackageRateCounter m_ReceiveRate;
};
