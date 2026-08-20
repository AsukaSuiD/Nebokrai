#include "onmsg_m2w_auction.h"
void SetMiscAuctionMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::MiscAuction,std::move(handler));}
void WorldMessageHandlers::OnMiscAuction(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::MiscAuction,message);}
