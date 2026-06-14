local ConnectionEvent = require("events.network.connection")
local Connection = require("network.connection")

---@type RawPacketEvent
---Fired when raw (decrypted) data arrives from a TCP connection.
---@class RawPacketEvent : ConnectionEvent
---@field _connection Connection
---@field data string
local M = ConnectionEvent:define()

local MT = getmetatable(M)
---@return RawPacketEvent
MT.__call = function(self, id, data)
	return setmetatable({
		args = {
			id,
			data
		},
		_connection = Connection(id),
		data = data,
	}, self)
end

---@return string data # raw bytes from the client
function M:getData()
	return self.data
end

return M
