---@type HttpStatus
---HTTP status codes.
---@class HttpStatus
---@field OK integer
---@field CREATED integer
---@field ACCEPTED integer
---@field NO_CONTENT integer
---@field MOVED_PERMANENTLY integer
---@field FOUND integer
---@field BAD_REQUEST integer
---@field UNAUTHORIZED integer
---@field FORBIDDEN integer
---@field NOT_FOUND integer
---@field METHOD_NOT_ALLOWED integer
---@field CONFLICT integer
---@field GONE integer
---@field UNPROCESSABLE_ENTITY integer
---@field TOO_MANY_REQUESTS integer
---@field INTERNAL_SERVER_ERROR integer
---@field NOT_IMPLEMENTED integer
---@field SERVICE_UNAVAILABLE integer
local M = {
	OK = 200,
	CREATED = 201,
	ACCEPTED = 202,
	NO_CONTENT = 204,
	MOVED_PERMANENTLY = 301,
	FOUND = 302,
	NOT_MODIFIED = 304,
	BAD_REQUEST = 400,
	UNAUTHORIZED = 401,
	FORBIDDEN = 403,
	NOT_FOUND = 404,
	METHOD_NOT_ALLOWED = 405,
	CONFLICT = 409,
	GONE = 410,
	UNPROCESSABLE_ENTITY = 422,
	TOO_MANY_REQUESTS = 429,
	INTERNAL_SERVER_ERROR = 500,
	NOT_IMPLEMENTED = 501,
	SERVICE_UNAVAILABLE = 503,
}

return M
