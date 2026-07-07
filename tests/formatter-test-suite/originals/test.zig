const std = @import("std");

// ── CASE 1: Struct and comptime ────────────────────────────────────────────
const User = struct {
    id:    u32,
    name:  []const u8,
    email: []const u8,
    age:   u8 = 0,

    pub fn init( id: u32 , name: []const u8 , email: []const u8 ) User {
        return User{ .id=id, .name=name, .email=email };
    }

    pub fn greet(self: User, writer: anytype) !void {
        try writer.print("Hello, {s}!\n", .{self.name});
    }
};

// ── CASE 2: Error union and optional ──────────────────────────────────────
const ParseError = error{
    InvalidInput,
    Overflow,
    EmptyString,
};

fn parseAge(s: []const u8) ParseError!u8 {
    if (s.len == 0) return ParseError.EmptyString;
    const n = std.fmt.parseInt(u8, s, 10) catch return ParseError.InvalidInput;
    return n;
}

// ── CASE 3: Generics via comptime ─────────────────────────────────────────
fn Stack(comptime T: type) type {
    return struct {
        items: std.ArrayList(T),

        const Self = @This();

        pub fn init(allocator: std.mem.Allocator) Self {
            return Self{ .items = std.ArrayList(T).init(allocator) };
        }

        pub fn push(self: *Self, item: T) !void {
            try self.items.append(item);
        }

        pub fn pop(self: *Self) ?T {
            if (self.items.items.len == 0) return null;
            return self.items.pop();
        }
    };
}

// ── CASE 4: Allocators and memory ─────────────────────────────────────────
pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var list = std.ArrayList(u32).init(allocator);
    defer list.deinit();

    var i: u32 = 0;
    while (i < 10) : (i += 1) {
        try list.append(i * i);
    }

    for (list.items) |item| {
        std.debug.print("{d}\n", .{item});
    }
}

// ── CASE 5: Long function signature ───────────────────────────────────────
pub fn processWithVeryLongFunctionNameExceedingLineWidth(allocator: std.mem.Allocator, input: []const u8, transform: fn (u8) u8) ![]u8 {
    const result = try allocator.alloc(u8, input.len);
    for (input, 0..) |byte, i| {
        result[i] = transform(byte);
    }
    return result;
}
