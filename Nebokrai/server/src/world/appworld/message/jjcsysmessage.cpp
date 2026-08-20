#include "jjcsysmessage.h"
void SetJjcMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Jjc,std::move(handler));}
void WorldMessageHandlers::OnJjcSystem(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Jjc,message);}
