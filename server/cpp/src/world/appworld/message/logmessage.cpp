#include "logmessage.h"
void SetLogMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Log,std::move(handler));}
void WorldMessageHandlers::OnLog(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Log,message);}
