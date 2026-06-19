---StartupEvent
---Fired once after all plugins are loaded and systems have run.
---Cancelling this event aborts the server startup.
---@see ShutdownEvent
local CancellableEvent = require("events.cancellable")

local M = CancellableEvent:define()

StartupEvent = M

return M
