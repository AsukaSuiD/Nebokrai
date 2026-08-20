#include "playermessage.h"
void SetPlayerMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Player,std::move(handler));}
void WorldMessageHandlers::OnPlayer(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Player,message);}
