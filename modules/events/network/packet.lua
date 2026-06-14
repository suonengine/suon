local IncomingMessage = require("network.incoming_msg")

---@type PacketEvent
---Handles opcode-level dispatch.  `RawPacketEvent` feeds raw decrypted
---data here; the opcode is parsed and routed to per-opcode handlers.
---@class PacketEvent
local M = {}
M.__index = M

---@type table<integer, { handler: fun(Connection, IncomingMessage), priority: integer }[]>
local opcode_handlers = {}

---Register a handler for a specific opcode.
---
---Multiple handlers may be registered for the same opcode; they are
---sorted by `priority` (highest runs first).
---
---@param opcode integer
---@param handler fun(connection: Connection, msg: IncomingMessage)
---@param priority? EventPriority
function M:on(opcode, handler, priority)
    if not opcode_handlers[opcode] then
        opcode_handlers[opcode] = {}
    end

    local EventPriority = require("events.priority")

    table.insert(opcode_handlers[opcode], {
        handler = handler,
        priority = priority or EventPriority.NORMAL,
    })

    table.sort(opcode_handlers[opcode], function(a, b)
        return a.priority > b.priority
    end)
end

---@param opcode integer
---@param connection Connection
---@param msg IncomingMessage
function M:dispatch(opcode, connection, msg)
    local list = opcode_handlers[opcode]
    if not list then
        return
    end

    for _, entry in ipairs(list) do
        local ok, error = pcall(entry.handler, connection, msg)
        if not ok then
            print(string.format("[PacketEvent] Handler error for opcode 0x%04X: %s", opcode, tostring(error)))
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
