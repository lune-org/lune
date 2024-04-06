local M = {}

M.state = 10
_state_test_global = 10

function M.set_state(n: number)
    M.state = n
    _state_test_global = n
end

return M
