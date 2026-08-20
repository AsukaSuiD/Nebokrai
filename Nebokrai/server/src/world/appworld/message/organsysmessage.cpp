#include "organsysmessage.h"
void SetOrganizingMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Organizing,std::move(handler));}
void WorldMessageHandlers::OnOrganizingSystem(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Organizing,message);}
