local byId = {}

---@class Characters
local M = {}

---@class Characters
Characters = M

---Returns the character list for an account.
---@param accountId integer
---@return Character[]
function M.get(accountId)
	return byId[accountId] or {}
end

---Removes the character list for an account.
---@param accountId integer
function M.remove(accountId)
	byId[accountId] = nil
end

---Returns true when the account has any characters stored.
---@param accountId integer
---@return boolean
function M.has(accountId)
	local chars = byId[accountId]
	return chars and #chars > 0
end

return M
