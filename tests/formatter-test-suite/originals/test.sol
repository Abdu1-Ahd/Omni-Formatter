// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

// ── CASE 1: Interface ─────────────────────────────────────────────────────
interface IERC20 {
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to,uint256 amount) external returns (bool);
    function allowance(address owner,address spender) external view returns (uint256);
    function approve(address spender,uint256 amount) external returns (bool);
    function transferFrom(address from,address to,uint256 amount) external returns (bool);

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

// ── CASE 2: Contract — mixed spacing ──────────────────────────────────────
contract Token is IERC20 {
    string  public name;
    string  public symbol;
    uint8   public decimals;
    uint256 private _totalSupply;

    mapping(address=>uint256) private _balances;
    mapping(address=>mapping(address=>uint256)) private _allowances;

    address public   owner;

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    constructor(string memory _name, string memory _symbol, uint256 initialSupply) {
        name     = _name;
        symbol   = _symbol;
        decimals = 18;
        owner    = msg.sender;
        _mint(msg.sender, initialSupply * 10**decimals);
    }

    // ── CASE 3: State-mutating functions ─────────────────────────────────
    function transfer(address to, uint256 amount) external override returns (bool) {
        _transfer(msg.sender,to,amount);
        return true;
    }

    function approve(address spender,uint256 amount) external override returns (bool) {
        _approve(msg.sender,spender,amount);
        return true;
    }

    // ── CASE 4: View functions ────────────────────────────────────────────
    function totalSupply() external view override returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(address account) external view override returns (uint256) {
        return _balances[account];
    }

    function allowance(address owner_,address spender) external view override returns (uint256) {
        return _allowances[owner_][spender];
    }

    // ── CASE 5: Internal functions ────────────────────────────────────────
    function _transfer(address from,address to,uint256 amount) internal {
        require(from != address(0), "Transfer from zero");
        require(to != address(0),   "Transfer to zero");
        require(_balances[from] >= amount, "Insufficient balance");

        _balances[from] -= amount;
        _balances[to]   += amount;
        emit Transfer(from, to, amount);
    }

    function _mint(address account,uint256 amount) internal {
        require(account != address(0));
        _totalSupply += amount;
        _balances[account] += amount;
        emit Transfer(address(0), account, amount);
    }

    function _approve(address owner_,address spender,uint256 amount) internal {
        _allowances[owner_][spender] = amount;
        emit Approval(owner_, spender, amount);
    }

    // ── CASE 6: Custom errors ─────────────────────────────────────────────
    error Unauthorized(address caller);
    error InsufficientAllowance(uint256 needed,uint256 available);
}
