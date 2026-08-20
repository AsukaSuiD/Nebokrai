#include "othermessage.h"
void SetOtherMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Other,std::move(handler));}
void WorldMessageHandlers::OnOther(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Other,message);}
