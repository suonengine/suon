---Fired when a raw HTTP request arrives. Carries the port, method,
---path, headers, body and a respond function.
---
---Modules should NOT attach to this directly. Use HttpRequestEvent instead,
---which provides method+path routing and HttpRequest/HttpResponse objects.
---@class RawHttpRequestEvent : Event
---@field request_id integer
---@field port integer
---@field method string
---@field path string
---@field headers table<string, string>
---@field body string
---@field _respond fun(status: integer, body: string)
local M = Event:define()

---@class RawHttpRequestEvent : Event
RawHttpRequestEvent = M

local MT = getmetatable(M)
---@return RawHttpRequestEvent
MT.__call = function(self, request_id, port, method, path, headers, body, respond)
	return setmetatable({
		args = { request_id, port, method, path, headers, body, respond },
		request_id = request_id,
		port = port,
		method = method,
		path = path,
		headers = headers,
		body = body,
		_respond = respond,
	}, self)
end

---@return integer
function M:getRequestId()
	return self.request_id
end

---@return integer
function M:getPort()
	return self.port
end

---@return string
function M:getMethod()
	return self.method
end

---@return string
function M:getPath()
	return self.path
end

---@return table<string, string>
function M:getHeaders()
	return self.headers
end

---@return string
function M:getBody()
	return self.body
end

---Sends an HTTP response.
---@param status integer
---@param body string
function M:send(status, body)
	if self._respond then
		self._respond(status, body)
		self._respond = nil
	end
end

return M
