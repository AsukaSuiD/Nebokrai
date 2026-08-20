#pragma once

namespace BillingNet
{
class CMessage;
class CServerForGS;
}

namespace Billing
{
struct ServerMessageOutcome
{
    enum class Kind { Unsupported, Connected, Disconnected } kind{Kind::Unsupported};
    bool allowed{};
};

/* Исходный владелец: appbilling/servermessage.cpp, ветви 0xEF101/0x10EF01. */
[[nodiscard]] ServerMessageOutcome
OnServerMessage(BillingNet::CMessage& message,
                BillingNet::CServerForGS* server);
}
