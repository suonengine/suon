---ShutdownEvent
---Fired when the server is shutting down, after the task loop exits.
---Modules should save state and release resources here.
---@see StartupEvent
local M = Event:define()

ShutdownEvent = M

return M
