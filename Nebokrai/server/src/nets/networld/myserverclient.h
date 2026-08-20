#pragma once

#include "../msgqueue.h"
#include "../serverclient.h"
#include "message.h"

#include <cstdint>
#include <optional>

/*
 * Исходный владелец: nets/networld/myserverclient.cpp / .h.
 *
 * Принятое GameServer-соединение WorldServer. Сохраняет исходный accumulator
 * 0x1400000, CRC-проверку обоих слов envelope, metadata соединения и synthetic
 * close 0x3FC02. Ручные receive/send buffers заменены CServerClient.
 */
namespace WorldNet
{
enum class GameServerReceiveErrorKind
{
    LengthChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    ContentChecksumMismatch,
};

struct GameServerReceiveError
{
    GameServerReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::int32_t messageType{};
    std::optional<CreateMessageError> messageError;
};

struct GameServerReceiveResult
{
    std::int32_t messages{};
    std::size_t pendingBytes{};
    std::optional<GameServerReceiveError> error;
    [[nodiscard]] explicit operator bool() const noexcept { return !error.has_value(); }
};

class CMyServerClient final : public CServerClient
{
public:
    CMyServerClient(std::int32_t socketId,
                    std::uint32_t peerIPv4,
                    std::uint32_t nowMs);

    void OnClose(CMsgQueue<CMessage>& messages);
    [[nodiscard]] GameServerReceiveResult OnReceive(CMsgQueue<CMessage>& messages,
                                                    std::uint32_t receiveTimeMs);
};
}
