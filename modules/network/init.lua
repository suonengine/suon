---Network module.  Provides connection management, message
---serialisation, and default event handlers.
---@class Network

local Connection = require("network.connection")
local IncomingMessage = require("network.incoming_msg")
local ConnectionBeginEvent = require("events.network.connection_begin")
local ConnectionEndEvent = require("events.network.connection_end")
local RawPacketEvent = require("events.network.raw_packet")
local PacketEvent = require("events.network.packet")
local RawHttpRequestEvent = require("events.http.raw_request")
local HttpRequestEvent = require("events.http.request")

ConnectionBeginEvent:on(function(event)
end)

---Remove from cache on disconnect.
ConnectionEndEvent:on(function(event)
	local connection = event:getConnection()
	if connection then
		connection:remove()
	end
end)

---Feed raw data into the opcode dispatcher.
RawPacketEvent:on(function(event)
	local connection = event:getConnection()
	if not connection then
		return
	end

	PacketEvent:trigger(connection, event:getData())
end)

---Feed raw HTTP requests into the method+path dispatcher.
RawHttpRequestEvent:on(function(event)
	HttpRequestEvent:trigger(event)
end)

return {
	Connection = Connection,
	IncomingMessage = IncomingMessage,
	OutgoingMessage = OutgoingMessage,
}
