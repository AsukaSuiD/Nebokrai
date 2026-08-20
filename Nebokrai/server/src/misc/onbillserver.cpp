#include "onbillserver.h"

#include "game.h"

namespace Misc
{
void OnMSG_M2M_Fuction(MiscNet::CMessage& message, CGame& game)
{
    if (message.MessageType() == 0x0016'EA01) {
        game.RequestReconnect();
        return;
    }
    if (message.MessageType() == 0x0016'EA02) {
        game.AuctionRoom().Clear();
        MiscNet::CMessage response(0x0015'EB05);
        static_cast<void>(response.Send(game.SendQueue(), false));
    }
}
}
