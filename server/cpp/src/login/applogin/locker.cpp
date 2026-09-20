#include "locker.h"

namespace Login
{
void Locker::Acquire()
{
    m_Mutex.lock();
}

void Locker::Release()
{
    m_Mutex.unlock();
}
}
