---@type OutgoingMessage
---Builder for outgoing network messages.
---@class OutgoingMessage
---@field _buffer string[]
local M = {}
M.__index = M
M.__tostring = function(self)
	return string.format("OutgoingMessage[%d]", self:getLength())
end

---@return OutgoingMessage
local function construct()
	return setmetatable({
		_buffer = {},
	}, M)
end

---@type OutgoingMessage
---@overload fun(): OutgoingMessage
local callable = setmetatable({}, {
	__index = M,
	__call = construct,
})

---Unsigned 8-bit integer.
---@param value integer
function M:addU8(value)
	table.insert(self._buffer, string.char(value & 0xFF))
end

---Signed 8-bit integer.
---@param value integer
function M:addI8(value)
	self:addU8(value & 0xFF)
end

---Little-endian unsigned 16-bit integer.
---@param value integer
function M:addU16(value)
	self:addU8(value & 0xFF)
	self:addU8((value >> 8) & 0xFF)
end

---Little-endian signed 16-bit integer.
---@param value integer
function M:addI16(value)
	self:addU16(value & 0xFFFF)
end

---Little-endian unsigned 32-bit integer.
---@param value integer
function M:addU32(value)
	self:addU16(value & 0xFFFF)
	self:addU16((value >> 16) & 0xFFFF)
end

---Little-endian signed 32-bit integer.
---@param value integer
function M:addI32(value)
	self:addU32(value & 0xFFFFFFFF)
end

---Little-endian unsigned 64-bit integer.
---@param value integer
function M:addU64(value)
	self:addU32(value & 0xFFFFFFFF)
	self:addU32((value >> 32) & 0xFFFFFFFF)
end

---Little-endian signed 64-bit integer.
---@param value integer
function M:addI64(value)
	self:addU64(value)
end

---Boolean (0x00 or 0x01).
---@param value boolean
function M:addBoolean(value)
	self:addU8(value and 1 or 0)
end

---Pascal-style string (U16 length prefix).
---@param value string
function M:addString(value)
	value = value or ""
	self:addU16(#value)
	table.insert(self._buffer, value)
end

---Raw bytes appended directly.
---@param data string
function M:addRaw(data)
	if data and #data > 0 then
		table.insert(self._buffer, data)
	end
end

---Little-endian 32-bit float.
---@param value number
function M:addFloat(value)
	table.insert(self._buffer, string.pack("<f", value))
end

---Little-endian 64-bit double.
---@param value number
function M:addDouble(value)
	table.insert(self._buffer, string.pack("<d", value))
end

---Current byte length of the buffer.
---@return integer
function M:getLength()
	local total = 0
	for _, part in ipairs(self._buffer) do
		total = total + #part
	end
	return total
end

---Sends the buffer through the given connection.
---@param connection Connection Connection object with `_id` field
function M:send(connection)
	connection:send(table.concat(self._buffer))
end

return callable
