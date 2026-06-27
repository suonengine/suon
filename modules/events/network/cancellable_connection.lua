---Base class for cancellable connection events.
---@class CancellableConnectionEvent : ConnectionEvent
---@field _connection Connection
---@field cancelled boolean
local M = ConnectionEvent:define()

---@class CancellableConnectionEvent : ConnectionEvent
CancellableConnectionEvent = M

---@return boolean isCancelled
function M:isCancelled()
	return self.cancelled or false
end

---@param value boolean
function M:setCancelled(value)
	self.cancelled = value
end

return M
