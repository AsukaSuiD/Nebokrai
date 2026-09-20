#include "gmmessage.h"

#include <vector>

namespace Login
{
namespace
{
constexpr std::int32_t kCdKeyBanMessageType = 0x00020001;
constexpr std::size_t kAccountLimit = 0x100U;
}

GmMessageHandler::GmMessageHandler(IGmMessageContext& context) noexcept
    : m_Context(context)
{
}

GmMessageResult GmMessageHandler::OnGMMessage(LoginNet::CMessage& message)
{
    const std::int32_t messageType = message.MessageType();
    if (messageType != kCdKeyBanMessageType) {
        return GmMessageOutcome{
            .kind = GmMessageOutcomeKind::Unsupported,
            .messageType = messageType,
        };
    }

    const auto account =
        message.Base().GetStrBytes(kAccountLimit).value_or(
            std::vector<std::uint8_t>{});
    const std::int32_t durationMinutes = message.Base().GetLong();
    const auto banResult = m_Context.CdKeyBan(account, durationMinutes);
    if (!banResult.has_value()) {
        return GmMessageError::DatabaseOwnerMissing;
    }

    return GmMessageOutcome{
        .kind = GmMessageOutcomeKind::BanAttempted,
        .messageType = messageType,
        .banSucceeded = *banResult,
    };
}
}
