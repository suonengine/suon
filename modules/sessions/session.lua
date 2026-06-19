---@type Session
---An authenticated user session.
---@class Session
---@field _id integer
---@field _accountId integer
---@field _key string
---@field _createdAt integer
---@field _expiresAt integer
local M = {}
M.__index = M

local SESSION_TIMEOUT = 3600

---@overload fun(id: integer, accountId: integer, key: string): Session
setmetatable(M, {
	__call = function(_, id, accountId, key)
		local now = os.time()
		return setmetatable({
			_id = id,
			_accountId = accountId,
			_key = key,
			_createdAt = now,
			_expiresAt = now + SESSION_TIMEOUT,
		}, M)
	end,
})

---@return integer
function M:getId()
	return self._id
end

---@return integer
function M:getAccountId()
	return self._accountId
end

---@return string
function M:getKey()
	return self._key
end

---@return integer
function M:getCreatedAt()
	return self._createdAt
end

---@return integer
function M:getExpiresAt()
	return self._expiresAt
end

---@return boolean
function M:isValid()
	return self._expiresAt > os.time()
end

---@return boolean
function M:isExpired()
	return not self:isValid()
end

---@return integer
function M:remaining()
	return math.max(0, self._expiresAt - os.time())
end

---@param seconds integer
function M:extend(seconds)
	self._expiresAt = os.time() + seconds
end

return M
