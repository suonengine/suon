---Base event class.
---@class Event
---@field handlers { handler: fun(Event), priority: EventPriority, filter?: fun(Event): boolean }[]
---@field args any[]
local M = {}
M.__index = M

---@class Event
Event = M

setmetatable(M, {
	__index = M,
	__call = function(self, ...)
		return setmetatable({ args = { ... } }, self)
	end,
	__tostring = function()
		return "Event"
	end,
})

---Register a handler for this event class.
---@generic T : Event
---@param self T
---@param handler fun(event: T)
---@param priority? EventPriority
---@param filter? fun(event: T): boolean
function M:on(handler, priority, filter)
	local EventPriority = require("events.priority")

	---@type Event
	local s = self

	table.insert(s.handlers, {
		handler = handler,
		priority = priority or EventPriority.NORMAL,
		filter = filter,
	})
end

---Creates an event instance and dispatches to all registered handlers.
---@param ... any
---@return boolean success # true if all handlers returned true
function M:trigger(...)
	if not self.handlers or #self.handlers == 0 then
		return true
	end
	return self:dispatch(self(...))
end

---Dispatches handlers sorted by priority.  Skips those whose filter
---returns false.
---@param eventInstance Event
---@return boolean success
function M:dispatch(eventInstance)
	table.sort(self.handlers, function(a, b)
		return a.priority > b.priority
	end)

	for _, entry in ipairs(self.handlers) do
		if not entry.filter or entry.filter(eventInstance) then
			local ok, error = pcall(entry.handler, eventInstance)
			if not ok then
				local source = "?"

				if debug then
					local information = debug.getinfo(entry.handler, "S")
					if information then
						source = string.format("%s:%d", information.short_src, information.linedefined) or "?"
					end
				end

				print(string.format("[Event] Handler error [%s]: %s", source, tostring(error)))
			end
		end
	end

	return true
end

---Create a new event subclass.
---@generic T : Event
---@param self T
---@param parent T?
---@return T
function M:define(parent)
	parent = parent or self

	local class = {
		handlers = {},
	}

	setmetatable(class, {
		__index = parent,
		__call = getmetatable(parent).__call,
		__tostring = function()
			return string.format("Event[%d handlers]", #(class.handlers or {}))
		end,
	})

	class.__index = class
	return class
end

return M
