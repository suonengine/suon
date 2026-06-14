local Event = require("events.event")
local Connection = require("network.connection")

---@type ConnectionEvent
---Base class for events involving a network connection.
---@class ConnectionEvent : Event
---@field _connection Connection
local M = Event:define()

local MT = getmetatable(M)
---@return ConnectionEvent
MT.__call = function(self, id, ip, port)
	return setmetatable({
		args = {
			id,
			ip,
			port
		},
		_connection = Connection(id, ip, port),
	}, self)
end

---@return Connection connection
function M:getConnection()
	return self._connection
end

return M
