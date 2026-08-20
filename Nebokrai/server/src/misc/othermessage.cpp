#include "othermessage.h"

#include "game.h"

namespace Misc
{
void OnOtherMsg(MiscNet::CMessage& message, CGame& game)
{
    std::int32_t responseType{};
    if (message.MessageType() == 0x0007'F809) {
        if (game.NetClient() == nullptr) return;
        responseType = 0x0005'FA0A;
    } else if (message.MessageType() == 0x0007'F80B) {
        responseType = 0x0005'FA0C;
    } else {
        return;
    }

    MiscNet::CMessage response(responseType);
    response.Base().Add(std::int32_t{0});
    static_cast<void>(response.Send(game.SendQueue(), false));
}
}
