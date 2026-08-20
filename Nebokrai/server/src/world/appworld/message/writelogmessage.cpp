#include "writelogmessage.h"
void SetWriteLogMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::WriteLog,std::move(handler));}
void WorldMessageHandlers::OnWriteLog(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::WriteLog,message);}
