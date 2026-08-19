#pragma once

#include "../msgqueue.h"
#include "myserverclient.h"
#include "message.h"

#include <cstdint>
#include <optional>

/*
 * Owner: nets/netlogin/mynetserverclient_world.cpp
 *
 * LoginServer EXE/PDB: ctor 0x0006EC60, OnClose 0x0006ED10,
 * SetSendRevBuf 0x0006EDA0, OnReceive 0x0006EDF0.
 *
 * World envelope всегда [len, crc(len), crc(normalized message), message].
 * Ошибка CRC очищает accumulator, но этот component не добавляет IP-ban и не
 * ставит QUIT. OnClose всегда публикует 0xFF01 только с текущим map ID.
 *
 * Windows SetSendRevBuf(SO_SNDBUF=0) намеренно не имитируется похожим Linux
 * setsockopt: наблюдаемая backpressure-семантика пока не доказана.
 */
namespace LoginNet
{
enum class WorldReceiveErrorKind
{
    LengthChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    ContentChecksumMismatch,
};

struct WorldReceiveError
{
    WorldReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::int32_t messageType{};
    std::optional<CreateMessageError> messageError;
};

struct WorldReceiveResult
{
    std::int32_t messages{};
    std::size_t pendingBytes{};
    std::optional<WorldReceiveError> error;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return !error.has_value();
    }
};

class CMyNetServerClientWorld final : public CMyServerClient
{
public:
    CMyNetServerClientWorld(std::int32_t socketId,
                            std::uint32_t peerIPv4,
                            std::uint32_t nowMs);

    void OnClose(CMsgQueue<CMessage>& messages);
    [[nodiscard]] WorldReceiveResult OnReceive(CMsgQueue<CMessage>& messages);
};
}
