#pragma once

namespace BillingNet
{
class CMessage;
}

namespace Billing
{
class CBillingPlayerManager;

/*
 * Исходный владелец: appbilling/billingmessage.cpp.
 * Три ветви 0xEF201..0xEF203 восстановлены по BillingServer EXE/PDB и Rust.
 * Обработчик только читает payload и ставит глубокую копию записи в FIFO;
 * результат финансовой операции принадлежит DB-worker.
 */
void OnBillingMessage(BillingNet::CMessage& message,
                      CBillingPlayerManager& playerManager);
}
