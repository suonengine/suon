---HTTP request parser.
---@class HttpRequest
---@field id integer
---@field method string
---@field path string
---@field body string
---@field headers table<string, string>
---@field query table<string, string>?
local M = {}
M.__index = M

---@class HttpRequest
HttpRequest = M

setmetatable(M, {
	__call = function(_, id, method, path, body, headers)
		return setmetatable({
			id = id,
			method = method,
			path = path,
			body = body,
			headers = headers,
		}, M)
	end,
})

---@return integer
function M:getId()
	return self.id
end

---@return string
function M:getMethod()
	return self.method
end

---@return string
function M:getPath()
	return self.path
end

---@return string
function M:getBody()
	return self.body
end

---@return table<string, string>
function M:getHeaders()
	return self.headers
end

---@param name string
---@return string?
function M:getHeader(name)
	return self.headers[name:lower()]
end

---@return boolean
function M:isGet()
	return self.method == "GET"
end

---@return boolean
function M:isPost()
	return self.method == "POST"
end

---@return boolean
function M:isPut()
	return self.method == "PUT"
end

---@return boolean
function M:isDelete()
	return self.method == "DELETE"
end

---Match path against a route pattern with :param support.
---@param pattern string
---@param params table<string, string>?
---@return boolean
function M:route(pattern, params)
	local cleanpath = self.path:gsub("%?.*$", "")
	if not pattern:find("[*:]") then
		return cleanpath == pattern
	end

	local pattern_parts = {}
	for part in pattern:gmatch("[^/]+") do
		pattern_parts[#pattern_parts + 1] = part
	end

	local path_parts = {}
	for part in cleanpath:gmatch("[^/]+") do
		path_parts[#path_parts + 1] = part
	end

	if #pattern_parts ~= #path_parts then
		return false
	end

	for i, pattern_part in ipairs(pattern_parts) do
		if pattern_part == "*" then
		elseif pattern_part:sub(1, 1) == ":" then
			if params then
				params[pattern_part:sub(2)] = path_parts[i]
			end
		elseif pattern_part ~= path_parts[i] then
			return false
		end
	end
	return true
end

---@return table<string, string>
function M:getQuery()
	if self.query then
		return self.query
	end

	local result = {}
	local query_string = self.path:match("%?(.+)$")
	if query_string then
		for pair in query_string:gmatch("[^&]+") do
			local key, value = pair:match("^(.-)=(.*)$")
			if key then
				result[key] = value or ""
			else
				result[pair] = ""
			end
		end
	end

	self.query = result
	return result
end

---@param name string
---@return string?
function M:getCookie(name)
	local cookie_header = self.headers["cookie"]
	if not cookie_header then
		return nil
	end

	local start = cookie_header:find(name .. "=", 1, true)
	if not start then
		return nil
	end

	local value_start = start + #name + 1
	local value_end = cookie_header:find(";", value_start, true)
	if not value_end then
		return cookie_header:sub(value_start)
	end
	return cookie_header:sub(value_start, value_end - 1)
end

return M
