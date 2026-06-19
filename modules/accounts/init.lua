print(">> Loading accounts module...")

require("accounts.type")
require("accounts.status")
require("accounts.startup")

local Account = require("accounts.account")

local loadedById = {}
local loadedByName = {}
local loadedByEmail = {}
local nextId = 1

---@class Accounts
local M = {}

---Loads all accounts from storage into memory.
---Seeds a default admin account when the store is empty.
function M.loadAll()
	if not loadedById[1] then
		local account = Account(1, "admin", "admin@suon.dev", "admin")
		account:setPremium(true)
		account:setPremiumUntil(9999999999)
		loadedById[1] = account
		loadedByName["admin"] = account
		loadedByEmail["admin@suon.dev"] = account
		nextId = 2
	end

	print(string.format(">> Loaded %d accounts", M.count()))
end

---Looks up an account by its numeric ID.
---@param id integer
---@return Account?
function M.findById(id)
	return loadedById[id]
end

---Looks up an account by its username.
---@param name string
---@return Account?
function M.findByName(name)
	return loadedByName[name]
end

---Looks up an account by its email address.
---@param email string
---@return Account?
function M.findByEmail(email)
	return loadedByEmail[email]
end

---Authenticates an account by login (name or email) and password.
---@param login string
---@param password string
---@return Account?
function M.authenticate(login, password)
	local account = loadedByName[login] or loadedByEmail[login]
	if not account then
		return nil
	end

	if not account:verifyPassword(password) then
		return nil
	end
	return account
end

---Creates a new account and stores it in the cache.
---@param name string
---@param email string
---@param password string
---@return Account
function M.create(name, email, password)
	local id = nextId
	nextId = nextId + 1

	local account = Account(id, name, email, password)
	loadedById[id] = account
	loadedByName[name] = account
	loadedByEmail[email] = account
	return account
end

---Removes an account from the cache by ID.
---@param id integer
function M.remove(id)
	local account = loadedById[id]
	if account then
		loadedByName[account:getName()] = nil
		loadedByEmail[account:getEmail()] = nil
		loadedById[id] = nil
	end
end

---Returns the total number of cached accounts.
---@return integer
function M.count()
	local n = 0
	for _ in pairs(loadedById) do
		n = n + 1
	end
	return n
end

---Updates the last login timestamp for an account.
---@param id integer
function M.updateLastLogin(id)
	local account = loadedById[id]
	if account then
		account:setLastLogin(os.time())
	end
end

return M
