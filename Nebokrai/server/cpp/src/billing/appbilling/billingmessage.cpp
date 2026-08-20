#include "billingmessage.h"

#include "billingplayermanager.h"
#include "../../nets/netbilling/message.h"

#include <bit>
#include <utility>

namespace Billing
{
namespace
{
constexpr std::int32_t kAccountBalanceRequest = 0x000E'F201;
constexpr std::int32_t kIncrementPurchaseRequest = 0x000E'F202;
constexpr std::int32_t kPlayerTradeRequest = 0x000E'F203;
constexpr std::size_t kBillingStringLimit = 0x20;

std::vector<std::uint8_t> ReadString(BillingNet::CMessage& message)
{
    auto value = message.Base().GetStrBytes(kBillingStringLimit);
    return value ? std::move(*value) : std::vector<std::uint8_t>{};
}
}

void OnBillingMessage(BillingNet::CMessage& message,
                      CBillingPlayerManager& playerManager)
{
    switch (message.MessageType()) {
    case kAccountBalanceRequest: {
        std::vector<std::uint8_t> identity = ReadString(message);
        if (identity.empty()) {
            return;
        }
        static_cast<void>(playerManager.PushAccountRequest(TagAccInfo{
            .playerId = message.Base().GetLong(),
            .playerIdentity = std::move(identity),
            .gameServerId = message.SocketID()}));
        break;
    }
    case kIncrementPurchaseRequest: {
        TagTradeNode request;
        request.buyerId = message.Base().GetLong();
        request.buyerIdentity = ReadString(message);
        request.buyerIp = ReadString(message);
        request.buyerName = ReadString(message);
        request.yuanbao = std::bit_cast<std::uint32_t>(message.Base().GetLong());
        request.goodsId = message.Base().GetLong();
        request.goodsNumber = message.Base().GetLong();
        request.sessionId = message.Base().GetLong();
        request.loginServerId = message.Base().GetLong();
        request.worldServerId = message.Base().GetLong();
        request.gameServerId = message.SocketID();
        static_cast<void>(playerManager.PushTradeRequest(std::move(request)));
        break;
    }
    case kPlayerTradeRequest: {
        TagTradeNode request;
        request.tradeType = message.Base().GetLong();
        request.buyerId = message.Base().GetLong();
        request.sellerId = message.Base().GetLong();
        request.buyerIdentity = ReadString(message);
        request.sellerIdentity = ReadString(message);
        request.buyerIp = ReadString(message);
        request.sellerIp = ReadString(message);
        request.buyerName = ReadString(message);
        request.sellerName = ReadString(message);
        request.yuanbao = std::bit_cast<std::uint32_t>(message.Base().GetLong());
        request.goodsId = message.Base().GetLong();
        request.goodsNumber = message.Base().GetLong();
        request.sessionId = message.Base().GetLong();
        request.pluginId = message.Base().GetLong();
        request.loginServerId = message.Base().GetLong();
        request.worldServerId = message.Base().GetLong();
        static_cast<void>(message.Base().GetGUID(request.goodsGuid));
        request.gameServerId = message.SocketID();
        static_cast<void>(playerManager.PushTradeRequest(std::move(request)));
        break;
    }
    default:
        break;
    }
}
}
