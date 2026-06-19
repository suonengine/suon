---@type SessionStatus
---Session status constants.
---@class SessionStatus
---@field Current string  Active, valid session
---@field Expired string  Session that has timed out
local M = {
	Current = "current",
	Expired = "expired",
}

return M
