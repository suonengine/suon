local Players = require("players")

---@class PlayerPacketEvent
local M = {}

---@class PlayerPacketEvent
PlayerPacketEvent = M

---Register a handler for a specific opcode on the game port (7171).
---The handler receives a `Player` instead of a `Connection`.
---It is only called if the player is found.
---@param opcode integer
---@param handler fun(player: Player, msg: IncomingMessage)
---@param priority? integer
function M:on(opcode, handler, priority)
	PacketEvent:onPort(7171, opcode, function(connection, msg)
		local player = Players.findByConnection(connection)
		if not player then
			return
		end
		handler(player, msg)
	end, priority)
end

---Register a handler for a specific opcode on any port.
---The handler receives a `Player` instead of a `Connection`.
---It is only called if the player is found.
---@param opcode integer
---@param handler fun(player: Player, msg: IncomingMessage)
---@param priority? integer
function M:onAny(opcode, handler, priority)
	PacketEvent:onAny(opcode, function(connection, msg)
		local player = Players.findByConnection(connection)
		if not player then
			return
		end
		handler(player, msg)
	end, priority)
end

return M
