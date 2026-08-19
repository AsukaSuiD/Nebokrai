#pragma once

#include <mutex>

/*
 * Owner: loginserver/applogin/locker.cpp
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\complite_version\fengyun_russia\trunk\server\loginserver\applogin\locker.cpp
 * RVA: ctor 0x00023CD0, dtor 0x00023CE0, Acquire 0x00023CF0,
 * Release 0x00023D00.
 *
 * Все четыре функции — тонкая оболочка над Win32 CRITICAL_SECTION:
 * Initialize/Delete/Enter/LeaveCriticalSection. Существенная для вызывающего
 * кода деталь — рекурсивность critical section. Поэтому Windows primitive и
 * ручной lifecycle не переносятся: std::recursive_mutex сохраняет контракт.
 *
 * StringCopyWorkerA, попавший рядом в декомпиляцию, является CRT/library
 * helper и не относится к семантике Locker; отдельная копия этого кода здесь
 * намеренно не создаётся.
 */
namespace Login
{
class Locker
{
public:
    Locker() = default;
    ~Locker() = default;

    Locker(const Locker&) = delete;
    Locker& operator=(const Locker&) = delete;

    void Acquire();
    void Release();

private:
    std::recursive_mutex m_Mutex;
};
}
