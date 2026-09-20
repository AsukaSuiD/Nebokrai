#pragma once

#include "../msgqueue.h"
#include "../serverclient.h"
#include "message.h"

#include <cstdint>
#include <optional>

/*
 * Исходный владелец: nets/netbilling/clientforgs.cpp / .h
 *
 * BillingServer EXE/PDB и Rust-реконструкция подтверждают accumulator
 * 0xA00000, 12-байтовый CRC-envelope, публикацию полного CMessage в FIFO и
 * безусловное 0x10EF01 при закрытии. Windows SO_SNDBUF=0 не имитируется:
 * совместимая Linux-семантика этого вызова не доказана.
 */
namespace BillingNet
{
enum class BillingReceiveErrorKind
{
    LengthChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    ContentChecksumMismatch,
};

struct BillingReceiveError
{
    BillingReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::int32_t messageType{};
    std::optional<CreateMessageError> messageError;
};

struct BillingReceiveResult
{
    std::int32_t messages{};
    std::size_t pendingBytes{};
    std::optional<BillingReceiveError> error;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return !error.has_value();
    }
};

class CClientForGS final : public CServerClient
{
public:
    CClientForGS(std::int32_t socketId,
                 std::uint32_t peerIPv4,
                 std::uint32_t nowMs);

    void OnClose(CMsgQueue<CMessage>& messages);
    [[nodiscard]] BillingReceiveResult OnReceive(CMsgQueue<CMessage>& messages);
};
}
