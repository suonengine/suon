---Fired when a TCP connection is closed.
---@class ConnectionEndEvent : ConnectionEvent
---@field _connection Connection
local M = ConnectionEvent:define()

---@class ConnectionEndEvent : ConnectionEvent
ConnectionEndEvent = M

local MT = getmetatable(M)
---@return ConnectionEndEvent
MT.__call = function(self, id)
	return setmetatable({
		args = {
			id,
		},
		_connection = Connection(id),
	}, self)
end

return M
