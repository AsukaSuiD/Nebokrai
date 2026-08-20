#include "countrymessage.h"
void SetCountryMessageHandler(WorldMessageHandlers& handlers,WorldMessageHandlers::Handler handler){handlers.Set(WorldMessageFamily::Country,std::move(handler));}
void WorldMessageHandlers::OnCountry(WorldNet::CMessage& message){Dispatch(WorldMessageFamily::Country,message);}
