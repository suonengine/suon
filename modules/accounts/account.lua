local AccountType = require("accounts.type")
local AccountStatus = require("accounts.status")

---@type Account
---A registered user account.
---@class Account
---@field _id integer
---@field _name string
---@field _email string
---@field _password string
---@field _premium boolean
---@field _premiumUntil integer
---@field _type AccountType
---@field _status AccountStatus
---@field _lastLogin integer?
local M = {}
M.__index = M

---@overload fun(id: integer, name: string, email: string, password: string): Account
setmetatable(M, {
	__call = function(_, id, name, email, password)
		return setmetatable({
			_id = id,
			_name = name,
			_email = email,
			_password = password,
			_premium = false,
			_premiumUntil = 0,
			_type = AccountType.Normal,
			_status = AccountStatus.Active,
			_lastLogin = nil,
		}, M)
	end,
})

---@return integer
function M:getId()
	return self._id
end

---@return string
function M:getName()
	return self._name
end

---@return string
function M:getEmail()
	return self._email
end

---@return boolean
function M:isPremium()
	return self._premium
end

---@param value boolean
function M:setPremium(value)
	self._premium = value
end

---@return integer
function M:getPremiumUntil()
	return self._premiumUntil
end

---@param timestamp integer
function M:setPremiumUntil(timestamp)
	self._premiumUntil = timestamp
end

---@return AccountType
function M:getType()
	return self._type
end

---@return AccountStatus
function M:getStatus()
	return self._status
end

---@param status AccountStatus
function M:setStatus(status)
	self._status = status
end

---@return integer?
function M:getLastLogin()
	return self._lastLogin
end

---@param timestamp integer
function M:setLastLogin(timestamp)
	self._lastLogin = timestamp
end

---@param password string
---@return boolean
function M:verifyPassword(password)
	return self._password == password
end

return M
