local storage = setmetatable({}, { __mode = "v" })

---A remote TCP connection.
---@class Connection
---@field _id integer
---@field _ip string
---@field _port integer
---@field _authenticated boolean
---@field _accountId integer?
---@field _sessionKey string?
---@field _characterName string?
---@field _serverName string?
---@field _handshakeSent boolean?
---@field send fun(self: Connection, data: string)
---@field sendRaw fun(self: Connection, data: string)
---@field close fun(self: Connection)
local M = {}
M.__index = M

---@class Connection
Connection = M

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
			_authenticated = false,
			_accountId = nil,
			_sessionKey = nil,
			_characterName = nil,
			_serverName = nil,
			_handshakeSent = nil,
		}, M)

		storage[id] = self
		return self
	end,
})

---@return integer
function M:getId()
	return self._id
end

---@return string
function M:getIp()
	return self._ip
end

---@return integer
function M:getPort()
	return self._port
end

---@return boolean
function M:isAuthenticated()
	return self._authenticated
end

---@return integer?
function M:getAccountId()
	return self._accountId
end

---@return string?
function M:getSessionKey()
	return self._sessionKey
end

---@return string?
function M:getCharacterName()
	return self._characterName
end

---@return string?
function M:getServerName()
	return self._serverName
end

---Marks the connection as authenticated after a successful login.
---@param accountId integer
---@param sessionKey string
---@param characterName string?
function M:authenticate(accountId, sessionKey, characterName)
	self._authenticated = true
	self._accountId = accountId
	self._sessionKey = sessionKey
	self._characterName = characterName
end

---Sets the server name received during the handshake.
---@param name string
function M:setServerName(name)
	self._serverName = name
end

---Removes the connection from the internal cache.
function M:remove()
	storage[self._id] = nil
end

Connection = M

return M
