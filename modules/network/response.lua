local HttpStatus = require("network.http_status")

---@type HttpResponse
---HTTP response builder.
---@class HttpResponse
---@field status integer
---@field headers table<string, string>
---@field body string
---@field _sent boolean
---@field callback fun(status: integer, headers: table<string, string>, body: string)
local M = {}
M.__index = M

setmetatable(M, {
	__call = function(_, callback)
		return setmetatable({
			status = HttpStatus.OK,
			headers = {},
			body = "",
			_sent = false,
			callback = callback,
		}, M)
	end
})

HttpResponse = M

---@param code integer
---@param message string?
function M:setStatus(code, message)
	self.status = code
	if message then
		self.body = message
	end
end

---@param name string
---@param value string
function M:setHeader(name, value)
	self.headers[name] = value
end

---@param body string
function M:setBody(body)
	self.body = body
end

---Sends 200 OK.
---@param body string?
function M:ok(body)
	self:setStatus(HttpStatus.OK)
	if body then
		self.body = body
	end
end

---Sends 201 Created.
function M:created(location)
	self:setStatus(HttpStatus.CREATED, "Created")
	if location then
		self.headers["Location"] = location
	end
end

---Sends 204 No Content.
function M:noContent()
	self:setStatus(HttpStatus.NO_CONTENT, "")
end

---Sends redirect.
---@param url string
function M:redirect(url)
	self:setStatus(HttpStatus.FOUND, "")
	self.headers["Location"] = url
end

---Sends 400 Bad Request.
---@param message string?
function M:badRequest(message)
	self:setStatus(HttpStatus.BAD_REQUEST, message or "Bad request")
end

---Sends 401 Unauthorized.
---@param message string?
function M:unauthorized(message)
	self:setStatus(HttpStatus.UNAUTHORIZED, message or "Unauthorized")
end

---Sends 403 Forbidden.
---@param message string?
function M:forbidden(message)
	self:setStatus(HttpStatus.FORBIDDEN, message or "Forbidden")
end

---Sends 404 Not Found.
---@param message string?
function M:notFound(message)
	self:setStatus(HttpStatus.NOT_FOUND, message or "Not found")
end

---Sends 405 Method Not Allowed.
---@param allowed string?
function M:methodNotAllowed(allowed)
	self:setStatus(HttpStatus.METHOD_NOT_ALLOWED, "Method not allowed")
	if allowed then
		self.headers["Allow"] = allowed
	end
end

---Sends 500 Internal Server Error.
---@param message string?
function M:serverError(message)
	self:setStatus(HttpStatus.INTERNAL_SERVER_ERROR, message or "Internal server error")
end

---Sends the response through the callback.
function M:send()
	if self._sent then return end
	self._sent = true
	self.callback(self.status, self.headers, self.body)
end

return M
