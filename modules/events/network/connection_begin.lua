local CancellableConnectionEvent = require("events.network.cancellable_connection")

---@type ConnectionBeginEvent
---Fired when a new TCP connection is established.
---@class ConnectionBeginEvent : CancellableConnectionEvent
---@field _connection Connection
---@field cancelled boolean
local M = CancellableConnectionEvent:define()

return M
