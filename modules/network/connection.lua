local storage = setmetatable({}, { __mode = "k" })

---@type Connection
---A remote TCP connection.
---@class Connection
---@field _id integer
---@field _ip string
---@field _port integer
---@field send fun(self: Connection, data: string)
---@field close fun(self: Connection)
local M = {}
M.__index = M

---@overload fun(identifier: integer): Connection?
---@overload fun(identifier: integer, ip: string, port: integer): Connection
setmetatable(M, {
	__call = function(_, id, ip, port)
		local existing = storage[id]
		if existing then
			return existing
		end

		if not ip or not port then
			return nil
		end

		local self = setmetatable({
			_id = id,
			_ip = ip,
			_port = port,
		}, M)

		storage[id] = self
		return self
	end,
})

---Removes the connection from the internal cache.
---@return nil
function M:remove()
	storage[self._id] = nil
end

Connection = M

return M
