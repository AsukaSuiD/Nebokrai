#include "teammessage.h"
void SetTeamMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Team,std::move(handler));}
void WorldMessageHandlers::OnTeam(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Team,message);}
