#pragma once

#include "../msgqueue.h"
#include "myserverclient.h"
#include "message.h"

#include <cstdint>
#include <optional>

/*
 * Owner: nets/netlogin/mynetserverclient_client.cpp
 *
 * LoginServer EXE/PDB: ctor 0x0006E590, OnClose 0x0006E660,
 * OnOneMessageSizeOver 0x0006E730, OnTotalMessageSizeOver 0x0006E7A0,
 * OnReceive 0x0006E810.
 *
 * Client envelope:
 * [total_len, optional crc(total_len), optional crc(rle), rle(message)].
 * Length/content CRC включаются независимо. Предел одного frame проверяется
 * только при включённой length-проверке. Допустимый opcode после RLE decode:
 * 0x2FD01..=0x3FBFF. Четыре доказанных ошибки требуют forbid IP + QUIT:
 * oversize, length CRC, content CRC и opcode range.
 *
 * OnClose публикует 0x10001 только при уже назначенном непустом CD-key.
 * Signed/short length и trailing RLE marker остаются неизвестными границами.
 */
namespace LoginNet
{
struct ClientReceiveSettings
{
    bool checkLengthCrc{};
    bool checkContentCrc{};
    std::uint32_t maximumMessageLength{};
};

enum class ClientReceiveErrorKind
{
    MessageLengthExceeded,
    LengthChecksumMismatch,
    ContentChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    OpcodeOutsideAllowedRange,
};

struct ClientReceiveError
{
    ClientReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t permitted{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::uint32_t opcode{};
    std::optional<CreateMessageError> messageError;

    [[nodiscard]] bool RequiresForbidAndQuit() const noexcept;
};

struct ClientReceiveResult
{
    std::int32_t messages{};
    std::size_t pendingBytes{};
    std::optional<ClientReceiveError> error;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return !error.has_value();
    }
};

class CMyNetServerClientClient final : public CMyServerClient
{
public:
    CMyNetServerClientClient(std::int32_t socketId,
                             std::uint32_t peerIPv4,
                             std::uint32_t nowMs);

    [[nodiscard]] bool OnClose(CMsgQueue<CMessage>& messages);
    [[nodiscard]] ClientReceiveResult OnReceive(ClientReceiveSettings settings,
                                                CMsgQueue<CMessage>& messages);
};
}
