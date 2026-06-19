local IncomingMessage = require("network.incoming_msg")

---@type PacketEvent
---Handles opcode-level dispatch. `RawPacketEvent` feeds raw decrypted
---data here; the opcode is parsed and routed to per-opcode handlers.
---
---Handlers can be registered with an optional port filter so they only
---fire for connections on a specific server port:
---
---    PacketEvent:on(7171, 0x14, function(connection, msg) ... end)
---    PacketEvent:on(0x01, function(connection, msg) ... end) -- any port
---
---@class PacketEvent
local M = {}
M.__index = M

---@type table<integer, { port: integer?, handler: fun(Connection, IncomingMessage), priority: integer }[]>
local opcode_handlers = {}

---Tracks whether any handler list needs re-sorting before dispatch.
local dirty = false

---Register a handler for a specific opcode.
---
---When `port` is given (first argument), the handler only fires for
---connections on that server port.
---
---@param portOrOpcode integer
---@param opcodeOrHandler integer | fun(connection: Connection, msg: IncomingMessage)
---@param handlerOrPriority? fun(connection: Connection, msg: IncomingMessage) | integer
---@param optPriority? integer
function M:on(portOrOpcode, opcodeOrHandler, handlerOrPriority, optPriority)
	local port, opcode, handler, priority

	if type(portOrOpcode) == "number" and type(opcodeOrHandler) == "number" then
		port = portOrOpcode
		opcode = opcodeOrHandler
		handler = handlerOrPriority
		priority = optPriority
	else
		port = nil
		opcode = portOrOpcode
		handler = opcodeOrHandler
		priority = handlerOrPriority
	end

	if not opcode_handlers[opcode] then
		opcode_handlers[opcode] = {}
	end

	local EventPriority = require("events.priority")

	table.insert(opcode_handlers[opcode], {
		port = port,
		handler = handler,
		priority = priority or EventPriority.NORMAL,
	})

	dirty = true
end

---@param opcode integer
---@param connection Connection
---@param msg IncomingMessage
function M:dispatch(opcode, connection, msg)
	if dirty then
		for _, handlers in pairs(opcode_handlers) do
			table.sort(handlers, function(a, b)
				return a.priority > b.priority
			end)
		end
		dirty = false
	end

	local list = opcode_handlers[opcode]
	if not list then
		return
	end

	for _, entry in ipairs(list) do
		if not entry.port or entry.port == connection:getPort() then
			local ok, error = pcall(entry.handler, connection, msg)
			if not ok then
				print(string.format("[PacketEvent] Handler error for opcode 0x%04X: %s", opcode, tostring(error)))
			end
		end
	end
end

---Receive raw decrypted data from a connection, parse the opcode,
---and dispatch to the registered handler(s).
---@param connection Connection
---@param raw string
function M:trigger(connection, raw)
	if #raw < 1 then
		return
	end

	local msg = IncomingMessage(raw)
	local opcode = msg:getU8()
	self:dispatch(opcode, connection, msg)
end

return M
