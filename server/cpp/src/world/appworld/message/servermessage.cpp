#include "servermessage.h"

void WorldMessageHandlers::Set(const WorldMessageFamily family, Handler handler){m_Handlers[static_cast<std::size_t>(family)]=std::move(handler);}
void WorldMessageHandlers::Dispatch(const WorldMessageFamily family,WorldNet::CMessage& message){if(auto& handler=m_Handlers[static_cast<std::size_t>(family)];handler)handler(message);}
void WorldMessageHandlers::OnServer(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Server,message);}
