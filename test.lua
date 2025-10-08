#!/usr/bin/env lua5.4
---@version 5.4
--- SPDX-license-identifier: MIT

assert(_VERSION == "Lua 5.4")

local term = require("term")
print("term = " .. tostring(term))

term.input_loop(function(event)
	print("event = " .. tostring(event))
end)
