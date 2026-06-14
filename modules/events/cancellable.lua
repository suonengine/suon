local Event = require("events.event")

---@type CancellableEvent
---Base class for events that can be cancelled.
---@class CancellableEvent : Event
---@field cancelled boolean
local M = Event:define()

---@return boolean isCancelled # true if a handler called setCancelled(true)
function M:isCancelled()
	return self.cancelled or false
end

---@param value boolean
---@return nil
function M:setCancelled(value)
	self.cancelled = value
end

return M
