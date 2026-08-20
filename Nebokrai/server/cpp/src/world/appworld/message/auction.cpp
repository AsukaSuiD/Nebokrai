#include "auction.h"
void SetServerAuctionMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::ServerAuction,std::move(handler));}
void WorldMessageHandlers::OnServerAuction(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::ServerAuction,message);}
