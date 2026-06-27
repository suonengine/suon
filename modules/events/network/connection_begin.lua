---Fired when a new TCP connection is established.
---@class ConnectionBeginEvent : CancellableConnectionEvent
---@field _connection Connection
---@field cancelled boolean
local M = CancellableConnectionEvent:define()

---@class ConnectionBeginEvent : CancellableConnectionEvent
ConnectionBeginEvent = M

return M
