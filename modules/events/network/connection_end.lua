local ConnectionEvent = require("events.network.connection")
local Connection = require("network.connection")

---@type ConnectionEndEvent
---Fired when a TCP connection is closed.
---@class ConnectionEndEvent : ConnectionEvent
---@field _connection Connection
local M = ConnectionEvent:define()

local MT = getmetatable(M)
---@return ConnectionEndEvent
MT.__call = function(self, id)
	return setmetatable({
		args = {
			id
		},
		_connection = Connection(id),
	}, self)
end

return M
