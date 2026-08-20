#pragma once

#include "../serverclient.h"

/*
 * Исходный владелец: nets/netlogin/myserverclient.cpp / .h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * PDB подтверждает только CMyServerClient::CMyServerClient, destructor/vtable;
 * отдельного состояния и отдельной сетевой семантики владелец не добавляет.
 * `SetSendRevBuf` принадлежит общему CServerClient, а отдельный override есть у
 * CMyNetServerClient_World. Это точечно проверено по exact LoginServer.pdb.
 *
 * В современном C++ владелец поэтому остаётся тонким типом над CServerClient:
 * наследует безопасный owned state и не получает фиктивных методов/полей ради
 * старого 32-bit layout.
 */
namespace LoginNet
{
class CMyServerClient : public CServerClient
{
public:
    using CServerClient::CServerClient;
    ~CMyServerClient() override = default;
};
}
