---Handles HTTP method+path dispatch. `RawHttpRequestEvent` feeds raw
---request data here; the method and path are parsed and routed to
---registered handlers.
---@class HttpRequestEvent
local M = {}
M.__index = M

---@class HttpRequestEvent
HttpRequestEvent = M

---@type table<string, { port: integer?, method: string, pattern: string, handler: fun(event: HttpRequestEvent) }[]>
local routes = {}

---Register a handler for an HTTP method and path pattern on a specific port.
---
---@param port integer   Server port (e.g. 8080)
---@param method string  "GET", "POST", etc.
---@param pattern string e.g. "/login", "/api/:id"
---@param handler fun(event: HttpRequestEvent)
function M:on(port, method, pattern, handler)
	if not routes[method] then
		routes[method] = {}
	end

	table.insert(routes[method], {
		port = port,
		method = method,
		pattern = pattern,
		handler = handler,
	})
end

---Dispatches to all handlers matching the method, path and port.
---@param rawEvent RawHttpRequestEvent
function M:trigger(rawEvent)
	local method = rawEvent:getMethod()
	local path = rawEvent:getPath()
	local port = rawEvent:getPort()
	local list = routes[method]
	if not list then
		rawEvent:send(HttpStatus.NOT_FOUND, "Not Found")
		return
	end

	local req = HttpRequest(rawEvent:getRequestId(), method, path, rawEvent:getBody(), rawEvent:getHeaders())

	local res = HttpResponse(function(status, headers, body)
		rawEvent:send(status, body)
	end)

	local matched = false
	for _, entry in ipairs(list) do
		if entry.port == port then
			if req:route(entry.pattern) then
				matched = true
				local ok, err = pcall(entry.handler, {
					getRequest = function()
						return req
					end,
					getResponse = function()
						return res
					end,
				})
				if not ok then
					print(string.format("[HttpRequestEvent] Handler error for %s %s: %s", method, path, tostring(err)))
				end
			end
		end
	end

	if not matched then
		rawEvent:send(HttpStatus.NOT_FOUND, "Not Found")
	end
end

return M
