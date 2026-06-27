---Handles opcode-level dispatch. RawPacketEvent feeds raw decrypted
---data here; the opcode is parsed and routed to per-opcode handlers.
---@class PacketEvent
local M = {}
M.__index = M

---@class PacketEvent
PacketEvent = M

---@class PacketHandlerEntry
---@field port integer?
---@field handler fun(connection: Connection, msg: IncomingMessage)
---@field priority integer

---@type table<integer, PacketHandlerEntry[]>
local opcode_handlers = {}

local dirty = false

local EventPriority = require("events.priority")

---Register a handler for a specific opcode on a given server port.
---@param port integer
---@param opcode integer
---@param handler fun(connection: Connection, msg: IncomingMessage)
---@param priority? integer
function M:onPort(port, opcode, handler, priority)
	if not opcode_handlers[opcode] then
		opcode_handlers[opcode] = {}
	end

	table.insert(opcode_handlers[opcode], {
		port = port,
		handler = handler,
		priority = priority or EventPriority.NORMAL,
	})

	dirty = true
end

---Register a handler for a specific opcode on any port.
---@param opcode integer
---@param handler fun(connection: Connection, msg: IncomingMessage)
---@param priority? integer
function M:onAny(opcode, handler, priority)
	if not opcode_handlers[opcode] then
		opcode_handlers[opcode] = {}
	end

	table.insert(opcode_handlers[opcode], {
		port = nil,
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
