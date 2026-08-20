#include "gmamessage.h"
void SetGmaMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Gma,std::move(handler));}
void WorldMessageHandlers::OnGma(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Gma,message);}
