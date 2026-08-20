#include "ccontainer.h"

void CContainerListener::OnObjectAdded(CContainer&, CBaseObject&, void*) {}
void CContainerListener::OnObjectRemoved(CContainer&, CBaseObject&, void*) {}
void CContainerListener::OnTraversingContainer(CContainer&, CBaseObject&) {}

bool CContainer::AddListener(CContainerListener& listener)
{
    if (std::ranges::find(m_Listeners, &listener) != m_Listeners.end()) {
        return false;
    }
    m_Listeners.push_back(&listener);
    return true;
}

bool CContainer::RemoveListener(CContainerListener& listener) noexcept
{
    return std::erase(m_Listeners, &listener) != 0;
}

void CContainer::NotifyAdded(CBaseObject& object, void* context)
{
    const auto listeners = m_Listeners;
    for (CContainerListener* listener : listeners) {
        if (listener != nullptr) {
            listener->OnObjectAdded(*this, object, context);
        }
    }
}

void CContainer::NotifyRemoved(CBaseObject& object, void* context)
{
    const auto listeners = m_Listeners;
    for (CContainerListener* listener : listeners) {
        if (listener != nullptr) {
            listener->OnObjectRemoved(*this, object, context);
        }
    }
}
