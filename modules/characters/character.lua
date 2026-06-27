---A playable character on a world.
---@class Character
---@field _name string
---@field _level integer
---@field _vocation string
---@field _worldId integer
---@field _isMale boolean
---@field _outfitId integer
---@field _headColor integer
---@field _torsoColor integer
---@field _legsColor integer
---@field _detailColor integer
---@field _addonsFlags integer
---@field _lastLogin integer
---@field _isHidden boolean
local M = {}
M.__index = M

---@class Character
Character = M

---@overload fun(name: string): Character
setmetatable(M, {
	__call = function(_, name)
		return setmetatable({
			_name = name,
			_level = 1,
			_vocation = "None",
			_worldId = 0,
			_isMale = true,
			_outfitId = 0,
			_headColor = 0,
			_torsoColor = 0,
			_legsColor = 0,
			_detailColor = 0,
			_addonsFlags = 0,
			_lastLogin = 0,
			_isHidden = false,
		}, M)
	end,
})

---@return string
function M:getName()
	return self._name
end

---@return integer
function M:getLevel()
	return self._level
end

---@param level integer
function M:setLevel(level)
	self._level = level
end

---@return string
function M:getVocation()
	return self._vocation
end

---@param vocation string
function M:setVocation(vocation)
	self._vocation = vocation
end

---@return integer
function M:getWorldId()
	return self._worldId
end

---@param worldId integer
function M:setWorldId(worldId)
	self._worldId = worldId
end

---@return boolean
function M:isMale()
	return self._isMale
end

---@return integer
function M:getOutfitId()
	return self._outfitId
end

---@param outfitId integer
function M:setOutfitId(outfitId)
	self._outfitId = outfitId
end

---@return integer
function M:getHeadColor()
	return self._headColor
end

---@param color integer
function M:setHeadColor(color)
	self._headColor = color
end

---@return integer
function M:getTorsoColor()
	return self._torsoColor
end

---@param color integer
function M:setTorsoColor(color)
	self._torsoColor = color
end

---@return integer
function M:getLegsColor()
	return self._legsColor
end

---@param color integer
function M:setLegsColor(color)
	self._legsColor = color
end

---@return integer
function M:getDetailColor()
	return self._detailColor
end

---@param color integer
function M:setDetailColor(color)
	self._detailColor = color
end

---@return integer
function M:getAddonsFlags()
	return self._addonsFlags
end

---@param flags integer
function M:setAddonsFlags(flags)
	self._addonsFlags = flags
end

---@return integer
function M:getLastLogin()
	return self._lastLogin
end

---@param timestamp integer
function M:setLastLogin(timestamp)
	self._lastLogin = timestamp
end

---@return boolean
function M:isHidden()
	return self._isHidden
end

---@param hidden boolean
function M:setHidden(hidden)
	self._isHidden = hidden
end

return M
