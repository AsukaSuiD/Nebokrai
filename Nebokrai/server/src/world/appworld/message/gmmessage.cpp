#include "gmmessage.h"
void SetGmMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Gm,std::move(handler));}
void WorldMessageHandlers::OnGm(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Gm,message);}
