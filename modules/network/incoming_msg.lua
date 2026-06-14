---@type IncomingMessage
---Read-only binary buffer for decoding network packets.
---@class IncomingMessage
---@field _buffer string
---@field _position integer
---@field _length integer
local M = {}
M.__index = M
M.__tostring = function(self)
	return string.format("IncomingMessage[%d]", #self._buffer)
end

---@return IncomingMessage
local function construct(data)
	return setmetatable({
		_buffer = data or "",
		_position = 1,
		_length = #(data or ""),
	}, M)
end

---@type IncomingMessage
---@overload fun(data: string): IncomingMessage
local callable = setmetatable({}, {
	__index = M,
	__call = construct,
})

---Unsigned 8-bit integer.
---@return integer
function M:getU8()
	local value = string.byte(self._buffer, self._position) or 0
	self._position = self._position + 1
	return value
end

---Signed 8-bit integer.
---@return integer
function M:getI8()
	local value = self:getU8()
	if value > 127 then
		value = value - 256
	end
	return value
end

---Little-endian unsigned 16-bit integer.
---@return integer
function M:getU16()
	local low = self:getU8()
	local high = self:getU8()
	return low + high * 256
end

---Little-endian signed 16-bit integer.
---@return integer
function M:getI16()
	local value = self:getU16()
	if value > 32767 then
		value = value - 65536
	end
	return value
end

---Little-endian unsigned 32-bit integer.
---@return integer
function M:getU32()
	local low = self:getU16()
	local high = self:getU16()
	return low + high * 65536
end

---Little-endian signed 32-bit integer.
---@return integer
function M:getI32()
	local value = self:getU32()
	if value > 2147483647 then
		value = value - 4294967296
	end
	return value
end

---Little-endian unsigned 64-bit integer.
---@return integer
function M:getU64()
	local low = self:getU32()
	local high = self:getU32()
	return low + high * 4294967296
end

---Little-endian signed 64-bit integer.
---@return integer
function M:getI64()
	local low = self:getU32()
	local high = self:getU32()
	if high > 2147483647 then
		high = high - 4294967296
	end
	return low + high * 4294967296
end

---Boolean (0x00 = false, anything else = true).
---@return boolean
function M:getBoolean()
	return self:getU8() ~= 0
end

---Pascal-style string (U16 length prefix).
---@return string
function M:getString()
	local length = self:getU16()
	if length == 0 or self._position > self._length then
		return ""
	end

	local value = self._buffer:sub(self._position, self._position + length - 1)
	self._position = self._position + length
	return value
end

---Little-endian 32-bit float.
---@return number
function M:getFloat()
	if self._position + 3 > self._length then
		return 0.0
	end

	local value = string.unpack("<f", self._buffer, self._position)
	self._position = self._position + 4
	return value
end

---Little-endian 64-bit double.
---@return number
function M:getDouble()
	if self._position + 7 > self._length then
		return 0.0
	end

	local value = string.unpack("<d", self._buffer, self._position)
	self._position = self._position + 8
	return value
end

---Total buffer length in bytes.
---@return integer
function M:getLength()
	return self._length
end

---Current read position (1-based).
---@return integer
function M:getPosition()
	return self._position
end

---Sets the read position.
---@param position integer
---@return boolean true if the position is valid
function M:setPosition(position)
	if position < 1 or position > self._length + 1 then
		return false
	end

	self._position = position
	return true
end

---Skips `count` bytes without reading them.
---@param count integer
function M:skip(count)
	self._position = math.min(self._position + count, self._length + 1)
end

---Returns true when all data has been consumed.
---@return boolean
function M:eof()
	return self._position > self._length
end

---Reads a fixed number of raw bytes.
---@param count integer
---@return string
function M:readBytes(count)
	if count <= 0 then
		return ""
	end

	if self._position > self._length then
		return ""
	end

	local available = self._length - self._position + 1
	local actual = count > available and available or count
	local value = self._buffer:sub(self._position, self._position + actual - 1)
	self._position = self._position + actual
	return value
end

---Unsigned 8-bit integer without advancing.
---@return integer
function M:peekU8()
	local saved = self:getPosition()
	local value = self:getU8()
	self._position = saved
	return value
end

---Signed 8-bit integer without advancing.
---@return integer
function M:peekI8()
	local saved = self:getPosition()
	local value = self:getI8()
	self._position = saved
	return value
end

---Little-endian unsigned 16-bit integer without advancing.
---@return integer
function M:peekU16()
	local saved = self:getPosition()
	local value = self:getU16()
	self._position = saved
	return value
end

---Little-endian signed 16-bit integer without advancing.
---@return integer
function M:peekI16()
	local saved = self:getPosition()
	local value = self:getI16()
	self._position = saved
	return value
end

---Little-endian unsigned 32-bit integer without advancing.
---@return integer
function M:peekU32()
	local saved = self:getPosition()
	local value = self:getU32()
	self._position = saved
	return value
end

---Little-endian signed 32-bit integer without advancing.
---@return integer
function M:peekI32()
	local saved = self:getPosition()
	local value = self:getI32()
	self._position = saved
	return value
end

---Little-endian unsigned 64-bit integer without advancing.
---@return integer
function M:peekU64()
	local saved = self:getPosition()
	local value = self:getU64()
	self._position = saved
	return value
end

---Little-endian signed 64-bit integer without advancing.
---@return integer
function M:peekI64()
	local saved = self:getPosition()
	local value = self:getI64()
	self._position = saved
	return value
end

---Boolean without advancing.
---@return boolean
function M:peekBoolean()
	local saved = self:getPosition()
	local value = self:getBoolean()
	self._position = saved
	return value
end

---Pascal-style string without advancing.
---@return string
function M:peekString()
	local saved = self:getPosition()
	local value = self:getString()
	self._position = saved
	return value
end

---Little-endian 32-bit float without advancing.
---@return number
function M:peekFloat()
	local saved = self:getPosition()
	local value = self:getFloat()
	self._position = saved
	return value
end

---Little-endian 64-bit double without advancing.
---@return number
function M:peekDouble()
	local saved = self:getPosition()
	local value = self:getDouble()
	self._position = saved
	return value
end

return callable
