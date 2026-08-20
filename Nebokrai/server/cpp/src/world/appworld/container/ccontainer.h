#pragma once

#include <algorithm>
#include <vector>

class CBaseObject;
class CContainer;

/*
 * Исходный владелец: WorldServer/appworld/container/ccontainer.cpp / .h.
 * Listener-ы являются non-owning синхронными наблюдателями. Встроенные
 * listener-ы стандартных goods-контейнеров в оригинале возвращали 1 без
 * побочных эффектов; интерфейс сохранён для DB/player listeners.
 */
class CContainerListener
{
public:
    virtual ~CContainerListener() = default;
    virtual void OnObjectAdded(CContainer& container, CBaseObject& object, void* context);
    virtual void OnObjectRemoved(CContainer& container, CBaseObject& object, void* context);
    virtual void OnTraversingContainer(CContainer& container, CBaseObject& object);
};

class CContainer
{
public:
    virtual ~CContainer() = default;

    bool AddListener(CContainerListener& listener);
    bool RemoveListener(CContainerListener& listener) noexcept;
    virtual void Traverse(CContainerListener& listener) = 0;

protected:
    void NotifyAdded(CBaseObject& object, void* context);
    void NotifyRemoved(CBaseObject& object, void* context);

private:
    std::vector<CContainerListener*> m_Listeners;
};
