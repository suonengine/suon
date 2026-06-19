print(">> Loading session module...")

local Session = require("sessions.session")

math.randomseed(os.time() + (tostring({}):byte(-1) or 0))
math.random()
math.random()

local byKey = setmetatable({}, { __mode = "v" })
local byAccount = setmetatable({}, { __mode = "v" })
local nextId = 1

---@class Sessions
local M = {}

---Creates a new session for the given account.
---Any existing session for the account is invalidated first.
---@param accountId integer
---@return Session
function M.create(accountId)
	M.removeByAccount(accountId)

	local id = nextId
	nextId = nextId + 1

	local key = string.format("%08x%08x", math.random(0, 0x7FFFFFFF), os.time())
	local session = Session(id, accountId, key)
	byKey[key] = session
	byAccount[accountId] = session
	return session
end

---Finds a session by its key. Returns nil if not found or expired.
---Expired sessions are removed automatically.
---@param key string
---@return Session?
function M.find(key)
	local session = byKey[key]
	if not session then
		return nil
	end

	if session:isExpired() then
		M.remove(key)
		return nil
	end
	return session
end

---Finds a session by account ID. Returns nil if not found or expired.
---@param accountId integer
---@return Session?
function M.findByAccount(accountId)
	local session = byAccount[accountId]
	if not session then
		return nil
	end

	if session:isExpired() then
		M.removeByAccount(accountId)
		return nil
	end
	return session
end

---Returns true when the account has a valid, non-expired session.
---@param accountId integer
---@return boolean
function M.isValid(accountId)
	local session = byAccount[accountId]
	if not session then
		return false
	end

	if session:isExpired() then
		M.removeByAccount(accountId)
		return false
	end
	return true
end

---Removes a session by its key.
---@param key string
function M.remove(key)
	local session = byKey[key]
	if session then
		byAccount[session:getAccountId()] = nil
		byKey[key] = nil
	end
end

---Removes the session for the given account.
---@param accountId integer
function M.removeByAccount(accountId)
	local session = byAccount[accountId]
	if session then
		byKey[session:getKey()] = nil
		byAccount[accountId] = nil
	end
end

---Removes all expired sessions from the cache.
function M.cleanup()
	for key, session in pairs(byKey) do
		if session:isExpired() then
			byAccount[session:getAccountId()] = nil
			byKey[key] = nil
		end
	end
end

---Returns the number of active sessions.
---@return integer
function M.count()
	local n = 0
	for _ in pairs(byKey) do
		n = n + 1
	end
	return n
end

return M
